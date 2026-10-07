//! Error types of `rust-template-core`.

use miette::Diagnostic;
use thiserror::Error;

/// Errors returned by this crate.
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    /// The given name was empty or whitespace only.
    #[error("name must not be empty")]
    #[diagnostic(code(rust_template_core::empty_name), help("pass a name, e.g. `world`"))]
    EmptyName,
}

/// Result type of this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;
