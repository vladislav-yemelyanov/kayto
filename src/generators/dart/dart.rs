use super::Generator;
use crate::parser;
use std::fs;
use std::io;
use std::path::Path;

mod typing;
mod identifiers;
mod model_catalog;
mod assemble;
mod format;

/// Dart backend entry point.
pub struct DartGenerator;

impl Generator for DartGenerator {
    /// Builds and writes the Dart schema file to disk.
    fn generate(
        &self,
        requests: &[parser::Request],
        models: &std::collections::BTreeMap<String, parser::SchemaType>,
        output: &Path,
    ) -> io::Result<()> {
        let file_content = assemble::build_schema_file(requests, models);
        fs::write(output, file_content)
    }
}

