use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::manifest::{
    BundleManifest, MAX_BUNDLE_BYTES, MAX_FILE_BYTES, MAX_PACKAGE_BYTES, MAX_PACKAGE_FILES,
    Manifest, PayloadFile, contains_secret, sha256_hex, validate_payload_path,
};
use crate::{LoomError, Result};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const MANIFEST_FILE: &str = "loom.toml";
const BUNDLE_FILE: &str = "bundle.json";

pub(crate) struct InspectedProject {
    pub manifest: Manifest,
    pub files: Vec<SourceFile>,
}

pub(crate) struct SourceFile {
    pub(crate) inventory: PayloadFile,
    pub(crate) bytes: Vec<u8>,
}

pub(crate) struct BuildResult {
    pub output: PathBuf,
    pub manifest: BundleManifest,
}

fn fail(message: impl Into<String>) -> LoomError {
    LoomError::Message(message.into())
}

mod archive;
mod scan;
mod storage;
#[cfg(test)]
mod tests;

pub use archive::{VerifiedArchive, pack_project, read_archive, verify_archive};
pub(crate) use scan::inspect_project;
pub(crate) use storage::write_new;
use storage::{ensure_directory, set_file_mode, unique_directory, unique_file};
pub(crate) fn build_project(project_path: &Path) -> Result<BuildResult> {
    let inspected = inspect_project(project_path)?;
    crate::adapters::host::file_profile::check_inputs(
        &inspected.manifest.package_identity(),
        &inspected.files,
        true,
    )?;
    let manifest = BundleManifest::new(
        inspected.manifest.package_identity(),
        inspected
            .files
            .iter()
            .map(|file| file.inventory.clone())
            .collect(),
    )?;
    let root = fs::canonicalize(project_path)?;
    let dist = root.join("dist");
    ensure_directory(&dist)?;
    let output = dist.join(format!(
        "{}-{}-{}",
        manifest.package.name,
        manifest.package.version,
        &manifest.release_digest[..12]
    ));
    if fs::symlink_metadata(&output).is_ok() {
        verify_build_directory(&output, &manifest)?;
        return Ok(BuildResult { output, manifest });
    }
    let staging = unique_directory(&dist, ".loom-stage")?;
    let result = write_build_directory(&staging, &inspected.files, &manifest)
        .and_then(|()| fs::rename(&staging, &output).map_err(LoomError::from));
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result?;
    Ok(BuildResult { output, manifest })
}

fn write_build_directory(
    directory: &Path,
    files: &[SourceFile],
    manifest: &BundleManifest,
) -> Result<()> {
    for file in files {
        let destination = directory.join(Path::new(&file.inventory.path));
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        write_new(&destination, &file.bytes)?;
        set_file_mode(&destination, file.inventory.mode)?;
    }
    let bytes = format!("{}\n", serde_json::to_string_pretty(manifest)?);
    write_new(&directory.join(BUNDLE_FILE), bytes.as_bytes())?;
    Ok(())
}

fn verify_build_directory(directory: &Path, expected: &BundleManifest) -> Result<()> {
    let directory_metadata = fs::symlink_metadata(directory)?;
    if directory_metadata.file_type().is_symlink() || !directory_metadata.is_dir() {
        return Err(fail("cached build must be a regular directory"));
    }
    let bundle_path = directory.join(BUNDLE_FILE);
    let metadata = fs::symlink_metadata(&bundle_path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(fail("existing build bundle is not a regular file"));
    }
    let actual: BundleManifest =
        serde_json::from_slice(&crate::files::read(&bundle_path, MAX_BUNDLE_BYTES)?)?;
    actual.validate()?;
    if &actual != expected {
        return Err(fail(format!(
            "build destination exists but does not match this release: {}",
            directory.display()
        )));
    }
    for file in &expected.files {
        let path = directory.join(Path::new(&file.path));
        let metadata = fs::symlink_metadata(&path)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(fail(format!(
                "existing build payload is not a regular file: {}",
                file.path
            )));
        }
        let bytes = crate::files::read(&path, MAX_FILE_BYTES)?;
        if bytes.len() != file.size || sha256_hex(&bytes) != file.sha256 {
            return Err(fail(format!(
                "existing build payload was modified: {}",
                file.path
            )));
        }
    }
    Ok(())
}

/// Exclusive local publication with no symlinked ancestors.
pub(crate) fn new_directory(path: &Path) -> Result<()> {
    let mut cursor = PathBuf::new();
    for part in path.components() {
        if part == std::path::Component::ParentDir {
            return Err(fail("parent traversal in output path"));
        }
        cursor.push(part);
        match fs::symlink_metadata(&cursor) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(fail("symlink in output path"));
            }
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error.into()),
            _ => (),
        }
    }
    fs::create_dir(path)?;
    Ok(())
}
