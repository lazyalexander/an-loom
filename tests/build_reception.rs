//! Fixed file ABI acceptance; Loom never links or executes the kit.
use serde_json::Value;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
static NEXT_DIRECTORY: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

struct Directory(PathBuf);
impl Directory {
    fn new() -> TestResult<Self> {
        let path = std::env::temp_dir().canonicalize()?.join(format!(
            "loom-files-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos(),
            // Wall-clock resolution is not a uniqueness guarantee for parallel tests.
            NEXT_DIRECTORY.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fixture(root: &Path) -> TestResult<(PathBuf, PathBuf)> {
    let project = root.join("kit");
    an_loom::project::scaffold(&project, true)?;
    let job = root.join("job");
    fs::create_dir_all(job.join("accepted"))?;
    let source = fs::read(project.join("kit/main.rs"))?;
    let program = b"not executed in reception tests";
    let manifest = json!({"schema":"an-rust-build-artifacts-v1","source_sha256":hash(&source),"program_sha256":hash(program),"tests":"passed"});
    fs::write(job.join("accepted/main.rs"), &source)?;
    fs::write(job.join("accepted/program"), program)?;
    fs::write(job.join("accepted/manifest.json"), manifest.to_string())?;
    fs::write(job.join("receipt.json"),json!({"schema":"an-rust-build-receipt-v1","status":"succeeded","source_sha256":hash(&source),"result":{"status":"succeeded","artifacts":manifest}}).to_string())?;
    Ok((project, job))
}

#[test]
fn reception_is_source_bound_portable_and_non_overwriting() -> TestResult {
    let root = Directory::new()?;
    let (project, job) = fixture(&root.0)?;
    let mut receipt: Value = serde_json::from_slice(&fs::read(job.join("receipt.json"))?)?;
    receipt["private_plan_path"] = json!(root.0.join("private-plan.json"));
    fs::write(job.join("receipt.json"), receipt.to_string())?;
    an_loom::build_artifacts::receive(&project, &job)?;
    assert!(
        !fs::read_to_string(project.join("kit/build.json"))?
            .contains(&root.0.to_string_lossy().to_string())
    );
    an_loom::spool::declare(&project)?;
    let yaml = fs::read_to_string(project.join("kit/spool.yaml"))?;
    assert!(yaml.contains("an-agent-file-v1"));
    assert!(yaml.contains("path: /work"));
    assert!(an_loom::build_artifacts::receive(&project, &job).is_err());
    fs::write(project.join("kit/program"), "changed after reception")?;
    assert!(an_loom::spool::declare(&project).is_err());
    Ok(())
}

#[test]
fn failed_tampered_or_stale_builds_are_not_received() -> TestResult {
    for target in [
        "source",
        "binary",
        "receipt",
        "hardlink",
        "symlink",
        "destination",
    ] {
        let root = Directory::new()?;
        let (project, job) = fixture(&root.0)?;
        match target {
            "source" => fs::write(project.join("kit/main.rs"), "fn main() {}")?,
            "binary" => fs::write(job.join("accepted/program"), "tampered")?,
            "receipt" => fs::write(job.join("receipt.json"), r#"{"status":"failed"}"#)?,
            "hardlink" => fs::hard_link(job.join("accepted/program"), root.0.join("alias"))?,
            "symlink" => {
                #[cfg(unix)]
                {
                    fs::rename(job.join("accepted/program"), root.0.join("program"))?;
                    std::os::unix::fs::symlink(
                        root.0.join("program"),
                        job.join("accepted/program"),
                    )?;
                }
                #[cfg(not(unix))]
                {
                    continue;
                }
            }
            "destination" => fs::write(project.join("kit/build.json"), "existing")?,
            _ => unreachable!(),
        }
        assert!(
            an_loom::build_artifacts::receive(&project, &job).is_err(),
            "{target}"
        );
        assert!(!project.join("kit/program").exists(), "{target}");
    }
    Ok(())
}

#[test]
fn private_or_unknown_build_metadata_never_enters_the_package() -> TestResult {
    for case in ["unknown", "compiler", "empty", "result-status", "duplicate"] {
        let root = Directory::new()?;
        let (project, job) = fixture(&root.0)?;
        let mut manifest: Value =
            serde_json::from_slice(&fs::read(job.join("accepted/manifest.json"))?)?;
        let mut receipt: Value = serde_json::from_slice(&fs::read(job.join("receipt.json"))?)?;
        match case {
            "unknown" => manifest["local_path"] = json!("/private/build/job"),
            "compiler" => manifest["compiler"] = json!("/private/toolchain/bin/rustc"),
            "empty" => {
                fs::write(job.join("accepted/program"), b"")?;
                manifest["program_sha256"] = json!(hash(b""));
            }
            "result-status" => receipt["result"]["status"] = json!("failed"),
            _ => (),
        }
        receipt["result"]["artifacts"] = manifest.clone();
        let bytes = if case == "duplicate" {
            // Value parsing alone silently discards the earlier duplicate field.
            manifest
                .to_string()
                .replacen('{', "{\"tests\":\"failed\",", 1)
        } else {
            manifest.to_string()
        };
        fs::write(job.join("accepted/manifest.json"), bytes)?;
        fs::write(job.join("receipt.json"), receipt.to_string())?;
        assert!(
            an_loom::build_artifacts::receive(&project, &job).is_err(),
            "{case}"
        );
        assert!(!project.join("kit/program").exists(), "{case}");
        assert!(!project.join("kit/build.json").exists(), "{case}");
    }
    Ok(())
}

#[test]
fn package_checks_reject_metadata_added_after_reception() -> TestResult {
    let root = Directory::new()?;
    let (project, job) = fixture(&root.0)?;
    an_loom::build_artifacts::receive(&project, &job)?;
    an_loom::spool::declare(&project)?;
    // Check the spool-export validator independently of the generic byte verifier.
    an_loom::run(["pack".into(), project.to_string_lossy().into_owned()])?;
    let archive = fs::read_dir(project.join("dist/artifacts"))?
        .next()
        .ok_or("missing archive")??
        .path();
    let mut verified = an_loom::read_archive(&archive)?;
    an_loom::spool::validate(&verified)?;
    let record = project.join("kit/build.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&record)?)?;
    manifest["local_path"] = json!("/private/build/job");
    verified
        .files
        .insert("kit/build.json".into(), manifest.to_string().into_bytes());
    assert!(an_loom::spool::validate(&verified).is_err());
    fs::write(record, manifest.to_string())?;
    for command in ["declare", "check", "pack"] {
        assert!(
            an_loom::run([command.into(), project.to_string_lossy().into_owned()]).is_err(),
            "{command}"
        );
    }
    // The earlier identified archive remains unchanged and valid.
    an_loom::spool::validate(&an_loom::read_archive(&archive)?)?;
    Ok(())
}
