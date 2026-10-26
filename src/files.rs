//! Bounded regular-file reads for authoring and packaging; no host protocol.
use crate::{Result, fail};
use std::{
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

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
