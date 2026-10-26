//! Trusted operator-selected host CLI transport; not a spool sandbox.
use crate::{Result, fail, files::read, manifest::sha256_hex, plan::HostCall};
use serde_json::Value;
use std::{path::Path, process::Command};

pub(super) fn call(submission: &HostCall, expected_digest: &str) -> Result<Value> {
    if sha256_hex(&read(Path::new(&submission.executable), 100 * 1024 * 1024)?) != expected_digest {
        return Err(fail("operator-selected host executable changed"));
    }
    let output = Command::new(&submission.executable)
        .args(&submission.args)
        .current_dir(&submission.cwd)
        .env_clear()
        .envs(&submission.environment)
        .output()?;
    if output.stdout.len() > 1024 * 1024 || output.stderr.len() > 65536 {
        return Err(fail(
            "host CLI response exceeds the adapter's accepted record limit",
        ));
    }
    if !output.status.success() {
        return Err(fail(format!(
            "host request failed; inspect the retained host job: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
