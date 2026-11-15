//! Explicit offline runtime delivery. No downloads, installers or host execution.
mod manifest;
pub mod stage;
mod verified;
use crate::{Result, fail};
use manifest::Manifest;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub fn pack(source: &Path, selection: &Path, output: &Path) -> Result<serde_json::Value> {
    let bytes = crate::files::read(selection, 2 * 1024 * 1024)?;
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    manifest.validate().map_err(fail)?;
    reject_links(source)?;
    let root = source.canonicalize()?;
    for (name, file) in &manifest.files {
        let path = root.join(name);
        reject_links(&path)?;
        // Canonical containment also rejects a selected symlink to an outside tree.
        if !path.canonicalize()?.starts_with(&root) {
            return Err(fail("runtime source escapes root"));
        }
        let mut input = fs::File::open(&path)?;
        if input.metadata()?.len() != file.bytes {
            return Err(fail("runtime source size mismatch"));
        }
        let mut hash = Sha256::new();
        let mut block = [0u8; 65536];
        loop {
            let count = input.read(&mut block)?;
            if count == 0 {
                break;
            }
            hash.update(&block[..count]);
        }
        if format!("{:x}", hash.finalize()) != file.sha256 {
            return Err(fail("runtime source hash mismatch"));
        }
    }
    validate_image_component(&root, &manifest)?;
    let target = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)?;
    let result = (|| -> Result<()> {
        let gzip = flate2::GzBuilder::new()
            .mtime(0)
            .write(target, flate2::Compression::default());
        let mut archive = tar::Builder::new(gzip);
        let data = serde_json::to_vec(&manifest)?;
        append(&mut archive, "runtime.json", data.len() as u64, &data[..])?;
        for (name, file) in &manifest.files {
            verified::append(&mut archive, name, file, fs::File::open(root.join(name))?)?;
        }
        archive.finish()?;
        let mut file = archive.into_inner()?.finish()?;
        file.flush()?;
        file.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(output);
    }
    result?;
    Ok(serde_json::json!({"archive":output,"manifest":manifest,"execution_authorized":false}))
}
/// A declaration pins a finite host API; it cannot qualify VM execution or memory restore.
fn validate_image_component(root: &Path, manifest: &Manifest) -> Result<()> {
    let Some(path) = manifest.components.get("image_api") else {
        return Ok(());
    };
    for name in ["cocoon", "runner", "kernel", "initrd", "root_image"] {
        if !manifest.components.contains_key(name) {
            return Err(fail(
                "disk images require pinned Cocoon, runner and boot components",
            ));
        }
    }
    if manifest.files[path].executable {
        return Err(fail("image API declaration must be data"));
    }
    let value: serde_json::Value =
        serde_json::from_slice(&crate::files::read(&root.join(path), 4096)?)?;
    if value
        != serde_json::json!({"schema":"cocoon-image-api-v1","disk_images":true,"memory_restore_qualified":false})
    {
        return Err(fail("unsupported image API declaration"));
    }
    Ok(())
}

fn append<W: Write>(
    archive: &mut tar::Builder<W>,
    name: &str,
    size: u64,
    input: impl Read,
) -> Result<()> {
    let mut header = tar::Header::new_ustar();
    header.set_path(name)?;
    header.set_size(size);
    header.set_mode(0o400);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_cksum();
    archive.append(&header, input)?;
    Ok(())
}

fn reject_links(path: &Path) -> Result<()> {
    let mut cursor = std::path::PathBuf::new();
    for part in path.components() {
        if matches!(part, std::path::Component::ParentDir) {
            return Err(fail("runtime parent traversal"));
        }
        cursor.push(part);
        if fs::symlink_metadata(&cursor)?.file_type().is_symlink() {
            return Err(fail("runtime symlink refused"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
