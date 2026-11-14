//! Title checks: duplicates and near-duplicates (see `specs/features/write-documents`).

use std::collections::BTreeSet;

/// The words of a title: runs of letters and digits, lowercase.
pub fn words(title: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in title.chars() {
        if c.is_alphanumeric() {
            cur.extend(c.to_lowercase());
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clash {
    /// The same words in the same order.
    Exact(String),
    /// At least 80 percent of the words of the shorter title are shared.
    Near(String),
}

/// How close two titles are. Exact match, near match, or none.
pub fn compare(a: &str, b: &str) -> Option<bool> {
    let wa = words(a);
    let wb = words(b);
    if wa.is_empty() || wb.is_empty() {
        return None;
    }
    if wa == wb {
        return Some(true);
    }
    let sa: BTreeSet<&String> = wa.iter().collect();
    let sb: BTreeSet<&String> = wb.iter().collect();
    let shorter = sa.len().min(sb.len());
    if shorter < 3 {
        return None;
    }
    let shared = sa.intersection(&sb).count();
    // shared / shorter >= 0.8, in integers.
    if shared * 10 >= shorter * 8 {
        Some(false)
    } else {
        None
    }
}

/// All active documents whose title clashes with `title`. `docs` holds (ID, title) of active
/// documents. IDs in `skip` are ignored.
pub fn clashes(title: &str, docs: &[(String, String)], skip: &[&str]) -> Vec<Clash> {
    let mut out = Vec::new();
    for (id, other) in docs {
        if skip.contains(&id.as_str()) {
            continue;
        }
        match compare(title, other) {
            Some(true) => out.push(Clash::Exact(id.clone())),
            Some(false) => out.push(Clash::Near(id.clone())),
            None => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_are_lowercase_runs() {
        assert_eq!(
            words("Deploy: the API (v3)!"),
            vec!["deploy", "the", "api", "v3"]
        );
    }

    #[test]
    fn same_words_in_the_same_order_is_exact() {
        assert_eq!(compare("Deploy the API", "deploy,  THE api"), Some(true));
    }

    #[test]
    fn the_boundary_is_80_percent_of_the_shorter_title() {
        // 4 of 5 shared words: 0.8, a near duplicate.
        assert_eq!(
            compare(
                "one two three four five",
                "one two three four six seven eight"
            ),
            Some(false)
        );
        // 3 of 5: 0.6, no clash.
        assert_eq!(
            compare("one two three four five", "one two three x y z"),
            None
        );
    }

    #[test]
    fn a_title_of_fewer_than_three_words_matches_only_exactly() {
        assert_eq!(compare("Deploy API", "Deploy API guide for the team"), None);
        assert_eq!(compare("Deploy API", "api deploy"), None);
        assert_eq!(compare("Deploy API", "Deploy API"), Some(true));
    }

    #[test]
    fn skipped_documents_are_ignored() {
        let docs = vec![
            ("ops/a".to_string(), "Deploy the API now".to_string()),
            ("ops/b".to_string(), "Deploy the API now".to_string()),
        ];
        assert_eq!(
            clashes("Deploy the API now", &docs, &["ops/a"]),
            vec![Clash::Exact("ops/b".into())]
        );
    }
}
