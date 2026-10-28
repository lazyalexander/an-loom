//! Export a bounded native-spool contract. Authoring records are not executed.
use crate::{
    Result, fail,
    manifest::{sha256_hex, validate_payload_path},
    package,
    project::Project,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

const SCHEMA: &str = "an-native-spool-v1";
/// The only runtime this adapter qualifies. Other labels are refused.
const RUNTIME: &str = "macos-native-process-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Native {
    schema: String,
    #[serde(skip)]
    legacy: bool,
    #[serde(default)]
    runtime: String,
    entry: String,
    silk: Vec<String>,
    kit: Vec<String>,
    #[serde(default)]
    editable: Vec<String>,
    #[serde(default)]
    inputs: Vec<String>,
    outputs: Vec<String>,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    acceptance: Vec<String>,
}

/// Read the authoring contract. Callers must not spawn `acceptance` or fetch `dependencies`.
pub(crate) fn validate(files: &BTreeMap<&str, &[u8]>) -> Result<()> {
    load(files).map(|_| ())
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Authoring {
    pub entry: String,
    pub runtime: String,
    pub silk: Vec<String>,
    pub kit: Vec<String>,
    pub editable: Vec<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
    pub acceptance: Vec<String>,
}

pub(crate) fn authoring(files: &BTreeMap<&str, &[u8]>) -> Result<Authoring> {
    let native = load(files)?;
    if native.legacy {
        return Err(fail(
            "legacy native release has no authoring contract; add explicit metadata before creating a task",
        ));
    }
    Ok(Authoring {
        entry: native.entry,
        runtime: native.runtime,
        silk: native.silk,
        kit: native.kit,
        editable: native.editable,
        inputs: native.inputs,
        outputs: native.outputs,
        capabilities: native.capabilities,
        dependencies: native.dependencies,
        acceptance: native.acceptance,
    })
}

fn load(files: &BTreeMap<&str, &[u8]>) -> Result<Native> {
    let bytes = files
        .get("native-spool.toml")
        .copied()
        .ok_or_else(|| fail("missing native-spool.toml"))?;
    let raw: toml::Value = toml::from_str(
        std::str::from_utf8(bytes).map_err(|_| fail("native contract must be UTF-8"))?,
    )?;
    let fields = [
        "runtime",
        "editable",
        "inputs",
        "capabilities",
        "dependencies",
        "acceptance",
    ];
    let count = fields.iter().filter(|key| raw.get(**key).is_some()).count();
    if count != 0 && count != fields.len() {
        return Err(fail(
            "partial native authoring metadata; provide all authoring fields",
        ));
    }
    let mut config: Native = toml::from_str(
        std::str::from_utf8(bytes).map_err(|_| fail("native contract must be UTF-8"))?,
    )?;
    config.legacy = count == 0;
    interpret(&config, files)?;
    Ok(config)
}

fn interpret(config: &Native, files: &BTreeMap<&str, &[u8]>) -> Result<()> {
    if config.schema != SCHEMA {
        return Err(fail("unsupported native spool contract"));
    }
    if config.legacy {
        roles(config, files)?;
        return slots(&config.outputs, "invalid output slot");
    }
    if config.runtime != RUNTIME {
        return Err(fail("native runtime is not qualified for this adapter"));
    }
    roles(config, files)?;
    if config.inputs.is_empty() {
        return Err(fail("missing native input"));
    }
    slots(&config.inputs, "invalid input slot")?;
    slots(&config.outputs, "invalid output slot")?;
    editable(config)?;
    records(
        &config.capabilities,
        "invalid capability request",
        identifier,
    )?;
    records(
        &config.dependencies,
        "native dependency must be an unresolved label",
        dependency_label,
    )?;
    if config.acceptance.is_empty() {
        return Err(fail("missing acceptance command record"));
    }
    records(
        &config.acceptance,
        "invalid acceptance command record",
        command_record,
    )?;
    domain_description(files)
}

fn roles(config: &Native, files: &BTreeMap<&str, &[u8]>) -> Result<()> {
    if config.silk.len() + config.kit.len() > 12 {
        return Err(fail("native code file limit exceeded"));
    }
    let silk = unique(&config.silk)?;
    let kit = unique(&config.kit)?;
    if silk.intersection(&kit).next().is_some() {
        return Err(fail("native role conflict"));
    }
    if !silk.contains(config.entry.as_str()) || !files.contains_key(config.entry.as_str()) {
        return Err(fail("missing native entry"));
    }
    let mut names = BTreeSet::new();
    let mut total = 0usize;
    for (role, paths) in [("silk", &config.silk), ("kit", &config.kit)] {
        for path in paths {
            validate_payload_path(path)?;
            let name = path
                .strip_prefix(&format!("{role}/"))
                .ok_or_else(|| fail("code role prefix mismatch"))?;
            if !identifier(name) {
                return Err(fail("invalid or duplicate code basename"));
            }
            if !names.insert(name) {
                return Err(fail("native role conflict"));
            }
            let bytes = files
                .get(path.as_str())
                .copied()
                .ok_or_else(|| fail("native code not in selected archive"))?;
            total += bytes.len();
            if bytes.len() > 1024 * 1024 || total > 8 * 1024 * 1024 {
                return Err(fail("native code size exceeded"));
            }
        }
    }
    for path in files.keys() {
        if !config.legacy
            && (path.starts_with("silk/") || path.starts_with("kit/"))
            && !silk.contains(path)
            && !kit.contains(path)
        {
            return Err(fail(format!("undeclared native code file: {path}")));
        }
    }
    Ok(())
}

fn unique(paths: &[String]) -> Result<BTreeSet<&str>> {
    let mut set = BTreeSet::new();
    for path in paths {
        if !set.insert(path.as_str()) {
            return Err(fail("native role conflict"));
        }
    }
    Ok(set)
}

fn slots(paths: &[String], message: &str) -> Result<()> {
    if paths.is_empty() || paths.len() > 16 {
        return Err(fail(message));
    }
    let mut seen = BTreeSet::new();
    for path in paths {
        if !identifier(path)
            || matches!(path.as_str(), "kit" | "tmp")
            || !seen.insert(path.as_str())
        {
            return Err(fail(message));
        }
    }
    Ok(())
}

fn editable(config: &Native) -> Result<()> {
    let mut seen = BTreeSet::new();
    for path in &config.editable {
        validate_payload_path(path)?;
        if !seen.insert(path.as_str()) {
            return Err(fail("native role conflict"));
        }
        if !config.kit.iter().any(|kit| kit == path) {
            return Err(fail("editable path is outside the kit role"));
        }
    }
    Ok(())
}

fn records(values: &[String], message: &str, accept: fn(&str) -> bool) -> Result<()> {
    if values.len() > 16 {
        return Err(fail(message));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if !accept(value) || !seen.insert(value.as_str()) {
            return Err(fail(message));
        }
    }
    Ok(())
}

fn domain_description(files: &BTreeMap<&str, &[u8]>) -> Result<()> {
    let bytes = files
        .get("loom.toml")
        .copied()
        .ok_or_else(|| fail("native project is missing loom.toml"))?;
    let project =
        Project::parse(std::str::from_utf8(bytes).map_err(|_| fail("project must be UTF-8"))?)?;
    match project.description.as_deref() {
        Some(description) if !description.trim().is_empty() => Ok(()),
        _ => Err(fail("native project needs a domain description")),
    }
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}

fn dependency_label(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value.bytes().any(|c| c.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._+-".contains(&c))
}

fn command_record(value: &str) -> bool {
    (1..=256).contains(&value.len()) && value.bytes().all(|c| (0x20..0x7f).contains(&c))
}

pub fn export(archive: &Path, destination: &Path) -> Result<Value> {
    let verified = crate::read_archive(archive)?;
    let selected = verified
        .files
        .iter()
        .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
        .collect();
    let config = load(&selected)?;
    let mut payload = BTreeMap::new();
    let mut hashes = BTreeMap::new();
    for paths in [&config.silk, &config.kit] {
        for path in paths {
            let bytes = verified
                .files
                .get(path)
                .ok_or_else(|| fail("native code not in selected archive"))?;
            hashes.insert(path.clone(), sha256_hex(bytes));
            payload.insert(path.clone(), bytes);
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
