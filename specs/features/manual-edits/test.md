# Tests: Manual edits and sync

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_AI checks off `[auto]` items when tests pass. A human checks off `[manual]` items. The feature moves to `done` only when all items are checked._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** an edit in Obsidian, **When** `sync` runs, **Then** a commit `waymark: sync "<title>"` under my account reaches the remote. | T1, T16 |
| **Given** a failing edit and no other, **When** `sync` runs, **Then** `validation_failed`, nothing pushed, file unchanged. | T2 |
| **Given** one passing and one failing edit, **When** `sync` runs, **Then** the passing one is pushed and the failing one is `blocked`. | T3 |
| **Given** pending edits, **When** an agent writes, **Then** the edits are committed first in their own commit. | T4 |
| **Given** a failing pending edit, **When** an agent writes another document, **Then** the write works with a notice. | T5 |
| **Given** a failing pending edit, **When** an agent writes the same document, **Then** `stale_base`. | T6 |
| **Given** approval is required, **When** edits are ready, **Then** the request opens before the commit. | T7 |
| **Given** `access = "read-only"`, **When** `sync` runs, **Then** it pulls and does not commit or push. | T8 |
| **Given** a changed document, **When** `sync` commits it, **Then** `updated` is today and `updated_by` the author. | T9 |
| **Given** a hand move, **When** `sync` and `show` with the old ID run, **Then** `show` returns the new document. | T10 |
| **Given** a file that is not a document, **When** `sync` runs, **Then** it is skipped. | T11 |
| **Given** `preview: true`, **When** `sync` runs, **Then** nothing is committed, pulled, or pushed. | T12 |
| **Given** an edit to `.mkbase/waymark.toml`, **When** `sync` runs, **Then** it is not committed. | T13 |
| **Given** a remote change to the same document as a pending edit, **When** `sync` runs, **Then** both versions are kept and a notice says so. | T14 |

## Test Cases

All auto tests are in `tests/it_manual_edits.rs`, with a `World` and an `McpClient` on clone A. A "manual edit" is a file change that the test makes with plain file writes in the clone, as Obsidian would.

- [x] [auto] T1: edit the body of `ops/deploy.md`. `sync` returns `committed` with `ops/deploy`. The remote has a commit with the message `waymark: sync "How we deploy"` and the author of the person.
- [x] [auto] T2: create `ops/new.md` with no frontmatter. `sync` fails with `validation_failed`. The remote has no new commit. `ops/new.md` is unchanged.
- [x] [auto] T3: one good edit and one bad new file. `committed` has the good one and `blocked` has the bad one with `no-frontmatter`. The remote has the good commit only.
- [x] [auto] T4: edit `ops/deploy.md`, then call `add`. `git log` shows a `sync` commit and then an `add` commit.
- [x] [auto] T5: a bad new file and a call to `add` for another document. `add` works, and the result has the notice "1 manual edit is not published. Run `sync`."
- [x] [auto] T6: a bad edit to `ops/deploy.md` (a 16-word title) and a call to `update_document` for it fails with `stale_base` and the `next` sentence.
- [ ] [auto] T7: `approval.required_for = ["sync"]`, and the harness answers the elicitation. With "no", nothing is committed and the error is `approval_denied`. With "yes", it commits. Waits for `write-approval`.
- [x] [auto] T8: `--access read-only`. A pending edit and a remote commit from B. `sync` is listed, `committed` is empty, `pushed` is false, the clone has B's commit, the edit is still pending.
- [x] [auto] T9: edit `ops/deploy.md` without touching `updated`. After `sync`, the file has `updated` of today and `updated_by` of the author. The body is the same bytes.
- [x] [auto] T10: `git mv`-free hand move (delete and create in another folder). `sync`. `show` with the old ID returns `moved_from`.
- [x] [auto] T11: add `images/diagram.png`. `sync` lists it under `skipped` and it is not in the commit.
- [x] [auto] T12: `sync` with `preview: true`: no commit, no push, and `committed` lists the document without `commit`.
- [x] [auto] T13: edit `.mkbase/waymark.toml`. `sync` does not commit it.
- [x] [auto] T14: B edits the same document and pushes. A has a pending edit to it. `sync` keeps both files, and the result has `conflicts` and a notice.
- [x] [auto] T15: a pending delete is committed with kind `deleted`.
- [x] [auto] T16: the `lint` rules apply: a duplicate title in a new file blocks it with `duplicate-title`.
- [x] [auto] T17: a clean tree: `sync` returns `committed: []`, pulls, and does not create an empty commit.
- [x] [auto] T18: a repository with no remote commits locally and the result has a notice that there is no remote.
- [x] [auto] T19: a `preview: true` write does not commit pending edits.
- [ ] [manual] T20: edit in a real Obsidian vault and publish.
  - Setup: install the current Obsidian. Create a knowledge base with `init` and a real Git remote (a private repository on a Git host that you control). Open the knowledge base folder as a vault. Add the MCP entry to a real agent tool.
  - Steps: in Obsidian, open `ops/deploy`, change one sentence, and save. Create a new document through Obsidian with all required properties (use the properties panel). In the agent tool, ask it to run `sync`.
  - Expected: the agent reports 2 documents published. The remote has one commit `waymark: sync "2 manual edits"` under your name. Obsidian shows no error and still shows the properties and links. `git status` in the folder is clean.
  - Teardown: delete the test documents with `delete`.
- [ ] [manual] T21: a new document without properties in Obsidian is blocked.
  - Setup: as T20.
  - Steps: create a note in Obsidian with only text. Ask the agent to run `sync`.
  - Expected: the result lists the file under `blocked` with `no-frontmatter` and a clear message. The file is unchanged and nothing is pushed.
  - Teardown: delete the note.
- [x] [auto] A new broken body link in a hand edit is blocked, an old one is not, and a linked document deleted by hand is blocked until the links are gone (`tests/it_manual_edits.rs`)

## Notes

- T7 needs `write-approval`. Until that spec is implemented, it stays unchecked and the rest can run.
- The harness writes the manual edits with `tests/common/kb.rs` helpers that write files without Git.
