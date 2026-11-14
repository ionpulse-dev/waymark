# Decisions: Move documents

> **Note 2026-11-11:** All decisions stay valid. `move` is now a tool.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Moves keep history — 2026-11-07

**Decision:** `git mv`, no content change in the move, rewrites and move in one commit, and a check with `git log --follow`.

**Alternatives considered:**
- Delete and re-create — loses the history of the document.

**Reason:** Users do not want to lose the history of a document. Git detects a rename only if the files are similar.

## One command for reorganizations — 2026-11-06

**Decision:** `move` handles folders and documents and rewrites links.

**Reason:** Folders are teams, and teams reorganize.

## Two commits and rename — 2026-11-09

**Decision:** A move is two commits: first a pure `git mv`, then the link fixes. A rename is a move.

**Reason:** A pure rename commit makes Git detect the rename for sure (similarity 100%). The link fixes in a second commit keep the history of the moved file. This is the rule of the tested waymark-trail skills.

## `move` is a tool — 2026-11-11

**Decision:** The move rules and the two-commit rule stay. A team can switch the tool off with `tools_denied`.

**Reason:** MCP is the only interface.

## Two commits, one transaction — 2026-11-14

**Decision:** The two commits of the earlier decision are made inside one `Repo::write` call. They share one lock and one push. The second commit is skipped when no link needs a fix. The second commit carries `Move: <hash of commit 1>` in its message body.

**Alternatives considered:** Two separate write calls (rejected: another write could come between them, and the first push would publish broken links). One commit (rejected: breaks rename detection).

**Reason:** The pair must stay together for `undo` and for readers of the history. Decided autonomously.

## Folder or document by a trailing slash — 2026-11-14

**Decision:** `from` and `to` are IDs for documents and paths with a trailing `/` for folders. A folder never merges into an existing folder. `to` must not exist.

**Alternatives considered:** A `kind` field (rejected: one more field for every call). Guess by checking the files (rejected: a document and a folder can have the same name).

**Reason:** The call is short and exact. Decided autonomously.

## No `base` in `move` — 2026-11-14

**Decision:** `move` takes no `base`. The tool reads the current files after the fetch, inside the lock.

**Alternatives considered:** One `base` per document (rejected: a folder move has many documents, and a move changes no content).

**Reason:** A move cannot overwrite content. Decided autonomously.

## Inbound links from outside the knowledge base — 2026-11-14

**Decision:** They are not checked. Every result has the notice "Links from outside this knowledge base are not checked."

**Alternatives considered:** Scan the whole repository (rejected: the root of the knowledge base is the boundary of the standard, and other repositories are out of reach).

**Reason:** Answers the open question of the original spec. Decided autonomously.

## The history check runs before the push — 2026-11-14

**Decision:** After the commits, and before the push, each moved document is checked with `git log --follow`. A failure removes the unpushed commits of this call and fails with `validation_failed`.

**Alternatives considered:** Check after the push (rejected: a bad move would already be public). Only warn (rejected: the rules say the server verifies).

**Reason:** The history is the point of the two-commit rule. This needs a check step and a rollback in `git-sync`. Decided autonomously.

## `show` redirect belongs to `search-and-read` — 2026-11-14

**Decision:** The criterion "`show` with an old ID names the new ID" is removed from this spec. This spec only guarantees pure rename commits, so `git log --follow` works.

**Reason:** `move-documents` does not require `search-and-read`, and `show` is a tool of that spec. Decided autonomously. Reported as a conflict.

## Implementation notes — 2026-11-14

**Decided autonomously.**

- **Two commits through one write path.** `writes::Plan` has an optional `first: FirstCommit` (the moves, the title, and the history list). `writes::execute` makes that commit first, then the link fixes as the usual commit, then one push. The link fixes are computed on the text before the move and written to the new paths.
- **History check.** For each moved document, `git log --follow` on the new path must reach the last commit of the old path. If not, `git reset --mixed` removes the move commit, the transaction restores the files, and the call fails with `validation_failed`. The test hook `WAYMARK_TEST_HIDE_HISTORY` makes the check fail.
- **Push of a lone commit.** `Repo::write` pushes unpushed commits also when its own commit has no change, so a move without link fixes is pushed.
- **Folder moves** move every file below the folder, also images and other files. Empty source folders are removed.
- **Relative links.** A link text is rewritten only if it no longer resolves to the moved target from the new location of the linking document. Links between documents that move together keep their text. A leading `/` stays absolute.
- **Short wikilinks** are rewritten to the full new path only if the short name no longer finds exactly one document, which was the target before.
- **Edits that wait.** A moved document with an unpublished manual edit gives `stale_base`, as in the other write tools.
- **Kind.** `move` is a write tool (`Kind::Write`). The commit verbs are `move` and `update`.
