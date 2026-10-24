//! Deterministic plans bind operator-selected host requests to project inputs.
use crate::{Result, fail, manifest::sha256_hex};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct HostCall {
    pub executable: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct DevelopmentPlan {
    pub schema: &'static str,
    pub project_sha256: String,
    pub source_sha256: String,
    pub toolchain_sha256: String,
    pub host_sha256: String,
    pub host_preview: Value,
    pub submission: HostCall,
    pub automatic_retries: u8,
}

impl DevelopmentPlan {
    /// Pure validation and assembly. Host preview bytes carry no Loom authority.
    pub fn assemble(
        project: &[u8],
        source: &[u8],
        toolchain: &[u8],
        host_digest: String,
        host_preview: Value,
        submission: HostCall,
    ) -> Result<Self> {
        if host_preview["plan"]["schema"] != "an-rust-build-plan-v1"
            || host_preview["plan"]["source_sha256"] != sha256_hex(source)
            || host_preview["plan"]["toolchain_profile_sha256"] != sha256_hex(toolchain)
            || host_preview["approval_sha256"]
                .as_str()
                .is_none_or(|s| !crate::manifest::valid_digest(s))
        {
            return Err(fail(
                "host preview does not bind the selected source and toolchain",
            ));
        }
        Ok(Self {
            schema: "an-loom-development-plan-v1",
            project_sha256: sha256_hex(project),
            source_sha256: sha256_hex(source),
            toolchain_sha256: sha256_hex(toolchain),
            host_sha256: host_digest,
            host_preview,
            submission,
            automatic_retries: 0,
        })
    }

    pub fn digest(&self) -> Result<String> {
        Ok(sha256_hex(&serde_json::to_vec(self)?))
    }

    pub fn authorize(&self, approved: &str) -> Result<()> {
        if self.digest()? != approved {
            return Err(fail("development approval mismatch; inspect a fresh plan"));
        }
        if self.host_preview["plan"]["supported"] != true {
            return Err(fail("host build profile is unsupported; no fallback"));
        }
        Ok(())
    }
}
