//! Pure functions on repository URLs (see `project-binding`).

/// The host of a URL, in lower case. A local path or a `file://` URL has the host `localhost`.
pub fn host(url: &str) -> String {
    let url = url.trim();
    if let Some((scheme, rest)) = url.split_once("://") {
        if scheme.eq_ignore_ascii_case("file") {
            return "localhost".to_string();
        }
        let authority = rest.split('/').next().unwrap_or("");
        return host_of_authority(authority);
    }
    // scp-like: [user@]host:path
    if let Some((left, _path)) = url.split_once(':') {
        if !left.contains('/') && !left.is_empty() {
            return host_of_authority(left);
        }
    }
    "localhost".to_string()
}

fn host_of_authority(authority: &str) -> String {
    let after_user = authority.rsplit('@').next().unwrap_or(authority);
    // An IPv6 host is in brackets.
    let h = if let Some(rest) = after_user.strip_prefix('[') {
        rest.split(']').next().unwrap_or("")
    } else {
        after_user.split(':').next().unwrap_or("")
    };
    h.to_ascii_lowercase()
}

/// The repository name for the default folder: the last part of the URL, without a trailing `/`
/// and without `.git`. `None` if the URL gives no usable name.
pub fn name(url: &str) -> Option<String> {
    let last = url
        .trim()
        .trim_end_matches('/')
        .rsplit(['/', ':'])
        .next()
        .unwrap_or("");
    let last = last.strip_suffix(".git").unwrap_or(last);
    if last.is_empty() || last == "." || last == ".." || last.contains('\\') {
        None
    } else {
        Some(last.to_string())
    }
}

/// The path of a URL: no scheme, user, port, or host.
fn path_part(url: &str) -> String {
    let u = url.trim().trim_end_matches('/');
    let u = u.strip_suffix(".git").unwrap_or(u);
    let path = if let Some((_, rest)) = u.split_once("://") {
        rest.find('/').map(|i| &rest[i..]).unwrap_or("")
    } else if let Some((left, path)) = u.split_once(':') {
        if !left.contains('/') && !left.is_empty() {
            path
        } else {
            u
        }
    } else {
        u
    };
    path.trim_start_matches('/').to_string()
}

/// Are two remote URLs the same repository? The form (SSH or HTTPS), the user, the port, a trailing
/// `/`, and `.git` do not count. The host is compared in lower case, the path as it is.
pub fn same_remote(a: &str, b: &str) -> bool {
    host(a) == host(b) && path_part(a) == path_part(b)
}

/// The URL without a user and a password. A scp-like address keeps its form.
pub fn redact(url: &str) -> String {
    if let Some((scheme, rest)) = url.split_once("://") {
        let (authority, path) = match rest.split_once('/') {
            Some((a, p)) => (a, format!("/{p}")),
            None => (rest, String::new()),
        };
        let authority = authority.rsplit('@').next().unwrap_or(authority);
        return format!("{scheme}://{authority}{path}");
    }
    if let Some((left, path)) = url.split_once(':') {
        if !left.contains('/') && left.contains('@') {
            let h = left.rsplit('@').next().unwrap_or(left);
            return format!("{h}:{path}");
        }
    }
    url.to_string()
}

/// Replace every copy of `url` in a text by its redacted form.
pub fn redact_in(text: &str, url: &str) -> String {
    text.replace(url, &redact(url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosts() {
        assert_eq!(
            host("git@github.com:example/waymark-agent-memory.git"),
            "github.com"
        );
        assert_eq!(host("ssh://git@example.org:2222/team/k.git"), "example.org");
        assert_eq!(
            host("https://user:secret@Git.Example.org/a/b.git"),
            "git.example.org"
        );
        assert_eq!(host("file:///tmp/k.git"), "localhost");
        assert_eq!(host("/tmp/k.git"), "localhost");
        assert_eq!(host("../k.git"), "localhost");
    }

    #[test]
    fn names() {
        assert_eq!(
            name("git@example.org:team/Example_Agent-Memory.git").as_deref(),
            Some("Example_Agent-Memory")
        );
        assert_eq!(
            name("https://example.org/acme/notes.git/").as_deref(),
            Some("notes")
        );
        assert_eq!(
            name("ssh://git@h:2222/acme/notes").as_deref(),
            Some("notes")
        );
        assert_eq!(name("host:notes.git").as_deref(), Some("notes"));
        assert_eq!(name("https://example.org/.git"), None);
        assert_eq!(name("https://example.org/a/.."), None);
        assert_eq!(name(""), None);
    }

    #[test]
    fn same_remotes() {
        assert!(same_remote("file:///a/b.git", "file:///a/b"));
        assert!(same_remote("file:///a/b/", "file:///a/b.git"));
        assert!(!same_remote("file:///a/b", "file:///a/c"));
        assert!(same_remote("git@h:a/b.git", "https://h/a/b.git"));
        assert!(same_remote(
            "git@GitHub.com:a/b.git",
            "https://github.com/a/b"
        ));
        assert!(same_remote(
            "ssh://git@h:2222/a/b.git",
            "https://user:pw@h/a/b/"
        ));
        assert!(!same_remote("git@h:a/b.git", "https://other/a/b.git"));
        assert!(!same_remote("git@h:a/b.git", "https://h/c/b.git"));
        assert!(!same_remote("git@h:a/b.git", "https://h/a/B.git"));
    }

    #[test]
    fn redaction() {
        assert_eq!(
            redact("https://user:secret@example.org/a/b.git"),
            "https://example.org/a/b.git"
        );
        assert_eq!(
            redact("ssh://git@example.org:2222/a.git"),
            "ssh://example.org:2222/a.git"
        );
        assert_eq!(redact("git@example.org:a/b.git"), "example.org:a/b.git");
        assert_eq!(redact("file:///tmp/k.git"), "file:///tmp/k.git");
        assert_eq!(
            redact_in("fatal: https://u:p@h.org/x failed", "https://u:p@h.org/x"),
            "fatal: https://h.org/x failed"
        );
    }
}
