//! Qualified operator workflow, actual host execution, deterministic candidate.
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn text(p: &Path) -> Result<&str> {
    p.to_str().ok_or_else(|| "UTF-8 path".into())
}
fn assignment(base: &Path, name: &str, code: &[u8]) -> Result<PathBuf> {
    let project = base.join(format!("{name}-source"));
    an_loom::run(["init-native".into(), text(&project)?.into()])?;
    let config = project.join("native-spool.toml");
    let mut doc: toml::Value = fs::read_to_string(&config)?.parse()?;
    doc["acceptance"]=toml::Value::Array(vec![toml::Value::String("bash -n kit/transform.sh".into()),
        toml::Value::String("bash kit/transform.sh loom.toml > \"$AN_LOOM_WORK/out\"; grep -q '^edited:' \"$AN_LOOM_WORK/out\"".into())]);
    fs::write(config, toml::to_string(&doc)?)?;
    let task = base.join(name);
    an_loom::task::prepare(&project, &task)?;
    let candidate = base.join(format!("{name}.sh"));
    fs::write(&candidate, code)?;
    an_loom::task::receive(&task, "kit/transform.sh", &candidate)?;
    Ok(task)
}
fn call(host: &Path, args: &[&str]) -> Result<Value> {
    let output = Command::new(host).args(args).output()?;
    if !output.status.success() {
        return Err(format!(
            "host failed: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn approve(host: &Path, args: &[&str]) -> Result<Value> {
    let p = call(host, args)?;
    let mut a = args.to_vec();
    a.extend([
        "--approve",
        p["approval_sha256"].as_str().ok_or("approval")?,
    ]);
    call(host, &a)
}
pub(super) fn run() -> Result {
    let host = PathBuf::from(std::env::var_os("AN_AGENT_HOST_BIN").ok_or("host binary required")?);
    let t = tempfile::tempdir()?;
    let base = t.path().canonicalize()?;
    let task = assignment(&base, "good", b"printf 'edited:'; cat \"$1\"\n")?;
    let job = base.join("job");
    let delivery = base.join("delivery");
    let preview = an_loom::adapters::develop_native(&task, &host, &job, &delivery, None)?;
    assert_eq!(preview["executed"], false);
    assert!(!job.exists());
    assert!(!delivery.exists());
    assert!(!task.join("authority/develop-native.json").exists());
    assert!(
        an_loom::adapters::develop_native(&task, &host, &job, &delivery, Some(&"0".repeat(64)))
            .is_err()
    );
    assert!(!job.exists());
    an_loom::run([
        "develop-native".into(),
        text(&task)?.into(),
        text(&host)?.into(),
        text(&job)?.into(),
        text(&delivery)?.into(),
        "--approve".into(),
        preview["approval_sha256"]
            .as_str()
            .ok_or("approval")?
            .into(),
    ])?;
    let result: Value = serde_json::from_slice(&fs::read(delivery.join("result.json"))?)?;
    assert_eq!(result["status"], "succeeded");
    assert_eq!(result["model_invoked"], false);
    assert_eq!(
        fs::read(delivery.join("project/checked-release.tar.gz"))?,
        fs::read(task.join("authority/candidate.tar.gz"))?
    );
    let instance = delivery.join("instance");
    let input = base.join("input.txt");
    fs::write(&input, b"hello\n")?;
    let run = approve(
        &host,
        &["spool-start", text(&instance)?, "demo", text(&input)?, "15"],
    )?;
    assert_eq!(run["status"], "succeeded");
    approve(&host, &["spool-accept", text(&instance)?, "demo"])?;
    let versions = call(&host, &["spool-artifacts", text(&instance)?, "demo"])?;
    let output = base.join("output.txt");
    call(
        &host,
        &[
            "artifact-checkout",
            versions["catalog"].as_str().ok_or("catalog")?,
            versions["outputs"][0]["binding"]["revision"]
                .as_str()
                .ok_or("revision")?,
            text(&output)?,
        ],
    )?;
    assert_eq!(fs::read(output)?, b"Report\nedited:hello\n");
    assert!(
        an_loom::adapters::develop_native(
            &task,
            &host,
            &base.join("retry-job"),
            &base.join("retry-delivery"),
            None
        )
        .is_err()
    );
    // An admitted submission with no resulting execution must stay fenced, too.
    let bad = task_fixture(&base)?;
    let bad_job = base.join("bad-job");
    let bad_delivery = base.join("bad-delivery");
    let p = an_loom::adapters::develop_native(&bad, &host, &bad_job, &bad_delivery, None)?;
    assert!(
        an_loom::adapters::develop_native(
            &bad,
            &host,
            &bad_job,
            &bad_delivery,
            p["approval_sha256"].as_str()
        )
        .is_err()
    );
    assert!(bad.join("authority/develop-native.json").exists());
    assert!(bad_job.exists());
    assert!(!bad_delivery.join("spool").exists());
    assert!(
        an_loom::adapters::develop_native(
            &bad,
            &host,
            &base.join("new-job"),
            &base.join("new-delivery"),
            None
        )
        .is_err()
    );
    assert!(!base.join("new-job").exists());
    let diagnostic: Value = serde_json::from_slice(&fs::read(bad_delivery.join("result.json"))?)?;
    assert_eq!(diagnostic["automatic_retries"], 0);
    // Simulate a client interruption after intent, without claiming a workload stopped.
    let interrupted = assignment(&base, "interrupted", b"printf 'edited:'; cat \"$1\"\n")?;
    fs::write(
        interrupted.join("authority/develop-native.json"),
        serde_json::to_vec(&json!({"submitted":true}))?,
    )?;
    assert!(
        an_loom::adapters::develop_native(
            &interrupted,
            &host,
            &base.join("unknown-job"),
            &base.join("unknown-delivery"),
            None
        )
        .is_err()
    );
    assert!(!base.join("unknown-job").exists());
    if let Some(path) = std::env::var_os("LOOM_DEVELOP_NATIVE_EVIDENCE") {
        let run: Value = serde_json::from_slice(&fs::read(delivery.join("host-run.json"))?)?;
        fs::write(
            path,
            serde_json::to_vec_pretty(
                &json!({"preview":preview,"delivery":result,"check_run":run,
            "qualification":"real macOS host execution; deterministic candidate; no model invocation",
            "verified":["preview does not submit","wrong approval does not submit","same checked archive delivered",
                "host loads matching release","separately approved spool executes and produces pinned output",
                "failed check prevents delivery","submitted task cannot rerun with new destinations",
                "simulated interrupted intent prevents submission"]}),
            )?,
        )?;
    }
    Ok(())
}
fn task_fixture(base: &Path) -> Result<PathBuf> {
    assignment(base, "bad", b"if then\n")
}
