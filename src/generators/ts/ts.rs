use super::Generator;
use crate::parser;
use std::fs;
use std::io;
use std::path::Path;

mod assemble;
mod format;
mod identifiers;
mod model_catalog;
mod typing;

/// TypeScript backend entry point.
pub struct TsGenerator;

impl Generator for TsGenerator {
    /// Builds and writes the TypeScript schema file to disk.
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
