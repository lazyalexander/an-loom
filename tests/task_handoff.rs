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

#[test]
fn unverified_operator_success_cannot_authorize_packaging() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let assignment = task(&base, &root)?;
    let candidate = base.join("candidate.sh");
    fs::write(&candidate, b"printf 'edited:'\n")?;
    an_loom::task::receive(&assignment, "kit/transform.sh", &candidate)?;
    let plan = an_loom::task::check(&assignment)?;
    assert_eq!(plan["automatic_retries"], 0);
    assert_eq!(plan["host_invoked_by_loom"], false);
    assert_eq!(plan["diff"]["changed"], true);
    let fake = base.join("operator-evidence");
    fs::create_dir(&fake)?;
    fs::write(
        fake.join("receipt.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema":"an-loom-check-evidence-v1","attempt":plan["attempt"],"status":"succeeded",
            "candidate_sha256":plan["candidate"]["sha256"]
        }))?,
    )?;
    assert!(an_loom::task::receive_check(&assignment, &fake).is_err());
    assert!(an_loom::task::pack_candidate(&assignment, &base.join("pack")).is_err());
    assert!(!base.join("pack").exists());
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

#[test]
fn scaffold_instructions_are_packaged_protected_and_bound_to_the_task() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let assignment = task(&base, &root)?;
    let contract: serde_json::Value =
        serde_json::from_slice(&fs::read(assignment.join("authority/task.json"))?)?;
    assert!(
        contract["protected"]
            .as_array()
            .ok_or("protected")?
            .contains(&serde_json::json!("AGENTS.md"))
    );
    let archive = an_loom::package::pack_project(&root)?;
    assert_eq!(
        an_loom::read_archive(&archive)?.files.get("AGENTS.md"),
        Some(&fs::read(root.join("AGENTS.md"))?)
    );
    let candidate = base.join("candidate.sh");
    fs::write(&candidate, b"printf changed\n")?;
    assert!(an_loom::task::receive(&assignment, "AGENTS.md", &candidate).is_err());
    fs::write(root.join("AGENTS.md"), b"changed instructions\n")?;
    assert!(an_loom::task::receive(&assignment, "kit/transform.sh", &candidate).is_err());
    Ok(())
}
