//! Bounded kit-edit assignment. No model call and no host CLI.
mod evidence;
use crate::adapters::host::candidate_check as host_check;
mod snapshot;
pub use evidence::{check, pack_candidate, receive_check};

use crate::{
    Result, fail,
    manifest::{sha256_hex, validate_payload_path},
    package,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, io::Read, path::Path};

pub(super) const CANDIDATE_LIMIT: usize = 64 * 1024;

pub fn prepare(project: &Path, destination: &Path) -> Result<Value> {
    let project = fs::canonicalize(project)?;
    let destination = fresh_directory(destination, &project)?;
    let document = document(&project)?;
    package::new_directory(&destination.join("authority"))?;
    package::new_directory(&destination.join("workspace"))?;
    package::write_new(
        &destination.join("authority/task.json"),
        &serde_json::to_vec_pretty(&document)?,
    )?;
    let editable = paths(&document, "editable")?;
    let inspected = crate::package::inspect_project(&project)?;
    for path in &editable {
        let bytes = inspected
            .files
            .iter()
            .find(|file| file.inventory.path == *path)
            .map(|file| file.bytes.as_slice())
            .ok_or_else(|| fail("editable kit file is not in the project"))?;
        let slot = destination.join("workspace").join(path);
        if let Some(parent) = slot.parent() {
            fs::create_dir_all(parent)?;
        }
        package::write_new(&slot, bytes)?;
    }
    package::write_new(
        &destination.join("workspace/BRIEF.md"),
        brief(&document).as_bytes(),
    )?;
    Ok(json!({
        "schema": "an-loom-task-v1",
        "directory": destination,
        "editable": editable,
        "model_invoked": false,
        "host_cli_included": false,
        "applied_to_project": false,
    }))
}

pub fn receive(task: &Path, editable: &str, candidate: &Path) -> Result<Value> {
    let task = fs::canonicalize(task)?;
    if revoked(&task)? {
        return Err(fail("revoked task cannot receive a candidate"));
    }
    let stored = read_document(&task)?;
    let project = project_path(&stored)?;
    let fresh = document(&project)?;
    if stored["baseline"] != fresh["baseline"] {
        return Err(fail("stale task baseline"));
    }
    if stored != fresh {
        return Err(fail("task contract does not match the project"));
    }
    validate_payload_path(editable)?;
    if !paths(&stored, "editable")?
        .iter()
        .any(|path| path == editable)
    {
        return Err(fail("path is not an editable kit file"));
    }
    let bytes = read_candidate(candidate)?;
    let slot = task.join("candidate").join(editable);
    if slot.starts_with(&project) {
        return Err(fail("candidate must stay outside the project"));
    }
    if let Some(parent) = slot.parent() {
        fs::create_dir_all(parent)?;
    }
    package::write_new(&slot, &bytes)?;
    Ok(json!({
        "schema": "an-loom-task-candidate-v1",
        "path": editable,
        "sha256": sha256_hex(&bytes),
        "model_invoked": false,
        "applied_to_project": false,
        "release_updated": false,
    }))
}

pub fn revoke(task: &Path) -> Result<()> {
    let task = fs::canonicalize(task)?;
    let _ = read_document(&task)?;
    if revoked(&task)? {
        return Err(fail("task already revoked"));
    }
    package::write_new(
        &task.join("authority/revoked.json"),
        &serde_json::to_vec_pretty(&json!({
            "schema": "an-loom-task-revoked-v1",
            "model_invoked": false,
        }))?,
    )
}

pub(super) fn document(project: &Path) -> Result<Value> {
    let inspected = crate::package::inspect_project(project)?;
    let files: BTreeMap<&str, &[u8]> = inspected
        .files
        .iter()
        .map(|file| (file.inventory.path.as_str(), file.bytes.as_slice()))
        .collect();
    let authoring = crate::adapters::host::native_spool::authoring(&files)?;
    if authoring.editable.is_empty() {
        return Err(fail("task has no editable kit path"));
    }
    let mut baseline = BTreeMap::new();
    let mut baseline_modes = BTreeMap::new();
    for file in &inspected.files {
        baseline.insert(file.inventory.path.clone(), file.inventory.sha256.clone());
        baseline_modes.insert(file.inventory.path.clone(), file.inventory.mode);
    }
    let protected: Vec<_> = baseline
        .keys()
        .filter(|path| !authoring.editable.contains(path))
        .cloned()
        .collect();
    let mut task = json!({
        "schema": "an-loom-task-v1",
        "model_invoked": false,
        "host_cli_included": false,
        "project": {
            "name": inspected.manifest.name,
            "version": inspected.manifest.version,
            "description": inspected.manifest.description,
            "path": project,
        },
        "entry": authoring.entry,
        "runtime": authoring.runtime,
        "editable": authoring.editable,
        "protected": protected,
        "inputs": authoring.inputs,
        "outputs": authoring.outputs,
        "capabilities": authoring.capabilities,
        "dependencies": authoring.dependencies,
        "acceptance": authoring.acceptance,
        "baseline": baseline,
        "baseline_modes": baseline_modes,
        "limits": {"candidate_bytes": CANDIDATE_LIMIT, "files": 1},
        "escalation": "Stop and ask the operator. Do not invoke an-agent-host.",
    });
    if !authoring.binary_outputs.is_empty() {
        task["binary_outputs"] = json!(authoring.binary_outputs);
    }
    if let Some(slot) = authoring.state_input {
        task["state_input"] = json!(slot);
    }
    Ok(task)
}

fn brief(document: &Value) -> String {
    format!(
        "Edit only the listed kit working copies.\n\
         Protected silk, policy and check files are not editable.\n\
         A silk change needs a separately authorized task and a new release.\n\
         Acceptance commands and dependency labels are records. Do not run or download them.\n\
         Capability names are not grants. This brief does not include a host CLI.\n\
         Write the replacement outside the project and submit it with receive-task.\n\
         The native session stays with the coding tool.\n\n\
         {document}\n"
    )
}

pub(super) fn fresh_directory(destination: &Path, project: &Path) -> Result<std::path::PathBuf> {
    let destination = if destination.is_absolute() {
        destination.to_path_buf()
    } else {
        std::env::current_dir()?.join(destination)
    };
    let parent = destination
        .parent()
        .ok_or_else(|| fail("task directory needs an existing parent"))?;
    let name = destination
        .file_name()
        .ok_or_else(|| fail("task directory needs a name"))?;
    let destination = parent.canonicalize()?.join(name);
    if destination.starts_with(project) {
        return Err(fail("task directory must stay outside the project"));
    }
    package::new_directory(&destination)?;
    Ok(destination)
}

pub(super) fn read_document(task: &Path) -> Result<Value> {
    let bytes = crate::files::read(&task.join("authority/task.json"), 1024 * 1024)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(super) fn project_path(document: &Value) -> Result<std::path::PathBuf> {
    let path = document["project"]["path"]
        .as_str()
        .ok_or_else(|| fail("task is missing its project"))?;
    fs::canonicalize(path).map_err(|_| fail("task project is unavailable"))
}

pub(super) fn paths(document: &Value, key: &str) -> Result<Vec<String>> {
    document[key]
        .as_array()
        .ok_or_else(|| fail("task contract is incomplete"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| fail("task contract is incomplete"))
        })
        .collect()
}

pub(super) fn read_candidate(path: &Path) -> Result<Vec<u8>> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| fail("candidate must be a regular file"))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(fail("candidate must be a regular file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err(fail("candidate must be a regular file"));
        }
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(CANDIDATE_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > CANDIDATE_LIMIT {
        return Err(fail("candidate exceeds the task limit"));
    }
    Ok(bytes)
}

pub(super) fn revoked(task: &Path) -> Result<bool> {
    match fs::symlink_metadata(task.join("authority/revoked.json")) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(true),
        Ok(_) => Err(fail("task revocation is not a regular file")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}
