//! The safe log of the server (see `specs/foundation/personal-server`).
//!
//! One line for each start, sign-in, and tool call, on standard output. The fields are fixed in
//! code. Free text from clients and documents never goes into a line. The text that does go in
//! (such as an error) is redacted first. There is no stored log and no log page.

use crate::identity::redact::redact_text;

/// Make a value safe for one log line: no line breaks, no quotes, no URL passwords, and short.
pub fn clean(value: &str) -> String {
    let redacted = redact_text(value);
    let one_line: String = redacted
        .chars()
        .map(|c| if c.is_control() || c == '"' { ' ' } else { c })
        .collect();
    let trimmed: String = one_line.split_whitespace().collect::<Vec<_>>().join(" ");
    trimmed.chars().take(200).collect()
}

/// The text of a line: `time=... event=... key="value"`.
pub fn format_line(time: &str, event: &str, fields: &[(&str, &str)]) -> String {
    let mut line = format!("time={time} event={event}");
    for (k, v) in fields {
        line.push_str(&format!(" {k}=\"{}\"", clean(v)));
    }
    line
}

/// Write one line to standard output.
pub fn event(event: &str, fields: &[(&str, &str)]) {
    let time = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    println!("{}", format_line(&time, event, fields));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_has_the_fixed_shape() {
        let l = format_line(
            "2026-11-15T10:00:00Z",
            "tool_call",
            &[("tool", "search"), ("outcome", "ok")],
        );
        assert_eq!(
            l,
            "time=2026-11-15T10:00:00Z event=tool_call tool=\"search\" outcome=\"ok\""
        );
    }

    #[test]
    fn line_breaks_and_quotes_cannot_break_the_line() {
        let l = format_line("t", "e", &[("error", "a\"b\nc event=fake")]);
        assert!(!l.contains('\n'));
        assert_eq!(l.matches('"').count(), 2);
    }

    #[test]
    fn url_passwords_are_removed() {
        let l = format_line(
            "t",
            "e",
            &[(
                "error",
                "cannot clone https://user:secret@host.example/x.git now",
            )],
        );
        assert!(!l.contains("secret"), "{l}");
    }

    #[test]
    fn a_long_value_is_cut() {
        let l = format_line("t", "e", &[("error", &"x".repeat(5000))]);
        assert!(l.len() < 300);
    }
}
