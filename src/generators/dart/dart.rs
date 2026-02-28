use super::Generator;
use crate::parser;
use std::fs;
use std::io;
use std::path::Path;

mod convert;
mod names;
mod prepare_model_data;
mod render;
mod utils;

/// Dart backend entry point.
pub struct DartGenerator;

impl Generator for DartGenerator {
    /// Renders and writes the Dart schema file to disk.
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

