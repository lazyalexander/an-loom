use super::*;
use flate2::Compression;
use std::io::Cursor;
use tar::{Archive, Builder, EntryType, Header};
pub fn pack_project(project_path: &Path) -> Result<PathBuf> {
    let build = build_project(project_path)?;
    let artifacts = build
        .output
        .parent()
        .ok_or_else(|| fail("build directory has no parent"))?
        .join("artifacts");
    ensure_directory(&artifacts)?;
    let archive_path = artifacts.join(format!(
        "{}-{}-{}.tar.gz",
        build.manifest.package.name,
        build.manifest.package.version,
        &build.manifest.release_digest[..12]
    ));
    match fs::symlink_metadata(&archive_path) {
        Ok(metadata) => {
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(fail(format!(
                    "archive destination is not a regular file: {}",
                    archive_path.display()
                )));
            }
            if verify_archive(&archive_path)? != build.manifest {
                return Err(fail("cached archive belongs to a different release"));
            }
            return Ok(archive_path);
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let (staging, output_file) = unique_file(&artifacts, ".loom-archive")?;
    let result = create_archive(output_file, &build.output, &build.manifest)
        .and_then(|()| fs::rename(&staging, &archive_path).map_err(LoomError::from));
    if result.is_err() {
        let _ = fs::remove_file(&staging);
    }
    result?;
    verify_archive(&archive_path)?;
    Ok(archive_path)
}

fn create_archive(file: File, build_directory: &Path, manifest: &BundleManifest) -> Result<()> {
    let encoder = flate2::GzBuilder::new()
        .mtime(0)
        .write(file, Compression::default());
    let mut builder = Builder::new(encoder);
    builder.mode(tar::HeaderMode::Deterministic);
    for payload in &manifest.files {
        let path = build_directory.join(Path::new(&payload.path));
        let bytes = crate::files::read(&path, MAX_FILE_BYTES)?;
        if bytes.len() != payload.size || sha256_hex(&bytes) != payload.sha256 {
            return Err(fail(format!(
                "build payload changed before packaging: {}",
                payload.path
            )));
        }
        append_file(&mut builder, &payload.path, &bytes, payload.mode)?;
    }
    let bundle = format!("{}\n", serde_json::to_string_pretty(manifest)?);
    append_file(&mut builder, BUNDLE_FILE, bundle.as_bytes(), 0o644)?;
    builder.finish()?;
    let encoder = builder.into_inner()?;
    let mut file = encoder.finish()?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

fn append_file<W: Write>(
    builder: &mut Builder<W>,
    name: &str,
    bytes: &[u8],
    mode: u32,
) -> Result<()> {
    let mut header = Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(mode);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_entry_type(EntryType::Regular);
    header.set_cksum();
    builder.append_data(&mut header, name, Cursor::new(bytes))?;
    Ok(())
}

pub fn verify_archive(archive_path: &Path) -> Result<BundleManifest> {
    let verified = read_archive(archive_path)?;
    if verified.files.contains_key("native-spool.toml") {
        let selected = verified
            .files
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
            .collect();
        crate::adapters::host::native_spool::validate(&selected)?;
    }
    Ok(verified.manifest)
}

/// Verified payload bytes, never extracted using archive-provided paths.
pub struct VerifiedArchive {
    pub manifest: BundleManifest,
    pub files: BTreeMap<String, Vec<u8>>,
}

pub fn read_archive(archive_path: &Path) -> Result<VerifiedArchive> {
    let file = File::open(archive_path)?;
    read_archive_from(file)
}

pub(crate) fn read_archive_bytes(bytes: &[u8]) -> Result<VerifiedArchive> {
    read_archive_from(Cursor::new(bytes))
}

fn read_archive_from(reader: impl Read) -> Result<VerifiedArchive> {
    let decoder = flate2::read::MultiGzDecoder::new(reader);
    let mut archive = Archive::new(decoder);
    let mut contents = BTreeMap::<String, Vec<u8>>::new();
    let mut expanded_bytes = 0usize;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let raw_path = entry.path_bytes();
        let name = std::str::from_utf8(&raw_path)
            .map_err(|_| fail("archive paths must be UTF-8"))?
            .to_owned();
        if name.ends_with('/') || name.is_empty() {
            return Err(fail(format!(
                "directory or empty archive entry is not allowed: {name:?}"
            )));
        }
        validate_archive_path(&name)?;
        if !entry.header().entry_type().is_file() {
            return Err(fail(format!("unsupported archive entry type: {name}")));
        }
        let declared_size = usize::try_from(entry.header().size()?)
            .map_err(|_| fail("archive entry size is too large"))?;
        let entry_limit = if name == BUNDLE_FILE {
            MAX_BUNDLE_BYTES
        } else {
            MAX_FILE_BYTES
        };
        if declared_size > entry_limit {
            return Err(fail(format!(
                "archive entry exceeds its size limit: {name}"
            )));
        }
        expanded_bytes = expanded_bytes
            .checked_add(declared_size)
            .ok_or_else(|| fail("archive size overflow"))?;
        if expanded_bytes > MAX_PACKAGE_BYTES + MAX_BUNDLE_BYTES {
            return Err(fail("archive exceeds the 101 MiB expanded size limit"));
        }
        let mut bytes = Vec::with_capacity(declared_size);
        entry.read_to_end(&mut bytes)?;
        if bytes.len() != declared_size {
            return Err(fail(format!(
                "archive entry size does not match its header: {name}"
            )));
        }
        if contents.insert(name.clone(), bytes).is_some() {
            return Err(fail(format!("duplicate archive entry: {name}")));
        }
        if contents.len() > MAX_PACKAGE_FILES + 1 {
            return Err(fail("archive exceeds the 1,000-payload-file limit"));
        }
    }
    // tar stops at its end marker; gzip integrity is only checked when drained.
    // Bound trailing padding and reject hidden concatenated archive payloads.
    let mut remainder = Vec::new();
    archive
        .into_inner()
        .take(MAX_BUNDLE_BYTES as u64 + 1)
        .read_to_end(&mut remainder)?;
    if remainder.len() > MAX_BUNDLE_BYTES || remainder.iter().any(|byte| *byte != 0) {
        return Err(fail("unexpected data after the tar end marker"));
    }
    let bundle_bytes = contents
        .remove(BUNDLE_FILE)
        .ok_or_else(|| fail("archive is missing bundle.json"))?;
    let manifest: BundleManifest = serde_json::from_slice(&bundle_bytes)?;
    manifest.validate()?;
    if contents.len() != manifest.files.len() {
        return Err(fail("archive entries do not match bundle.json"));
    }
    for payload in &manifest.files {
        let bytes = contents
            .get(&payload.path)
            .ok_or_else(|| fail(format!("archive is missing payload: {}", payload.path)))?;
        if bytes.len() != payload.size || sha256_hex(bytes) != payload.sha256 {
            return Err(fail(format!(
                "payload hash or size mismatch: {}",
                payload.path
            )));
        }
    }
    Ok(VerifiedArchive {
        manifest,
        files: contents,
    })
}

fn validate_archive_path(value: &str) -> Result<()> {
    if value == BUNDLE_FILE {
        return Ok(());
    }
    validate_payload_path(value)
}
