//! Export a bounded native-spool contract; no host runtime or execution grants.
use crate::{
    Result, fail,
    manifest::{sha256_hex, validate_payload_path},
    package,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Native {
    schema: String,
    entry: String,
    silk: Vec<String>,
    kit: Vec<String>,
    outputs: Vec<String>,
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s != "."
        && s != ".."
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}

pub fn export(archive: &Path, destination: &Path) -> Result<Value> {
    let verified = crate::read_archive(archive)?;
    let config = verified
        .files
        .get("native-spool.toml")
        .ok_or_else(|| fail("missing native-spool.toml"))?;
    let config: Native = toml::from_str(
        std::str::from_utf8(config).map_err(|_| fail("native contract must be UTF-8"))?,
    )?;
    if config.schema != "an-native-spool-v1"
        || !config.silk.contains(&config.entry)
        || config.silk.len() + config.kit.len() > 12
        || config.outputs.is_empty()
        || config.outputs.len() > 16
    {
        return Err(fail("unsupported native spool contract"));
    }
    let mut payload = BTreeMap::new();
    let mut hashes = BTreeMap::new();
    let mut names = BTreeSet::new();
    let mut total = 0;
    for (role, paths) in [("silk", &config.silk), ("kit", &config.kit)] {
        for path in paths {
            validate_payload_path(path)?;
            let name = path
                .strip_prefix(&format!("{role}/"))
                .ok_or_else(|| fail("code role prefix mismatch"))?;
            if !identifier(name) || !names.insert(name) {
                return Err(fail("invalid or duplicate code basename"));
            }
            let bytes = verified
                .files
                .get(path)
                .ok_or_else(|| fail("native code not in selected archive"))?;
            total += bytes.len();
            if bytes.len() > 1024 * 1024 || total > 8 * 1024 * 1024 {
                return Err(fail("native code size exceeded"));
            }
            hashes.insert(path.clone(), sha256_hex(bytes));
            payload.insert(path.clone(), bytes);
        }
    }
    let mut outputs = BTreeSet::new();
    for path in &config.outputs {
        if !identifier(path) || matches!(path.as_str(), "kit" | "tmp") || !outputs.insert(path) {
            return Err(fail("invalid output slot"));
        }
    }
    let manifest = json!({"schema":config.schema,"name":verified.manifest.package.name,
        "version":verified.manifest.package.version,"entry":config.entry,"files":hashes,"outputs":config.outputs});
    let bytes = serde_json::to_vec_pretty(&manifest)?;
    package::new_directory(destination)?;
    for role in ["silk", "kit"] {
        package::new_directory(&destination.join(role))?;
    }
    for (path, data) in payload {
        package::write_new(&destination.join(path), data)?;
    }
    package::write_new(&destination.join("native-spool.json"), &bytes)?;
    Ok(
        json!({"schema":"an-loom-native-export-v1","release":sha256_hex(&bytes),
        "source_bundle":verified.manifest.release_digest,"execution_granted":false}),
    )
}
