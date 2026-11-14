//! Remove passwords from URLs, in one URL or in free text.

/// The URL without its user info. A scp-like address keeps its form.
pub fn redact_url(url: &str) -> String {
    crate::mcp::repo_url::redact(url)
}

/// Redact every URL with user info in a text. The text keeps its other words.
pub fn redact_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        if word.contains("://") && word.contains('@') {
            // Keep the characters around the URL, such as quotes and a final dot.
            let start = word.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(0);
            let (lead, rest) = word.split_at(start);
            let trail_len = rest
                .chars()
                .rev()
                .take_while(|c| matches!(c, '.' | ',' | ';' | ')' | '\'' | '"' | '`'))
                .count();
            let (url, trail) = rest.split_at(rest.len() - trail_len);
            out.push_str(lead);
            out.push_str(&redact_url(url));
            out.push_str(trail);
        } else {
            out.push_str(word);
        }
        word.clear();
    };
    for c in text.chars() {
        if c.is_whitespace() {
            flush(&mut word, &mut out);
            out.push(c);
        } else {
            word.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_is_removed() {
        assert_eq!(
            redact_url("https://jane:s3cret@example.org/x.git"),
            "https://example.org/x.git"
        );
        assert_eq!(
            redact_url("git@github.com:example/k.git"),
            "github.com:example/k.git"
        );
    }

    #[test]
    fn free_text_is_redacted() {
        let t = "fatal: unable to access 'https://jane:s3cret@example.org/x.git/': failed.";
        let r = redact_text(t);
        assert!(!r.contains("s3cret"), "{r}");
        assert!(r.contains("example.org/x.git/"));
        assert_eq!(redact_text("no url here"), "no url here");
    }
}
