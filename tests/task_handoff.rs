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
