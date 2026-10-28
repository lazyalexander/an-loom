//! Freeze a complete candidate before any check; publication reuses this archive.
use super::{document, paths, project_path, read_candidate, read_document, revoked};
use crate::{Result, fail, files::read, manifest::sha256_hex, package};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

pub(super) fn open(task: &Path) -> Result<Value> {
    if revoked(task)? {
        return Err(fail("revoked task is closed"));
    }
    let stored = read_document(task)?;
    let fresh = document(&project_path(&stored)?)?;
    if stored["baseline"] != fresh["baseline"] {
        return Err(fail("stale task baseline"));
    }
    if stored != fresh {
        return Err(fail("task contract does not match the project"));
    }
    Ok(stored)
}

pub(super) fn freeze(task: &Path) -> Result<Value> {
    let doc = open(task)?;
    let mut candidates = Vec::new();
    collect(
        &task.join("candidate"),
        &task.join("candidate"),
        &mut candidates,
        0,
    )?;
    if candidates.len() != 1 || !paths(&doc, "editable")?.contains(&candidates[0]) {
        return Err(fail("task candidate must be exactly one editable kit file"));
    }
    let path = &candidates[0];
    let candidate = read_candidate(&task.join("candidate").join(path))?;
    let inspected = package::inspect_project(&project_path(&doc)?)?;
    let baseline: BTreeMap<_, _> = inspected
        .files
        .iter()
        .map(|f| (f.inventory.path.clone(), f.inventory.sha256.clone()))
        .collect();
    let modes: BTreeMap<_, _> = inspected
        .files
        .iter()
        .map(|f| (f.inventory.path.clone(), f.inventory.mode))
        .collect();
    if json!(baseline) != doc["baseline"] || json!(modes) != doc["baseline_modes"] {
        return Err(fail("project changed while freezing candidate"));
    }
    let root = task.join("authority/snapshot");
    package::new_directory(&root)?;
    for file in &inspected.files {
        let bytes = if &file.inventory.path == path {
            &candidate
        } else {
            &file.bytes
        };
        write_payload(&root, &file.inventory.path, bytes, file.inventory.mode)?;
    }
    let archive = package::pack_project(&root)?;
    let archive_bytes = read(&archive, 110 * 1024 * 1024)?;
    let verified = package::read_archive_bytes(&archive_bytes)?;
    // Compare the archive against the bytes captured above, not another source read.
    for file in &inspected.files {
        let bytes = if &file.inventory.path == path {
            &candidate
        } else {
            &file.bytes
        };
        if verified.files.get(&file.inventory.path) != Some(bytes)
            || !verified
                .manifest
                .files
                .iter()
                .any(|f| f.path == file.inventory.path && f.mode == file.inventory.mode)
        {
            return Err(fail("snapshot changed while packaging"));
        }
    }
    if verified.files.len() != inspected.files.len() {
        return Err(fail("snapshot inventory changed while packaging"));
    }
    package::write_new(&task.join("authority/candidate.tar.gz"), &archive_bytes)?;
    let before = inspected
        .files
        .iter()
        .find(|f| &f.inventory.path == path)
        .ok_or_else(|| fail("candidate path missing"))?;
    let mut plan = json!({
        "schema":"an-loom-check-plan-v2", "attempt":uuid::Uuid::new_v4().to_string(),
        "baseline":doc["baseline"], "release":verified.manifest,
        "archive_sha256":sha256_hex(&archive_bytes),
        "candidate":{"path":path,"sha256":sha256_hex(&candidate)},
        "diff":{"changed":before.bytes != candidate},
        "acceptance":doc["acceptance"], "outputs":["checks.txt"],
        "model_invoked":false,"host_invoked_by_loom":false,"automatic_retries":0,
        "host_execution":{"required":true,"owner":"an-agent"}
    });
    let profile = super::host_check::prepare(task, &plan)?;
    plan["profile_sha256"] = json!(sha256_hex(&profile));
    plan["plan_sha256"] = json!(sha256_hex(&serde_json::to_vec(&plan)?));
    package::write_new(
        &task.join("authority/check-plan.json"),
        &serde_json::to_vec_pretty(&plan)?,
    )?;
    Ok(plan)
}

pub(super) fn load(task: &Path) -> Result<(Value, Vec<u8>, package::VerifiedArchive)> {
    let doc = open(task)?;
    let mut plan: Value =
        serde_json::from_slice(&read(&task.join("authority/check-plan.json"), 1024 * 1024)?)?;
    let hash = plan
        .as_object_mut()
        .ok_or_else(|| fail("invalid check plan"))?
        .remove("plan_sha256")
        .ok_or_else(|| fail("legacy check evidence must be regenerated"))?;
    if plan["schema"] != "an-loom-check-plan-v2"
        || hash != sha256_hex(&serde_json::to_vec(&plan)?)
        || plan["baseline"] != doc["baseline"]
    {
        return Err(fail("changed check plan"));
    }
    plan["plan_sha256"] = hash;
    let path = plan["candidate"]["path"]
        .as_str()
        .ok_or_else(|| fail("candidate path"))?;
    crate::manifest::validate_payload_path(path)?;
    if sha256_hex(&read_candidate(&task.join("candidate").join(path))?)
        != plan["candidate"]["sha256"]
    {
        return Err(fail("candidate changed after freezing"));
    }
    let archive_path = task.join("authority/candidate.tar.gz");
    let bytes = read(&archive_path, 110 * 1024 * 1024)?;
    if sha256_hex(&bytes) != plan["archive_sha256"] {
        return Err(fail("frozen archive changed"));
    }
    let archive = package::read_archive_bytes(&bytes)?;
    if json!(archive.manifest) != plan["release"] {
        return Err(fail("frozen release changed"));
    }
    for (name, content) in &archive.files {
        if read(
            &task.join("authority/snapshot").join(name),
            20 * 1024 * 1024,
        )? != *content
        {
            return Err(fail("candidate snapshot changed"));
        }
    }
    Ok((plan, bytes, archive))
}

pub(super) fn write_payload(root: &Path, name: &str, bytes: &[u8], mode: u32) -> Result<()> {
    crate::manifest::validate_payload_path(name)?;
    let path = root.join(name);
    fs::create_dir_all(path.parent().ok_or_else(|| fail("payload parent"))?)?;
    package::write_new(&path, bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
    #[cfg(not(unix))]
    let _ = mode;
    Ok(())
}

fn collect(root: &Path, directory: &Path, found: &mut Vec<String>, depth: usize) -> Result<()> {
    if depth > 16 || found.len() > 1 {
        return Err(fail("candidate inventory exceeds task limit"));
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let meta = fs::symlink_metadata(entry.path())?;
        if meta.is_dir() {
            collect(root, &entry.path(), found, depth + 1)?;
        } else if meta.is_file() && !meta.is_symlink() {
            found.push(
                entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|_| fail("candidate escaped"))?
                    .to_str()
                    .ok_or_else(|| fail("candidate UTF-8 path"))?
                    .into(),
            );
        } else {
            return Err(fail("candidate must be a regular file"));
        }
    }
    Ok(())
}
