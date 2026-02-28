/// Builds a TypeScript object literal type from formatted field fragments.
pub fn object_type(fields: Vec<String>) -> String {
    if fields.is_empty() {
        "{}".to_string()
    } else {
        let mut out = String::new();
        out.push_str("{\n");

        for field in fields {
            let indented = indent_block(&field, "  ");
            out.push_str(&indented);
            out.push_str(";\n");
        }

        out.push('}');
        out
    }
}

/// Quotes and escapes a string for safe usage as a TS string literal.
pub fn quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Applies inline indentation to every new line in a rendered type fragment.
pub fn indent_multiline(value: &str, prefix: &str) -> String {
    value.replace('\n', &format!("\n{prefix}"))
}

/// Indents each line in a multi-line fragment using the provided prefix.
fn indent_block(value: &str, prefix: &str) -> String {
    value
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
