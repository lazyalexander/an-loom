//! Check and publication orchestration; execution belongs to the selected host.
use super::{fresh_directory, host_check, project_path, read_document, snapshot};
use crate::{Result, fail, files::read, manifest::sha256_hex, package};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub fn check(task: &Path) -> Result<Value> {
    let task = fs::canonicalize(task)?;
    if task.join("authority/check-plan.json").exists() {
        return Ok(snapshot::load(&task)?.0);
    }
    snapshot::freeze(&task)
}

/// JOB is an operator-owned host workspace, never agent-provided success JSON.
pub fn receive_check(task: &Path, job: &Path) -> Result<Value> {
    let task = fs::canonicalize(task)?;
    let (plan, _, _) = snapshot::load(&task)?;
    let job = fs::canonicalize(job)?;
    let project = project_path(&read_document(&task)?)?;
    if job.starts_with(&task) || job.starts_with(&project) {
        return Err(fail("host evidence must stay outside the task and project"));
    }
    if task.join("authority/evidence.json").exists() {
        return Err(fail("check evidence already recorded"));
    }
    let evidence = host_check::verify(&task, &plan, &job)?;
    package::write_new(
        &task.join("authority/evidence.json"),
        &serde_json::to_vec_pretty(&evidence)?,
    )?;
    Ok(
        json!({"schema":"an-loom-check-evidence-v2","attempt":plan["attempt"],
        "recorded":true,"model_invoked":false,"host_invoked_by_loom":false,"published":false}),
    )
}

pub fn pack_candidate(task: &Path, destination: &Path) -> Result<Value> {
    let task = fs::canonicalize(task)?;
    let (plan, bytes, archive) = snapshot::load(&task)?;
    let evidence: Value = serde_json::from_slice(
        &read(&task.join("authority/evidence.json"), 1024 * 1024).map_err(|_| {
            fail("passing host check evidence is required before packaging a candidate")
        })?,
    )?;
    let job = Path::new(
        evidence["job"]
            .as_str()
            .ok_or_else(|| fail("host evidence missing"))?,
    );
    // Reconcile current host custody as well as candidate identity, including late mutation.
    if host_check::verify(&task, &plan, job)? != evidence {
        return Err(fail("recorded host evidence changed"));
    }
    let project = project_path(&read_document(&task)?)?;
    let absolute = if destination.is_absolute() {
        destination.to_path_buf()
    } else {
        std::env::current_dir()?.join(destination)
    };
    let parent = absolute
        .parent()
        .ok_or_else(|| fail("destination parent"))?
        .canonicalize()?;
    if parent.starts_with(&task) {
        return Err(fail("package destination must stay outside the task"));
    }
    let destination = fresh_directory(destination, &project)?;
    for file in &archive.manifest.files {
        snapshot::write_payload(
            &destination,
            &file.path,
            archive
                .files
                .get(&file.path)
                .ok_or_else(|| fail("missing payload"))?,
            file.mode,
        )?;
    }
    // Do not re-read the project or reconstructed tree to select release bytes.
    let output = destination.join("checked-release.tar.gz");
    package::write_new(&output, &bytes)?;
    Ok(json!({"archive":output,"archive_sha256":sha256_hex(&bytes),
        "release_digest":archive.manifest.release_digest,"attempt":plan["attempt"],
        "candidate_sha256":plan["candidate"]["sha256"],"model_invoked":false,
        "host_invoked_by_loom":false,"applied_to_source_project":false}))
}
