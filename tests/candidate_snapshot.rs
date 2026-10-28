//! Full candidate identities and legacy authoring compatibility, without execution.
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn project(base: &Path) -> Result<PathBuf> {
    let root = base.join("source");
    an_loom::run(["init-native".into(), root.to_string_lossy().into_owned()])?;
    Ok(root)
}
fn task(base: &Path, root: &Path, name: &str) -> Result<(PathBuf, Value)> {
    let task = base.join(name);
    an_loom::task::prepare(root, &task)?;
    let file = base.join(format!("{name}.sh"));
    fs::write(&file, b"printf candidate\n")?;
    an_loom::task::receive(&task, "kit/transform.sh", &file)?;
    let plan = an_loom::task::check(&task)?;
    Ok((task, plan))
}
#[test]
fn content_identity_and_execution_identity_are_distinct() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let (a, first) = task(&base, &root, "a")?;
    let (_, second) = task(&base, &root, "b")?;
    assert_eq!(first["release"], second["release"]);
    assert_eq!(first["archive_sha256"], second["archive_sha256"]);
    assert_ne!(first["attempt"], second["attempt"]);
    assert_ne!(first["profile_sha256"], second["profile_sha256"]);
    assert_eq!(an_loom::task::check(&a)?, first);
    fs::write(root.join("silk/report.sh"), b"printf changed\n")?;
    assert!(an_loom::task::check(&a).is_err());
    let (_, third) = task(&base, &root, "c")?;
    assert_eq!(first["candidate"], third["candidate"]);
    assert_ne!(first["baseline"], third["baseline"]);
    assert_ne!(first["release"], third["release"]);
    Ok(())
}
#[test]
fn changes_to_frozen_material_and_revocation_refuse_publication() -> Result {
    for path in [
        "authority/candidate.tar.gz",
        "authority/snapshot/silk/report.sh",
        "candidate/kit/transform.sh",
    ] {
        let tmp = tempfile::tempdir()?;
        let base = tmp.path().canonicalize()?;
        let root = project(&base)?;
        let (task, _) = task(&base, &root, "task")?;
        fs::write(task.join(path), b"changed")?;
        assert!(an_loom::task::check(&task).is_err(), "{path}");
        assert!(an_loom::task::pack_candidate(&task, &base.join("pack")).is_err());
        assert!(!base.join("pack").exists());
    }
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let (task, _) = task(&base, &root, "task")?;
    an_loom::task::revoke(&task)?;
    assert!(an_loom::task::check(&task).is_err());
    Ok(())
}
#[test]
fn legacy_v1_still_packages_verifies_exports_but_cannot_invent_task_authority() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let path = root.join("native-spool.toml");
    let mut config: toml::Value = fs::read_to_string(&path)?.parse()?;
    for key in [
        "runtime",
        "editable",
        "inputs",
        "capabilities",
        "dependencies",
        "acceptance",
    ] {
        config.as_table_mut().ok_or("table")?.remove(key);
    }
    fs::write(&path, toml::to_string(&config)?)?;
    let archive = an_loom::package::pack_project(&root)?;
    let verified = an_loom::package::verify_archive(&archive)?;
    an_loom::run([
        "export-native-spool".into(),
        archive.to_string_lossy().into_owned(),
        base.join("release").to_string_lossy().into_owned(),
    ])?;
    assert_eq!(verified.package.name, "native-text-spool");
    assert!(
        an_loom::task::prepare(&root, &base.join("task"))
            .err()
            .ok_or("legacy task unexpectedly accepted")?
            .to_string()
            .contains("legacy")
    );
    // Partial additions and an empty runtime must not silently fall back to legacy.
    config.as_table_mut().ok_or("table")?.insert(
        "runtime".into(),
        toml::Value::String("macos-native-process-v1".into()),
    );
    fs::write(&path, toml::to_string(&config)?)?;
    assert!(an_loom::package::pack_project(&root).is_err());
    Ok(())
}

#[test]
fn historical_archive_is_read_without_rewriting_its_identity() -> Result {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/native-spool-85c832a.tar.gz");
    let before = fs::read(&path)?;
    let manifest = an_loom::package::verify_archive(&path)?;
    assert!(manifest.release_digest.starts_with("631ebec19015"));
    let temp = tempfile::tempdir()?;
    let release = temp.path().canonicalize()?.join("delivery");
    an_loom::run([
        "export-native-spool".into(),
        path.to_string_lossy().into_owned(),
        release.to_string_lossy().into_owned(),
    ])?;
    assert_eq!(before, fs::read(path)?);
    Ok(())
}
