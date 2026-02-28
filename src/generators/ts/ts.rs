use super::Generator;
use crate::parser;
use std::fs;
use std::io;
use std::path::Path;

mod prepare_model_data;
mod convert;
mod names;
mod render;
mod utils;

/// TypeScript backend entry point.
pub struct TsGenerator;

impl Generator for TsGenerator {
    /// Renders and writes the TypeScript schema file to disk.
    fn generate(
        &self,
        requests: &[parser::Request],
        models: &std::collections::BTreeMap<String, parser::SchemaType>,
        output: &Path,
    ) -> io::Result<()> {
        let file_content = render::render_schema_file(requests, models);
        fs::write(output, file_content)
    }
}

