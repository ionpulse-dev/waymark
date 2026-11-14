# Tests: Notices and review

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_AI checks off `[auto]` items when tests pass. A human checks off `[manual]` items. The feature moves to `done` only when all items are checked._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** someone disputes my document, **When** the next result is built after the next fetch, **Then** it carries a notice that names it. | T1 |
| **Given** someone changes my document, **When** the next result is built after the next fetch, **Then** it carries a notice. | T2 |
| **Given** a failed fetch, **When** a read runs, **Then** the result carries a notice that the clone may be behind. | T3 |
| **Given** the same problem in one session, **When** several results are built, **Then** the notice appears once. | T4 |
| **Given** no problems, **When** a result is built, **Then** `notices` is absent. | T5 |
| **Given** a notice about my documents, **When** I run `updates` without `since`, **Then** later results do not carry it. | T6 |
| **Given** the same notice in 3 sessions, **When** a fourth session starts, **Then** it does not carry it. | T7 |
| **Given** a dispute that someone then clears, **When** the next result is built, **Then** there is no notice about it. | T8 |
| **Given** `paused = true`, **When** any tool runs, **Then** the result carries "Writes are paused." once. | T9 |
| **Given** a change by me, **When** a result is built, **Then** there is no notice about it. | T10 |
| **Given** `status` runs, **When** notices are open, **Then** it lists all of them. | T11 |

## Test Cases

All tests are in `tests/it_notifications.rs`, with a `World` of two clones. Person A runs the server. Person B commits. The harness sets `WAYMARK_FETCH_MAX_AGE=0` and `WAYMARK_STATE_DIR`.

- [x] [auto] T1: A's document is disputed by B and pushed. A's next `search` has one notice that names the document and says "disputed".
- [x] [auto] T2: B edits A's document and pushes. A's next `search` has a notice that says "changed".
- [x] [auto] T3: the remote path is broken. `search` has the notice "The last update from the remote failed. The clone may be behind."
- [x] [auto] T4: after T1, a second `search` and a `list` in the same session have no such notice.
- [x] [auto] T5: a clean world: `search` has no `notices` key.
- [x] [auto] T6: after T1, a new session runs `updates` without `since`. Its result has no notice about documents. A following `search` has none, and a third new session has none.
- [x] [auto] T7: the problem of T1 stays open. Sessions 1, 2, and 3 each show the notice once. Session 4 shows none.
- [x] [auto] T8: B disputes and then clears the dispute with a second commit. A's `search` has no notice.
- [x] [auto] T9: a personal config with `paused = true`. Two calls: only the first has "Writes are paused."
- [x] [auto] T10: A's own commit is the only change. No notice.
- [x] [auto] T11: after T1 and T4, `status` still lists the notice about the document.
- [x] [auto] T12: a notice with more than 3 documents names 3 and says "and N more".
- [x] [auto] T13: an error result (`not_found` from `show`) also carries a pending notice.
- [x] [auto] T14: a broken state file does not fail the call. The notice is skipped.
- [ ] [manual] T15: a real agent tool shows the notice to the person.
  - Setup: a knowledge base with a remote and a second clone. Install waymark. Add the MCP entry to Claude Code. Write a document `ops/deploy` with `add` through the agent. Push.
  - Steps: in the second clone, run `git pull`, then edit `ops/deploy.md` so that `disputed: true` and `dispute_reason: "Test"`, set the author fields, commit, and push. In Claude Code, start a new session in the project and ask "search the knowledge base for deploy".
  - Expected: Claude Code shows or mentions the notice that someone disputed `ops/deploy`. If the client hides `notices`, record that in `todo.md`.
  - Teardown: delete the test document.

## Notes

- Disputes in tests are made with plain `git` commits in clone B, so no second server is needed.
