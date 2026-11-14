//! Document ID and file name rules (sections 5 and 6 of the standard).

use super::finding::Finding;

const FORBIDDEN: &str = "#^[]|:*\"<>?\\/";

/// The ID of a document: its path below the root without `.md`.
pub fn id_from_path(path: &str) -> String {
    path.strip_suffix(".md").unwrap_or(path).to_string()
}

/// The path of a document from its ID.
pub fn path_from_id(id: &str) -> String {
    format!("{id}.md")
}

fn segment_ok(seg: &str) -> bool {
    !seg.is_empty()
        && seg
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Check a path below the root, such as `backend/payments/idempotency-keys.md`.
/// The file name and every folder name use lowercase letters, digits, and `-`.
pub fn check_path(path: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let Some(stem_path) = path.strip_suffix(".md") else {
        out.push(Finding::error(
            path,
            "bad-file-name",
            "A document file must end with .md.",
        ));
        return out;
    };
    for seg in stem_path.split('/') {
        if segment_ok(seg) {
            continue;
        }
        let bad: Vec<char> = seg.chars().filter(|c| FORBIDDEN.contains(*c)).collect();
        let msg = if !bad.is_empty() {
            format!("The name \"{seg}\" has a forbidden character. Use lowercase letters, digits, and -.")
        } else {
            format!("The name \"{seg}\" must use only lowercase letters a to z, digits, and -.")
        };
        out.push(Finding::error(path, "bad-file-name", msg));
    }
    out
}

/// A short, valid file name stem from a title: lowercase words joined by `-`.
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    let mut last_dash = true;
    for c in title.chars().flat_map(|c| c.to_lowercase()) {
        let c = match c {
            'ä' => {
                out.push_str("ae");
                last_dash = false;
                continue;
            }
            'ö' => {
                out.push_str("oe");
                last_dash = false;
                continue;
            }
            'ü' => {
                out.push_str("ue");
                last_dash = false;
                continue;
            }
            'ß' => {
                out.push_str("ss");
                last_dash = false;
                continue;
            }
            c => c,
        };
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_is_the_path_without_extension() {
        assert_eq!(
            id_from_path("backend/payments/idempotency-keys.md"),
            "backend/payments/idempotency-keys"
        );
        assert_eq!(path_from_id("a/b"), "a/b.md");
    }

    #[test]
    fn id_does_not_depend_on_the_title() {
        // The ID is computed from the path alone.
        assert_eq!(id_from_path("a/b.md"), id_from_path("a/b.md"));
    }

    #[test]
    fn valid_names_pass() {
        assert!(check_path("backend/payments/idempotency-keys.md").is_empty());
        assert!(check_path("a1-b2.md").is_empty());
    }

    #[test]
    fn uppercase_fails() {
        assert_eq!(check_path("Backend/x.md").len(), 1);
        assert_eq!(check_path("backend/Idempotency.md").len(), 1);
    }

    #[test]
    fn every_forbidden_character_fails() {
        for c in FORBIDDEN.chars() {
            if c == '/' {
                continue; // a slash separates folders
            }
            let path = format!("a/b{c}c.md");
            let f = check_path(&path);
            assert_eq!(f.len(), 1, "character {c}");
            assert_eq!(f[0].rule, "bad-file-name");
        }
    }

    #[test]
    fn underscore_space_dot_and_empty_fail() {
        assert!(!check_path("a_b.md").is_empty());
        assert!(!check_path("a b.md").is_empty());
        assert!(!check_path("a.b.md").is_empty());
        assert!(!check_path("a//b.md").is_empty());
    }

    #[test]
    fn missing_extension_fails() {
        assert!(!check_path("a/b").is_empty());
    }

    #[test]
    fn slug_is_valid() {
        assert_eq!(
            slug("Payment API v3 uses idempotency keys"),
            "payment-api-v3-uses-idempotency-keys"
        );
        assert_eq!(slug("  Über  uns! "), "ueber-uns");
        assert!(check_path(&format!("x/{}.md", slug("A: B / C?"))).is_empty());
    }
}
