//! Authoring metadata is data: it contains no executable commands or host authority.
use crate::{Manifest, Result, fail, manifest::validate_payload_path};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const FILE: &str = "loom.toml";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub schema_version: u8,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub include: Vec<String>,
    /// Legacy Rust field; new authoring output uses `[silk]`. Both keys cannot coexist.
    #[serde(rename = "silk", alias = "kit")]
    pub kit: Option<Silk>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Silk {
    pub source: String,
    pub program: String,
    pub build_record: String,
    pub declaration: String,
    pub analysis: String,
    pub profile: String,
}

/// Source compatibility for callers using the former authoring type.
pub type Kit = Silk;

impl Silk {
    pub fn files(&self) -> [&str; 5] {
        [
            &self.source,
            &self.program,
            &self.build_record,
            &self.declaration,
            &self.analysis,
        ]
    }
}

impl Project {
    pub fn parse(text: &str) -> Result<Self> {
        if text.len() > 1024 * 1024 {
            return Err(fail("project manifest exceeds 1 MiB"));
        }
        let project: Self = toml::from_str(text)?;
        project.validate()?;
        Ok(project)
    }

    pub fn read(root: &Path) -> Result<Self> {
        let bytes = crate::files::read(&root.join(FILE), 1024 * 1024)?;
        Self::parse(std::str::from_utf8(&bytes).map_err(|_| fail("project must be UTF-8"))?)
    }

    pub fn from_manifest(manifest: Manifest) -> Self {
        Self {
            schema_version: manifest.schema_version,
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            include: manifest.include,
            kit: None,
        }
    }

    pub fn manifest(&self) -> Manifest {
        Manifest {
            schema_version: self.schema_version,
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            include: self.include.clone(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.manifest().validate()?;
        if !self.include.iter().any(|path| path == FILE) {
            return Err(fail(
                "include must contain loom.toml so the project contract is identified",
            ));
        }
        for path in &self.include {
            validate_payload_path(path)?;
        }
        if let Some(kit) = &self.kit {
            if kit.profile != "file-kit-v1" {
                return Err(fail("this release adapter supports only file-kit-v1"));
            }
            let paths = kit.files();
            for (index, path) in paths.iter().enumerate() {
                validate_payload_path(path)?;
                if *path == FILE || *path == "bundle.json" || paths[..index].contains(path) {
                    return Err(fail(
                        "kit paths must be distinct and cannot replace package metadata",
                    ));
                }
                if !self
                    .include
                    .iter()
                    .any(|include| path == include || path.starts_with(&format!("{include}/")))
                {
                    return Err(fail(
                        "every kit binding must be selected by the package allowlist",
                    ));
                }
                if paths
                    .iter()
                    .any(|other| other.starts_with(&format!("{path}/")))
                {
                    return Err(fail("a kit file cannot be another kit file's ancestor"));
                }
            }
        }
        Ok(())
    }

    pub fn silk(&self) -> Result<&Silk> {
        self.kit
            .as_ref()
            .ok_or_else(|| fail("project has no silk binding"))
    }

    /// Legacy accessor retained for existing integration code.
    pub fn kit(&self) -> Result<&Kit> {
        self.silk()
    }
}

/// A deterministic starter, not a claim that a model generated the program.
pub fn scaffold(target: &Path, file_kit: bool) -> Result<()> {
    fs::create_dir(target)?;
    fs::create_dir(target.join("kit"))?;
    let manifest = Manifest {
        schema_version: 1,
        name: if file_kit {
            "file-list".into()
        } else {
            target
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| fail("invalid project name"))?
                .into()
        },
        version: "0.1.0".into(),
        description: Some("A developer-owned CLI component".into()),
        include: vec![FILE.into(), "README.md".into(), "kit".into()],
    };
    let mut project = Project::from_manifest(manifest);
    if file_kit {
        project.include.push("generation.md".into());
        project.kit = Some(Kit {
            source: "kit/main.rs".into(),
            program: "kit/program".into(),
            build_record: "kit/build.json".into(),
            declaration: "kit/spool.yaml".into(),
            analysis: "kit/analysis.json".into(),
            profile: "file-kit-v1".into(),
        });
    }
    project.validate()?;
    crate::package::write_new(
        &target.join(FILE),
        toml::to_string_pretty(&project)?.as_bytes(),
    )?;
    if file_kit {
        for (path, bytes) in [
            (
                "kit/main.rs",
                include_bytes!("../examples/file-list/kit/main.rs").as_slice(),
            ),
            (
                "README.md",
                include_bytes!("../examples/file-list/README.md").as_slice(),
            ),
            (
                "generation.md",
                include_bytes!("../examples/file-list/generation.md").as_slice(),
            ),
        ] {
            crate::package::write_new(&target.join(path), bytes)?;
        }
    } else {
        crate::package::write_new(
            &target.join("README.md"),
            b"# Spool project\n\nDescribe its internal silks, inputs, outputs and acceptance. The host owns the spool execution boundary.\n",
        )?;
        crate::package::write_new(&target.join("kit/README.md"), b"# Internal silk files\n\nThis legacy directory name is not an independent runtime boundary. Loom packages selected files; it does not run project commands.\n")?;
    }
    Ok(())
}

/// Copy the reviewed native text reference. This writes files and runs nothing.
pub fn scaffold_native(target: &Path) -> Result<()> {
    fs::create_dir(target)?;
    fs::create_dir(target.join("silk"))?;
    fs::create_dir(target.join("kit"))?;
    for (path, bytes) in [
        (
            "AGENTS.md",
            include_bytes!("../examples/native-spool/AGENTS.md").as_slice(),
        ),
        (
            "loom.toml",
            include_bytes!("../examples/native-spool/loom.toml").as_slice(),
        ),
        (
            "native-spool.toml",
            include_bytes!("../examples/native-spool/native-spool.toml").as_slice(),
        ),
        (
            "README.md",
            include_bytes!("../examples/native-spool/README.md").as_slice(),
        ),
        (
            "silk/main.sh",
            include_bytes!("../examples/native-spool/silk/main.sh").as_slice(),
        ),
        (
            "silk/report.sh",
            include_bytes!("../examples/native-spool/silk/report.sh").as_slice(),
        ),
        (
            "kit/transform.sh",
            include_bytes!("../examples/native-spool/kit/transform.sh").as_slice(),
        ),
    ] {
        crate::package::write_new(&target.join(path), bytes)?;
    }
    Ok(())
}
