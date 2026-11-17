# Tests: Menu bar app

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_AI checks off `[auto]` items when tests pass. A human checks off `[manual]` items. The feature moves to `done` only when all items are checked._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** the app is running, **When** the person looks at the menu bar, **Then** the icon is there and no Dock icon shows. | M1 |
| **Given** no memory repository in the settings, **When** the app starts, **Then** it shows the setup help. | M2 |
| **Given** a watched document, **When** another person pushes a change and the poll interval passes, **Then** the document window shows the new text and who changed it. | A6, M3 |
| **Given** a watched folder, **When** a new document arrives in a subfolder and the poll interval passes, **Then** the folder window lists it and marks it as new. | A7, M4 |
| **Given** a watched document, **When** nothing changes, **Then** the window does not reload and no notification shows. | A8, U5 |
| **Given** a document window, **When** the person clicks a link to another document, **Then** that document opens in the same window, and Back returns. | U3, M5 |
| **Given** a deprecated document with a replacement, **When** its window opens, **Then** it shows a banner with a link to the replacement. | A9, M5 |
| **Given** no network, **When** a poll runs, **Then** the windows keep the last content and the icon shows "Behind". | A10, U6 |
| **Given** `waymark` is not installed, **When** the app starts, **Then** it shows the install help and the icon shows "Error". | A2, U2, M6 |
| **Given** the app, **When** it lists the tools of its `waymark` process, **Then** `undo` is the only tool with `readOnlyHint: false`. | A1 |
| **Given** a write of the person, **When** the person chooses Undo, **Then** the app shows the preview first, and nothing changes before the person confirms. | A13, M7 |
| **Given** the person confirms an undo, **When** it finishes, **Then** a revert commit under the person's name is pushed, and the write shows as "Undone". | A14, M7 |
| **Given** a write whose files another person changed later, so that the revert conflicts, **When** the person chooses Undo, **Then** the app shows the reason from `undo` and changes nothing. | A15 |
| **Given** `approval.required_for = ["undo"]`, **When** the person confirms an undo, **Then** the approval dialog shows in the app, and the undo runs only after a yes. | A16, M8 |
| **Given** the shared setting `access = "read-only"`, **When** the recent writes window shows, **Then** it has no Undo button and says why. | A17 |
| **Given** notifications on, **When** a watched item changes, **Then** one notification shows, and a click opens its window. | M9 |
| **Given** "Start at login" on, **When** the person logs in, **Then** the app starts. | M10 |
| **Given** an agent of the person added a document, **When** the next poll runs, **Then** the recent writes window lists it as not reviewed and the menu shows the count 1. | A11, M7 |
| **Given** an update of a document by an agent of the person, **When** the person opens the write, **Then** the app shows the diff between the version before and the version of the write. | A12, U1 |
| **Given** a write that was undone with `undo`, **When** the list shows it, **Then** it is marked "Undone". | A14 |
| **Given** a write marked as reviewed, **When** the app restarts, **Then** it is still reviewed. | U4 |
| **Given** writes of another person, **When** the recent writes window shows, **Then** they are not listed. | A11 |
| **Given** the app polls `updates`, **When** an agent tool of the person then calls `updates` without `since`, **Then** it still lists the changes since its own last call. | A18 |

## Test Cases

Unit tests (`swift test` in `apps/macos`, no `waymark` needed):

- [ ] [auto] U1: the line diff marks added, removed, and kept lines, and lists changed properties (`DiffTests.swift`).
- [ ] [auto] U2: the binary is found in the setting, then `/opt/homebrew/bin`, then `/usr/local/bin`. None found is the error "not installed". The arguments hold the deny list with every write tool except `undo` (`BinaryTests.swift`).
- [ ] [auto] U3: `[[ops/a]]`, `[[ops/a|Label]]`, `[[ops/a#part]]`, `[x](../b.md)`, and `[x](c.md)` from `ops/deep/d` resolve to document IDs. External links stay as they are (`LinksTests.swift`).
- [ ] [auto] U4: the reviewed set, the watched list, and the settings survive a new store on the same defaults. The reviewed set keeps the last 1000 (`StoresTests.swift`).
- [ ] [auto] U5: with a fake session, a poll with the same `version` and no `updates` gives no event (`PollerTests.swift`).
- [ ] [auto] U6: with a fake session, a notice "may be behind" sets the state "Behind" and keeps the last content (`PollerTests.swift`).

Integration tests (`swift test`, a real `waymark` built from this repository against a temporary memory repository with a bare remote and two clones):

- [ ] [auto] A1: `tools/list` of the app's session has `undo` as the only tool with `readOnlyHint: false` (`IntegrationTests.swift`).
- [ ] [auto] A2: a session with a missing binary fails with "not installed" (`IntegrationTests.swift`).
- [ ] [auto] A6: B updates a watched document and pushes. The next poll of A gives a change with the new `version`, the new body, and B's name (`IntegrationTests.swift`).
- [ ] [auto] A7: B adds a document in a subfolder of a watched folder. The next poll gives the folder as changed and the new ID as new (`IntegrationTests.swift`).
- [ ] [auto] A8: a second poll with no change gives no event (`IntegrationTests.swift`).
- [ ] [auto] A9: a deprecated document has `replaced_by`, and the model gives the replacement link (`IntegrationTests.swift`).
- [ ] [auto] A10: the remote is gone. The poll gives the state "Behind" and the last content stays (`IntegrationTests.swift`).
- [ ] [auto] A11: A's agent adds a document. The recent writes list has it as not reviewed, count 1. B's writes are not listed (`IntegrationTests.swift`).
- [ ] [auto] A12: A's agent updates a document. The change of the write has the version before and after and a diff with the new line (`IntegrationTests.swift`).
- [ ] [auto] A13: Undo preview lists the change and writes nothing (the head of the remote is the same) (`IntegrationTests.swift`).
- [ ] [auto] A14: Undo pushes a revert under A's name. The write shows "Undone", and the undo commit is listed (`IntegrationTests.swift`).
- [ ] [auto] A15: B changes the document after A's write. Undo refuses with a message and changes nothing (`IntegrationTests.swift`).
- [ ] [auto] A16: with `approval.required_for = ["undo"]`, the handler gets the request. A no writes nothing. A yes runs the undo (`IntegrationTests.swift`).
- [ ] [auto] A17: with `access = "read-only"` in the shared settings, the session says that Undo is not allowed and why (`IntegrationTests.swift`).
- [ ] [auto] A18: the app polls `updates` with `since`. An agent of A then calls `updates` without `since` and still lists B's change (`IntegrationTests.swift`).
- [ ] [auto] A19: with a low `max_documents_opened`, more `show` calls than the budget still work, through a new session (`IntegrationTests.swift`).
- [ ] [auto] A20: `scripts/macos-app.sh` builds `WaymarkBar.app`, and its `Info.plist` has `LSUIElement` (`make app`).

Manual tests (on a Mac, with the app built by `make app`):

Setup for M3 to M8: a test memory repository. Run `bash scripts/sample-memory.sh create`, then `waymark --path tmp/sample-memory/clone` is the memory repository. In the app settings choose the folder `tmp/sample-memory/clone`. A second clone for "another person": `git clone tmp/sample-memory/remote.git tmp/sample-memory/b`.

- [ ] [manual] M1: open `apps/macos/build/WaymarkBar.app`. Expected: an icon in the menu bar, no Dock icon, no window.
- [ ] [manual] M2: with empty settings (`defaults delete dev.waymark.bar` first), open the app and its menu. Expected: the menu says that no memory repository is set and offers Settings. Settings shows the setup help.
- [ ] [manual] M3: watch a document (Search, Watch). Open its window. In clone `b`, edit the body of that file, commit with another name, and push. Wait for the poll interval (set it to 10 seconds). Expected: the window shows the new text and "Updated just now by" with the other name. The menu marks the item with a dot until the window is shown.
- [ ] [manual] M4: watch a folder. In clone `b`, add a document in a subfolder of it (a valid file, for example a copy of another document with a new title), commit, push. Expected: after the interval, the folder window lists it and marks it as new.
- [ ] [manual] M5: open a document with a `[[...]]` link. Click the link. Expected: the linked document opens in the same window, Back returns. Open a deprecated document with a replacement: a banner with a link to the replacement.
- [ ] [manual] M6: in Settings, set the `waymark` binary to a path that does not exist. Expected: the icon shows the error state and the menu says "waymark is not installed. Install it with Homebrew, then choose Retry."
- [ ] [manual] M7: let an agent (or the sample clone through `waymark`) add and update documents under your name. Expected: Recent writes shows the count, the list marks them as not reviewed, an update shows a diff. Choose Undo on the update: a sheet with the preview, Cancel changes nothing, Undo makes the write "Undone" and lists the undo commit.
- [ ] [manual] M8: add `[approval]` `required_for = ["undo"]` to `~/.config/waymark/config.toml`. Restart the app. Undo a write. Expected: an approval dialog of the app with the commit message. Decline: nothing changes. Approve: the undo runs.
- [ ] [manual] M9: switch on notifications in Settings and allow them in macOS. Change a watched document from clone `b`. Expected: one notification. A click opens its window.
- [ ] [manual] M10: switch on Start at login, log out and in. Expected: the app runs. It shows in System Settings, General, Login Items.
- [ ] [manual] M11: a real memory repository on GitHub with the person's own login, watched for a day. Expected: no error icon while the network works, updates show within the interval.

## Notes

- The integration tests need `git` and the `waymark` binary of this repository. They use `WAYMARK_BIN`, or `../../target/debug/waymark`. `make app-test` builds it first.
- The tests set `HOME` to a temporary folder, so the person's own config is never read.
