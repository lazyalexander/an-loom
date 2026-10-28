//! Kit task gates. Candidate bytes are fixtures, not model output.
use std::{fs, path::Path};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn project(base: &Path) -> Result<std::path::PathBuf> {
    let root = base.join("spool");
    an_loom::run([
        "init-native".to_owned(),
        root.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    Ok(root)
}

fn failed(args: Vec<String>) -> Result<String> {
    match an_loom::run(args) {
        Ok(()) => Err("command unexpectedly succeeded".into()),
        Err(error) => Ok(error.to_string()),
    }
}

fn task(base: &Path, project: &Path) -> Result<std::path::PathBuf> {
    let destination = base.join("task");
    an_loom::run([
        "task".to_owned(),
        project.to_str().ok_or("non-UTF8 path")?.to_owned(),
        destination.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    Ok(destination)
}

#[test]
fn kit_candidate_stays_outside_the_project_and_release() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let assignment = task(&base, &root)?;
    let original = fs::read(root.join("kit/transform.sh"))?;
    let replacement = base.join("replacement.sh");
    fs::write(&replacement, b"printf 'edited\\n'\n")?;
    an_loom::run([
        "receive-task".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
        "kit/transform.sh".to_owned(),
        replacement.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    assert_eq!(fs::read(root.join("kit/transform.sh"))?, original);
    assert_eq!(
        fs::read(assignment.join("candidate/kit/transform.sh"))?,
        b"printf 'edited\\n'\n"
    );
    assert!(!root.join("dist").exists());
    assert!(
        an_loom::run([
            "receive-task".to_owned(),
            assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
            "kit/transform.sh".to_owned(),
            replacement.to_str().ok_or("non-UTF8 path")?.to_owned(),
        ])
        .is_err()
    );
    Ok(())
}

#[test]
fn task_refuses_silk_stale_baseline_tampering_and_late_results() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let marker = base.join("acceptance-ran");
    let command = format!("touch {}", marker.display());
    let contract = root.join("native-spool.toml");
    let text = fs::read_to_string(&contract)?.replace(
        "acceptance = [\n    \"bash -n silk/main.sh\",\n    \"bash -n silk/report.sh\",\n    \"bash -n kit/transform.sh\",\n]",
        &format!("acceptance = [\"{command}\"]"),
    );
    fs::write(&contract, text)?;
    assert!(
        an_loom::run([
            "task".to_owned(),
            root.to_str().ok_or("non-UTF8 path")?.to_owned(),
            root.join("inside")
                .to_str()
                .ok_or("non-UTF8 path")?
                .to_owned(),
        ])
        .is_err()
    );
    let assignment = task(&base, &root)?;
    assert!(!marker.exists());
    let replacement = base.join("replacement.sh");
    fs::write(&replacement, b"printf 'edited\\n'\n")?;
    let silk = failed(vec![
        "receive-task".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
        "silk/main.sh".to_owned(),
        replacement.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    assert!(silk.contains("path is not an editable kit file"), "{silk}");
    assert!(!assignment.join("candidate").exists());
    fs::write(
        root.join("kit/transform.sh"),
        b"printf 'changed source\\n'\n",
    )?;
    let stale = failed(vec![
        "receive-task".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
        "kit/transform.sh".to_owned(),
        replacement.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    assert!(stale.contains("stale task baseline"), "{stale}");
    fs::write(
        root.join("kit/transform.sh"),
        fs::read(assignment.join("workspace/kit/transform.sh"))?,
    )?;
    let mut authority: serde_json::Value =
        serde_json::from_slice(&fs::read(assignment.join("authority/task.json"))?)?;
    authority["editable"]
        .as_array_mut()
        .ok_or("editable")?
        .push(serde_json::json!("silk/main.sh"));
    fs::write(
        assignment.join("authority/task.json"),
        serde_json::to_vec_pretty(&authority)?,
    )?;
    let tampered = failed(vec![
        "receive-task".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
        "kit/transform.sh".to_owned(),
        replacement.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    assert!(
        tampered.contains("task contract does not match the project"),
        "{tampered}"
    );
    Ok(())
}

#[test]
fn revoked_task_drops_a_late_candidate() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let assignment = task(&base, &root)?;
    an_loom::run([
        "revoke-task".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    let replacement = base.join("late.sh");
    fs::write(&replacement, b"printf 'late\\n'\n")?;
    let error = failed(vec![
        "receive-task".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
        "kit/transform.sh".to_owned(),
        replacement.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    assert!(error.contains("revoked task"), "{error}");
    assert!(!assignment.join("candidate").exists());
    Ok(())
}

fn admit(
    base: &std::path::Path,
    assignment: &std::path::Path,
    status: &str,
    output: &str,
) -> Result<String> {
    let plan: serde_json::Value =
        serde_json::from_slice(&fs::read(assignment.join("authority/check-plan.json"))?)?;
    let mut receipt = evidence_for(&plan, b"Report\nedited\n", status)?;
    if status == "failed" {
        receipt["checks"][0]["status"] = serde_json::json!("failed");
        receipt["checks"][0]["output"] = serde_json::json!(output);
    }
    if status == "old" {
        receipt["status"] = serde_json::json!("succeeded");
        receipt["attempt"] = serde_json::json!("0".repeat(64));
    }
    let dir = base.join(format!("evidence-{status}"));
    fs::create_dir(&dir)?;
    fs::write(
        dir.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    fs::write(dir.join("report.txt"), b"Report\nedited\n")?;
    failed(vec![
        "receive-check".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
        dir.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])
}

fn evidence_for(
    plan: &serde_json::Value,
    report: &[u8],
    status: &str,
) -> Result<serde_json::Value> {
    let checks = plan["acceptance"]
        .as_array()
        .ok_or("acceptance")?
        .iter()
        .map(|command| {
            serde_json::json!({
                "command": command,
                "status": "succeeded",
                "output": "",
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema": "an-loom-check-evidence-v1",
        "attempt": plan["attempt"],
        "candidate_sha256": plan["candidate"]["sha256"],
        "status": status,
        "checks": checks,
        "artifacts": [{
            "path": "report.txt",
            "sha256": sha256(report),
        }],
    }))
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn check_evidence_gates_packaging_without_running_commands_or_the_host() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let assignment = task(&base, &root)?;
    let replacement = base.join("replacement.sh");
    fs::write(&replacement, b"printf 'edited:'\n")?;
    an_loom::run([
        "receive-task".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
        "kit/transform.sh".to_owned(),
        replacement.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    an_loom::run([
        "check-task".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    let plan: serde_json::Value =
        serde_json::from_slice(&fs::read(assignment.join("authority/check-plan.json"))?)?;
    assert_eq!(plan["automatic_retries"], 0);
    assert_eq!(plan["host_invoked_by_loom"], false);
    assert_eq!(plan["acceptance"].as_array().ok_or("acceptance")?.len(), 3);
    assert_eq!(plan["diff"]["changed"], true);
    let failed_check = admit(&base, &assignment, "failed", "domain mismatch")?;
    assert!(
        failed_check.contains("command bash -n silk/main.sh")
            && failed_check.contains("domain mismatch"),
        "{failed_check}"
    );
    assert!(!assignment.join("authority/evidence.json").exists());
    let cancelled = admit(&base, &assignment, "cancelled", "")?;
    assert!(cancelled.contains("check cancelled"), "{cancelled}");
    let unknown = admit(&base, &assignment, "unknown", "")?;
    assert!(unknown.contains("no automatic retry"), "{unknown}");
    let stale_attempt = admit(&base, &assignment, "old", "")?;
    assert!(
        stale_attempt.contains("does not match the current check plan"),
        "{stale_attempt}"
    );
    assert!(
        an_loom::run([
            "pack-task".to_owned(),
            assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
            base.join("unpublished")
                .to_str()
                .ok_or("non-UTF8 path")?
                .to_owned(),
        ])
        .is_err()
    );
    assert!(!base.join("unpublished").exists());
    let receipt = evidence_for(&plan, b"Report\nedited\n", "succeeded")?;
    let dir = base.join("evidence-ok");
    fs::create_dir(&dir)?;
    fs::write(
        dir.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    fs::write(dir.join("report.txt"), b"Report\nedited\n")?;
    an_loom::run([
        "receive-check".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
        dir.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    let packed = base.join("packed");
    an_loom::run([
        "pack-task".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
        packed.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    let archive = an_loom::package::pack_project(&packed)?;
    let verified = an_loom::read_archive(&archive)?;
    assert_eq!(
        verified
            .files
            .get("kit/transform.sh")
            .ok_or("kit")?
            .as_slice(),
        b"printf 'edited:'\n"
    );
    assert_eq!(
        fs::read(root.join("kit/transform.sh"))?,
        fs::read(assignment.join("workspace/kit/transform.sh"))?
    );
    assert!(
        an_loom::run([
            "receive-check".to_owned(),
            assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
            dir.to_str().ok_or("non-UTF8 path")?.to_owned(),
        ])
        .is_err()
    );
    Ok(())
}

#[test]
fn project_acceptance_commands_enter_the_plan_without_a_loom_change() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let contract = root.join("native-spool.toml");
    let text =
        fs::read_to_string(&contract)?.replace("bash -n silk/main.sh", "project-domain-check");
    fs::write(&contract, text)?;
    let assignment = task(&base, &root)?;
    fs::write(base.join("replacement.sh"), b"printf 'edited:'\n")?;
    an_loom::run([
        "receive-task".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
        "kit/transform.sh".to_owned(),
        base.join("replacement.sh")
            .to_str()
            .ok_or("non-UTF8 path")?
            .to_owned(),
    ])?;
    an_loom::run([
        "check-task".to_owned(),
        assignment.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    let plan = fs::read_to_string(assignment.join("authority/check-plan.json"))?;
    assert!(plan.contains("project-domain-check"), "{plan}");
    assert!(!plan.contains("bash -n silk/main.sh"));
    Ok(())
}
