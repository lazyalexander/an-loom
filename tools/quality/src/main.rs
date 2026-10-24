//! Fail-closed repository quality gate over the existing, revision-pinned checker.
use pplx_space_archcheck::{
    Baseline, Context, Mode, Renames, check,
    config::CrateNamesPolicy,
    report::{Format, render},
};
use serde::Deserialize;
use std::{error::Error, fs, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    version: u8,
    forbidden_runtime_dependencies: Vec<String>,
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git").current_dir(root).args(args).output()?;
    if !output.status.success() {
        return Err(format!("git failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(output.stdout)
}

fn bootstrap(base: &str) -> bool {
    // Exactly the pre-gate main and fixed Nextflow input trees.
    // A merge or squash with the same tree is equivalent. After activation, every
    // later comparison must load committed policy.
    matches!(
        base,
        "cff1c0f792b4876f2227195502180cc159837967" | "5b15f313ecb559f0a226f4daad43f9b474999368"
    )
}

fn run() -> Result<()> {
    let mut root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "--root") && args.len() >= 2 {
        root = Path::new(&args[1]).canonicalize()?;
        args.drain(..2);
    }
    let reference = match args.as_slice() {
        [flag, value] if flag == "--base" => value.as_str(),
        [] => "origin/main",
        _ => return Err("usage: cargo quality [--base COMMIT]".into()),
    };
    let base = String::from_utf8(git(
        &root,
        &["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
    )?)?
    .trim()
    .to_owned();
    let mut ctx = Context::load(&root)?;
    let scratch = tempfile::tempdir()?;
    // An explicit one-time bootstrap cannot grandfather violations or weaken
    // checks in follow-up changes. Missing comparison history is otherwise fatal.
    let tree = String::from_utf8(git(&root, &["rev-parse", &format!("{base}^{{tree}}")])?)?
        .trim()
        .to_owned();
    let bootstrap = bootstrap(&tree);
    for name in [
        "ARCHITECTURE.toml",
        "architecture-baseline.toml",
        "quality.toml",
    ] {
        let current = fs::read(root.join(name))?;
        let previous = if bootstrap {
            current.clone()
        } else {
            git(&root, &["show", &format!("{base}:{name}")])?
        };
        if name != "architecture-baseline.toml" && !bootstrap {
            let before: toml::Value = toml::from_str(std::str::from_utf8(&previous)?)?;
            let after: toml::Value = toml::from_str(std::str::from_utf8(&current)?)?;
            if before != after {
                return Err("quality policy changes require a separate reviewed migration".into());
            }
        }
        fs::write(scratch.path().join(name), previous)?;
    }
    if bootstrap {
        let baseline: toml::Value = toml::from_str(&fs::read_to_string(
            root.join("architecture-baseline.toml"),
        )?)?;
        fn empty(v: &toml::Value) -> bool {
            v.as_table().is_some_and(|t| t.values().all(empty))
        }
        if !empty(&baseline) {
            return Err("initial quality baseline must contain no exceptions".into());
        }
    }
    Baseline::load(&scratch.path().join("architecture-baseline.toml"))?;
    ctx.base_baseline = Baseline::load_base(&scratch.path().join("architecture-baseline.toml"))?;
    ctx.base_architecture = CrateNamesPolicy::load_base(&scratch.path().join("ARCHITECTURE.toml"))?;
    let renames = scratch.path().join("renames");
    fs::write(
        &renames,
        git(
            &root,
            &["diff", "-z", "--name-status", "--find-renames", &base],
        )?,
    )?;
    ctx.renames = Some(Renames::load(&renames)?);
    let findings = check(&ctx);
    print!("{}", render(Format::Human, &findings, ctx.files.len()));
    if !pplx_space_archcheck::skipped(&ctx).is_empty() {
        return Err("incomplete quality comparison".into());
    }
    let policy: Policy = toml::from_str(&fs::read_to_string(root.join("quality.toml"))?)?;
    if policy.version != 1 {
        return Err("unsupported quality policy".into());
    }
    let metadata = Command::new("cargo")
        .current_dir(&root)
        .args(["metadata", "--format-version=1", "--no-deps", "--locked"])
        .output()?;
    if !metadata.status.success() {
        return Err("cargo metadata failed".into());
    }
    let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout)?;
    for package in metadata["packages"]
        .as_array()
        .ok_or("missing metadata packages")?
    {
        for dependency in package["dependencies"]
            .as_array()
            .ok_or("missing dependencies")?
        {
            if dependency["kind"] != "dev"
                && policy
                    .forbidden_runtime_dependencies
                    .iter()
                    .any(|n| dependency["name"] == *n)
            {
                return Err(format!("forbidden runtime dependency: {}", dependency["name"]).into());
            }
        }
    }
    if findings.iter().any(|f| f.mode == Mode::Required) {
        return Err("required quality findings".into());
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("quality gate: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bootstrap_never_accepts_a_moving_reference() {
        for name in ["main", "origin/main", "HEAD", ""] {
            assert!(!super::bootstrap(name));
        }
    }
}
