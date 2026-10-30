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
