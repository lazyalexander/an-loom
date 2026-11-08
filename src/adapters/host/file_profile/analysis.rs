use crate::{Result, fail, manifest::sha256_hex};
use serde::Serialize;
use std::collections::BTreeSet;
use syn::visit::Visit;
#[derive(Debug, Serialize)]
pub struct Analysis {
    pub schema_version: u8,
    pub source_sha256: String,
    pub observed_paths: BTreeSet<String>,
    pub unknowns: Vec<String>,
}

#[derive(Default)]
struct Evidence {
    paths: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for Evidence {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let value = path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        self.paths.insert(value);
        syn::visit::visit_path(self, path);
    }
}

pub fn analyze(source: &[u8]) -> Result<Analysis> {
    let text = std::str::from_utf8(source).map_err(|_| fail("Rust source must be UTF-8"))?;
    let ast = syn::parse_file(text).map_err(|e| fail(format!("Rust syntax: {e}")))?;
    let mut evidence = Evidence::default();
    evidence.visit_file(&ast);
    Ok(Analysis {
        schema_version: 1,
        source_sha256: sha256_hex(source),
        observed_paths: evidence.paths,
        unknowns: vec![
            "Native behavior, aliases, macros and dependencies are not proven by this syntax scan.".into(),
            "Source/binary correspondence requires a trusted build; hashes do not attest a compiler.".into(),
        ],
    })
}
