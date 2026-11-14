//! Search: all terms must occur, and parts have fixed weights.

use super::Doc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Path,
    Title,
    Description,
    Tags,
    Body,
}

impl Part {
    pub const ALL: [Part; 5] = [
        Part::Path,
        Part::Title,
        Part::Description,
        Part::Tags,
        Part::Body,
    ];

    pub fn parse(s: &str) -> Option<Part> {
        Some(match s {
            "path" => Part::Path,
            "title" => Part::Title,
            "description" => Part::Description,
            "tags" => Part::Tags,
            "body" => Part::Body,
            _ => return None,
        })
    }

    fn weight(self) -> u32 {
        match self {
            Part::Title => 5,
            Part::Tags => 3,
            Part::Path | Part::Description => 2,
            Part::Body => 1,
        }
    }

    fn text(self, d: &Doc) -> String {
        match self {
            Part::Path => d.id.to_lowercase(),
            Part::Title => d.title().to_lowercase(),
            Part::Description => d.description().to_lowercase(),
            Part::Tags => d.tags().join(" ").to_lowercase(),
            Part::Body => d.body.to_lowercase(),
        }
    }
}

pub fn terms(query: &str) -> Vec<String> {
    query.split_whitespace().map(str::to_lowercase).collect()
}

/// The score of a document, or `None` if one term is in none of the parts.
pub fn score(d: &Doc, terms: &[String], parts: &[Part]) -> Option<u32> {
    let texts: Vec<(Part, String)> = parts.iter().map(|p| (*p, p.text(d))).collect();
    let mut total = 0;
    for t in terms {
        let s: u32 = texts
            .iter()
            .filter(|(_, text)| text.contains(t.as_str()))
            .map(|(p, _)| p.weight())
            .sum();
        if s == 0 {
            return None;
        }
        total += s;
    }
    Some(total)
}

/// Sort by score (high first), then `updated` (new first), then `id`.
pub fn sort(hits: &mut [(&Doc, u32)]) {
    hits.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.0.updated().cmp(a.0.updated()))
            .then_with(|| a.0.id.cmp(&b.0.id))
    });
}

/// About 25 words of the body around the first match. `...` marks a cut end.
pub fn snippet(d: &Doc, terms: &[String]) -> String {
    const SIDE: usize = 12;
    let words: Vec<&str> = d.body.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }
    let at = words
        .iter()
        .position(|w| {
            let lw = w.to_lowercase();
            terms.iter().any(|t| lw.contains(t.as_str()))
        })
        .unwrap_or(0);
    let start = at.saturating_sub(SIDE);
    let end = (at + SIDE + 1).min(words.len());
    let mut out = String::new();
    if start > 0 {
        out.push_str("... ");
    }
    out.push_str(&words[start..end].join(" "));
    if end < words.len() {
        out.push_str(" ...");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::parse;

    fn doc(id: &str, title: &str, tags: &str, body: &str) -> Doc {
        let text = format!(
            "---\ntitle: {title}\ndescription: Desc\nstatus: active\nupdated: 2026-01-01\ntags: [{tags}]\n---\n\n{body}\n"
        );
        let p = parse(&text).unwrap();
        Doc {
            id: id.into(),
            raw: text,
            front: Some(p.frontmatter),
            body: p.body,
        }
    }

    #[test]
    fn every_term_must_occur_in_some_part() {
        let d = doc("ops/deploy", "How we deploy", "", "Use the pipeline.");
        assert!(score(&d, &terms("deploy pipeline"), &Part::ALL).is_some());
        assert!(score(&d, &terms("deploy missing"), &Part::ALL).is_none());
        assert!(score(&d, &terms("DEPLOY"), &Part::ALL).is_some());
    }

    #[test]
    fn weights_put_titles_first() {
        let a = doc("a", "Other", "", "keys keys");
        let b = doc("b", "Idempotency keys", "", "x");
        let ta = score(&a, &terms("keys"), &Part::ALL).unwrap();
        let tb = score(&b, &terms("keys"), &Part::ALL).unwrap();
        assert!(tb > ta);
    }

    #[test]
    fn the_searched_parts_can_be_narrowed() {
        let d = doc("a", "Title", "", "needle");
        assert!(score(&d, &terms("needle"), &[Part::Title]).is_none());
        assert!(score(&d, &terms("needle"), &[Part::Body]).is_some());
    }

    #[test]
    fn ties_go_to_the_newer_update_then_the_id() {
        let mut a = doc("b", "T", "", "x");
        let b = doc("a", "T", "", "x");
        a.front.as_mut().unwrap().set_date("updated", "2026-05-05");
        let mut hits = vec![(&b, 1), (&a, 1)];
        sort(&mut hits);
        assert_eq!(hits[0].0.id, "b");
        let c = doc("c", "T", "", "x");
        let mut hits = vec![(&c, 1), (&b, 1)];
        sort(&mut hits);
        assert_eq!(hits[0].0.id, "a");
    }

    #[test]
    fn the_snippet_cuts_around_the_match() {
        let body: Vec<String> = (0..60).map(|i| format!("w{i}")).collect();
        let mut text = body.join(" ");
        text.push_str(" needle ");
        text.push_str(&body.join(" "));
        let d = doc("a", "T", "", &text);
        let s = snippet(&d, &terms("needle"));
        assert!(s.starts_with("... ") && s.ends_with(" ..."));
        assert!(s.contains("needle"));
        assert!(s.split_whitespace().count() <= 27);
    }
}
