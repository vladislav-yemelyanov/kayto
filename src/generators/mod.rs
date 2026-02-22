#[path = "dart/dart.rs"]
pub mod dart;
#[path = "ts/ts.rs"]
pub mod ts;

use crate::parser;
use std::io;
use std::path::Path;

/// Unified generator contract for language backends.
pub trait Generator {
    /// Generates language-specific artifacts from parsed requests into the target output path.
    fn generate(&self, requests: &[parser::Request], output: &Path) -> io::Result<()>;
}
