//! Check plan and evidence for one kit candidate. No model call and no host CLI.
use super::{
    CANDIDATE_LIMIT, document, fresh_directory, paths, project_path, read_candidate, read_document,
    revoked,
};
use crate::{
    Result, fail,
    manifest::{sha256_hex, validate_payload_path},
    package,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub fn check(task: &Path) -> Result<Value> {
    let task = fs::canonicalize(task)?;
    ensure_open(&task)?;
    let (plan, _) = current_plan(&task)?;
    let path = task.join("authority/check-plan.json");
    if path.exists() {
        let existing: Value = serde_json::from_slice(&crate::files::read(&path, 1024 * 1024)?)?;
        if existing != plan {
            return Err(fail(
                "check plan no longer matches the candidate; start a new task",
            ));
        }
    } else {
        package::write_new(&path, &serde_json::to_vec_pretty(&plan)?)?;
    }
    Ok(plan)
}

pub fn receive_check(task: &Path, evidence: &Path) -> Result<Value> {
    let task = fs::canonicalize(task)?;
    ensure_open(&task)?;
    if task.join("authority/evidence.json").exists() {
        return Err(fail("check evidence already recorded"));
    }
    let (plan, candidate) = current_plan(&task)?;
    if !task.join("authority/check-plan.json").exists() {
        return Err(fail("task has no check plan"));
    }
    let stored: Value = serde_json::from_slice(&crate::files::read(
        &task.join("authority/check-plan.json"),
        1024 * 1024,
    )?)?;
    if stored != plan {
        return Err(fail("stale check plan"));
    }
    let evidence = fs::canonicalize(evidence)?;
    let project = project_path(&read_document(&task)?)?;
    if evidence.starts_with(&task) || evidence.starts_with(&project) {
        return Err(fail(
            "check evidence must stay outside the task and project",
        ));
    }
    let receipt: Evidence = serde_json::from_slice(&crate::files::read(
        &evidence.join("receipt.json"),
        64 * 1024,
    )?)?;
    let path = plan["candidate"]["path"]
        .as_str()
        .ok_or_else(|| fail("check plan is incomplete"))?;
    let sha = plan["candidate"]["sha256"]
        .as_str()
        .ok_or_else(|| fail("check plan is incomplete"))?;
    if receipt.schema != "an-loom-check-evidence-v1"
        || receipt.attempt != plan["attempt"]
        || receipt.candidate_sha256 != sha
    {
        return Err(fail(format!(
            "evidence attempt does not match the current check plan for {path} candidate {sha}"
        )));
    }
    match receipt.status.as_str() {
        "cancelled" => {
            return Err(fail(format!(
                "check cancelled for {path} candidate {sha}; not publishing"
            )));
        }
        "unknown" => {
            return Err(fail(format!(
                "check outcome unknown for {path} candidate {sha}; no automatic retry"
            )));
        }
        "failed" => {
            let failed = receipt
                .checks
                .iter()
                .find(|check| check.status != "succeeded")
                .or(receipt.checks.first());
            let command = failed.map(|check| check.command.as_str()).unwrap_or("");
            let output = excerpt(failed.map(|check| check.output.as_str()).unwrap_or(""));
            return Err(fail(format!(
                "check failed for {path} candidate {sha}: command {command} status failed: {output}"
            )));
        }
        "succeeded" => {}
        _ => {
            return Err(fail(format!(
                "check outcome unknown for {path} candidate {sha}; no automatic retry"
            )));
        }
    }
    let acceptance = paths(&plan, "acceptance")?;
    if receipt.checks.len() != acceptance.len()
        || receipt
            .checks
            .iter()
            .zip(&acceptance)
            .any(|(check, command)| &check.command != command || check.status != "succeeded")
    {
        return Err(fail(format!(
            "check evidence does not match the planned commands for {path} candidate {sha}"
        )));
    }
    if receipt.checks.iter().any(|check| check.output.len() > 4096) {
        return Err(fail("check output exceeds the evidence limit"));
    }
    let outputs = paths(&plan, "outputs")?;
    if receipt.artifacts.len() != outputs.len()
        || receipt
            .artifacts
            .iter()
            .zip(&outputs)
            .any(|(artifact, output)| &artifact.path != output)
    {
        return Err(fail(format!(
            "check artifacts do not match the declared outputs for {path} candidate {sha}"
        )));
    }
    if sha256_hex(&candidate) != sha {
        return Err(fail(format!(
            "evidence candidate does not match the received file {path} {sha}"
        )));
    }
    let mut present = fs::read_dir(&evidence)?
        .map(|entry| {
            entry?
                .file_name()
                .into_string()
                .map_err(|_| fail("check evidence paths must be UTF-8"))
        })
        .collect::<Result<Vec<_>>>()?;
    present.sort();
    let mut expected = outputs.clone();
    expected.push("receipt.json".to_owned());
    expected.sort();
    if present != expected {
        return Err(fail("unregistered material in check evidence"));
    }
    for artifact in &receipt.artifacts {
        validate_payload_path(&artifact.path)?;
        let bytes = read_candidate(&evidence.join(&artifact.path)).map_err(|_| {
            fail(format!(
                "check artifact {} is missing or unreadable for {path} candidate {sha}",
                artifact.path
            ))
        })?;
        if sha256_hex(&bytes) != artifact.sha256 {
            return Err(fail(format!(
                "check artifact {} does not match its evidence hash for {path} candidate {sha}",
                artifact.path
            )));
        }
        let slot = task.join("authority/artifacts").join(&artifact.path);
        if let Some(parent) = slot.parent() {
            fs::create_dir_all(parent)?;
        }
        package::write_new(&slot, &bytes)?;
    }
    package::write_new(
        &task.join("authority/evidence.json"),
        &serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok(json!({
        "schema": "an-loom-check-evidence-v1",
        "attempt": receipt.attempt,
        "recorded": true,
        "model_invoked": false,
        "host_invoked_by_loom": false,
        "published": false,
    }))
}

pub fn pack_candidate(task: &Path, destination: &Path) -> Result<Value> {
    let task = fs::canonicalize(task)?;
    ensure_open(&task)?;
    let (plan, candidate) = current_plan(&task)?;
    let evidence: Evidence = serde_json::from_slice(
        &crate::files::read(&task.join("authority/evidence.json"), 64 * 1024)
            .map_err(|_| fail("passing check evidence is required before packaging a candidate"))?,
    )?;
    let path = plan["candidate"]["path"]
        .as_str()
        .ok_or_else(|| fail("check plan is incomplete"))?;
    let sha = plan["candidate"]["sha256"]
        .as_str()
        .ok_or_else(|| fail("check plan is incomplete"))?;
    if evidence.schema != "an-loom-check-evidence-v1"
        || evidence.status != "succeeded"
        || evidence.attempt != plan["attempt"]
        || evidence.candidate_sha256 != sha
        || sha256_hex(&candidate) != sha
    {
        return Err(fail(format!(
            "recorded evidence does not match the current candidate {path} {sha}"
        )));
    }
    for artifact in &evidence.artifacts {
        let bytes = crate::files::read(
            &task.join("authority/artifacts").join(&artifact.path),
            CANDIDATE_LIMIT,
        )?;
        if sha256_hex(&bytes) != artifact.sha256 {
            return Err(fail(format!(
                "recorded artifact {} does not match the admitted evidence",
                artifact.path
            )));
        }
    }
    let project = project_path(&read_document(&task)?)?;
    let destination = fresh_directory(destination, &project)?;
    if destination.starts_with(&task) {
        return Err(fail("package destination must stay outside the task"));
    }
    let inspected = crate::package::inspect_project(&project)?;
    for file in &inspected.files {
        let bytes = if file.inventory.path == path {
            candidate.as_slice()
        } else {
            file.bytes.as_slice()
        };
        let slot = destination.join(&file.inventory.path);
        if let Some(parent) = slot.parent() {
            fs::create_dir_all(parent)?;
        }
        package::write_new(&slot, bytes)?;
    }
    let archive = crate::package::pack_project(&destination)?;
    Ok(json!({
        "archive": archive,
        "attempt": plan["attempt"],
        "candidate_sha256": sha,
        "model_invoked": false,
        "host_invoked_by_loom": false,
        "applied_to_source_project": false,
    }))
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    schema: String,
    attempt: String,
    candidate_sha256: String,
    status: String,
    checks: Vec<EvidenceCheck>,
    artifacts: Vec<EvidenceArtifact>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EvidenceCheck {
    command: String,
    status: String,
    output: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EvidenceArtifact {
    path: String,
    sha256: String,
}

fn ensure_open(task: &Path) -> Result<()> {
    if revoked(task)? {
        return Err(fail("revoked task is closed"));
    }
    Ok(())
}

fn current_plan(task: &Path) -> Result<(Value, Vec<u8>)> {
    let stored = read_document(task)?;
    let project = project_path(&stored)?;
    let fresh = document(&project)?;
    if stored["baseline"] != fresh["baseline"] {
        return Err(fail("stale task baseline"));
    }
    if stored != fresh {
        return Err(fail("task contract does not match the project"));
    }
    let editable = paths(&stored, "editable")?;
    let (path, bytes) = received_candidate(task, &editable)?;
    let before = crate::files::read(&project.join(&path), CANDIDATE_LIMIT)?;
    let mut plan = json!({
        "schema": "an-loom-check-plan-v1",
        "model_invoked": false,
        "host_invoked_by_loom": false,
        "automatic_retries": 0,
        "candidate": {"path": path, "sha256": sha256_hex(&bytes)},
        "baseline_sha256": sha256_hex(&before),
        "diff": change_summary(&before, &bytes),
        "acceptance": stored["acceptance"],
        "outputs": stored["outputs"],
        "host_execution": {"required": true, "owner": "an-agent"},
    });
    plan["attempt"] = json!(sha256_hex(&serde_json::to_vec(&plan)?));
    Ok((plan, bytes))
}

fn received_candidate(task: &Path, editable: &[String]) -> Result<(String, Vec<u8>)> {
    let root = task.join("candidate");
    if !root.exists() {
        return Err(fail("task has no received candidate"));
    }
    let mut found = Vec::new();
    collect_files(&root, &root, &mut found)?;
    if found.len() != 1 || !editable.iter().any(|path| path == &found[0]) {
        return Err(fail("task candidate must be exactly one editable kit file"));
    }
    let path = found.remove(0);
    let bytes = read_candidate(&root.join(&path))?;
    Ok((path, bytes))
}

fn collect_files(root: &Path, directory: &Path, found: &mut Vec<String>) -> Result<()> {
    let mut entries = fs::read_dir(directory)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            return Err(fail("candidate must be a regular file"));
        }
        if metadata.is_dir() {
            collect_files(root, &entry.path(), found)?;
        } else if metadata.is_file() {
            let full = entry.path();
            let relative = full
                .strip_prefix(root)
                .map_err(|_| fail("candidate path escaped"))?;
            found.push(
                relative
                    .to_str()
                    .ok_or_else(|| fail("candidate path must be UTF-8"))?
                    .to_owned(),
            );
        } else {
            return Err(fail("candidate must be a regular file"));
        }
    }
    Ok(())
}

fn change_summary(before: &[u8], after: &[u8]) -> Value {
    if before == after {
        return json!({"changed": false});
    }
    let (Ok(before), Ok(after)) = (std::str::from_utf8(before), std::str::from_utf8(after)) else {
        return json!({"changed": true, "binary": true});
    };
    let mut excerpt = Vec::new();
    for (index, (left, right)) in before.lines().zip(after.lines()).enumerate() {
        if left != right {
            excerpt.push(format!(
                "line {}: {} -> {}",
                index + 1,
                clip(left),
                clip(right)
            ));
        }
        if excerpt.len() == 8 {
            break;
        }
    }
    json!({"changed": true, "excerpt": excerpt})
}

fn clip(value: &str) -> String {
    value.chars().take(80).collect()
}

fn excerpt(value: &str) -> String {
    value.chars().take(200).collect()
}
