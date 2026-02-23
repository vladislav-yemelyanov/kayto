/// Quotes and escapes a string for safe usage as a Dart string literal.
pub fn dart_quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}
