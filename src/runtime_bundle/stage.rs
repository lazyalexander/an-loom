//! Assemble a reviewed offline payload from explicit local sources, never download or run it.
use super::*;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    io::{Seek, SeekFrom},
    path::PathBuf,
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    name: String,
    version: String,
    platform: String,
    licenses: Vec<String>,
    components: BTreeMap<String, String>,
    sources: BTreeMap<String, PathBuf>,
}
pub fn stage(selection: &Path, destination: &Path) -> Result<serde_json::Value> {
    let value: Selection =
        serde_json::from_slice(&crate::files::read(selection, 2 * 1024 * 1024)?)?;
    if destination.exists() {
        return Err(fail("runtime stage requires a new destination"));
    }
    fs::create_dir(destination)?;
    let result = (|| -> Result<serde_json::Value> {
        let mut manifest = Manifest {
            schema: "an-offline-runtime-v1".into(),
            name: value.name,
            version: value.version,
            platform: value.platform,
            licenses: value.licenses,
            components: value.components,
            files: BTreeMap::new(),
        };
        let mut total = 0u64;
        for (target, source) in value.sources {
            if !manifest::safe_path(&target) {
                return Err(fail("invalid runtime destination"));
            }
            let source = source.canonicalize()?;
            let containment = if source.is_dir() {
                source.clone()
            } else {
                source
                    .parent()
                    .ok_or_else(|| fail("source parent"))?
                    .to_path_buf()
            };
            copy(
                &source,
                &containment,
                destination,
                &target,
                &mut manifest,
                &mut total,
                0,
            )?;
        }
        manifest.validate().map_err(fail)?;
        let path = destination.join("runtime.json");
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        file.write_all(&serde_json::to_vec(&manifest)?)?;
        file.sync_all()?;
        Ok(
            serde_json::json!({"directory":destination,"manifest":path,"files":manifest.files.len(),"execution_authorized":false}),
        )
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(destination);
    }
    result
}
fn copy(
    source: &Path,
    containment: &Path,
    destination: &Path,
    target: &str,
    manifest: &mut Manifest,
    total: &mut u64,
    depth: usize,
) -> Result<()> {
    if depth > 24 || !manifest::safe_path(target) {
        return Err(fail("runtime tree depth or path limit"));
    }
    let canonical = source.canonicalize()?;
    if !canonical.starts_with(containment) {
        return Err(fail("runtime source link escapes selected tree"));
    }
    if fs::symlink_metadata(source)?.file_type().is_symlink() && canonical.is_dir() {
        return Err(fail(
            "runtime directory links require explicit source selection",
        ));
    }
    if canonical.is_dir() {
        let mut entries = fs::read_dir(&canonical)?.collect::<std::result::Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| fail("runtime path must be UTF-8"))?;
            copy(
                &entry.path(),
                containment,
                destination,
                &format!("{target}/{name}"),
                manifest,
                total,
                depth + 1,
            )?;
        }
        return Ok(());
    }
    if !canonical.is_file() || manifest.files.contains_key(target) || manifest.files.len() >= 16384
    {
        return Err(fail("runtime file type, duplicate or count limit"));
    }
    let size = fs::metadata(&canonical)?.len();
    *total = total
        .checked_add(size)
        .ok_or_else(|| fail("runtime byte overflow"))?;
    if *total > 32 * 1024 * 1024 * 1024 {
        return Err(fail("runtime expanded size limit"));
    }
    let path = destination.join(target);
    fs::create_dir_all(path.parent().ok_or_else(|| fail("runtime parent"))?)?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let mut input = fs::File::open(&canonical)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    let mut written = 0u64;
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
        written += count as u64;
        if written > size {
            return Err(fail("runtime source grew while staging"));
        }
        if buffer[..count].iter().all(|b| *b == 0) {
            output.seek(SeekFrom::Current(count as i64))?;
        } else {
            output.write_all(&buffer[..count])?;
        }
    }
    if written != size {
        return Err(fail("runtime source changed size"));
    }
    output.set_len(size)?;
    output.sync_all()?;
    #[cfg(unix)]
    let executable = {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(&canonical)?.permissions().mode() & 0o111 != 0
    };
    #[cfg(not(unix))]
    let executable = false;
    manifest.files.insert(
        target.into(),
        manifest::File {
            sha256: format!("{:x}", hash.finalize()),
            bytes: size,
            executable,
        },
    );
    Ok(())
}
