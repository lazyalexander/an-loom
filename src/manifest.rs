use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{LoomError, Result};

pub(crate) const MAX_FILE_BYTES: usize = 20 * 1024 * 1024;
pub(crate) const MAX_PACKAGE_BYTES: usize = 100 * 1024 * 1024;
pub(crate) const MAX_PACKAGE_FILES: usize = 1_000;
pub(crate) const MAX_BUNDLE_BYTES: usize = 1024 * 1024;
const MAX_PATH_BYTES: usize = 1024;
const MAX_PATH_DEPTH: usize = 64;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u8,
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub include: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageIdentity {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadFile {
    pub path: String,
    pub size: usize,
    pub mode: u32,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct BundleManifest {
    pub schema_version: u8,
    pub package: PackageIdentity,
    pub files: Vec<PayloadFile>,
    pub release_digest: String,
}

#[derive(Serialize)]
struct BundleIdentity<'a> {
    schema_version: u8,
    package: &'a PackageIdentity,
    files: &'a [PayloadFile],
}

impl Manifest {
    pub fn parse(source: &str) -> Result<Self> {
        let manifest: Self = serde_json::from_str(source)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err(message("project schema_version must be 1"));
        }
        if !valid_name(&self.name) {
            return Err(message(
                "package name must be 1-64 lowercase letters, numbers, dots, underscores, or hyphens",
            ));
        }
        if !valid_version(&self.version) {
            return Err(message(
                "package version must use a numeric major.minor.patch form, optionally followed by a prerelease",
            ));
        }
        if self.include.is_empty() {
            return Err(message(
                "include must be a non-empty array of relative paths",
            ));
        }
        let mut unique = self.include.clone();
        for path in &unique {
            validate_relative_path(path)?;
        }
        unique.sort();
        unique.dedup();
        if unique.len() != self.include.len() {
            return Err(message("include paths must be unique"));
        }
        Ok(())
    }

    pub fn package_identity(&self) -> PackageIdentity {
        PackageIdentity {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
        }
    }
}

impl BundleManifest {
    pub fn new(package: PackageIdentity, files: Vec<PayloadFile>) -> Result<Self> {
        let mut bundle = Self {
            schema_version: 1,
            package,
            files,
            release_digest: String::new(),
        };
        bundle.release_digest = bundle.calculate_digest()?;
        Ok(bundle)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || !valid_name(&self.package.name)
            || !valid_version(&self.package.version)
        {
            return Err(message("bundle metadata has an unsupported format"));
        }
        if self.files.is_empty() || self.files.len() > MAX_PACKAGE_FILES {
            return Err(message(
                "bundle file count is empty or exceeds the 1,000-file limit",
            ));
        }
        let mut previous: Option<&str> = None;
        let mut total = 0usize;
        for file in &self.files {
            validate_payload_path(&file.path)?;
            if previous.is_some_and(|prior| prior >= file.path.as_str()) {
                return Err(message("bundle paths must be unique and sorted"));
            }
            previous = Some(&file.path);
            if file.size > MAX_FILE_BYTES
                || !valid_digest(&file.sha256)
                || !matches!(file.mode, 0o644 | 0o755)
            {
                return Err(message(format!(
                    "invalid bundle inventory entry: {}",
                    file.path
                )));
            }
            total = total
                .checked_add(file.size)
                .ok_or_else(|| message("bundle size overflow"))?;
            if total > MAX_PACKAGE_BYTES {
                return Err(message("bundle exceeds the 100 MiB total payload limit"));
            }
        }
        if self.calculate_digest()? != self.release_digest {
            return Err(message("bundle release digest does not match its metadata"));
        }
        Ok(())
    }

    fn calculate_digest(&self) -> Result<String> {
        let identity = BundleIdentity {
            schema_version: self.schema_version,
            package: &self.package,
            files: &self.files,
        };
        let bytes = serde_json::to_vec(&identity)?;
        Ok(sha256_hex(&bytes))
    }
}

pub(crate) fn validate_relative_path(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > MAX_PATH_BYTES
        || value.contains('\\')
        || value.starts_with('/')
        || value.contains(':')
    {
        return Err(message(format!("unsafe package path: {value:?}")));
    }
    let parts = value.split('/').collect::<Vec<_>>();
    if parts.len() > MAX_PATH_DEPTH
        || parts
            .iter()
            .any(|part| part.is_empty() || *part == "." || *part == ".." || part.len() > 255)
    {
        return Err(message(format!("unsafe package path: {value:?}")));
    }
    if matches!(
        parts[0],
        ".git" | ".loom" | ".local" | ".host-runtime" | "dist" | "target" | "node_modules"
    ) {
        return Err(message(format!(
            "generated or dependency path cannot be included: {value}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_payload_path(value: &str) -> Result<()> {
    validate_relative_path(value)?;
    for segment in value.split('/') {
        if matches!(
            segment,
            ".git" | ".loom" | ".local" | ".host-runtime" | "node_modules"
        ) {
            return Err(message(format!(
                "repository metadata or dependency paths cannot be packaged: {value}"
            )));
        }
    }
    let basename = value
        .rsplit('/')
        .next()
        .unwrap_or(value)
        .to_ascii_lowercase();
    if basename.ends_with(".local.json")
        || basename.ends_with(".local.toml")
        || basename == ".env"
        || basename.starts_with(".env.")
        || matches!(
            basename.as_str(),
            ".npmrc"
                | ".netrc"
                | ".pypirc"
                | "credentials"
                | "credentials.json"
                | "id_rsa"
                | "id_ed25519"
                | "secrets.json"
        )
        || [".pem", ".key", ".p12", ".pfx", ".jks"]
            .iter()
            .any(|extension| basename.ends_with(extension))
    {
        return Err(message(format!(
            "refusing to package credential-like file: {value}"
        )));
    }
    Ok(())
}

pub(crate) fn contains_secret(data: &[u8]) -> bool {
    if data.contains(&0) {
        return false;
    }
    let text = String::from_utf8_lossy(data);
    if text.contains("-----BEGIN PRIVATE KEY-----")
        || text.contains("-----BEGIN RSA PRIVATE KEY-----")
        || text.contains("-----BEGIN EC PRIVATE KEY-----")
        || text.contains("-----BEGIN OPENSSH PRIVATE KEY-----")
    {
        return true;
    }
    text.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| {
            ["ghp_", "gho_", "ghu_", "ghs_", "github_pat_"]
                .iter()
                .any(|prefix| {
                    token
                        .strip_prefix(prefix)
                        .is_some_and(|suffix| suffix.len() >= 20)
                })
                || token.strip_prefix("AKIA").is_some_and(|suffix| {
                    suffix.len() == 16
                        && suffix
                            .chars()
                            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
                })
                || token
                    .strip_prefix("sk-")
                    .is_some_and(|suffix| suffix.len() >= 24)
        })
}

pub(crate) fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn valid_name(value: &str) -> bool {
    let Some(first) = value.as_bytes().first() else {
        return false;
    };
    value.len() <= 64
        && (first.is_ascii_lowercase() || first.is_ascii_digit())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_version(value: &str) -> bool {
    let core = value.split_once('-').map_or(
        value,
        |(head, tail)| if tail.is_empty() { "" } else { head },
    );
    let mut parts = core.split('.');
    let numeric = (0..3).all(|_| {
        parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    }) && parts.next().is_none();
    numeric
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
}

pub(crate) fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn message(value: impl Into<String>) -> LoomError {
    LoomError::Message(value.into())
}

#[cfg(test)]
mod tests {
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
}
