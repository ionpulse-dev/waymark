# Tasks: Quality checks

_Prerequisite: `search-and-read` is `implemented` or `done`._

- [x] Add `mod lint;` to `src/lib.rs`. `regex` was a dependency already.
- [x] `src/gitsync/log.rs`: add `path_authors()` (first and last author of each path, one pass). Tested end to end (T5).
- [x] `src/lint/secrets.rs`: the built-in patterns, the placeholder skip, the extra patterns, and the `setup_problem` for a bad one. Unit tests: one positive and one negative sample for each pattern, and no matched text in the finding.
- [x] `src/lint/personal.rs`: IBAN with mod 97 and cards with Luhn. Unit tests.
- [x] Duplicate titles: the oldest-wins rule is in `src/lint/mod.rs`, and `writes::titles` has `find_title` logic. Tested end to end (T9).
- [x] `src/lint/authors.rs`: `author-mismatch` and `author-not-allowed`. Unit tests.
- [x] `src/lint/mod.rs`: `run`, `text_findings`, and the `next_for` table. Tested end to end (T1, T2, T22).
- [x] `tests/lint_conformance.rs`: every file of `standard/conformance/vault/` has no error, and every file of `invalid/` has the rule that its name says.
- [x] Write checks: `writes::check_document` calls the shared lint code (secrets, personal data, authors). `gitsync` does not call `lint` (see `decisions.md`). `commit_change` now ignores an empty path list.
- [x] `src/tools/lint.rs`: schema, call, description. Register it in `src/tools/mod.rs`.
- [x] `tests/it_quality_checks.rs`: the test cases of `test.md`.
- [x] Check the size of `tools/list` against the budget of `mcp-server`.
- [x] Run the full suite (`make check`).
