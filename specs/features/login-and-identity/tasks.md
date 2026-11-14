# Tasks: Login and identity

`requires`: `mcp-server`.

- [ ] `src/identity/redact.rs`: `redact_url`, `redact_text`, and unit tests.
- [ ] `src/identity/email.rs`: `check_email` with the default blocklist, the extra blocklist, and the allowed domains. Unit tests for every blocklist part.
- [ ] `src/identity/author.rs`: `resolve` with the three sources and the Git config reader.
- [ ] `src/identity/mod.rs`: the types and the exports. Add `mod identity` to `src/lib.rs`.
- [ ] `mcp-server`: pass `--author-name` and `--author-email` to `identity` (in `mcp::args`), and call `resolve` and `check_email` in `Context` before a write.
- [ ] `gitsync`: take a checked `Author` in `Repo::write` (replaces its own author step).
- [ ] Wrap messages with `redact_text` in `mcp::error`.
- [ ] Map the login failures of `git` to `sync_failed` with the fixed `next` sentence (in `gitsync::cmd`).
- [ ] `tests/it_login_and_identity.rs`: the acceptance criteria.
- [ ] Run the full suite.
