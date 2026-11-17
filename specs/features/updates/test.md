# Tests: Updates

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_AI checks off `[auto]` items when tests pass. A human checks off `[manual]` items. The feature moves to `done` only when all items are checked._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** a document I wrote that someone disputed, **When** `updates` runs, **Then** it lists the dispute with its reason. | T1 |
| **Given** no `since`, **When** `updates` runs twice, **Then** the second run covers only the time since the first. | T2 |
| **Given** a call with `since`, **When** it ends, **Then** the stored time does not change. | T3 |
| **Given** a folder, **When** `updates` runs, **Then** it lists only changes under it. | T4 |
| **Given** a document outside the `folders` limit, **When** `updates` runs, **Then** it is not listed. | T5 |
| **Given** a change that I made myself, **When** `updates` runs, **Then** it is not listed. | T6 |
| **Given** a document that someone moved, **When** `updates` runs, **Then** it is `moved` with `from`. | T7 |
| **Given** a new document by someone else, **When** `updates` runs, **Then** it is `new`. | T8 |
| **Given** a `since` that is not a date, **When** `updates` runs, **Then** it fails with `invalid_arguments`. | T9 |
| **Given** a change on the remote that the clone lacks, **When** `updates` runs, **Then** it fetches first and lists it. | T10 |

## Test Cases

All tests are in `tests/it_updates.rs`, with a `World` of two clones: person A runs the server, person B commits.

- [x] [auto] T1: A writes a document and pushes. B disputes it and pushes. `updates` on A has `kind: "disputed"`, `mine: true`, and the reason.
- [x] [auto] T2: B commits a document. `updates` lists it. B commits another. A second `updates` lists only the second.
- [x] [auto] T3: after T2 setup, `updates` with `since` a week ago lists both and `marked` is false. The next `updates` without `since` still lists only changes after the last marked call.
- [x] [auto] T4: B changes `ops/x` and `backend/y`. `updates` with `folders: ["ops"]` lists only `ops/x`.
- [x] [auto] T5: with the limit `folders = ["ops/**"]`, a change to `backend/y` is not listed.
- [x] [auto] T6: A's own commit is in the window. It is not listed.
- [x] [auto] T7: B moves `ops/undo` to `ops/rollback`. The entry has `kind: "moved"` and `from: "ops/undo"`.
- [x] [auto] T8: B adds a document. The entry is `new`, and `mine` is false.
- [x] [auto] T9: `since: "yesterday"` and a `since` in the future both fail with `invalid_arguments` and a `next` sentence.
- [x] [auto] T10: A runs `updates` once. B pushes. A runs `updates` again at once, with no freshness setting. It lists B's change.
- [x] [auto] T11: B deprecates a document. The entry is `deprecated`.
- [x] [auto] T12: 60 documents change. `updates` returns 50, `total: 60`, `truncated: true`.
- [x] [auto] T13: a document `ops/index.md` with the tag `index` changes. It is listed like any document.
- [x] [auto] T14: the state file lives in `WAYMARK_STATE_DIR` and holds `last_updates`. Another key in it stays after a call.
- [x] [auto] T15: a repository younger than `since` lists every document as `new`.

## Notes

- The harness sets `WAYMARK_STATE_DIR` to a folder in the temp root.
- No `[manual]` item.
