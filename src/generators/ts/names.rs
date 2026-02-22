use std::collections::{BTreeMap, BTreeSet, HashSet};

/// Creates stable and unique TS identifiers for schema registry keys.
pub fn build_model_identifiers(model_names: &BTreeSet<String>) -> BTreeMap<String, String> {
    let mut used: HashSet<String> = HashSet::new();
    let mut identifiers = BTreeMap::new();

    for name in model_names {
        let base = sanitize_type_name(name);
        let mut candidate = base.clone();
        let mut n = 2usize;

        while used.contains(&candidate) {
            candidate = format!("{base}_{n}");
            n += 1;
        }

        used.insert(candidate.clone());
        identifiers.insert(name.clone(), candidate);
    }

    identifiers
}

/// Normalizes arbitrary schema names into valid UpperCamelCase TS type names.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;

    /// Verifies snake_case model keys are normalized to UpperCamelCase TS identifiers.
    #[test]
    fn normalizes_to_upper_camel_case() {
        let names = BTreeSet::from_iter(["actions_artifact".to_string()]);
        let id_map = build_model_identifiers(&names);
        assert_eq!(
            id_map.get("actions_artifact"),
            Some(&"ActionsArtifact".to_string())
        );
    }

    /// Verifies names starting with digits are prefixed to remain valid TS identifiers.
    #[test]
    fn prefixes_numeric_leading_identifiers() {
        let names = BTreeSet::from_iter(["123_model".to_string()]);
        let id_map = build_model_identifiers(&names);
        assert_eq!(id_map.get("123_model"), Some(&"Model123Model".to_string()));
    }

    /// Verifies colliding normalized names get deterministic numeric suffixes.
    #[test]
    fn adds_suffix_for_colliding_identifiers() {
        let names = BTreeSet::from_iter(["pet-model".to_string(), "pet_model".to_string()]);
        let id_map = build_model_identifiers(&names);

        assert_eq!(id_map.get("pet-model"), Some(&"PetModel".to_string()));
        assert_eq!(id_map.get("pet_model"), Some(&"PetModel_2".to_string()));
    }
}
