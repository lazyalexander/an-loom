//! Real producer-to-consumer v2 admission and output custody.
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn call(args: &[&str]) -> Result<Value> {
    let o = Command::new(std::env::var_os("AN_AGENT_HOST_BIN").ok_or("host required")?)
        .args(args)
        .output()?;
    if !o.status.success() {
        return Err(format!(
            "{} {}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&o.stdout)?)
}
fn path(p: &Path) -> Result<&str> {
    p.to_str().ok_or_else(|| "UTF-8 path".into())
}
fn approve(args: &[&str]) -> Result<Value> {
    let p = call(args)?;
    let mut a = args.to_vec();
    a.extend([
        "--approve",
        p["approval_sha256"].as_str().ok_or("approval")?,
    ]);
    call(&a)
}
pub(super) fn run() -> Result {
    let t = tempfile::tempdir()?;
    let base = t.path().canonicalize()?;
    let project = base.join("source");
    an_loom::run(["init-native".into(), path(&project)?.into()])?;
    let archive = an_loom::package::pack_project(&project)?;
    let delivery = base.join("delivery");
    an_loom::run([
        "export-native-spool-v2".into(),
        archive.to_string_lossy().into_owned(),
        delivery.to_string_lossy().into_owned(),
    ])?;
    let file = delivery.join("native-spool.json");
    let original = fs::read(&file)?;
    let m: Value = serde_json::from_slice(&original)?;
    for (i, (key, value)) in [
        ("os", json!("linux")),
        ("arch", json!("unknown")),
        ("gpu", json!("required")),
        ("interaction", json!("external-application")),
        ("dependencies", json!(["unresolved-tool"])),
        ("capabilities", json!(["network"])),
        ("editable", json!(["silk/main.sh"])),
    ]
    .into_iter()
    .enumerate()
    {
        let mut bad = m.clone();
        bad["requirements"][key] = value;
        fs::write(&file, serde_json::to_vec(&bad)?)?;
        let instance = base.join(format!("denied-{i}"));
        assert!(
            call(&["spool-load", path(&delivery)?, path(&instance)?]).is_err(),
            "{key}"
        );
        assert!(!instance.exists(), "{key}");
    }
    fs::write(&file, &original)?;
    let instance = base.join("instance");
    let input = base.join("input.txt");
    fs::write(&input, b"hello\n")?;
    let loaded = call(&["spool-load", path(&delivery)?, path(&instance)?])?;
    assert_eq!(loaded["requested_requirements"], m["requirements"]);
    let run = approve(&["spool-start", path(&instance)?, "run", path(&input)?, "15"])?;
    assert_eq!(run["status"], "succeeded");
    let accepted = approve(&["spool-accept", path(&instance)?, "run"])?;
    assert_eq!(accepted["status"], "succeeded");
    let old = call(&["spool-start", path(&instance)?, "next", path(&input)?, "15"])?;
    let versions = call(&["spool-artifacts", path(&instance)?, "run"])?;
    approve(&["spool-reclaim", path(&instance)?, "run"])?;
    approve(&["spool-unload", path(&instance)?])?;
    assert_eq!(
        call(&["spool-artifacts", path(&instance)?, "run"])?,
        versions
    );
    let checkout = base.join("fixed-output.txt");
    call(&[
        "artifact-checkout",
        versions["catalog"].as_str().ok_or("catalog")?,
        versions["outputs"][0]["binding"]["revision"]
            .as_str()
            .ok_or("revision")?,
        path(&checkout)?,
    ])?;
    assert_eq!(fs::read(checkout)?, b"Report\nHELLO\n");
    // Requested edit scope is not inferred from the presence of a kit payload.
    let mut locked = m.clone();
    locked["requirements"]["editable"] = json!([]);
    fs::write(&file, serde_json::to_vec(&locked)?)?;
    let second = base.join("locked");
    let other = call(&["spool-load", path(&delivery)?, path(&second)?])?;
    assert_ne!(other["release"], loaded["release"]);
    assert!(
        call(&[
            "spool-start",
            path(&second)?,
            "next",
            path(&input)?,
            "15",
            "--approve",
            old["approval_sha256"].as_str().ok_or("approval")?
        ])
        .is_err()
    );
    assert!(!second.join("runs/next/job").exists());
    let candidate = base.join("replacement.sh");
    fs::write(&candidate, b"exit 0\n")?;
    assert!(
        approve(&[
            "spool-kit-update",
            path(&second)?,
            "transform.sh",
            path(&candidate)?
        ])
        .is_err()
    );
    Ok(())
}
