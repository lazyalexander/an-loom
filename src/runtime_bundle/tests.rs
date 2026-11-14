use super::*;
#[test]
fn repeated_pack_is_identical_and_changed_source_is_rejected() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().canonicalize()?;
    fs::write(root.join("NOTICE"), b"licensed runtime")?;
    let selection = root.join("selection.json");
    let manifest = serde_json::json!({"schema":"an-offline-runtime-v1","name":"test","version":"1","platform":"macos-aarch64","licenses":["NOTICE"],"components":{"runtime":"NOTICE"},"files":{"NOTICE":{"sha256":format!("{:x}",Sha256::digest(b"licensed runtime")),"bytes":16,"executable":false}}});
    fs::write(&selection, serde_json::to_vec(&manifest)?)?;
    pack(&root, &selection, &root.join("a.tar.gz"))?;
    pack(&root, &selection, &root.join("b.tar.gz"))?;
    assert_eq!(
        fs::read(root.join("a.tar.gz"))?,
        fs::read(root.join("b.tar.gz"))?
    );
    fs::write(root.join("NOTICE"), b"modified runtime")?;
    assert!(pack(&root, &selection, &root.join("c.tar.gz")).is_err());
    assert!(!root.join("c.tar.gz").exists());
    Ok(())
}
#[test]
fn runtime_manifest_rejects_traversal_and_missing_license() -> Result<()> {
    let value = serde_json::json!({"schema":"an-offline-runtime-v1","name":"test","version":"1","platform":"macos-aarch64","licenses":["LICENSE"],"components":{"runtime":"../outside"},"files":{"../outside":{"sha256":"a".repeat(64),"bytes":1,"executable":false}}});
    let manifest: Manifest = serde_json::from_value(value)?;
    assert!(manifest.validate().is_err());
    Ok(())
}
