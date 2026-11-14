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

#[test]
fn long_paths_use_regular_ustar_entries_without_extension_records() -> Result<()> {
    let name = format!("{}/payload.bin", vec!["component"; 16].join("/"));
    let mut builder = tar::Builder::new(Vec::new());
    super::append(&mut builder, &name, 3, &b"abc"[..])?;
    let bytes = builder.into_inner()?;
    let mut archive = tar::Archive::new(&bytes[..]);
    let entries = archive
        .entries()?
        .raw(true)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(entries.len(), 1);
    assert!(entries[0].header().entry_type().is_file());
    assert_eq!(entries[0].path_bytes().as_ref(), name.as_bytes());
    Ok(())
}

#[test]
fn staging_freezes_selected_bytes_and_refuses_existing_destination() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().canonicalize()?;
    let source = root.join("source");
    fs::create_dir(&source)?;
    fs::write(source.join("NOTICE"), b"licensed")?;
    fs::write(source.join("runtime"), vec![0u8; 131073])?;
    let selection = root.join("sources.json");
    fs::write(
        &selection,
        serde_json::to_vec(
            &serde_json::json!({"name":"test","version":"1","platform":"macos-aarch64","licenses":["payload/NOTICE"],"components":{"runtime":"payload/runtime"},"sources":{"payload":source}}),
        )?,
    )?;
    let output = root.join("staged");
    stage::stage(&selection, &output)?;
    let manifest: Manifest = serde_json::from_slice(&fs::read(output.join("runtime.json"))?)?;
    assert_eq!(manifest.files.len(), 2);
    assert_eq!(manifest.files["payload/runtime"].bytes, 131073);
    assert_eq!(fs::read(output.join("payload/runtime"))?, vec![0u8; 131073]);
    assert!(stage::stage(&selection, &output).is_err());
    pack(
        &output,
        &output.join("runtime.json"),
        &root.join("runtime.tar.gz"),
    )?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn staging_normalizes_internal_file_aliases_but_refuses_escaping_links() -> Result<()> {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir()?;
    let root = temp.path().canonicalize()?;
    let source = root.join("source");
    fs::create_dir(&source)?;
    fs::write(source.join("NOTICE"), b"licensed")?;
    symlink("NOTICE", source.join("alias"))?;
    let selection = root.join("sources.json");
    fs::write(
        &selection,
        serde_json::to_vec(
            &serde_json::json!({"name":"test","version":"1","platform":"macos-aarch64","licenses":["payload/NOTICE"],"components":{"runtime":"payload/alias"},"sources":{"payload":source}}),
        )?,
    )?;
    let output = root.join("staged");
    stage::stage(&selection, &output)?;
    assert!(
        !fs::symlink_metadata(output.join("payload/alias"))?
            .file_type()
            .is_symlink()
    );
    fs::write(root.join("outside"), b"private")?;
    symlink("../outside", source.join("escape"))?;
    let refused = root.join("refused");
    assert!(stage::stage(&selection, &refused).is_err());
    assert!(!refused.exists());
    Ok(())
}
