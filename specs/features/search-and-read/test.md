# Tests: Search and read

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_AI checks off `[auto]` items when tests pass. A human checks off `[manual]` items. The feature moves to `done` only when all items are checked._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** a query with no match, **When** `search` runs, **Then** it returns an empty list and no error. | T1 |
| **Given** more matches than the cap, **When** `search` runs, **Then** it returns at most 5 documents within 800 tokens, and `truncated` is true. | T2, T3 |
| **Given** a call that asks for 50 results, **When** `search` runs, **Then** it returns at most the cap. | T4 |
| **Given** a title match and a body-only match, **When** `search` runs, **Then** the title match ranks first. | T5 |
| **Given** the read budget is used up, **When** `show` runs for a new document, **Then** it fails with `access_denied` and says why. | T6 |
| **Given** a document already opened, **When** `show` runs for it again after the budget is used up, **Then** it works. | T7 |
| **Given** `level` 0, **When** `list` runs, **Then** each entry is only the ID. | T8 |
| **Given** a document that others link to, **When** `show` runs, **Then** it lists the backlinks. | T9 |
| **Given** a disputed document, **When** `show` runs, **Then** it shows the `dispute_reason`. | T10 |
| **Given** a deprecated document that another supersedes, **When** `show` runs, **Then** `replaced_by` names the replacement. | T11 |
| **Given** the limit `folders = ["engineering/**"]`, **When** `search` runs, **Then** no document outside it appears, and `show` fails with `not_found`. | T12 |
| **Given** a folder, **When** `stats` runs with `by: "author"`, **Then** it counts per `created_by`. | T13 |
| **Given** a document that another clone pushed a moment ago, **When** `search` runs, **Then** it fetches first and finds the document. | T14 |
| **Given** a failed fetch, **When** `search` runs, **Then** it answers locally with a notice. | T15 |
| **Given** a document that moved, **When** `show` runs with the old ID, **Then** it returns the document and `moved_from`. | T16 |

## Test Cases

All tests are in `tests/it_search_and_read.rs`. Each starts a `World` and an `McpClient` (see `integration-testing`). Unit tests are in the files of `src/store/`.

- [x] [auto] T1: `search` for a word that no document holds returns `results: []` and no error.
- [x] [auto] T2: 8 matching documents: `search` returns 5 and `truncated: true`, `total_matches: 8`.
- [x] [auto] T3: documents with long descriptions: the result stays within 800 estimated tokens.
- [x] [auto] T4: `limit: 50` still returns at most 5. A personal config with `max_results = 3` returns at most 3.
- [x] [auto] T5: a title match ranks before a body-only match.
- [x] [auto] T6: with `--max-documents-opened 2` (a limit argument), the third different `show` fails with `access_denied`.
- [x] [auto] T7: after T6, `show` of one of the first two works.
- [x] [auto] T8: `list` with `level: 0` returns entries that have only `id`.
- [x] [auto] T9: `show` lists the IDs of documents that link to it by a body link and by `related`.
- [x] [auto] T10: `show` of a disputed document has `disputed: true` and the `dispute_reason`.
- [x] [auto] T11: `show` of a deprecated document names the superseding document in `replaced_by`.
- [x] [auto] T12: with a shared limit `folders = ["engineering/**"]`, `search` returns only `engineering/` documents and `show` of `ops/deploy` is `not_found`.
- [x] [auto] T13: `stats` with `by: "author"` returns one group per `created_by`, and the values add up to `total`.
- [ ] [auto] T14: clone A runs `search` once. Clone B commits a document and pushes. Clone A runs `search` again at once, with no freshness setting. It finds the document.
- [x] [auto] T15: the remote path is broken. `search` still returns results and has a notice that the clone may be behind.
- [x] [auto] T16: `git mv` a document and commit. `show` with the old ID returns the new document, `moved_from` is the old ID, and there is a notice.
- [x] [auto] T17: a document `ops/index.md` with the tag `index` is found by `search` and `list` like any document.
- [x] [auto] T18: `stats` with `metric: "documents"` and `by: "folder"` counts match the files.
- [x] [auto] T19: `list` with `offset` pages through 7 documents with `limit: 3` and returns all of them once.
- [x] [auto] T20: a manual, uncommitted edit shows in `search`, and `show` has `pending_edit: true`.
- T21: removed on 2026-11-17. It was a timing test (see `decisions.md`). The number stays free.
- [x] [auto] T22: `search` with an empty `query` fails with `invalid_arguments` and a `next` sentence.

## Notes

- No `[manual]` item. All behavior runs through the stdio server.
- Tests that need a short read budget or caps pass them as limit arguments of the MCP entry, as the harness allows.
