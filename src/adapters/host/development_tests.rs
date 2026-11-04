//! Exercise the actual submission path with an uncertain failed host response.
use super::*;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};

#[test]
fn approved_failure_is_submitted_once_and_symlink_jobs_are_refused() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let base = temp.path().canonicalize()?;
    let root = base.join("project");
    crate::project::scaffold(&root, true)?;
    let tools = base.join("tools.json");
    fs::write(&tools, b"{}")?;
    let source = read(&root.join("kit/main.rs"), 65536)?;
    let preview = base.join("preview.json");
    fs::write(
        &preview,
        serde_json::to_vec(&json!({"approval_sha256":"a".repeat(64),
        "plan":{"schema":"an-rust-build-plan-v1","source_sha256":sha256_hex(&source),
        "toolchain_profile_sha256":sha256_hex(b"{}"),"supported":true}}))?,
    )?;
    let marker = base.join("submitted");
    let host = base.join("host");
    fs::write(
        &host,
        format!(
            "#!/bin/sh\ncase \"$*\" in\n*--approve*) printf x >> '{}'; printf '{{\"status\":\"failed\"}}' ;;\n*) cat '{}' ;;\nesac\n",
            marker.display(),
            preview.display()
        ),
    )?;
    fs::set_permissions(&host, fs::Permissions::from_mode(0o700))?;
    let job = base.join("job");
    let planned = develop(&root, &host, &tools, &job, None)?;
    assert!(!job.exists());
    assert!(!base.join(".job.loom-submitted.json").exists());
    let approval = planned["approval_sha256"]
        .as_str()
        .ok_or_else(|| fail("approval"))?;
    assert!(develop(&root, &host, &tools, &job, Some(approval)).is_err());
    assert!(develop(&root, &host, &tools, &job, Some(approval)).is_err());
    assert_eq!(fs::read(&marker)?, b"x");
    assert!(base.join(".job.loom-submitted.json").is_file());
    let alias = base.join("alias-job");
    symlink(base.join("nonexistent"), &alias)?;
    assert!(develop(&root, &host, &tools, &alias, None).is_err());
    assert_eq!(fs::read(&marker)?, b"x");
    Ok(())
}
