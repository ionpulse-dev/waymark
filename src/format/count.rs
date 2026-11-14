//! Counting rules (section 9.1 of the standard).
//!
//! The server and `lint` use these functions, so they always agree.

use regex::Regex;
use std::sync::OnceLock;

/// The size of a document: bytes of the whole file in UTF-8.
pub fn bytes(text: &str) -> usize {
    text.len()
}

/// A word is a maximal run of characters that are not Unicode white space.
pub fn words(text: &str) -> usize {
    text.split_whitespace().count()
}

fn callout_marker() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[![A-Za-z][A-Za-z0-9-]*\][+-]?").unwrap())
}

/// Remove comments: the text from `%%` to the next `%%`.
/// A `%%` with no closing `%%` runs to the end of the text.
fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("%%") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("%%") {
            Some(end) => rest = &after[end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// The text of a body without the markup that people write for Obsidian.
/// Comments go with their text. Other markup loses only its markers.
pub fn strip_markup(body: &str) -> String {
    let no_comments = strip_comments(body);
    let mut out = String::with_capacity(no_comments.len());
    for line in no_comments.split_inclusive('\n') {
        // The leading `>` of a quoted line (every callout is one).
        let trimmed = line.trim_start();
        let line = if trimmed.starts_with('>') {
            let mut t = trimmed;
            while let Some(r) = t.strip_prefix('>') {
                t = r.trim_start_matches(' ');
            }
            t
        } else {
            line
        };
        out.push_str(line);
    }
    let out = callout_marker().replace_all(&out, "").into_owned();
    out.replace("![[", " ")
        .replace("]]", " ")
        .replace("==", " ")
}

/// Words of a body, without markup.
pub fn body_words(body: &str) -> usize {
    words(&strip_markup(body))
}

/// An estimate for planning reads: words times 1.5, rounded up.
pub fn estimated_tokens(words: usize) -> usize {
    (words * 3).div_ceil(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_split_on_unicode_white_space() {
        assert_eq!(words("one two\tthree\nfour\u{00a0}five"), 5);
        assert_eq!(words("   "), 0);
        assert_eq!(words("Überprüfung der Zahlungs-Schnittstelle"), 3);
        assert_eq!(words("日本語 のテキスト"), 2);
    }

    #[test]
    fn bytes_count_utf8() {
        assert_eq!(bytes("aä"), 3);
    }

    #[test]
    fn tokens_are_words_times_one_and_a_half_rounded_up() {
        assert_eq!(estimated_tokens(0), 0);
        assert_eq!(estimated_tokens(1), 2);
        assert_eq!(estimated_tokens(2), 3);
        assert_eq!(estimated_tokens(10), 15);
    }

    #[test]
    fn comment_text_does_not_count() {
        assert_eq!(body_words("a %% hidden words here %% b"), 2);
    }

    #[test]
    fn unclosed_comment_runs_to_the_end() {
        assert_eq!(body_words("a b %% c d\ne f"), 2);
    }

    #[test]
    fn callout_marker_adds_no_word_and_its_text_counts() {
        assert_eq!(body_words("> [!note] Hello world"), 2);
        assert_eq!(body_words("> [!warning]- Folded title\n> More text"), 4);
    }

    #[test]
    fn embed_markers_add_no_word() {
        assert_eq!(body_words("![[note-a]] and ![[note-b]]"), 3);
    }

    #[test]
    fn highlight_markers_add_no_word() {
        assert_eq!(body_words("a ==marked text== b"), 4);
        assert_eq!(body_words("a == b"), 2);
    }

    #[test]
    fn quoted_lines_lose_their_marker() {
        assert_eq!(body_words("> one two\n>> three"), 3);
    }

    #[test]
    fn plain_text_is_unchanged() {
        assert_eq!(body_words("one two three"), 3);
    }
}
