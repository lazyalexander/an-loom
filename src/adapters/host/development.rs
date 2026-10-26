use super::transport::call;
use crate::{
    Result, fail,
    files::read,
    manifest::sha256_hex,
    plan::{DevelopmentPlan, HostCall},
    project::{FILE, Project},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

/// Preview has no project/job writes. Approval submits once; the host owns lifecycle.
pub fn develop(
    project: &Path,
    host: &Path,
    toolchain: &Path,
    job: &Path,
    approval: Option<&str>,
) -> Result<Value> {
    let root = project.canonicalize()?;
    let project_bytes = read(&root.join(FILE), 1024 * 1024)?;
    let contract = Project::parse(
        std::str::from_utf8(&project_bytes).map_err(|_| fail("project must be UTF-8"))?,
    )?;
    let kit = contract.kit()?;
    let source = read(&root.join(&kit.source), 65536)?;
    super::file_profile::analyze(&source)?;
    // Inspect the allowlist before asking the trusted host to build anything.
    crate::package::inspect_project(&root)?;
    for path in [
        &kit.program,
        &kit.build_record,
        &kit.declaration,
        &kit.analysis,
    ] {
        if std::fs::symlink_metadata(root.join(path)).is_ok() {
            return Err(fail(
                "development requires fresh program and evidence destinations",
            ));
        }
    }
    let host = host.canonicalize()?;
    let host_digest = sha256_hex(&read(&host, 100 * 1024 * 1024)?);
    let toolchain = toolchain.canonicalize()?;
    let tools = read(&toolchain, 16384)?;
    let job = if job.is_absolute() {
        job.to_path_buf()
    } else {
        std::env::current_dir()?.join(job)
    };
    let job = job
        .parent()
        .ok_or_else(|| fail("job needs an existing parent"))?
        .canonicalize()?
        .join(job.file_name().ok_or_else(|| fail("job needs a name"))?);
    if job.starts_with(&root) {
        return Err(fail("host job must stay outside the distributable project"));
    }
    let mut submission = HostCall {
        executable: host.to_string_lossy().into_owned(),
        args: vec![
            "build-kit".into(),
            root.join(&kit.source).to_string_lossy().into_owned(),
            toolchain.to_string_lossy().into_owned(),
            job.to_string_lossy().into_owned(),
        ],
        cwd: root.to_string_lossy().into_owned(),
        environment: BTreeMap::new(),
    };
    let preview = call(&submission, &host_digest)?;
    let host_approval = preview["approval_sha256"]
        .as_str()
        .ok_or_else(|| fail("host omitted approval digest"))?
        .to_owned();
    submission.args.extend(["--approve".into(), host_approval]);
    let plan = DevelopmentPlan::assemble(
        &project_bytes,
        &source,
        &tools,
        host_digest,
        preview,
        submission,
    )?;
    let Some(approval) = approval else {
        return Ok(json!({"plan":plan,"approval_sha256":plan.digest()?,"executed":false}));
    };
    plan.authorize(approval)?;
    let result = call(&plan.submission, &plan.host_sha256)?;
    if result["status"] != "succeeded" {
        return Err(fail(
            "host did not accept a successful build; inspect its job",
        ));
    }
    super::build_artifacts::receive(&root, &job)?;
    super::file_profile::declare(&root)?;
    Ok(
        json!({"schema":"an-loom-development-result-v1","status":"succeeded",
        "approval_sha256":approval,"source_sha256":plan.source_sha256,
        "program_sha256":sha256_hex(&read(&root.join(&kit.program), 20*1024*1024)?),
        "executed":true,"automatic_retries":0,"host_job":job}),
    )
}
