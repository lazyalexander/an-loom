//! The first host deployment adapter. Domain algorithms remain in project code.
mod analysis;
use crate::{
    Result, VerifiedArchive, fail,
    manifest::PackageIdentity,
    project::{FILE, Project},
};
pub use analysis::{Analysis, analyze};
use std::{collections::BTreeMap, path::Path};

fn required<'a>(files: &'a BTreeMap<String, Vec<u8>>, path: &str) -> Result<&'a [u8]> {
    files
        .get(path)
        .map(Vec::as_slice)
        .ok_or_else(|| fail(format!("missing selected file: {path}")))
}

fn project(files: &BTreeMap<String, Vec<u8>>, package: &PackageIdentity) -> Result<Project> {
    let project = Project::parse(
        std::str::from_utf8(required(files, FILE)?).map_err(|_| fail("project must be UTF-8"))?,
    )?;
    if &project.manifest().package_identity() != package {
        return Err(fail("project identity differs from the bundle"));
    }
    Ok(project)
}

/// Pure, profile-bound emission. The host's parser remains authoritative.
fn generated(project: &Project, files: &BTreeMap<String, Vec<u8>>) -> Result<(String, Vec<u8>)> {
    let kit = project.silk()?;
    let source = required(files, &kit.source)?;
    let report = analyze(source)?;
    let program = required(files, &kit.program)?;
    crate::build_artifacts::validate(required(files, &kit.build_record)?, source, program)?;
    let yaml = format!(
        "v: 1\nkind: spool\nname: {}\nversion: {}\nsummary: CLI file kit\nsort: beat\nconstructor: host\nhost: an-agent-file-v1\neffect:\n  file: {{ op: rw, path: /work, recursive: true }}\n  memory: {{ op: ignore }}\n  net: none\n  proc: spawn\n  model: {{ op: none }}\n  flow: both\ninverse: irreversible\nrequires: []\nconsumes: [kit.request]\nproduces: [kit.execute]\nconfig:\n  profile: file-kit-v1\n  executable_sha256: {}\n  source_sha256: {}\n",
        serde_json::to_string(&project.name)?,
        serde_json::to_string(&project.version)?,
        serde_json::to_string(&crate::manifest::sha256_hex(program))?,
        serde_json::to_string(&report.source_sha256)?,
    );
    let mut report = serde_json::to_vec_pretty(&report)?;
    report.push(b'\n');
    Ok((yaml, report))
}

pub(crate) fn check_inputs(
    package: &PackageIdentity,
    files: &[crate::package::SourceFile],
    complete: bool,
) -> Result<()> {
    let files = files
        .iter()
        .map(|file| (file.inventory.path.clone(), file.bytes.clone()))
        .collect();
    let project = project(&files, package)?;
    if let Some(kit) = &project.kit {
        analyze(required(&files, &kit.source)?)?;
        if complete || files.contains_key(&kit.declaration) || files.contains_key(&kit.analysis) {
            let (yaml, report) = generated(&project, &files)?;
            if required(&files, &kit.declaration)? != yaml.as_bytes()
                || required(&files, &kit.analysis)? != report
            {
                return Err(fail("stale or missing spool declaration/analysis"));
            }
        }
    }
    Ok(())
}

pub fn validate(archive: &VerifiedArchive) -> Result<(Project, String)> {
    let project = project(&archive.files, &archive.manifest.package)?;
    let kit = project.silk()?;
    let (yaml, report) = generated(&project, &archive.files)?;
    if required(&archive.files, &kit.declaration)? != yaml.as_bytes()
        || required(&archive.files, &kit.analysis)? != report
    {
        return Err(fail("stale or modified declaration/analysis"));
    }
    Ok((project, yaml))
}

pub fn declare(root: &Path) -> Result<()> {
    let inspected = crate::package::inspect_project(root)?;
    let files = inspected
        .files
        .into_iter()
        .map(|file| (file.inventory.path, file.bytes))
        .collect();
    let project = project(&files, &inspected.manifest.package_identity())?;
    let kit = project.silk()?;
    let (yaml, report) = generated(&project, &files)?;
    for (path, bytes) in [
        (&kit.declaration, yaml.as_bytes()),
        (&kit.analysis, report.as_slice()),
    ] {
        let path = root.join(path);
        if path.try_exists()? {
            if crate::files::read(&path, 65536)? != bytes {
                return Err(fail("refusing to overwrite stale generated evidence"));
            }
        } else {
            crate::package::write_new(&path, bytes)?;
        }
    }
    Ok(())
}

/// Only the fixed host deployment files cross this boundary. No registration occurs.
pub fn export(archive: &Path, destination: &Path) -> Result<()> {
    let archive = crate::read_archive(archive)?;
    let (project, yaml) = validate(&archive)?;
    crate::package::new_directory(destination)?;
    crate::package::write_new(&destination.join("spool.yaml"), yaml.as_bytes())?;
    crate::package::write_new(
        &destination.join("program"),
        required(&archive.files, &project.silk()?.program)?,
    )?;
    Ok(())
}
