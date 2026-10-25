use super::support::Result;
use an_loom::project::{FILE, Project};
use std::fs;

#[test]
fn silk_authoring_preserves_legacy_binding_checks_and_rejects_ambiguity() -> Result {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("spool");
    an_loom::project::scaffold(&root, true)?;
    let canonical = fs::read_to_string(root.join(FILE))?;
    assert!(canonical.contains("[silk]"));
    assert!(!canonical.contains("[kit]"));
    let legacy = canonical.replace("[silk]", "[kit]");
    let current = Project::parse(&canonical)?;
    let old = Project::parse(&legacy)?;
    assert_eq!(current, old);
    assert_eq!(current.silk()?, old.kit()?);
    let duplicate = format!(
        "{canonical}\n[kit]\n{}",
        legacy.split("[kit]").nth(1).ok_or("binding")?
    );
    assert!(Project::parse(&duplicate).is_err());
    for text in [&canonical, &legacy] {
        let escaped = text.replace("kit/main.rs", "../outside.rs");
        assert!(Project::parse(&escaped).is_err());
        let wrong_profile = text.replace("file-kit-v1", "unrestricted");
        assert!(Project::parse(&wrong_profile).is_err());
    }
    Ok(())
}
