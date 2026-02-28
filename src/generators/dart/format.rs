/// Quotes and escapes a string for safe usage as a Dart string literal.
pub fn quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}
