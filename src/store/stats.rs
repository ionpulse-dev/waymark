//! The metrics of `stats` and the grouping.

use super::Doc;
use crate::format::count;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    Words,
    Tokens,
    Chars,
    Documents,
}

impl Metric {
    pub fn parse(s: &str) -> Option<Metric> {
        Some(match s {
            "words" => Metric::Words,
            "tokens" => Metric::Tokens,
            "chars" => Metric::Chars,
            "documents" => Metric::Documents,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Metric::Words => "words",
            Metric::Tokens => "tokens",
            Metric::Chars => "chars",
            Metric::Documents => "documents",
        }
    }

    pub fn of(self, d: &Doc) -> usize {
        match self {
            Metric::Words => d.words(),
            Metric::Tokens => count::estimated_tokens(d.words()),
            Metric::Chars => d.raw.chars().count(),
            Metric::Documents => 1,
        }
    }
}

/// The group key of a document for `by: folder`: the folder below the first matching base folder.
/// A document that lies directly in the base folder has the base folder as key (`.` for the root).
pub fn folder_key(id: &str, bases: &[String]) -> String {
    let base = bases
        .iter()
        .filter(|b| !b.is_empty() && (id.starts_with(&format!("{b}/"))))
        .max_by_key(|b| b.len())
        .map(String::as_str)
        .unwrap_or("");
    let rest = if base.is_empty() {
        id
    } else {
        &id[base.len() + 1..]
    };
    let join = |seg: &str| {
        if base.is_empty() {
            seg.to_string()
        } else {
            format!("{base}/{seg}")
        }
    };
    match rest.split_once('/') {
        Some((seg, _)) => join(seg),
        None if base.is_empty() => ".".to_string(),
        None => base.to_string(),
    }
}

/// Sum a metric per group. Sorted by value, largest first, then by key.
pub fn group<'a>(
    docs: impl Iterator<Item = &'a Doc>,
    metric: Metric,
    key: &dyn Fn(&Doc) -> String,
) -> Vec<(String, usize, usize)> {
    let mut map: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for d in docs {
        let e = map.entry(key(d)).or_default();
        e.0 += metric.of(d);
        e.1 += 1;
    }
    let mut v: Vec<(String, usize, usize)> = map.into_iter().map(|(k, (v, n))| (k, v, n)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_keys_use_the_first_level_below_the_base() {
        assert_eq!(folder_key("backend/payments/x", &[]), "backend");
        assert_eq!(folder_key("readme", &[]), ".");
        let bases = vec!["backend".to_string()];
        assert_eq!(folder_key("backend/payments/x", &bases), "backend/payments");
        assert_eq!(folder_key("backend/x", &bases), "backend");
    }
}
