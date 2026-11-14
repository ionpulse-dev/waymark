# Tasks: Write documents

`requires`: `mcp-server`, `git-sync`, `document-format`. All three must be `implemented` first.

- [ ] Confirm that `Repo::write` takes a closure, that `gitsync::message` supports a body, and that `format::links` returns link spans. If not, add these to the foundation code first (see `plan.md`).
- [ ] `src/writes/slug.rs`: file name from a title, with the hex suffix rule, and unit tests.
- [ ] `src/writes/titles.rs`: words, duplicate, near-duplicate, and unit tests (80 percent boundary, 3-word minimum).
- [ ] `src/writes/secrets.rs`: built-in and custom patterns, scan by rule and line, and unit tests.
- [ ] `src/writes/linkfix.rs`: find and remove links, find and retarget links, and unit tests (body, property, alias, embed, code fence).
- [ ] `src/writes/mod.rs`: `DocPlan`, common argument checks, the `folders` check, the glue to `Repo::write`, and the result and preview builders.
- [ ] `src/tools/add.rs`, with `supersedes`.
- [ ] `src/tools/update_document.rs`, with the trust reset.
- [ ] `src/tools/dispute.rs`.
- [ ] `src/tools/deprecate.rs`.
- [ ] `src/tools/delete.rs`.
- [ ] `src/tools/mod.rs`: register the five tools.
- [ ] `tests/it_write_documents.rs`: the cases of `test.md`.
- [ ] Check the size budget of `tools/list` (see `mcp-server`) with the five schemas.
- [ ] Run the full suite.
