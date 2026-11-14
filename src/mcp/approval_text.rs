//! The text of an approval request (see `write-approval`). No `rmcp` types here.

/// The body of a request is cut after this many characters.
pub const MAX_CHARS: usize = 20000;
const MAX_PATHS: usize = 20;

pub struct Request<'a> {
    pub tool: &'a str,
    pub kb_name: &'a str,
    pub author: &'a str,
    /// What the tool shows: a first line, and for document tools a blank line and the full body.
    pub summary: &'a str,
    /// The changed paths. They are listed when the summary has no body.
    pub paths: &'a [String],
}

pub fn render(r: &Request) -> String {
    let (first, rest) = match r.summary.split_once("\n\n") {
        Some((a, b)) => (a, Some(b)),
        None => (r.summary, None),
    };
    let mut out = format!(
        "waymark asks to publish a write to \"{}\".\n{}\nTool: {}\nAuthor: {}\n",
        r.kb_name, first, r.tool, r.author
    );
    match rest {
        Some(body) => {
            out.push_str("\n--- Body ---\n");
            let total = body.chars().count();
            if total > MAX_CHARS {
                let cut: String = body.chars().take(MAX_CHARS).collect();
                out.push_str(&cut);
                out.push_str(&format!(
                    "\n[Cut: {} more characters. The document has {} words.]",
                    total - MAX_CHARS,
                    body.split_whitespace().count()
                ));
            } else {
                out.push_str(body);
            }
        }
        None if !r.paths.is_empty() => {
            out.push_str("\nChanged paths:\n");
            for p in r.paths.iter().take(MAX_PATHS) {
                out.push_str(&format!("- {p}\n"));
            }
            if r.paths.len() > MAX_PATHS {
                out.push_str(&format!("[{} more paths]\n", r.paths.len() - MAX_PATHS));
            }
        }
        None => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req<'a>(summary: &'a str, paths: &'a [String]) -> Request<'a> {
        Request {
            tool: "add",
            kb_name: "Example",
            author: "Jane <j@example.org>",
            summary,
            paths,
        }
    }

    #[test]
    fn a_document_write_shows_the_body() {
        let t = render(&req("Add \"T\" in \"ops\"\n\nHello body", &[]));
        assert!(t.starts_with(
            "waymark asks to publish a write to \"Example\".\nAdd \"T\" in \"ops\"\n"
        ));
        assert!(t.contains("Tool: add\nAuthor: Jane <j@example.org>"));
        assert!(t.contains("--- Body ---\nHello body"));
    }

    #[test]
    fn a_summary_without_body_lists_paths() {
        let paths: Vec<String> = (0..25).map(|i| format!("p{i}.md")).collect();
        let t = render(&req("Move 25 documents", &paths));
        assert!(t.contains("- p0.md") && t.contains("- p19.md") && !t.contains("- p20.md"));
        assert!(t.contains("[5 more paths]"));
    }

    #[test]
    fn a_long_body_is_cut_and_says_how_much() {
        let body = "word ".repeat(5000);
        let s = format!("Add \"T\"\n\n{body}");
        let t = render(&req(&s, &[]));
        assert!(t.contains("[Cut: 5000 more characters. The document has 5000 words.]"));
        assert!(t.chars().count() < 20400);
    }
}
