//! Mutations must fail the actual CLI gate, including policy weakening.
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;
fn git(root: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout).unwrap().trim().into()
}
#[test]
fn rejects_large_files_unmediated_commands_and_policy_weakening() {
    let t = TempDir::new().unwrap();
    let root = t.path();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for p in [
        "ARCHITECTURE.toml",
        "architecture-baseline.toml",
        "quality.toml",
    ] {
        fs::copy(repo.join(p), root.join(p)).unwrap();
    }
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.1.0'\nedition='2024'\n[workspace]\n",
    )
    .unwrap();
    assert!(
        Command::new("cargo")
            .current_dir(root)
            .arg("generate-lockfile")
            .status()
            .unwrap()
            .success()
    );
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Fixture"]);
    git(root, &["config", "user.email", "fixture@example.invalid"]);
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "initial rules"]);
    let base = git(root, &["rev-parse", "HEAD"]);
    let gate = || {
        Command::new(env!("CARGO_BIN_EXE_an-loom-quality"))
            .args(["--root", root.to_str().unwrap(), "--base", &base])
            .output()
            .unwrap()
    };
    let clean = gate();
    assert!(
        clean.status.success(),
        "{}",
        String::from_utf8_lossy(&clean.stderr)
    );
    let original = fs::read(root.join("src/lib.rs")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        (0..501)
            .map(|n| format!("pub const X{n}: u8 = 1;\n"))
            .collect::<String>(),
    )
    .unwrap();
    assert!(!gate().status.success());
    fs::write(
        root.join("src/lib.rs"),
        "pub fn bad(){let _ = std::process::Command::new(\"true\");}\n",
    )
    .unwrap();
    assert!(!gate().status.success());
    fs::write(root.join("src/lib.rs"), &original).unwrap();
    let policy = fs::read_to_string(root.join("ARCHITECTURE.toml")).unwrap();
    fs::write(
        root.join("ARCHITECTURE.toml"),
        policy.replace("limit = 500", "limit = 900"),
    )
    .unwrap();
    assert!(!gate().status.success());
}
