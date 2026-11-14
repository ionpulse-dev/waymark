# Tasks: Login and identity

`requires`: `mcp-server`.

- [x] `src/identity/redact.rs`: `redact_url`, `redact_text`, and unit tests.
- [x] `src/identity/email.rs`: `check_email` with the default blocklist, the extra blocklist, and the allowed domains. Unit tests for every blocklist part.
- [x] `src/identity/author.rs`: `resolve` with the three sources and the Git config reader.
- [x] `src/identity/mod.rs`: the types and the exports. Add `mod identity` to `src/lib.rs`.
- [x] `mcp-server`: pass `--author-name` and `--author-email` to `identity` (in `mcp::args`), and call `resolve` and `check_email` in `Context` before a write.
- [x] `gitsync`: take a checked `Author` in `Repo::write` (replaces its own author step).
- [x] Wrap messages with `redact_text` in `mcp::error`.
- [x] Map the login failures of `git` to `sync_failed` with the fixed `next` sentence (in `gitsync::cmd`).
- [x] `tests/it_login_and_identity.rs`: the acceptance criteria.
- [x] Run the full suite.
