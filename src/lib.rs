//! Pure project and evidence contracts with explicit filesystem and host adapters.
pub mod adapters;
pub mod build_artifacts;
mod cli;
mod files;
mod manifest;
pub mod package;
pub mod plan;
pub mod project;
pub mod release;
pub mod spool;

pub use manifest::{BundleManifest, Manifest, PackageIdentity, PayloadFile};
pub use package::{VerifiedArchive, read_archive};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoomError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    TomlWrite(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, LoomError>;

pub(crate) fn fail(message: impl Into<String>) -> LoomError {
    LoomError::Message(message.into())
}

pub fn run(args: impl IntoIterator<Item = String>) -> Result<()> {
    cli::run(args.into_iter().collect())
}
