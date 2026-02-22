/// Quotes and escapes a string for safe usage as a Dart string literal.
pub fn dart_quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}

/// Applies inline indentation to every new line in a rendered fragment.
pub fn indent_inline(value: &str, prefix: &str) -> String {
    value.replace('\n', &format!("\n{prefix}"))
}
