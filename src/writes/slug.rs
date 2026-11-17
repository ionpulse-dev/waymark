//! File names from titles (see `specs/features/write-documents`).

use crate::format::id;
use sha2::{Digest, Sha256};

const MAX_LEN: usize = 60;

/// The file name stem of a title: lowercase ASCII letters and digits, joined by `-`,
/// at most 60 characters, cut at a `-`. An empty result is `document`.
pub fn stem(title: &str) -> String {
    let s = id::slug(title);
    let s = if s.len() > MAX_LEN {
        let head = &s[..MAX_LEN];
        // Cut at the last `-` that still fits, if the cut would split a word.
        if s.as_bytes()[MAX_LEN] == b'-' {
            head.to_string()
        } else {
            match head.rfind('-') {
                Some(i) => head[..i].to_string(),
                None => head.to_string(),
            }
        }
    } else {
        s
    };
    if s.is_empty() {
        "document".to_string()
    } else {
        s
    }
}

/// A file name that the caller chose (see `document-names`): lowercase letters, digits, and `-`,
/// not starting with `-`, at most 60 characters, no `/`, no `.md`.
pub fn check_name(name: &str) -> crate::error::Result<String> {
    let n = name;
    let ok = !n.is_empty()
        && n.len() <= MAX_LEN
        && !n.starts_with('-')
        && n.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if ok {
        Ok(n.to_string())
    } else {
        Err(crate::error::Error::invalid(format!(
            "\"{name}\" is not a valid name."
        ))
        .with_next("Use lowercase letters, digits, and -, at most 60 characters, without .md, such as \"readme\"."))
    }
}

fn hex_of(folder: &str, title: &str, len: usize) -> String {
    let mut h = Sha256::new();
    h.update(folder.as_bytes());
    h.update(b"\n");
    h.update(title.as_bytes());
    let digest = h.finalize();
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    hex[..len].to_string()
}

/// The ID of a new document. If the name is taken, `-<4 hex>` is added, then 6, then 8 hex.
/// `exists` says whether an ID is taken. `None` if all four names are taken.
pub fn new_id(folder: &str, title: &str, exists: &dyn Fn(&str) -> bool) -> Option<String> {
    let stem = stem(title);
    let join = |name: &str| {
        if folder.is_empty() {
            name.to_string()
        } else {
            format!("{folder}/{name}")
        }
    };
    let first = join(&stem);
    if !exists(&first) {
        return Some(first);
    }
    for len in [4, 6, 8] {
        let candidate = join(&format!("{stem}-{}", hex_of(folder, title, len)));
        if !exists(&candidate) {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_is_lowercase_ascii_with_dashes() {
        assert_eq!(
            stem("Payment API v3 uses idempotency keys"),
            "payment-api-v3-uses-idempotency-keys"
        );
        assert_eq!(stem("  Hello,   World!! "), "hello-world");
        assert_eq!(stem("Über Größe"), "ueber-groesse");
    }

    #[test]
    fn an_empty_stem_becomes_document() {
        assert_eq!(stem("!!!"), "document");
        assert_eq!(stem(""), "document");
    }

    #[test]
    fn a_long_stem_is_cut_at_a_dash() {
        let title = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu";
        let s = stem(title);
        assert!(s.len() <= 60, "{s}");
        assert!(!s.ends_with('-'));
        assert!(title.replace(' ', "-").starts_with(&s));
        // A single long word is cut hard.
        assert_eq!(stem(&"a".repeat(100)).len(), 60);
    }

    #[test]
    fn a_taken_name_gets_a_hex_suffix_then_longer_ones() {
        let none = |_: &str| false;
        assert_eq!(new_id("ops", "Deploy", &none).unwrap(), "ops/deploy");
        let taken = |id: &str| id == "ops/deploy";
        let id = new_id("ops", "Deploy", &taken).unwrap();
        assert!(id.starts_with("ops/deploy-") && id.len() == "ops/deploy-".len() + 4);
        let first = id.clone();
        let taken2 = move |i: &str| i == "ops/deploy" || i == first;
        let id2 = new_id("ops", "Deploy", &taken2).unwrap();
        assert_eq!(id2.len(), "ops/deploy-".len() + 6);
        assert!(new_id("", "Deploy", &|_: &str| true).is_none());
    }

    #[test]
    fn the_root_folder_has_no_prefix() {
        assert_eq!(new_id("", "Deploy", &|_: &str| false).unwrap(), "deploy");
    }

    #[test]
    fn a_chosen_name_is_checked() {
        for good in ["readme", "a1-b2", "x", &"a".repeat(60)] {
            assert_eq!(check_name(good).unwrap(), good);
        }
        for bad in [
            "",
            "Readme",
            "a/b",
            "readme.md",
            "-a",
            "a b",
            "ü",
            &"a".repeat(61),
        ] {
            assert!(check_name(bad).is_err(), "{bad}");
        }
    }
}
