//! Operator-approved composition of existing candidate and host contracts.
use super::{native_spool, transport::call};
use crate::{Result, fail, files::read, manifest::sha256_hex, package, plan::HostCall, task};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub fn develop_native(
    task_path: &Path,
    host: &Path,
    job: &Path,
    delivery: &Path,
    approved: Option<&str>,
) -> Result<Value> {
    let task = task_path.canonicalize()?;
    let fence = task.join("authority/develop-native.json");
    if fs::symlink_metadata(&fence).is_ok() {
        return Err(fail(
            "native development was already submitted; inspect retained evidence, do not rerun",
        ));
    }
    let job = fresh_path(job)?;
    let delivery = fresh_path(delivery)?;
    let doc: Value =
        serde_json::from_slice(&read(&task.join("authority/task.json"), 1024 * 1024)?)?;
    let project = Path::new(
        doc["project"]["path"]
            .as_str()
            .ok_or_else(|| fail("missing task project"))?,
    )
    .canonicalize()?;
    for path in [&job, &delivery] {
        if path.starts_with(&task)
            || task.starts_with(path)
            || path.starts_with(&project)
            || project.starts_with(path)
        {
            return Err(fail(
                "job and delivery must be outside the task and source project",
            ));
        }
    }
    if job.starts_with(&delivery) || delivery.starts_with(&job) {
        return Err(fail("job and delivery must be separate"));
    }
    let check = task::check(&task)?;
    let host = host.canonicalize()?;
    let host_sha = sha256_hex(&read(&host, 100 * 1024 * 1024)?);
    let profile = task.join("authority/host-profile.json");
    let profile_bytes = read(&profile, 65536)?;
    if sha256_hex(&profile_bytes) != check["profile_sha256"] {
        return Err(fail("candidate host profile changed"));
    }
    let mut request = HostCall {
        executable: host.to_string_lossy().into_owned(),
        args: vec![
            "workspace-run".into(),
            profile.to_string_lossy().into_owned(),
            job.to_string_lossy().into_owned(),
        ],
        cwd: task.to_string_lossy().into_owned(),
        environment: BTreeMap::new(),
    };
    let preview = call(&request, &host_sha)?;
    let hash = preview["approval_sha256"]
        .as_str()
        .filter(|s| crate::manifest::valid_digest(s))
        .ok_or_else(|| fail("host omitted workspace approval"))?;
    if preview["executed"] != false
        || preview["plan"]["schema"] != "an-workspace-plan-v2"
        || preview["plan"]["profile_sha256"] != check["profile_sha256"]
        || preview["plan"]["root"] != json!(job)
    {
        return Err(fail("host preview does not bind the candidate workspace"));
    }
    let plan = json!({"schema":"an-loom-native-development-v1","check":check,"host_sha256":host_sha,
        "host_preview":preview,"job":job,"delivery":delivery,"profile":serde_json::from_slice::<Value>(&profile_bytes)?,
        "steps":["workspace-run","workspace-accept","receive-check","pack-task","export-native-spool-v2","spool-load"],
        "authority":"one bounded check; accept declared check output only on confirmed success; deliver exact checked bytes",
        "automatic_retries":0,"model_invoked":false});
    let digest = sha256_hex(&serde_json::to_vec(&plan)?);
    let Some(approved) = approved else {
        return Ok(json!({"plan":plan,"approval_sha256":digest,"executed":false}));
    };
    if approved != digest {
        return Err(fail(
            "native development approval changed; inspect a fresh preview",
        ));
    }
    // Close admission before any submission. Even pre-submit errors require explicit inspection.
    package::write_new(&fence, &serde_json::to_vec_pretty(&plan)?)?;
    package::new_directory(&delivery)?;
    package::write_new(
        &delivery.join("intent.json"),
        &serde_json::to_vec_pretty(&plan)?,
    )?;
    request.args.extend(["--approve".into(), hash.into()]);
    let result = execute(&task, &job, &delivery, &host_sha, request);
    match result {
        Ok(value) => {
            package::write_new(
                &delivery.join("result.json"),
                &serde_json::to_vec_pretty(&value)?,
            )?;
            Ok(value)
        }
        Err(error) => {
            let outcome = json!({"status":"not-completed","automatic_retries":0,"error":error.to_string(),
                "next":"inspect retained host/task evidence; do not resubmit this task"});
            let _ = package::write_new(
                &delivery.join("result.json"),
                &serde_json::to_vec_pretty(&outcome)?,
            );
            Err(fail(format!(
                "native development stopped: {error}; retained job {}, delivery {}; no automatic retry",
                job.display(),
                delivery.display()
            )))
        }
    }
}

fn execute(
    task: &Path,
    job: &Path,
    delivery: &Path,
    host_sha: &str,
    mut request: HostCall,
) -> Result<Value> {
    let run = call(&request, host_sha)?;
    package::write_new(
        &delivery.join("host-run.json"),
        &serde_json::to_vec_pretty(&run)?,
    )?;
    if run["status"] != "succeeded" || run["termination_confirmed"] != true || run["reaped"] != true
    {
        return Err(fail("host check did not confirm successful termination"));
    }
    let attempt = run["attempt"]
        .as_str()
        .ok_or_else(|| fail("host omitted attempt"))?;
    request.args = vec![
        "workspace-accept".into(),
        job.to_string_lossy().into_owned(),
        attempt.into(),
    ];
    let preview = call(&request, host_sha)?;
    let hash = preview["approval_sha256"]
        .as_str()
        .filter(|s| crate::manifest::valid_digest(s))
        .ok_or_else(|| fail("host omitted acceptance approval"))?;
    if preview["executed"] != false
        || preview["plan"]["schema"] != "an-workspace-accept-v1"
        || preview["plan"]["attempt"] != attempt
    {
        return Err(fail("host acceptance preview identity mismatch"));
    }
    request.args.extend(["--approve".into(), hash.into()]);
    let accepted = call(&request, host_sha)?;
    package::write_new(
        &delivery.join("host-accepted.json"),
        &serde_json::to_vec_pretty(&accepted)?,
    )?;
    task::receive_check(task, job)?;
    let packed = task::pack_candidate(task, &delivery.join("project"))?;
    let archive = Path::new(
        packed["archive"]
            .as_str()
            .ok_or_else(|| fail("missing checked archive"))?,
    );
    let exported = native_spool::export_v2(archive, &delivery.join("spool"))?;
    request.args = vec![
        "spool-load".into(),
        delivery.join("spool").to_string_lossy().into_owned(),
        delivery.join("instance").to_string_lossy().into_owned(),
    ];
    let loaded = call(&request, host_sha)?;
    package::write_new(
        &delivery.join("host-load.json"),
        &serde_json::to_vec_pretty(&loaded)?,
    )?;
    if loaded["release"] != exported["release"]
        || loaded["execution_granted"] != false
        || loaded["loaded"] != true
    {
        return Err(fail("host admission did not bind the delivered release"));
    }

    Ok(
        json!({"schema":"an-loom-native-development-result-v1","status":"succeeded",
        "archive":archive,"spool":delivery.join("spool"),"instance":delivery.join("instance"),"release":exported["release"],
        "checked_release":packed["release_digest"],"host_attempt":attempt,"automatic_retries":0,
        "model_invoked":false,"execution_granted":false}),
    )
}

fn fresh_path(path: &Path) -> Result<PathBuf> {
    let full = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let result = full
        .parent()
        .ok_or_else(|| fail("destination parent"))?
        .canonicalize()?
        .join(full.file_name().ok_or_else(|| fail("destination name"))?);
    match fs::symlink_metadata(&result) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(result),
        _ => Err(fail(
            "native development requires fresh job and delivery destinations; inspect existing evidence",
        )),
    }
}
