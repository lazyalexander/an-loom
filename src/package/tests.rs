use super::*;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(label: &str) -> Result<PathBuf> {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| fail(error.to_string()))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("an-loom-{label}-{}-{time}", std::process::id()));
    fs::create_dir(&path)?;
    Ok(path)
}

fn fixture(root: &Path) -> Result<()> {
    fs::create_dir(root.join("kit"))?;
    write_new(&root.join("README.md"), b"A generic component.\n")?;
    write_new(
        &root.join("kit/contract.json"),
        b"{\"inputs\":[],\"outputs\":[]}\n",
    )?;
    let manifest = Manifest {
        schema_version: 1,
        name: "sample-kit".to_owned(),
        version: "0.1.0".to_owned(),
        description: None,
        include: vec![
            "README.md".to_owned(),
            "kit".to_owned(),
            "loom.toml".to_owned(),
        ],
    };
    write_new(
        &root.join(MANIFEST_FILE),
        toml::to_string(&crate::project::Project::from_manifest(manifest))
            .map_err(|e| fail(e.to_string()))?
            .as_bytes(),
    )
}

#[test]
fn complete_scaffold_build_pack_verify_loop() -> Result<()> {
    let root = temp_dir("loop")?;
    let project = root.join("hello-kit");
    crate::project::scaffold(&project, false)?;
    let checked = inspect_project(&project)?;
    assert_eq!(checked.files.len(), 3);
    let build = build_project(&project)?;
    let archive = pack_project(&project)?;
    let verified = verify_archive(&archive)?;
    assert_eq!(verified.release_digest, build.manifest.release_digest);
    assert_eq!(verified.package.name, "hello-kit");
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rejects_symlinks_and_common_secret_files() -> Result<()> {
    let root = temp_dir("unsafe")?;
    fixture(&root)?;
    fs::write(root.join("kit/.env.local"), b"TOKEN=placeholder")?;
    assert!(inspect_project(&root).is_err());
    fs::remove_file(root.join("kit/.env.local"))?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("README.md"), root.join("kit/link"))?;
    #[cfg(unix)]
    assert!(inspect_project(&root).is_err());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
#[cfg(unix)]
fn rejects_explicit_include_through_symlink_ancestor() -> Result<()> {
    let root = temp_dir("ancestor")?;
    fixture(&root)?;
    std::os::unix::fs::symlink(root.join("kit"), root.join("linked"))?;
    let mut manifest =
        crate::project::Project::parse(&fs::read_to_string(root.join(MANIFEST_FILE))?)?;
    manifest.include = vec!["loom.toml".into(), "linked/contract.json".into()];
    fs::write(
        root.join(MANIFEST_FILE),
        toml::to_string(&manifest).map_err(|e| fail(e.to_string()))?,
    )?;
    assert!(inspect_project(&root).is_err());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn identical_projects_have_identical_archives() -> Result<()> {
    let root = temp_dir("deterministic")?;
    let first = root.join("one");
    let second = root.join("two");
    fs::create_dir(&first)?;
    fs::create_dir(&second)?;
    fixture(&first)?;
    fixture(&second)?;
    let first_archive = pack_project(&first)?;
    let second_archive = pack_project(&second)?;
    assert_eq!(
        Sha256::digest(fs::read(first_archive)?),
        Sha256::digest(fs::read(second_archive)?)
    );
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn refuses_corrupted_payload_on_archive_verification() -> Result<()> {
    let root = temp_dir("corruption")?;
    let project = root.join("sample-kit");
    fs::create_dir(&project)?;
    fixture(&project)?;
    let archive = pack_project(&project)?;
    let mut bytes = fs::read(&archive)?;
    let last = bytes.len() - 12;
    bytes[last] ^= 0x7f;
    fs::write(&archive, bytes)?;
    assert!(verify_archive(&archive).is_err());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn refuses_current_home_paths_in_binary_payloads() -> Result<()> {
    let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) else {
        return Ok(());
    };
    let home = home.to_string_lossy();
    if home.len() <= 1 {
        return Ok(());
    }
    let root = temp_dir("private-binary")?;
    fixture(&root)?;
    let mut bytes = b"binary\0debug-path=".to_vec();
    bytes.extend(home.as_bytes());
    fs::write(root.join("kit/program"), bytes)?;
    assert!(inspect_project(&root).is_err());
    fs::remove_dir_all(root)?;
    Ok(())
}
