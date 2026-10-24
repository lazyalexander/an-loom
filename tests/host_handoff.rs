//! Explicit real-host acceptance; absence of a host is an error, never a passing skip.
#![cfg(target_os = "macos")]
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const HOST_REVISION: &str = "588d6c0652f7c2347b9255da677002e2c35dfa98";

fn call(executable: &Path, args: &[&str]) -> Result<Value> {
    let output = Command::new(executable).args(args).output()?;
    if !output.status.success() {
        return Err(format!(
            "CLI request failed: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    // The pinned host emits its resolved plan before the terminal execution record.
    // Validate that explicit JSON sequence rather than silently taking a text line.
    let mut records = serde_json::Deserializer::from_slice(&output.stdout)
        .into_iter::<Value>()
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let executing = args.first() == Some(&"run") && args.last() == Some(&"--execute");
    if executing {
        if records.len() != 2 || records[0]["execute"] != true || records[0]["supported"] != true {
            return Err("invalid pinned host execution result sequence".into());
        }
    } else if records.len() != 1 {
        return Err("expected exactly one CLI result".into());
    }
    records.pop().ok_or_else(|| "missing CLI result".into())
}

fn value(value: &Value, key: &str) -> Result<String> {
    Ok(value[key]
        .as_str()
        .ok_or_else(|| format!("missing {key}"))?
        .into())
}

fn text(path: &Path) -> Result<&str> {
    path.to_str().ok_or_else(|| "path must be UTF-8".into())
}

fn toolchain(root: &Path) -> Result<PathBuf> {
    let tool = |name: &str, args: &[&str]| -> Result<String> {
        let output = Command::new(name).args(args).output()?;
        if !output.status.success() {
            return Err(format!("tool discovery failed: {name}").into());
        }
        Ok(String::from_utf8(output.stdout)?.trim().into())
    };
    let path = root.join("toolchain.local.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({
        "rustc":Path::new(&tool("rustc", &["--print","sysroot"])?).join("bin/rustc"),
        "sdk":tool("xcrun", &["--show-sdk-path"])?,"linker":tool("xcrun", &["--find","clang"])?}))?,
    )?;
    Ok(path)
}

#[test]
#[ignore = "requires AN_AGENT_HOST_BIN and native macOS execution; mandatory in host-handoff CI"]
fn real_development_release_registration_and_controlled_file_execution() -> Result {
    let host = PathBuf::from(
        std::env::var_os("AN_AGENT_HOST_BIN").ok_or("AN_AGENT_HOST_BIN is required")?,
    )
    .canonicalize()?;
    let loom = Path::new(env!("CARGO_BIN_EXE_an-loom"));
    let temp = tempfile::tempdir()?;
    let root = match std::env::var_os("LOOM_ACCEPTANCE_DIR") {
        Some(path) => {
            let path = PathBuf::from(path);
            fs::create_dir(&path)?;
            path.canonicalize()?
        }
        None => temp.path().canonicalize()?,
    };
    let project = root.join("project");
    call(loom, &["init", text(&project)?])?;
    let mut contract = an_loom::project::Project::read(&project)?;
    let kit = contract.kit.as_mut().ok_or("missing kit")?;
    kit.source = "kit/normalize.rs".into();
    fs::rename(project.join("kit/main.rs"), project.join(&kit.source))?;
    fs::write(project.join("loom.toml"), toml::to_string(&contract)?)?;
    let tools = toolchain(&root)?;
    let job = root.join("host-job");
    let args = [
        "develop",
        text(&project)?,
        text(&host)?,
        text(&tools)?,
        text(&job)?,
    ];
    let preview = call(loom, &args)?;
    assert_eq!(preview["plan"]["host_preview"]["plan"]["supported"], true);
    assert!(!job.exists());
    let digest = value(&preview, "approval_sha256")?;
    let mut submit = args.to_vec();
    submit.extend(["--approve", &digest]);
    let source = project.join("kit/normalize.rs");
    let original = fs::read(&source)?;
    let mut changed = original.clone();
    changed.extend(b"\n// revised after preview\n");
    fs::write(&source, changed)?;
    assert!(call(loom, &submit).is_err());
    assert!(!job.exists());
    fs::write(&source, original)?;
    let result = call(loom, &submit)?;
    assert_eq!(result["status"], "succeeded");
    assert!(fs::read_to_string(job.join("audit.json"))?.contains("applied"));
    assert!(call(loom, &submit).is_err());
    let delivery = root.join("release");
    let release = call(loom, &["release", text(&project)?, text(&delivery)?])?;
    call(loom, &["verify-release", text(&delivery)?])?;
    let archive = an_loom::read_archive(&delivery.join("spool.tar.gz"))?;
    let private = root.to_str().ok_or("private root")?.as_bytes();
    for bytes in archive.files.values() {
        assert!(!bytes.windows(private.len()).any(|part| part == private));
        if let Some(home) = std::env::var_os("HOME") {
            let home = home.to_string_lossy();
            if !home.is_empty() {
                assert!(
                    !bytes
                        .windows(home.len())
                        .any(|part| part == home.as_bytes())
                );
            }
        }
    }
    let deployment = delivery.join("deployment");
    call(&host, &["check", text(&deployment)?])?;
    let store = root.join("host-store");
    let registered = call(&host, &["register", text(&deployment)?, text(&store)?])?;
    let deployment_digest = value(&registered, "registered")?;
    let input = root.join("input.txt");
    fs::write(&input, " beta\nalpha\nbeta\n\n")?;
    let run_args = ["run", text(&store)?, &deployment_digest, text(&input)?];
    call(&host, &run_args)?;
    assert!(!store.join("runs").exists());
    let mut run_args = run_args.to_vec();
    run_args.push("--execute");
    let executed = call(&host, &run_args)?;
    assert_eq!(executed["status"], "succeeded");
    let receipt = PathBuf::from(value(&executed, "receipt")?);
    let artifact = receipt
        .parent()
        .ok_or("missing run directory")?
        .join("artifact.txt");
    assert_eq!(fs::read(&artifact)?, b"alpha\nbeta\n");
    assert_eq!(fs::read(&input)?, b" beta\nalpha\nbeta\n\n");
    let receipt: Value = serde_json::from_slice(&fs::read(receipt)?)?;
    assert_eq!(receipt["process"]["reaped"], true);
    assert_eq!(
        receipt["artifact"]["sha256"],
        format!("{:x}", Sha256::digest(b"alpha\nbeta\n"))
    );
    // Share only a closed portable summary; full plans and logs remain local.
    fs::write(
        root.join("acceptance.json"),
        serde_json::to_vec_pretty(&json!({
        "schema":"an-loom-acceptance-v1","host_revision":HOST_REVISION,
        "os":std::env::consts::OS,"architecture":std::env::consts::ARCH,
        "build_backend":"macos-rust-build-v1","runtime_backend":receipt["backend"],
        "release":release,"deployment_digest":deployment_digest,
        "assertions":["preview creates no job","stale approval refused without execution",
            "host-isolated compilation and domain tests pass","duplicate build submission refused",
            "portable release verifies","private job and home paths absent from package payloads",
            "real host registration and file execution","input unchanged","accepted output matches",
            "process reaped and artifact hash verified"],"model_calls":0}))?,
    )?;
    Ok(())
}

#[test]
#[ignore = "requires AN_AGENT_HOST_BIN and native macOS execution; mandatory in host-handoff CI"]
fn failed_domain_checks_never_become_a_release() -> Result {
    let host = PathBuf::from(
        std::env::var_os("AN_AGENT_HOST_BIN").ok_or("AN_AGENT_HOST_BIN is required")?,
    )
    .canonicalize()?;
    let loom = Path::new(env!("CARGO_BIN_EXE_an-loom"));
    let temp = tempfile::tempdir()?;
    let root = temp.path().canonicalize()?;
    let project = root.join("project");
    call(loom, &["init", text(&project)?])?;
    let source = project.join("kit/main.rs");
    let mut code = fs::read_to_string(&source)?;
    code.push_str("\n#[cfg(test)] #[test] fn domain_rejection() { panic!(\"intentional acceptance failure\"); }\n");
    fs::write(source, code)?;
    let tools = toolchain(&root)?;
    let job = root.join("failed-job");
    let mut args = vec![
        "develop",
        text(&project)?,
        text(&host)?,
        text(&tools)?,
        text(&job)?,
    ];
    let preview = call(loom, &args)?;
    let digest = value(&preview, "approval_sha256")?;
    args.extend(["--approve", &digest]);
    assert!(call(loom, &args).is_err());
    let failure: Value = serde_json::from_slice(&fs::read(job.join("receipt.json"))?)?;
    assert_eq!(failure["status"], "failed");
    assert_eq!(failure["result"]["stage"], "tests");
    assert!(!project.join("kit/program").exists());
    assert!(!project.join("kit/spool.yaml").exists());
    assert!(
        call(
            loom,
            &["release", text(&project)?, text(&root.join("release"))?]
        )
        .is_err()
    );
    assert!(!root.join("release").exists());
    Ok(())
}
