use std::collections::{BTreeMap, BTreeSet, HashSet};

/// Creates stable and unique Dart identifiers for schema registry keys.
pub fn build_model_identifiers(model_names: &BTreeSet<String>) -> BTreeMap<String, String> {
    let mut used: HashSet<String> = HashSet::new();
    let mut identifiers = BTreeMap::new();

    for name in model_names {
        let base = sanitize_type_name(name);
        let mut candidate = base.clone();
        let mut n = 2usize;

        while used.contains(&candidate) {
            candidate = format!("{base}{n}");
            n += 1;
        }

        used.insert(candidate.clone());
        identifiers.insert(name.clone(), candidate);
    }

    identifiers
}

/// Normalizes arbitrary schema names into valid UpperCamelCase Dart type names.
fn sanitize_type_name(value: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch);
            continue;
        }

        if !current.is_empty() {
            parts.push(current);
            current = String::new();
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    let mut out = String::new();
    for part in parts {
        let mut chars = part.chars();
        let Some(first) = chars.next() else {
            continue;
        };
        out.push(first.to_ascii_uppercase());
        out.extend(chars);
    }

    if out.is_empty() {
        return "Model".to_string();
    }

    if out
        .chars()
        .next()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(false)
    {
        return format!("Model{out}");
    }

    out
}

