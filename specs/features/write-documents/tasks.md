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
- [x] `src/tools/delete.rs` (changes with the 2026-11-14 link rule, see below).
- [x] `src/tools/mod.rs`: register the five tools.
- [x] `tests/it_write_documents.rs`: the cases of `test.md`.
- [x] Check the size budget of `tools/list` (see `mcp-server`) with the five schemas.
- [x] Run the full suite.
- [x] Body links: `add_document` and `update_document` fail on a new link to a missing document (`src/writes/`, `format::links`).
- [x] `delete_document`: refuse while another document links to the target, with `data.linked_from`; remove the automatic link fix and `links_fixed`; change the tool description (`src/tools/delete.rs`).
- [x] Tests of the new cases in `test.md`; replace `delete_fixes_links`.
- [x] `sync` applies the same body link rule to documents that people edit by hand (see `manual-edits`).
- [x] Update `docs/install.md`, the README, and `standard/conformance` if they mention the old behaviour.
