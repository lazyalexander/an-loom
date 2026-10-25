use super::support::{Result, fixture};
use an_loom::project::{FILE, Project};
use std::fs;

#[test]
fn nondefault_bindings_release_portably_and_refuse_overwrites() -> Result {
    let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let root = temp.path().join("project");
    fixture(&root)?;
    let release = temp.path().join("release");
    let first = an_loom::release::create(&root, &release)?;
    assert!(an_loom::release::create(&root, &release).is_err());
    let relocated = temp.path().join("relocated");
    fs::rename(&release, &relocated)?;
    assert_eq!(an_loom::release::verify(&relocated)?, first);
    let second = temp.path().join("second");
    assert_eq!(an_loom::release::create(&root, &second)?, first);
    fs::write(relocated.join("deployment/program"), "tampered")?;
    assert!(an_loom::release::verify(&relocated).is_err());
    fs::write(second.join("private.local.json"), "not distributable")?;
    assert!(an_loom::release::verify(&second).is_err());
    Ok(())
}

#[test]
fn changed_source_and_private_config_block_release() -> Result {
    let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let root = temp.path().join("project");
    fixture(&root)?;
    fs::write(root.join("kit/normalize.rs"), "fn main() {}")?;
    assert!(an_loom::release::create(&root, &temp.path().join("release")).is_err());
    assert!(!temp.path().join("release").exists());
    let mut project = Project::read(&root)?;
    project.include.push("toolchain.local.json".into());
    assert!(project.validate().is_err());
    project.include.pop();
    project.kit.as_mut().ok_or("missing kit")?.source = "../secret.rs".into();
    assert!(project.validate().is_err());
    Ok(())
}

#[test]
fn cached_archive_must_match_the_current_project() -> Result {
    let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let one = temp.path().join("first");
    let two = temp.path().join("second");
    fixture(&one)?;
    fixture(&two)?;
    fs::write(two.join("README.md"), "Another selected document")?;
    let first = an_loom::package::pack_project(&one)?;
    let second = an_loom::package::pack_project(&two)?;
    fs::write(&first, fs::read(second)?)?;
    assert!(an_loom::package::pack_project(&one).is_err());
    Ok(())
}

#[test]
fn archive_verification_checks_gzip_crc_and_appended_content() -> Result {
    use std::io::Write;
    let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let root = temp.path().join("project");
    fixture(&root)?;
    let archive = an_loom::package::pack_project(&root)?;
    let original = fs::read(&archive)?;
    let mut corrupted = original.clone();
    let trailer = corrupted.len() - 1;
    corrupted[trailer] ^= 1;
    fs::write(&archive, corrupted)?;
    assert!(an_loom::read_archive(&archive).is_err());
    let mut extra = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    extra.write_all(b"hidden material after the tar archive")?;
    let mut appended = original;
    appended.extend(extra.finish()?);
    fs::write(&archive, appended)?;
    assert!(an_loom::read_archive(&archive).is_err());
    Ok(())
}

#[test]
fn legacy_project_bytes_survive_release_and_verification() -> Result {
    let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let root = temp.path().join("legacy");
    fixture(&root)?;
    let legacy = fs::read_to_string(root.join(FILE))?.replace("[silk]", "[kit]");
    fs::write(root.join(FILE), &legacy)?;
    let destination = temp.path().join("release");
    let record = an_loom::release::create(&root, &destination)?;
    let archive = an_loom::read_archive(&destination.join("spool.tar.gz"))?;
    assert_eq!(archive.files.get(FILE), Some(&legacy.as_bytes().to_vec()));
    assert_eq!(fs::read(root.join(FILE))?, legacy.as_bytes());
    assert_eq!(an_loom::release::verify(&destination)?, record);
    Ok(())
}
