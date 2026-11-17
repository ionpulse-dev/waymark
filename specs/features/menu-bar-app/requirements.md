---
type: feature
status: in-progress
created: 2026-11-17
updated: 2026-11-17
requires:
  - search-and-read
  - updates
  - activity-and-undo
  - document-history
  - write-approval
---

# Menu bar app

> **Scope on 2026-11-17:** the product owner approved this spec on 2026-11-17. It replaces "No macOS app — 2026-11-11" (see "The menu bar app is in scope" in [project decisions](../../../docs/decisions.md)). The old design was the spec `mac-app`, removed on 2026-11-17 (last version in commit `1e588d9`). This spec reuses its shell decisions, not its architecture.

## User Story

> As a person, I want to keep a document or a folder of my memory repository open in a small window on my Mac, and see it update by itself when someone changes it, so that I follow what my team and my agents learn without asking an agent.

## Personas

- **Private person** — keeps one document, for example a to-do list or a status page, open all day.
- **Team member without Git knowledge** — watches the folder of the team and sees new documents arrive.
- **Developer** — searches the memory repository quickly, without starting an agent session.
- **Every person who lets agents write freely** — reviews what the agents wrote under their name, quickly and in one place, and undoes a bad write with one click.

## Behaviour

### What the app is

- A native macOS app that lives in the menu bar. It has no Dock icon.
- It is an MCP client of the `waymark` binary. It starts its own `waymark` process over stdio, with the same memory repository as the agent tools of the person. It never runs `git` and never reads the files of the clone itself. All document logic stays in `waymark`.
- It reads, and it has one write: **Undo** of a write of the person (see "Recent writes window"). Every other write tool is switched off in its `waymark` process. It cannot add, change, move, or delete a document in any other way.
- It sees changes by polling: it calls read tools at an interval. Every read tool brings the clone up to date at its start (see `git-sync`), so each poll sees the latest state of the remote.

### Menu

- An icon that shows the state:
  - **Normal:** the last poll worked.
  - **Behind:** the last poll had the notice that the clone may be behind (for example, no network).
  - **Error:** `waymark` is missing, did not start, or reported `setup_problem`.
- **Search…**: opens the search window.
- **Recent writes**: opens the recent writes window. The menu shows the number of writes that are not reviewed yet, and the icon shows a small badge while that number is above 0.
- The list of watched documents and folders. A click brings its window to the front. A dot marks an item that changed since its window was last shown.
- **Refresh now**: polls all watched items at once, and reads every watched folder in full.
- **Settings…** and **Quit**.

### Search window

- One field. Typing calls `search` with the query, at level 2. The list shows title, description, folder, and the date of the last change. `search` returns a few results per call (`max_results`, 5 by default). **More results** reads the next page.
- A click on a result opens the document window.
- "Watch" on a result or on a folder adds it to the watched list.

### Document window

- It shows the title, the rendered Markdown body, the status, the tags, the date of the last change, and who changed it.
- A disputed document shows a banner with the reason. A deprecated document shows a banner with its replacement, as a link.
- A link to another document (`[[...]]` or a relative Markdown link) opens that document in the same window. Back and Forward buttons go through the visited documents.
- When the document changes, the window updates in place and shows "Updated just now by <name>" for a few seconds.
- When the document is deleted or moved, the window says so. For a move it offers to follow the document to its new path.

### Folder window

- It lists the documents in the folder and its subfolders: title, description, and the date of the last change, newest first.
- New and changed documents since the window was last shown are marked.
- A click opens the document window.

### Recent writes window

For a person whose agents write on their own: one place to see what was written under their name, and to review it fast.

- It lists the writes of the person, newest first, from the `activity` tool: the time, the verb (add, update, move, delete, and so on), the title, and the folder. It shows writes from every agent tool of the person, also from other computers, once they are pushed.
- Writes that the person has not reviewed are marked. "Load more" reads older writes.
- A click on a write shows it next to the list:
  - **add**: the new document, rendered.
  - **update, dispute, deprecate**: the change as a diff between the version before and the version of the write. A switch shows the full document instead.
  - **move**: the old and the new path.
  - **delete**: the title and the folder. `history` and `show` do not read a deleted document (see `document-history`), so the app shows no text.
  - A write that changed more than one document lists them. A click on one shows its change.
- A write that a later `undo` reverted is shown as "Undone". A write with later changes by anyone says "Changed later" and offers to open the current document.
- **Mark as reviewed**: one button for the shown write, and "Mark all as reviewed" for the list. Opening a write does not mark it. The reviewed state is stored by the app on this Mac only.
- **Undo**: a button on the shown write.
  1. The app calls `undo` with `preview: true` and shows what would change (removed, restored, or reverted documents).
  2. The person confirms with **Undo** or stops with **Cancel**.
  3. The app calls `undo`. The revert is a new commit under the name of the person, pushed at once (see `activity-and-undo`). The write shows as "Undone" and counts as reviewed. The new undo commit shows at the top of the list, already reviewed.
  - The button is not shown for a write that is already undone.
  - An undo of an undo is a redo. The app allows it, with the same steps.
  - If `undo` refuses, the app shows the `message` and the `next` sentence. For example, after later changes by someone else (`stale_base`), it offers to open the current document.
  - If an `approval.required_for` rule covers `undo`, `waymark` asks through MCP elicitation. The app is the MCP client, so the request shows as a dialog of the app, and the person answers it there. With `approval.via = "console"`, the undo fails with `approval_unavailable`, and the app says so.
- With notifications on, a new write of the person gives one notification per poll, for example "Your agent added 2 documents". A click opens the window.

### Notifications

- Optional, off by default. When on, a watched item that changed gives one macOS notification per poll, for example "backend/payments was updated by Jane Doe". A click opens its window. New writes of the person also give a notification (see "Recent writes window").

### Settings window

| Setting | Values | Default |
|---|---|---|
| Memory repository | A Git URL or a local folder, as in the MCP entry (`--repo`, `--path`, `--memory-repository-id`) | Empty. The app shows the setup help. |
| Poll interval | 10 seconds to 1 hour | 60 seconds |
| Notifications | on or off | off |
| Start at login | on or off | off |
| `waymark` binary | A path | Found by the app (see below) |

- The app finds `waymark` in `/opt/homebrew/bin`, then `/usr/local/bin`. If it finds none, it shows how to install it with Homebrew.
- The watched list is stored with the settings. It survives a restart.

### Error states

- `waymark` not found: "waymark is not installed. Install it with Homebrew, then choose Retry."
- `setup_problem` from `waymark`: the app shows the `message` and the `next` sentence of the error.
- The read budget is used up: `show` opens at most `max_documents_opened` documents in one session (20 by default), and a flag cannot raise it. The app counts its `show` calls and starts a new `waymark` process, which is a new session, before the budget runs out. If `show` still fails with `access_denied`, it starts a new process and tries once more.
- Writes are not allowed (`access_denied` because a layer sets `read-only`, or `undo` is in a `tools_denied`): the Undo button is hidden, and the recent writes window says why.
- Writes are paused (`paused`): the Undo button is disabled, with the line "Writes are paused."
- The push of an undo fails (`sync_failed`): the app shows the message. The undo commit stays local and goes out with the next write or `sync`, as in `git-sync`.
- A watched path that the `folders` limit hides: the item shows "Not available" and stays in the list.

## Interfaces & Contracts

The app uses only existing MCP tools and the MCP elicitation of `write-approval`. It needs no new tool and no change to `waymark`.

| Purpose | Call |
|---|---|
| Start | `waymark --tools-denied add_document,update_document,dispute_document,deprecate_document,restore_document,delete_document,move,create_folder,delete_folder,rename_tag,sync,init,verify_document,reject_document` plus the memory repository arguments, over stdio. The app passes no number limits: a layer can only tighten them, so the app works with the limits that `status` reports. |
| Check the switched-off tools | `tools/list` after the start. If a tool with `readOnlyHint: false` other than `undo` is listed (a newer `waymark` with a new write tool), the app adds it to `--tools-denied` and starts the process again. |
| Search | `search` with `query`, `level: 2` |
| Poll a document | `show` with the ID. A different `version` from the last poll means a change. |
| Poll a folder | `updates` with `folders: [<path>]` and `since` (the time of the last poll) gives the changes of other people and their names. The `activity` entries of the same poll give the person's own writes in the folder. Only when one of the two names the folder, or when the window opens, the app reads the full list: `search` with no query, `folders: [<path>]`, `level: 2` (for the description), `status: "any"`, paged with `offset` (`search` returns at most `max_results` per call). |
| Recent writes | `activity` with `limit: 50`, then `before` for older pages. A poll reads until it reaches the newest write it already knows. |
| Change of one write | `history` of the document, to find the version before the write. Then `show` with `version` for both versions. The app computes the diff of the two bodies and properties. |
| Undo | `undo` with `commit` and `preview: true`, then the same without `preview` after the person confirms. |
| Status | `status`, at start and after an error. |

- The app always passes `since` to `updates`. Without it, `updates` would move the stored time that the agent tools of the person share.
- One poll calls `activity` once, `updates` once per watched folder, and `show` once per watched document. `search` runs only for a folder that changed.

## Acceptance Criteria

- **Given** the app is running, **When** the person looks at the menu bar, **Then** the icon is there and no Dock icon shows.
- **Given** no memory repository in the settings, **When** the app starts, **Then** it shows the setup help.
- **Given** a watched document, **When** another person pushes a change and the poll interval passes, **Then** the document window shows the new text and who changed it.
- **Given** a watched folder, **When** a new document arrives in a subfolder and the poll interval passes, **Then** the folder window lists it and marks it as new.
- **Given** a watched document, **When** nothing changes, **Then** the window does not reload and no notification shows.
- **Given** a document window, **When** the person clicks a link to another document, **Then** that document opens in the same window, and Back returns.
- **Given** a deprecated document with a replacement, **When** its window opens, **Then** it shows a banner with a link to the replacement.
- **Given** no network, **When** a poll runs, **Then** the windows keep the last content and the icon shows "Behind".
- **Given** `waymark` is not installed, **When** the app starts, **Then** it shows the install help and the icon shows "Error".
- **Given** the app, **When** it lists the tools of its `waymark` process, **Then** `undo` is the only tool with `readOnlyHint: false`.
- **Given** a write of the person, **When** the person chooses Undo, **Then** the app shows the preview first, and nothing changes before the person confirms.
- **Given** the person confirms an undo, **When** it finishes, **Then** a revert commit under the person's name is pushed, and the write shows as "Undone".
- **Given** a write whose files another person changed later, so that the revert conflicts, **When** the person chooses Undo, **Then** the app shows the reason from `undo` and changes nothing.
- **Given** `approval.required_for = ["undo"]`, **When** the person confirms an undo, **Then** the approval dialog shows in the app, and the undo runs only after a yes.
- **Given** the shared setting `access = "read-only"`, **When** the recent writes window shows, **Then** it has no Undo button and says why.
- **Given** notifications on, **When** a watched item changes, **Then** one notification shows, and a click opens its window.
- **Given** "Start at login" on, **When** the person logs in, **Then** the app starts.
- **Given** an agent of the person added a document, **When** the next poll runs, **Then** the recent writes window lists it as not reviewed and the menu shows the count 1.
- **Given** an update of a document by an agent of the person, **When** the person opens the write, **Then** the app shows the diff between the version before and the version of the write.
- **Given** a write that was undone with `undo`, **When** the list shows it, **Then** it is marked "Undone".
- **Given** a write marked as reviewed, **When** the app restarts, **Then** it is still reviewed.
- **Given** writes of another person, **When** the recent writes window shows, **Then** they are not listed.
- **Given** the app polls `updates`, **When** an agent tool of the person then calls `updates` without `since`, **Then** it still lists the changes since its own last call.

## Constraints & Edge Cases

- **macOS only.** Apps for other systems are not planned.
- **Same clone as the agent tools.** With the same `--repo`, the app and an agent session use the same clone. The lock of `git-sync` keeps them apart. A long update of one waits for the other.
- **Load on the Git host.** Every poll is one fetch per tool call. A short interval with many watched items means many fetches. The minimum interval is 10 seconds.
- **Own changes in a folder.** `updates` does not list the person's own changes. The folder window sees them through `activity`, which lists every write of the person with its folder. The document window always sees a change, through `version`.
- **Manual edits in a folder.** A manual edit that is not committed is in no `updates` or `activity` result. The folder window shows it at the next full read: when the window opens, or after "Refresh now".
- **Version skew.** The app works with any `waymark` that has the tools above. If `waymark` is too old, `status` shows the version, and the app asks to update.
- **Manual edits** in the clone that are not committed show in the app, because the read tools read the files. They have no "changed by" name.
- **macOS 14 or later.**
- **Not in the Mac App Store.** The sandbox blocks starting the `waymark` binary.
- **Undo is the person's own action.** The app is not an agent. A click in the app is the person, so the app adds no extra approval of its own beyond the preview and the confirm step.
- **Undo only of the person's own writes.** `undo` refuses commits of other people (`not_found`). The app only offers it in the recent writes window, which lists the person's writes only.

## Open Questions

None. Answered on 2026-11-17 (decided autonomously, see `decisions.md`): no personal server connection in the first version, the folder poll uses `updates` and `activity` as its cheap signal, the minimum is macOS 14, no write other than Undo in the first version, and the reviewed state stays on one Mac.
