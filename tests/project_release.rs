use an_loom::{
    plan::{DevelopmentPlan, HostCall},
    project::{FILE, Project},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

// Synthetic bytes qualify contract checking only; this fixture is never executed.
fn fixture(root: &Path) -> Result {
    an_loom::project::scaffold(root, true)?;
    let mut project = Project::read(root)?;
    let kit = project.kit.as_mut().ok_or("missing binding")?;
    kit.source = "kit/normalize.rs".into();
    kit.program = "kit/normalize".into();
    kit.build_record = "kit/accepted.json".into();
    kit.declaration = "kit/contract.yaml".into();
    kit.analysis = "kit/evidence.json".into();
    fs::rename(root.join("kit/main.rs"), root.join(&kit.source))?;
    let source = fs::read(root.join(&kit.source))?;
    let program = b"synthetic reception fixture, not executable";
    fs::write(root.join(&kit.program), program)?;
    fs::write(
        root.join(&kit.build_record),
        serde_json::to_vec(&json!({
        "schema":"an-rust-build-artifacts-v1","source_sha256":hash(&source),
        "program_sha256":hash(program),"tests":"passed"}))?,
    )?;
    fs::write(root.join(FILE), toml::to_string(&project)?)?;
    an_loom::spool::declare(root)?;
    Ok(())
}

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
fn approvals_bind_the_project_and_refuse_unsupported_execution() -> Result {
    let source = b"fn main() {}";
    let tools = b"{}";
    let preview = json!({"approval_sha256":"a".repeat(64),"plan":{
        "schema":"an-rust-build-plan-v1","source_sha256":hash(source),
        "toolchain_profile_sha256":hash(tools),"supported":true}});
    let command = HostCall {
        executable: "host".into(),
        args: vec!["build-kit".into()],
        cwd: "project".into(),
        environment: BTreeMap::new(),
    };
    let plan = DevelopmentPlan::assemble(
        b"project-one",
        source,
        tools,
        "b".repeat(64),
        preview.clone(),
        command.clone(),
    )?;
    let digest = plan.digest()?;
    plan.authorize(&digest)?;
    assert!(plan.authorize(&"0".repeat(64)).is_err());
    let changed = DevelopmentPlan::assemble(
        b"project-two",
        source,
        tools,
        "b".repeat(64),
        preview.clone(),
        command.clone(),
    )?;
    assert!(changed.authorize(&digest).is_err());
    assert!(
        DevelopmentPlan::assemble(
            b"project-one",
            b"changed source",
            tools,
            "b".repeat(64),
            preview.clone(),
            command.clone()
        )
        .is_err()
    );
    let mut unsupported = preview;
    unsupported["plan"]["supported"] = json!(false);
    let plan = DevelopmentPlan::assemble(
        b"project-one",
        source,
        tools,
        "b".repeat(64),
        unsupported,
        command,
    )?;
    assert!(plan.authorize(&plan.digest()?).is_err());
    Ok(())
}

#[test]
fn source_signals_reject_known_incompatible_operations_without_claiming_completeness() -> Result {
    let evidence = an_loom::spool::analyze(include_bytes!("../examples/file-list/kit/main.rs"))?;
    assert!(!evidence.unknowns.is_empty());
    for source in [
        "fn main() { std::net::TcpStream::connect(\"server\"); }",
        "fn main() { unsafe {} }",
        "fn main( {",
    ] {
        assert!(an_loom::spool::analyze(source.as_bytes()).is_err());
    }
    Ok(())
}

#[test]
fn silk_authoring_preserves_legacy_binding_checks_and_rejects_ambiguity() -> Result {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("spool");
    an_loom::project::scaffold(&root, true)?;
    let canonical = fs::read_to_string(root.join(FILE))?;
    assert!(canonical.contains("[silk]"));
    assert!(!canonical.contains("[kit]"));
    let legacy = canonical.replace("[silk]", "[kit]");
    let current = Project::parse(&canonical)?;
    let old = Project::parse(&legacy)?;
    assert_eq!(current, old);
    assert_eq!(current.silk()?, old.kit()?);
    let duplicate = format!(
        "{canonical}\n[kit]\n{}",
        legacy.split("[kit]").nth(1).ok_or("binding")?
    );
    assert!(Project::parse(&duplicate).is_err());
    for text in [&canonical, &legacy] {
        let escaped = text.replace("kit/main.rs", "../outside.rs");
        assert!(Project::parse(&escaped).is_err());
        let wrong_profile = text.replace("file-kit-v1", "unrestricted");
        assert!(Project::parse(&wrong_profile).is_err());
    }
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
