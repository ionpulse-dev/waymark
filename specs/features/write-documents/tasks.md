# Tasks: Write documents

`requires`: `mcp-server`, `git-sync`, `document-format`. All three must be `implemented` first.

- [x] Confirm that `Repo::write` takes a closure, that `gitsync::message` supports a body, and that `format::links` returns link spans. If not, add these to the foundation code first (see `plan.md`).
- [x] `src/writes/slug.rs`: file name from a title, with the hex suffix rule, and unit tests.
- [x] `src/writes/titles.rs`: words, duplicate, near-duplicate, and unit tests (80 percent boundary, 3-word minimum).
- [x] `src/writes/secrets.rs`: built-in and custom patterns, scan by rule and line, and unit tests.
- [x] `src/writes/linkfix.rs`: find and remove links, and unit tests (body, property, alias, embed, code fence). Retargeting moves to `move-documents` (see `decisions.md`).
- [x] `src/writes/mod.rs`: `DocPlan`, common argument checks, the `folders` check, the glue to `Repo::write`, and the result and preview builders.
- [x] `src/tools/add.rs`, with `supersedes`.
- [x] `src/tools/update_document.rs`, with the trust reset.
- [x] `src/tools/dispute.rs`.
- [x] `src/tools/deprecate.rs`.
- [x] `src/tools/delete.rs`.
- [x] `src/tools/mod.rs`: register the five tools.
- [x] `tests/it_write_documents.rs`: the cases of `test.md`.
- [x] Check the size budget of `tools/list` (see `mcp-server`) with the five schemas.
- [x] Run the full suite.
