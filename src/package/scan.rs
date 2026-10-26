use super::*;
pub(crate) fn inspect_project(project_path: &Path) -> Result<InspectedProject> {
    let root = fs::canonicalize(project_path)?;
    let config_path = root.join(MANIFEST_FILE);
    let metadata = fs::symlink_metadata(&config_path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(fail(format!(
            "missing regular {MANIFEST_FILE} in {}",
            project_path.display()
        )));
    }
    let project = crate::project::Project::read(&root)?;
    let manifest = project.manifest();
    let mut paths = BTreeSet::new();
    for include in &manifest.include {
        collect_selected(&root, include, &mut paths)?;
    }
    if paths.is_empty() {
        return Err(fail("include paths resolve to no regular files"));
    }
    if paths.len() > MAX_PACKAGE_FILES {
        return Err(fail(format!(
            "package exceeds the {MAX_PACKAGE_FILES}-file limit"
        )));
    }
    // Local publication hygiene, not an authenticated provenance/safety decision.
    let local_home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|path| path.to_string_lossy().into_owned())
        .filter(|path| path.len() > 1);
    let mut total_bytes = 0usize;
    let mut files = Vec::with_capacity(paths.len());
    for relative in paths {
        let full_path = root.join(Path::new(&relative));
        let metadata = fs::symlink_metadata(&full_path)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(fail(format!(
                "selected input changed or became a symlink: {relative}"
            )));
        }
        let bytes = crate::files::read(&full_path, MAX_FILE_BYTES)?;
        if bytes.len() > MAX_FILE_BYTES {
            return Err(fail(format!("file exceeds the 20 MiB limit: {relative}")));
        }
        total_bytes = total_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| fail("package size overflow"))?;
        if total_bytes > MAX_PACKAGE_BYTES {
            return Err(fail("package exceeds the 100 MiB total payload limit"));
        }
        if local_home.as_ref().is_some_and(|home| {
            bytes
                .windows(home.len())
                .any(|part| part == home.as_bytes())
        }) {
            return Err(fail(format!(
                "local home path found in {relative}; inspect the input or trusted build recipe before packaging"
            )));
        }
        if contains_secret(&bytes) {
            return Err(fail(format!(
                "possible secret material found in {relative}; remove it before packaging"
            )));
        }
        files.push(SourceFile {
            inventory: PayloadFile {
                path: relative,
                size: bytes.len(),
                mode: if is_executable(&metadata) {
                    0o755
                } else {
                    0o644
                },
                sha256: sha256_hex(&bytes),
            },
            bytes,
        });
    }
    crate::adapters::host::file_profile::check_inputs(&manifest.package_identity(), &files, false)?;
    Ok(InspectedProject { manifest, files })
}

fn collect_selected(root: &Path, include: &str, selected: &mut BTreeSet<String>) -> Result<()> {
    validate_payload_path(include)?;
    // Check every ancestor too: an explicit include can traverse a symlink.
    let mut ancestor = root.to_path_buf();
    for component in Path::new(include).components() {
        ancestor.push(component);
        if fs::symlink_metadata(&ancestor)?.file_type().is_symlink() {
            return Err(fail("symlink ancestors are not allowed in package inputs"));
        }
    }
    let full_path = root.join(Path::new(include));
    let metadata = fs::symlink_metadata(&full_path)
        .map_err(|_| fail(format!("included path does not exist: {include}")))?;
    if metadata.file_type().is_symlink() {
        return Err(fail(format!(
            "symlinks are not allowed in package inputs: {include}"
        )));
    }
    if metadata.is_file() {
        selected.insert(include.to_owned());
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(fail(format!(
            "included path is not a regular file or directory: {include}"
        )));
    }
    collect_directory(&full_path, include, selected)
}

fn collect_directory(
    directory: &Path,
    relative: &str,
    selected: &mut BTreeSet<String>,
) -> Result<()> {
    let mut children = fs::read_dir(directory)?.collect::<std::io::Result<Vec<_>>>()?;
    children.sort_by_key(|entry| entry.file_name());
    for entry in children {
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| fail("package paths must be valid UTF-8"))?;
        let child_relative = format!("{relative}/{name}");
        validate_payload_path(&child_relative)?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            return Err(fail(format!(
                "symlinks are not allowed in package inputs: {child_relative}"
            )));
        }
        if metadata.is_dir() {
            collect_directory(&entry.path(), &child_relative, selected)?;
        } else if metadata.is_file() {
            selected.insert(child_relative);
            if selected.len() > MAX_PACKAGE_FILES {
                return Err(fail(format!(
                    "package exceeds the {MAX_PACKAGE_FILES}-file limit"
                )));
            }
        } else {
            return Err(fail(format!(
                "unsupported filesystem entry: {child_relative}"
            )));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &fs::Metadata) -> bool {
    false
}
