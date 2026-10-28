//! Qualified workspace-v2 wire adapter. No process supervision or host dependency.
//! The operator owns both ledgers; hashes detect mismatch, not forged host identity.
use crate::{Result, fail, files::read, manifest::sha256_hex, package};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub(crate) fn prepare(task: &Path, plan: &Value) -> Result<Vec<u8>> {
    let (runner, profile) = expected(task, plan)?;
    package::write_new(&task.join("authority/check.sh"), runner.as_bytes())?;
    let bytes = serde_json::to_vec_pretty(&profile)?;
    package::write_new(&task.join("authority/host-profile.json"), &bytes)?;
    Ok(bytes)
}

fn expected(task: &Path, plan: &Value) -> Result<(String, Value)> {
    let attempt = plan["attempt"]
        .as_str()
        .ok_or_else(|| fail("missing check identity"))?;
    if uuid::Uuid::parse_str(attempt).is_err() {
        return Err(fail("invalid check identity"));
    }
    let mut runner =
        String::from("set -eu\ncd \"$AN_LOOM_PROJECT\"\n: > \"$AN_LOOM_WORK/checks.txt\"\n");
    for command in crate::task::paths(plan, "acceptance")? {
        // Shell text is an explicit reviewed project input, never an implicit hook.
        runner.push_str(&format!(
            "/bin/bash -eu -c '{}' >> \"$AN_LOOM_WORK/checks.txt\" 2>&1\n",
            command.replace('\'', "'\\''")
        ));
    }
    runner.push_str(&format!(
        "printf '%s\\n' 'completed {attempt}' >> \"$AN_LOOM_WORK/checks.txt\"\n"
    ));
    let mut inputs = Vec::new();
    let mut layout = Vec::new();
    for file in plan["release"]["files"]
        .as_array()
        .ok_or_else(|| fail("missing release inventory"))?
    {
        let name = file["path"].as_str().ok_or_else(|| fail("release path"))?;
        crate::manifest::validate_payload_path(name)?;
        inputs.push(task.join("authority/snapshot").join(name));
        layout.push(format!("project/{name}"));
    }
    if inputs.len() >= 16 {
        return Err(fail(
            "candidate exceeds qualified host input limit (15 project files)",
        ));
    }
    let index = inputs.len();
    inputs.push(task.join("authority/check.sh"));
    layout.push("check.sh".into());
    Ok((
        runner,
        json!({"schema":"an-workspace-job-v2","executable":"/bin/bash",
        "args":[format!("{{input:{index}}}")],"environment":{
            "AN_LOOM_PROJECT":"{inputs}/project","AN_LOOM_WORK":"{work}"},
        "readonly":[],"inputs":inputs,"input_layout":layout,"outputs":["checks.txt"],
        "wall_seconds":30,"trusted_process_group":true}),
    ))
}

pub(crate) fn verify(task: &Path, check: &Value, job: &Path) -> Result<Value> {
    let (runner, profile) = expected(task, check)?;
    let profile_bytes = read(&task.join("authority/host-profile.json"), 65536)?;
    if serde_json::from_slice::<Value>(&profile_bytes)? != profile
        || sha256_hex(&profile_bytes) != check["profile_sha256"]
        || read(&task.join("authority/check.sh"), 65536)? != runner.as_bytes()
    {
        return Err(fail("candidate host profile or runner changed"));
    }
    let mut attempts = fs::read_dir(job.join("attempts"))?.collect::<std::io::Result<Vec<_>>>()?;
    if attempts.len() != 1 || job.join("reclaim-intent.json").exists() {
        return Err(fail(
            "check requires one retained host attempt; no resume or automatic retry",
        ));
    }
    let dir = attempts.remove(0).path();
    let id = dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| fail("host attempt identity"))?;
    if uuid::Uuid::parse_str(id).is_err() || dir.join("successor.json").exists() {
        return Err(fail("invalid or superseded host attempt"));
    }
    let receipt_bytes = read(&dir.join("receipt.json"), 2 * 1024 * 1024)?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)?;
    if receipt["schema"] != "an-workspace-receipt-v1"
        || receipt["attempt"] != id
        || receipt["status"] != "succeeded"
        || receipt["exit_code"] != 0
        || receipt["termination_confirmed"] != true
        || receipt["reaped"] != true
        || receipt["group_cleanup_required"] != false
        || receipt["automatic_retries"] != 0
        || !receipt["pid"]
            .as_u64()
            .is_some_and(|pid| pid > 0 && pid <= i32::MAX as u64)
        || receipt["group_observation"]["group"] != receipt["pid"]
        || receipt["group_observation"]["empty"] != true
        || receipt["group_observation"].get("error").is_some()
    {
        return Err(fail(
            "check failed, cancelled or termination unknown; no automatic retry",
        ));
    }
    let plan_bytes = read(&dir.join("plan.json"), 2 * 1024 * 1024)?;
    let plan: Value = serde_json::from_slice(&plan_bytes)?;
    let exec_bytes = read(&dir.join("execution.json"), 2 * 1024 * 1024)?;
    let exec: Value = serde_json::from_slice(&exec_bytes)?;
    if receipt["plan_sha256"] != sha256_hex(&plan_bytes)
        || receipt["execution_sha256"] != sha256_hex(&exec_bytes)
        || plan["schema"] != "an-workspace-plan-v2"
        || plan["root"] != json!(job)
        || !plan["predecessor"].is_null()
        || plan["profile_sha256"] != check["profile_sha256"]
        || plan["readonly"] != json!([])
        || plan["outputs"] != profile["outputs"]
        || plan["wall_seconds"] != 30
        || plan["network"] != false
        || plan["automatic_retries"] != 0
        || plan["backend"] != "macos-trusted-workspace-v1"
        || plan["executable"] != json!(Path::new("/bin/bash").canonicalize()?)
        || plan["executable_sha256"] != sha256_hex(&read(Path::new("/bin/bash"), 20 * 1024 * 1024)?)
    {
        return Err(fail("host plan does not bind the candidate check"));
    }
    verify_mapping(check, job, &dir, &plan, &exec, &profile, &runner)?;
    let output = accepted(job, &dir, &receipt_bytes, &plan)?;
    if !output.ends_with(
        format!(
            "completed {}\n",
            check["attempt"]
                .as_str()
                .ok_or_else(|| fail("check identity"))?
        )
        .as_bytes(),
    ) {
        return Err(fail("check runner did not finish all commands"));
    }
    Ok(
        json!({"schema":"an-loom-check-evidence-v2","job":job,"attempt":check["attempt"],
        "check_plan_sha256":check["plan_sha256"],"host_attempt":id,
        "host_receipt_sha256":sha256_hex(&receipt_bytes),"host_plan_sha256":sha256_hex(&plan_bytes),
        "host_execution_sha256":sha256_hex(&exec_bytes),"output_sha256":sha256_hex(&output),
        "release_digest":check["release"]["releaseDigest"],"status":"succeeded",
        "trust":"operator-owned local host ledger; not a signed attestation"}),
    )
}

fn verify_mapping(
    check: &Value,
    job: &Path,
    dir: &Path,
    plan: &Value,
    exec: &Value,
    profile: &Value,
    runner: &str,
) -> Result<()> {
    let inputs = plan["inputs"]
        .as_array()
        .ok_or_else(|| fail("host inputs"))?;
    let selected = profile["inputs"]
        .as_array()
        .ok_or_else(|| fail("profile inputs"))?;
    let layout = profile["input_layout"]
        .as_array()
        .ok_or_else(|| fail("profile layout"))?;
    if inputs.len() != selected.len() {
        return Err(fail("host input inventory mismatch"));
    }
    let mut expected_exec = plan.clone();
    let mut mapped = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
        let name = layout[index].as_str().ok_or_else(|| fail("layout name"))?;
        let sha = if index + 1 == inputs.len() {
            json!(sha256_hex(runner.as_bytes()))
        } else {
            check["release"]["files"][index]["sha256"].clone()
        };
        let source = Path::new(selected[index].as_str().ok_or_else(|| fail("input path"))?);
        let target = dir.join("snapshot/inputs").join(name);
        if input != &json!({"path":source,"name":name,"sha256":sha})
            || sha256_hex(&read(&target, 20 * 1024 * 1024)?) != sha
        {
            return Err(fail("host snapshot does not contain the checked candidate"));
        }
        mapped.push(json!({"path":target,"name":name,"sha256":sha}));
    }
    let env = json!({"PATH":"/usr/bin:/bin","TMPDIR":job.join("work/tmp"),"LANG":"C",
        "AN_LOOM_PROJECT":"{inputs}/project","AN_LOOM_WORK":job.join("work")});
    if plan["args"] != profile["args"] || plan["environment"] != env {
        return Err(fail(
            "host command or environment differs from check profile",
        ));
    }
    expected_exec["inputs"] = json!(mapped);
    expected_exec["args"] = json!([dir.join("snapshot/inputs/check.sh")]);
    expected_exec["environment"]["AN_LOOM_PROJECT"] = json!(dir.join("snapshot/inputs/project"));
    // Policy is host-generated, bound by the receipt; Loom does not implement the sandbox.
    expected_exec["policy"] = exec["policy"].clone();
    if exec != &expected_exec {
        return Err(fail("execution differs from the approved host mapping"));
    }
    let intent: Value = serde_json::from_slice(&read(&dir.join("intent.json"), 65536)?)?;
    if intent["attempt"] != plan_id(dir)?
        || intent["approval_sha256"] != sha256_hex(&serde_json::to_vec(plan)?)
    {
        return Err(fail("host approval identity mismatch"));
    }
    Ok(())
}

fn plan_id(dir: &Path) -> Result<&str> {
    dir.file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| fail("attempt identity"))
}

fn accepted(job: &Path, dir: &Path, receipt: &[u8], plan: &Value) -> Result<Vec<u8>> {
    let intent_bytes = read(&dir.join("accept-intent.json"), 65536)?;
    let intent: Value = serde_json::from_slice(&intent_bytes)?;
    let accepted: Value = serde_json::from_slice(&read(&dir.join("accepted.json"), 65536)?)?;
    if intent["schema"] != "an-workspace-accept-v1"
        || intent["attempt"] != plan_id(dir)?
        || intent["receipt_sha256"] != sha256_hex(receipt)
        || accepted["schema"] != "an-workspace-accepted-v1"
        || accepted["status"] != "succeeded"
        || accepted["approval_sha256"] != sha256_hex(&serde_json::to_vec(&intent)?)
        || accepted["outputs"].as_array().map(Vec::len) != Some(1)
        || intent["outputs"].as_array().map(Vec::len) != Some(1)
        || accepted["outputs"][0]["path"] != "checks.txt"
    {
        return Err(fail("missing or mismatched formal host acceptance"));
    }
    let binding = &accepted["outputs"][0]["binding"];
    let revision = binding["revision"]
        .as_str()
        .ok_or_else(|| fail("revision"))?;
    if revision.len() != 64 || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(fail("invalid revision"));
    }
    let root = job.join("artifacts/revisions").join(revision);
    let metadata = read(&root.join("revision.json"), 65536)?;
    let manifest: Value = serde_json::from_slice(&metadata)?;
    let output = read(&root.join("content.txt"), 65536)?;
    let sha = sha256_hex(&output);
    if sha256_hex(&metadata) != revision
        || manifest["schema"] != "an-artifact-revision-v1"
        || manifest["content_sha256"] != sha
        || manifest["bytes"] != output.len()
        || binding["content_sha256"] != sha
        || binding["bytes"] != output.len()
        || manifest["source"]
            != json!({"kind":"workspace-output","attempt":plan_id(dir)?,
            "path":"checks.txt","receipt_sha256":sha256_hex(receipt),"profile_sha256":plan["profile_sha256"]})
        || intent["outputs"][0] != json!({"path":"checks.txt","sha256":sha,"bytes":output.len()})
    {
        return Err(fail("accepted check artifact provenance mismatch"));
    }
    Ok(output)
}
