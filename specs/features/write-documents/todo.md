# Todo: Write documents

## Bugs

- [ ] None known.

## Deferred

- [x] ~~A tool or a field to make a deprecated document active again.~~ Done: `restore_document` (see `restore-and-replace`).
- [x] ~~Show the deprecation reason in `show`.~~ Done: `deprecation_reason` (see `lean-tools`).
- [ ] A secret scan with entropy checks and more patterns.
- [ ] A link fix for `delete_document` that keeps the context better than the plain text.
- [ ] Server mode: roles for write tools, and the web console for approval (see `server-mode` and `web-console`).

## Open Questions

- Conflict with `quality-checks`: `lint` has a rule for duplicate titles. The write tools ignore deprecated documents, so `lint` should do the same. `quality-checks` must reuse `src/writes/titles.rs` and `src/writes/secrets.rs`.
- Needs from the foundation: `Repo::write` with a closure, message body lines, and link spans from `format::links`. The owners of `git-sync` and `document-format` must confirm.

## Done in the implementation (2026-11-14)

- A test for `approval_asked_before_commit` waits for `write-approval` (`#[ignore]` in `tests/it_write_documents.rs`).
- The `[manual]` Obsidian check is for the product owner.
