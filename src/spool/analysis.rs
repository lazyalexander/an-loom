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
    incompatible: bool,
    file_profile: bool,
}

impl<'ast> Visit<'ast> for Evidence {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let value = path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        if ["std::fs", "std::net", "std::process", "std::os", "libc"]
            .iter()
            .any(|p| value == *p || value.starts_with(&format!("{p}::")))
            && !(self.file_profile && (value == "std::fs" || value.starts_with("std::fs::")))
        {
            self.incompatible = true;
        }
        self.paths.insert(value);
        syn::visit::visit_path(self, path);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.incompatible = true;
        syn::visit::visit_expr_unsafe(self, node);
    }

    fn visit_item_foreign_mod(&mut self, _node: &'ast syn::ItemForeignMod) {
        self.incompatible = true;
    }
}

pub fn analyze(source: &[u8]) -> Result<Analysis> {
    analyze_profile(source, true)
}

fn analyze_profile(source: &[u8], file: bool) -> Result<Analysis> {
    let text = std::str::from_utf8(source).map_err(|_| fail("Rust source must be UTF-8"))?;
    let ast = syn::parse_file(text).map_err(|e| fail(format!("Rust syntax: {e}")))?;
    let mut evidence = Evidence {
        file_profile: file,
        ..Evidence::default()
    };
    evidence.visit_file(&ast);
    if evidence.incompatible {
        return Err(fail(
            "observed source operation is incompatible with the selected CLI profile",
        ));
    }
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
