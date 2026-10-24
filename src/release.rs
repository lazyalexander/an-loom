//! Portable, exclusive local release delivery; registration remains a host request.
use crate::{Result, build_artifacts::read, fail, manifest::sha256_hex};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReleaseRecord {
    pub schema: String,
    pub package: crate::manifest::PackageIdentity,
    pub bundle_sha256: String,
    pub archive_sha256: String,
    pub program_sha256: String,
    pub declaration_sha256: String,
    pub source_sha256: String,
    pub profile: String,
}

pub fn create(project: &Path, destination: &Path) -> Result<ReleaseRecord> {
    let archive = crate::package::pack_project(project)?;
    let bytes = read(&archive, 101 * 1024 * 1024)?;
    let verified = crate::read_archive(&archive)?;
    let (project, yaml) = crate::spool::validate(&verified)?;
    let kit = project.kit()?;
    let program = verified
        .files
        .get(&kit.program)
        .ok_or_else(|| fail("missing program"))?;
    let source = verified
        .files
        .get(&kit.source)
        .ok_or_else(|| fail("missing source"))?;
    let record = ReleaseRecord {
        schema: "an-loom-release-v1".into(),
        package: verified.manifest.package,
        bundle_sha256: verified.manifest.release_digest,
        archive_sha256: sha256_hex(&bytes),
        program_sha256: sha256_hex(program),
        declaration_sha256: sha256_hex(yaml.as_bytes()),
        source_sha256: sha256_hex(source),
        profile: kit.profile.clone(),
    };
    crate::package::new_directory(destination)?;
    crate::package::write_new(&destination.join("spool.tar.gz"), &bytes)?;
    let deployment = destination.join("deployment");
    crate::package::new_directory(&deployment)?;
    crate::package::write_new(&deployment.join("spool.yaml"), yaml.as_bytes())?;
    crate::package::write_new(&deployment.join("program"), program)?;
    // The record is written last. Partial directories are never implicitly resumed.
    crate::package::write_new(
        &destination.join("release.json"),
        &serde_json::to_vec_pretty(&record)?,
    )?;
    verify(destination)
}

pub fn verify(root: &Path) -> Result<ReleaseRecord> {
    let bytes = read(&root.join("spool.tar.gz"), 101 * 1024 * 1024)?;
    let record: ReleaseRecord = serde_json::from_slice(&read(&root.join("release.json"), 16384)?)?;
    let archive = crate::read_archive(&root.join("spool.tar.gz"))?;
    let (project, yaml) = crate::spool::validate(&archive)?;
    let kit = project.kit()?;
    let program = read(&root.join("deployment/program"), 20 * 1024 * 1024)?;
    if record.schema != "an-loom-release-v1"
        || record.package != archive.manifest.package
        || record.bundle_sha256 != archive.manifest.release_digest
        || record.archive_sha256 != sha256_hex(&bytes)
        || record.program_sha256 != sha256_hex(&program)
        || record.declaration_sha256 != sha256_hex(yaml.as_bytes())
        || record.source_sha256
            != sha256_hex(
                archive
                    .files
                    .get(&kit.source)
                    .ok_or_else(|| fail("missing source"))?,
            )
        || record.profile != kit.profile
        || archive.files.get(&kit.program) != Some(&program)
        || read(&root.join("deployment/spool.yaml"), 65536)? != yaml.as_bytes()
    {
        return Err(fail(
            "release identity, deployment or source evidence mismatch",
        ));
    }
    for (path, expected) in [
        (root, vec!["deployment", "release.json", "spool.tar.gz"]),
        (
            root.join("deployment").as_path(),
            vec!["program", "spool.yaml"],
        ),
    ] {
        let mut actual = fs::read_dir(path)?
            .map(|entry| {
                entry?
                    .file_name()
                    .into_string()
                    .map_err(|_| fail("release paths must be UTF-8"))
            })
            .collect::<Result<Vec<_>>>()?;
        actual.sort();
        if actual != expected {
            return Err(fail("unregistered material in release delivery"));
        }
    }
    Ok(record)
}
