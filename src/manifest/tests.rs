use super::*;

fn manifest(include: Vec<&str>) -> Manifest {
    Manifest {
        schema_version: 1,
        name: "sample-kit".to_owned(),
        version: "0.1.0".to_owned(),
        description: None,
        include: include.into_iter().map(str::to_owned).collect(),
    }
}

#[test]
fn validates_manifest_and_rejects_unsafe_paths() {
    assert!(manifest(vec!["README.md", "kit"]).validate().is_ok());
    for path in ["../secret", "/absolute", "kit\\source", "dist/build"] {
        assert!(manifest(vec![path]).validate().is_err(), "accepted {path}");
    }
}

#[test]
fn secret_scanner_finds_common_credentials_without_domain_policy() {
    assert!(contains_secret(
        b"github_pat_ABCDEFGHIJKLMNOPQRSTUVWXYZ123456"
    ));
    assert!(contains_secret(b"-----BEGIN PRIVATE KEY-----"));
    assert!(!contains_secret(b"ordinary project source"));
    assert!(!contains_secret(b"binary\0payload"));
}

#[test]
fn bundle_digest_is_stable_and_metadata_bound() {
    let package = PackageIdentity {
        name: "sample-kit".to_owned(),
        version: "0.1.0".to_owned(),
        description: None,
    };
    let file = PayloadFile {
        path: "README.md".to_owned(),
        size: 4,
        mode: 0o644,
        sha256: sha256_hex(b"test"),
    };
    let bundle = BundleManifest::new(package, vec![file]);
    assert!(bundle.as_ref().is_ok_and(|value| value.validate().is_ok()));
}

#[test]
fn provider_key_detection_preserves_hyphens_and_requires_token_boundaries() {
    assert!(contains_secret(
        b"token=sk-abcdefghijklmnopqrstuvwxyz123456"
    ));
    assert!(contains_secret(
        b"\"sk-proj-abcdefghijklmnopqrstuvwxyz-123456\""
    ));
    assert!(!contains_secret(b"ask-abcdefghijklmnopqrstuvwxyz123456"));
    assert!(!contains_secret(b"sk-short-example"));
}
