//! Portable native export checks and an explicitly enabled real host round trip.
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn project(base: &Path) -> Result<PathBuf> {
    let root = base.join("project");
    fs::create_dir(&root)?;
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/native-spool");
    for dir in ["silk", "kit"] {
        fs::create_dir(root.join(dir))?;
    }
    for file in [
        "loom.toml",
        "native-spool.toml",
        "README.md",
        "silk/main.sh",
        "silk/report.sh",
        "kit/transform.sh",
    ] {
        fs::copy(sample.join(file), root.join(file))?;
    }
    Ok(root)
}
fn export(project: &Path, destination: &Path) -> Result {
    let archive = an_loom::package::pack_project(project)?;
    an_loom::run([
        "export-native-spool".to_owned(),
        archive.to_string_lossy().into_owned(),
        destination.to_string_lossy().into_owned(),
    ])?;
    Ok(())
}
#[test]
fn native_export_is_deterministic_exclusive_and_separate_from_source_archive() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let a = base.join("a");
    let b = base.join("b");
    export(&root, &a)?;
    export(&root, &b)?;
    assert_eq!(
        fs::read(a.join("native-spool.json"))?,
        fs::read(b.join("native-spool.json"))?
    );
    let m: Value = serde_json::from_slice(&fs::read(a.join("native-spool.json"))?)?;
    assert_eq!(m["schema"], "an-native-spool-v1");
    let mut keys = m
        .as_object()
        .ok_or("object")?
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    keys.sort();
    assert_eq!(
        keys,
        ["entry", "files", "name", "outputs", "schema", "version"]
    );
    assert_eq!(m["files"].as_object().ok_or("missing files")?.len(), 3);
    assert!(!a.join("loom.toml").exists());
    assert!(!a.join("README.md").exists());
    assert!(export(&root, &a).is_err());
    Ok(())
}
#[test]
fn native_export_refuses_unselected_code_and_ambiguous_roles() -> Result {
    for change in ["missing", "wrong-role", "output"] {
        let tmp = tempfile::tempdir()?;
        let base = tmp.path().canonicalize()?;
        let root = project(&base)?;
        let path = root.join("native-spool.toml");
        let text = fs::read_to_string(&path)?;
        let text = match change {
            "missing" => text.replace("kit/transform.sh", "kit/missing.sh"),
            "wrong-role" => text.replace("kit/transform.sh", "silk/report.sh"),
            _ => text.replace("report.txt", "../escape"),
        };
        fs::write(path, text)?;
        assert!(export(&root, &base.join("delivery")).is_err());
        assert!(!base.join("delivery").exists());
    }
    Ok(())
}
fn native_root(base: &Path) -> Result<PathBuf> {
    let root = base.join("spool");
    an_loom::run([
        "init-native".to_owned(),
        root.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    Ok(root)
}
fn check_error(root: &Path) -> Result<String> {
    Ok(an_loom::run([
        "check".to_owned(),
        root.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])
    .map(|()| "check unexpectedly succeeded".to_owned())
    .unwrap_or_else(|error| error.to_string()))
}
#[test]
fn native_scaffold_checks_packs_and_verifies_without_running_domain_code() -> Result {
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = native_root(&base)?;
    an_loom::run([
        "check".to_owned(),
        root.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    fs::write(root.join("kit/transform.sh"), b"printf 'changed\\n'\n")?;
    an_loom::run([
        "check".to_owned(),
        root.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    let archive = an_loom::package::pack_project(&root)?;
    let manifest = an_loom::package::verify_archive(&archive)?;
    assert_eq!(manifest.package.name, "native-text-spool");
    assert!(!root.join("side-effect.txt").exists());
    assert!(!root.join("transformed.txt").exists());
    assert!(!root.join("report.txt").exists());
    Ok(())
}
#[test]
fn native_check_refuses_bad_structure_and_does_not_execute_records() -> Result {
    let cases = [
        (
            "entry = \"silk/main.sh\"",
            "entry = \"silk/absent.sh\"",
            "missing native entry",
        ),
        (
            "kit = [\"kit/transform.sh\"]",
            "kit = [\"kit/transform.sh\", \"silk/main.sh\"]",
            "native role conflict",
        ),
        (
            "outputs = [\"report.txt\"]",
            "outputs = [\"../escape\"]",
            "invalid output slot",
        ),
        (
            "editable = [\"kit/transform.sh\"]",
            "editable = [\"silk/main.sh\"]",
            "editable path is outside the kit role",
        ),
        (
            "dependencies = []",
            "dependencies = [\"https://example.invalid/pkg\"]",
            "native dependency must be an unresolved label",
        ),
        (
            "runtime = \"macos-native-process-v1\"",
            "runtime = \"linux-gpu\"",
            "native runtime is not qualified for this adapter",
        ),
    ];
    for (from, to, expected) in cases {
        let tmp = tempfile::tempdir()?;
        let base = tmp.path().canonicalize()?;
        let root = native_root(&base)?;
        let path = root.join("native-spool.toml");
        let text = fs::read_to_string(&path)?.replace(from, to);
        fs::write(&path, text)?;
        let error = check_error(&root)?;
        if !error.contains(expected) {
            return Err(format!("{expected} not in {error}").into());
        }
        assert!(an_loom::package::pack_project(&root).is_err());
    }
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = native_root(&base)?;
    fs::write(root.join("silk/extra.sh"), b"printf extra\n")?;
    let error = check_error(&root)?;
    assert!(
        error.contains("undeclared native code file: silk/extra.sh"),
        "{error}"
    );
    assert!(an_loom::package::pack_project(&root).is_err());
    let marker = base.join("acceptance-ran");
    let command = format!("touch {}", marker.display());
    assert!(
        command.len() <= 256,
        "acceptance record longer than the contract allows"
    );
    let path = root.join("native-spool.toml");
    let text = fs::read_to_string(&path)?.replace(
        "acceptance = [\n    \"bash -n silk/main.sh\",\n    \"bash -n silk/report.sh\",\n    \"bash -n kit/transform.sh\",\n]",
        &format!("acceptance = [\"{command}\"]"),
    );
    fs::write(&path, text)?;
    fs::remove_file(root.join("silk/extra.sh"))?;
    an_loom::run([
        "check".to_owned(),
        root.to_str().ok_or("non-UTF8 path")?.to_owned(),
    ])?;
    an_loom::package::pack_project(&root)?;
    assert!(!marker.exists(), "acceptance command was executed");
    Ok(())
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires the qualified an-agent host binary and real macOS execution"]
fn native_host_consumes_export_edits_kit_and_reloads_baseline() -> Result {
    use std::process::Command;
    let host = std::env::var_os("AN_AGENT_HOST_BIN").ok_or("AN_AGENT_HOST_BIN is required")?;
    let call = |args: &[&str]| -> Result<Value> {
        let output = Command::new(&host).args(args).output()?;
        if !output.status.success() {
            return Err(format!(
                "host failed for {args:?}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(serde_json::from_slice(&output.stdout)?)
    };
    let approve = |args: &[&str]| -> Result<Value> {
        let preview = call(args)?;
        let hash = preview["approval_sha256"]
            .as_str()
            .ok_or("missing approval")?;
        let mut execute = args.to_vec();
        execute.extend(["--approve", hash]);
        call(&execute)
    };
    let tmp = tempfile::tempdir()?;
    let base = tmp.path().canonicalize()?;
    let root = project(&base)?;
    let delivery = base.join("delivery");
    export(&root, &delivery)?;
    let a = base.join("a");
    let b = base.join("b");
    let input = base.join("input.txt");
    let patch = base.join("patch.sh");
    fs::write(&input, b"hello\n")?;
    fs::write(&patch, b"printf 'edited:'; cat \"$1\"\n")?;
    let text = |p: &Path| -> Result<String> { Ok(p.to_str().ok_or("non-UTF8 fixture")?.into()) };
    let delivery = text(&delivery)?;
    let a = text(&a)?;
    let b = text(&b)?;
    let input = text(&input)?;
    let patch = text(&patch)?;
    let loaded = call(&["spool-load", &delivery, &a])?;
    let second = call(&["spool-load", &delivery, &b])?;
    assert_ne!(second["instance"], loaded["instance"]);
    // A deterministic file adapter exercises the slot, not model quality or containment.
    let request = call(&["spool-weaver-request", &a, "transform.sh"])?;
    let response = text(&base.join("response.json"))?;
    let mut proposal = serde_json::json!({"schema":"an-weaver-proposal-v1",
        "request":request["request"]["request"],"request_sha256":request["request_sha256"],
        "kit":"silk/main.sh","content":fs::read_to_string(&patch)?});
    fs::write(&response, serde_json::to_vec(&proposal)?)?;
    assert!(call(&["spool-weaver-apply", &a, &response]).is_err());
    proposal["kit"] = serde_json::json!("kit/transform.sh");
    fs::write(&response, serde_json::to_vec(&proposal)?)?;
    assert!(call(&["spool-weaver-apply", &b, &response]).is_err());
    approve(&["spool-weaver-apply", &a, &response])?;
    assert!(call(&["spool-weaver-apply", &a, &response]).is_err());
    let run = approve(&["spool-start", &a, "first", &input, "15"])?;
    assert_eq!(run["status"], "succeeded");
    let accepted = approve(&["spool-accept", &a, "first"])?;
    assert_eq!(accepted["release"], loaded["release"]);
    assert_eq!(
        fs::read_to_string(Path::new(&a).join("runs/first/job/work/report.txt"))?,
        "Report\nedited:hello\n"
    );
    let revision = accepted["result"]["outputs"][0]["binding"]["revision"]
        .as_str()
        .ok_or("revision")?;
    let pinned = text(&base.join("pinned.txt"))?;
    let catalog = text(&Path::new(&a).join("runs/first/job/artifacts"))?;
    let checkout = call(&["artifact-checkout", &catalog, revision, &pinned])?;
    fs::write(&input, "new material\n")?;
    approve(&["spool-start", &a, "new", &input, "15"])?;
    let result_path = Path::new(&a).join("runs/new/result.json");
    let current = fs::read(&result_path)?;
    fs::copy(Path::new(&a).join("runs/first/result.json"), &result_path)?;
    assert!(call(&["spool-accept", &a, "new"]).is_err());
    fs::write(result_path, current)?;
    approve(&["spool-accept", &a, "new"])?;
    approve(&["spool-start", &b, "consume", &pinned, "15"])?;
    assert_eq!(
        fs::read_to_string(Path::new(&b).join("runs/consume/job/work/report.txt"))?,
        "Report\nREPORT\nEDITED:HELLO\n"
    );
    approve(&["spool-reclaim", &a, "new"])?;
    approve(&["spool-reclaim", &a, "first"])?;
    approve(&["spool-unload", &a])?;
    let reloaded = text(&base.join("reloaded"))?;
    let fresh = call(&["spool-load", &delivery, &reloaded])?;
    fs::write(&input, "hello\n")?;
    assert_eq!(fresh["release"], loaded["release"]);
    assert_ne!(fresh["instance"], loaded["instance"]);
    approve(&["spool-start", &reloaded, "fresh", &input, "15"])?;
    assert_eq!(
        fs::read_to_string(Path::new(&reloaded).join("runs/fresh/job/work/report.txt"))?,
        "Report\nHELLO\n"
    );
    // Cancellation must pass through the public CLI, not only an in-process flag.
    fs::write(&patch, b"printf ready > ready; sleep 30; printf late\n")?;
    approve(&["spool-kit-update", &b, "transform.sh", &patch])?;
    let cancel_args = ["spool-start", &b, "cancelled", &input, "15"];
    let preview = call(&cancel_args)?;
    let mut child = Command::new(&host)
        .args(cancel_args)
        .args([
            "--approve",
            preview["approval_sha256"]
                .as_str()
                .ok_or("missing approval")?,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let ready = Path::new(&b).join("runs/cancelled/job/work/ready");
    let started = std::time::Instant::now();
    while !ready.exists() && started.elapsed() < std::time::Duration::from_secs(10) {
        if child.try_wait()?.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    // B remains blocked while the independently reloaded instance completes.
    let active_refusals = [
        call(&["spool-unload", &b]).is_err(),
        call(&["spool-reclaim", &b, "cancelled"]).is_err(),
        call(&["spool-weaver-request", &b, "transform.sh"]).is_err(),
    ];
    let independent = approve(&["spool-start", &reloaded, "independent", &input, "15"]);
    if child.try_wait()?.is_none() {
        let sent = Command::new("/bin/kill")
            .args(["-TERM", &child.id().to_string()])
            .status()?;
        if !sent.success() {
            child.kill()?;
        }
    }
    let stopped = child.wait_with_output()?;
    let independent = independent?;
    assert!(active_refusals.into_iter().all(|denied| denied));
    assert!(
        ready.exists(),
        "host did not reach the controlled cancellation point"
    );
    assert!(!stopped.status.success());
    let cancelled: Value = serde_json::from_slice(&stopped.stdout)?;
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(cancelled["workspace"]["termination_confirmed"], true);
    approve(&["spool-reclaim", &b, "cancelled"])?;
    assert!(call(&["spool-accept", &b, "cancelled"]).is_err());
    assert!(call(&["spool-reclaim", &b, "consume"]).is_err());
    let downstream = approve(&["spool-accept", &b, "consume"])?;
    approve(&["spool-reclaim", &b, "consume"])?;
    approve(&["spool-unload", &b])?;
    assert_eq!(fs::read_to_string(&pinned)?, "Report\nedited:hello\n");
    let evidence = serde_json::json!({"load":loaded,"second":second,"weaver_request":request,
        "adapter":"deterministic file proposal; no model invoked", "run":run,"accepted":accepted,
        "fixed_handoff":{"revision":revision,"checkout":checkout},"reload":fresh,
        "independent_run":independent,"downstream":downstream,"cancelled":cancelled});
    if let Some(path) = std::env::var_os("LOOM_NATIVE_EVIDENCE") {
        fs::write(path, serde_json::to_vec_pretty(&evidence)?)?;
    }
    Ok(())
}
