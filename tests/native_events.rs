//! Event interfaces are explicit package metadata, never host routing or grants.
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn project(base: &Path, suffix: &str) -> Result<PathBuf> {
    let root = base.join("project");
    an_loom::run(["init-native".into(), root.to_string_lossy().into_owned()])?;
    let config = root.join("native-spool.toml");
    let original = fs::read_to_string(&config)?;
    fs::write(&config, format!("{original}\n{suffix}"))?;
    Ok(root)
}
fn export(archive: &Path, target: &Path, command: &str) -> Result {
    an_loom::run([
        command.into(),
        archive.to_string_lossy().into_owned(),
        target.to_string_lossy().into_owned(),
    ])?;
    Ok(())
}
#[test]
fn v3_carries_exact_interfaces_and_cannot_be_downgraded_silently() -> Result {
    let temp = tempfile::tempdir()?;
    let base = temp.path().canonicalize()?;
    let root = project(
        &base,
        "[events]\nconsumes = [\"artifact.accepted\"]\nproduces = [\"artifact.accepted\"]\n",
    )?;
    let archive = an_loom::package::pack_project(&root)?;
    let target = base.join("delivery");
    export(&archive, &target, "export-native-spool-v3")?;
    let value: Value = serde_json::from_slice(&fs::read(target.join("native-spool.json"))?)?;
    assert_eq!(value["schema"], "an-native-spool-v3");
    assert_eq!(
        value["events"]["consumes"],
        serde_json::json!(["artifact.accepted"])
    );
    assert_eq!(value["requirements"]["gpu"], "none");
    assert!(value.get("subscriptions").is_none());
    for command in ["export-native-spool", "export-native-spool-v2"] {
        let target = base.join(command);
        assert!(export(&archive, &target, command).is_err());
        assert!(!target.exists());
    }
    Ok(())
}
#[test]
fn undeclared_unknown_and_duplicate_events_are_refused() -> Result {
    for suffix in [
        "",
        "[events]\nconsumes=[\"unknown\"]\nproduces=[]\n",
        "[events]\nconsumes=[\"artifact.accepted\",\"artifact.accepted\"]\nproduces=[]\n",
    ] {
        let temp = tempfile::tempdir()?;
        let base = temp.path().canonicalize()?;
        let root = project(&base, suffix)?;
        let target = base.join("delivery");
        match an_loom::package::pack_project(&root) {
            Ok(archive) => assert!(export(&archive, &target, "export-native-spool-v3").is_err()),
            Err(_) => assert!(!suffix.is_empty()),
        }
        assert!(!target.exists());
    }
    Ok(())
}

#[test]
fn binary_outputs_and_python_requests_are_explicit_and_version_bound() -> Result {
    let temp = tempfile::tempdir()?;
    let base = temp.path().canonicalize()?;
    let root = project(
        &base,
        "binary_outputs=[\"report.txt\"]\n[events]\nconsumes=[]\nproduces=[\"artifact.accepted\"]\n",
    )?;
    let config = root.join("native-spool.toml");
    let text =
        fs::read_to_string(&config)?.replace("macos-native-process-v1", "macos-native-python-v1");
    fs::write(&config, &text)?;
    let archive = an_loom::package::pack_project(&root)?;
    let target = base.join("python");
    export(&archive, &target, "export-native-spool-v3")?;
    let value: Value = serde_json::from_slice(&fs::read(target.join("native-spool.json"))?)?;
    assert_eq!(value["binary_outputs"], serde_json::json!(["report.txt"]));
    assert_eq!(value["requirements"]["runtime"], "macos-native-python-v1");
    assert!(value.get("runtime_path").is_none());
    let task = base.join("task");
    an_loom::run([
        "task".into(),
        root.to_string_lossy().into_owned(),
        task.to_string_lossy().into_owned(),
    ])?;
    let task: Value = serde_json::from_slice(&fs::read(task.join("authority/task.json"))?)?;
    assert_eq!(task["binary_outputs"], serde_json::json!(["report.txt"]));
    assert!(export(&archive, &base.join("downgrade"), "export-native-spool-v2").is_err());
    for invalid in ["[\"unknown.bin\"]", "[\"report.txt\",\"report.txt\"]"] {
        fs::write(
            &config,
            text.replace(
                "binary_outputs=[\"report.txt\"]",
                &format!("binary_outputs={invalid}"),
            ),
        )?;
        assert!(an_loom::package::pack_project(&root).is_err());
    }
    Ok(())
}

#[test]
fn state_input_is_preserved_protected_and_not_silently_downgraded() -> Result {
    let temp = tempfile::tempdir()?;
    let base = temp.path().canonicalize()?;
    let root = project(
        &base,
        "state_input=\"checkpoint.txt\"\n[events]\nconsumes=[\"artifact.accepted\"]\nproduces=[\"artifact.accepted\"]\n",
    )?;
    let archive = an_loom::package::pack_project(&root)?;
    let target = base.join("delivery");
    export(&archive, &target, "export-native-spool-v3")?;
    let manifest: Value = serde_json::from_slice(&fs::read(target.join("native-spool.json"))?)?;
    assert_eq!(manifest["requirements"]["state_input"], "checkpoint.txt");
    assert!(export(&archive, &base.join("legacy"), "export-native-spool-v2").is_err());
    let task = base.join("task");
    an_loom::task::prepare(&root, &task)?;
    let task: Value = serde_json::from_slice(&fs::read(task.join("authority/task.json"))?)?;
    assert_eq!(task["state_input"], "checkpoint.txt");
    let config = root.join("native-spool.toml");
    let text = fs::read_to_string(&config)?;
    for slot in ["input.txt", "main.sh", "transform.sh", "../state", "tmp"] {
        fs::write(
            &config,
            text.replace(
                "state_input=\"checkpoint.txt\"",
                &format!("state_input=\"{slot}\""),
            ),
        )?;
        assert!(an_loom::package::pack_project(&root).is_err());
    }
    Ok(())
}
