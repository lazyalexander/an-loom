use an_loom::project::{FILE, Project};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

pub(super) type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(super) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

// Synthetic bytes qualify contract checking only; this fixture is never executed.
pub(super) fn fixture(root: &Path) -> Result {
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
