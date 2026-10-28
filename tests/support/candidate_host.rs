//! Actual macOS host execution; deterministic candidates, no model invocation.
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn text(path: &Path) -> Result<&str> {
    path.to_str().ok_or_else(|| "UTF-8 path".into())
}
fn call(args: &[&str]) -> Result<Value> {
    let output = Command::new(std::env::var_os("AN_AGENT_HOST_BIN").ok_or("host binary")?)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "host {args:?}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn approve(args: &[&str]) -> Result<Value> {
    let p = call(args)?;
    let mut a = args.to_vec();
    a.extend([
        "--approve",
        p["approval_sha256"].as_str().ok_or("approval")?,
    ]);
    call(&a)
}
fn task(base: &Path, name: &str, candidate: &[u8], commands: &[&str]) -> Result<PathBuf> {
    let root = base.join(format!("{name}-source"));
    an_loom::run(["init-native".into(), text(&root)?.into()])?;
    let contract = root.join("native-spool.toml");
    let mut doc: toml::Value = fs::read_to_string(&contract)?.parse()?;
    doc["acceptance"] = toml::Value::Array(
        commands
            .iter()
            .map(|c| toml::Value::String(c.to_string()))
            .collect(),
    );
    fs::write(contract, toml::to_string(&doc)?)?;
    let assignment = base.join(name);
    an_loom::task::prepare(&root, &assignment)?;
    let file = base.join(format!("{name}-candidate.sh"));
    fs::write(&file, candidate)?;
    an_loom::task::receive(&assignment, "kit/transform.sh", &file)?;
    an_loom::task::check(&assignment)?;
    Ok(assignment)
}
fn execute(task: &Path, job: &Path) -> Result<Value> {
    approve(&[
        "workspace-run",
        text(&task.join("authority/host-profile.json"))?,
        text(job)?,
    ])
}
fn accept(job: &Path, run: &Value) -> Result<Value> {
    approve(&[
        "workspace-accept",
        text(job)?,
        run["attempt"].as_str().ok_or("attempt")?,
    ])
}
pub(super) fn run() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let commands = [
        "bash -n kit/transform.sh",
        "bash kit/transform.sh loom.toml > \"$AN_LOOM_WORK/report.txt\"; grep -q '^edited:' \"$AN_LOOM_WORK/report.txt\"",
        "if printf changed > kit/transform.sh; then exit 1; fi",
    ];
    let candidate = b"printf 'edited:'; cat \"$1\"\n";
    let assignment = task(&base, "passing", candidate, &commands)?;
    let job = base.join("job");
    let run = execute(&assignment, &job)?;
    assert_eq!(run["status"], "succeeded");
    assert_eq!(run["termination_confirmed"], true);
    assert!(an_loom::task::receive_check(&assignment, &job).is_err());
    let acceptance = accept(&job, &run)?;
    let id = run["attempt"].as_str().ok_or("attempt")?;
    let receipt_path = job.join("attempts").join(id).join("receipt.json");
    let original = fs::read(&receipt_path)?;
    for (field, value) in [
        ("termination_confirmed", json!(false)),
        ("status", json!("cancelled")),
        ("attempt", json!("00000000-0000-0000-0000-000000000000")),
        ("execution_sha256", json!("0".repeat(64))),
    ] {
        let mut receipt = run.clone();
        receipt[field] = value;
        fs::write(&receipt_path, serde_json::to_vec(&receipt)?)?;
        assert!(
            an_loom::task::receive_check(&assignment, &job).is_err(),
            "{field}"
        );
    }
    fs::write(&receipt_path, &original)?;
    let evidence = an_loom::task::receive_check(&assignment, &job)?;
    let packed = base.join("packed");
    let result = an_loom::task::pack_candidate(&assignment, &packed)?;
    assert_eq!(
        fs::read(result["archive"].as_str().ok_or("archive")?)?,
        fs::read(assignment.join("authority/candidate.tar.gz"))?
    );
    assert_ne!(
        fs::read(base.join("passing-source/kit/transform.sh"))?,
        candidate
    );
    let snapshot = assignment.join("authority/snapshot/silk/main.sh");
    let saved = fs::read(&snapshot)?;
    fs::write(&snapshot, b"exit 1\n")?;
    assert!(an_loom::task::pack_candidate(&assignment, &base.join("tampered")).is_err());
    fs::write(&snapshot, saved)?;
    let again = task(&base, "repeat", candidate, &commands)?;
    assert!(an_loom::task::receive_check(&again, &job).is_err());
    let broken = task(&base, "broken", b"if then\n", &["bash -n kit/transform.sh"])?;
    assert!(execute(&broken, &base.join("broken-job")).is_err());
    assert!(an_loom::task::receive_check(&broken, &base.join("broken-job")).is_err());
    let failed_dir = fs::read_dir(base.join("broken-job/attempts"))?
        .next()
        .ok_or("failed attempt")??
        .path();
    let failed: Value = serde_json::from_slice(&fs::read(failed_dir.join("receipt.json"))?)?;
    assert_eq!(failed["status"], "failed");
    assert_eq!(failed["termination_confirmed"], true);
    let cancelled = cancel(&base)?;
    if let Some(path) = std::env::var_os("LOOM_CANDIDATE_EVIDENCE") {
        fs::write(
            path,
            serde_json::to_vec_pretty(&json!({"run":run,"acceptance":acceptance,
            "evidence":evidence,"package":result,"cancelled":cancelled,"model_invoked":false,"retained_test_root":base}))?,
        )?;
        let _retained = tmp.keep();
    }
    Ok(())
}
fn cancel(base: &Path) -> Result<Value> {
    let task = task(
        base,
        "cancel",
        b"exit 0\n",
        &["echo ready > \"$AN_LOOM_WORK/ready\"; sleep 25"],
    )?;
    let job = base.join("cancel-job");
    let profile = task.join("authority/host-profile.json");
    let preview = call(&["workspace-run", text(&profile)?, text(&job)?])?;
    let stdout = fs::File::create(base.join("cancel.stdout"))?;
    let mut child = Command::new(std::env::var_os("AN_AGENT_HOST_BIN").ok_or("host")?)
        .args([
            "workspace-run",
            text(&profile)?,
            text(&job)?,
            "--approve",
            preview["approval_sha256"].as_str().ok_or("approval")?,
        ])
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(15);
    while !job.join("work/ready").exists()
        && Instant::now() < deadline
        && child.try_wait()?.is_none()
    {
        thread::sleep(Duration::from_millis(25));
    }
    let ready = job.join("work/ready").exists();
    if child.try_wait()?.is_none() {
        Command::new("/bin/kill")
            .args(["-TERM", &child.id().to_string()])
            .status()?;
    }
    let _ = child.wait()?;
    assert!(ready, "host did not reach cancellation point");
    let receipt: Value = serde_json::from_slice(&fs::read(base.join("cancel.stdout"))?)?;
    assert_eq!(receipt["status"], "cancelled");
    assert_eq!(receipt["termination_confirmed"], true);
    assert!(an_loom::task::receive_check(&task, &job).is_err());
    assert!(an_loom::task::pack_candidate(&task, &base.join("cancel-pack")).is_err());
    Ok(receipt)
}
