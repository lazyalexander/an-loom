//! Receive a fixed host build handoff. No code execution or runtime dependency.
use crate::{LoomError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

fn fail(message: &str) -> LoomError {
    LoomError::Message(message.into())
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The portable record is a closed wire contract, not a bag of host metadata.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuildManifest {
    schema: String,
    source_sha256: String,
    program_sha256: String,
    tests: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    compiler: Option<String>,
}

// Pure validation shared by reception, declaration, package build and spool export.
// Identified bytes remain untrusted; this is not a compiler attestation.
pub(crate) fn validate(bytes: &[u8], source: &[u8], program: &[u8]) -> Result<BuildManifest> {
    let manifest: BuildManifest = serde_json::from_slice(bytes)?;
    if manifest.schema != "an-rust-build-artifacts-v1"
        || manifest.tests != "passed"
        || manifest.source_sha256 != hash(source)
        || manifest.program_sha256 != hash(program)
        || program.is_empty()
        || manifest
            .compiler
            .as_deref()
            .is_some_and(|value| value != "operator-selected; not a reproducible-build attestation")
    {
        return Err(fail("stale or invalid accepted build evidence"));
    }
    Ok(manifest)
}

pub(crate) fn read(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut cursor = PathBuf::new();
    for part in path.components() {
        if part == Component::ParentDir {
            return Err(fail("parent traversal in build artifact"));
        }
        cursor.push(part);
        if fs::symlink_metadata(&cursor)?.file_type().is_symlink() {
            return Err(fail("symlink in build artifact"));
        }
    }
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        return Err(fail("expected regular build artifact"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err(fail("hard-linked build artifact"));
        }
    }
    let mut bytes = vec![];
    fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(fail("build artifact exceeds limit"));
    }
    Ok(bytes)
}

pub fn receive(project: &Path, job: &Path) -> Result<()> {
    // The operator supplies a trusted host job directory. Hashes are not signatures.
    let project_contract = crate::project::Project::read(project)?;
    let kit = project_contract.kit()?;
    let receipt: Value = serde_json::from_slice(&read(&job.join("receipt.json"), 65536)?)?;
    let bytes = read(&job.join("accepted/manifest.json"), 8192)?;
    let manifest: Value = serde_json::from_slice(&bytes)?;
    let source = read(&job.join("accepted/main.rs"), 65536)?;
    let program = read(&job.join("accepted/program"), 20 * 1024 * 1024)?;
    let portable = validate(&bytes, &source, &program)?;
    if receipt["schema"] != "an-rust-build-receipt-v1"
        || receipt["status"] != "succeeded"
        || receipt["result"]["status"] != "succeeded"
        || receipt["source_sha256"] != manifest["source_sha256"]
        || receipt["result"]["artifacts"] != manifest
        || read(&project.join(&kit.source), 65536)? != source
    {
        return Err(fail("build evidence or project source mismatch"));
    }
    let _ = crate::package::inspect_project(project)?;
    for name in [&kit.program, &kit.build_record] {
        match fs::symlink_metadata(project.join(name)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            _ => return Err(fail("build destination already exists or is inaccessible")),
        }
    }
    let mut options = fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o755);
    }
    let mut file = options.open(project.join(&kit.program))?;
    file.write_all(&program)?;
    file.sync_all()?;
    // The public package gets only the portable handoff, never local toolchain paths.
    let mut record = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(project.join(&kit.build_record))?;
    record.write_all(&serde_json::to_vec_pretty(&portable)?)?;
    record.sync_all()?;
    Ok(())
}
