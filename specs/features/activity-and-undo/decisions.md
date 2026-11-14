# Decisions: Activity and undo

> **Note 2026-11-11:** The decision "Undo makes autonomy safe" stays valid. Undo is now the `undo` tool and the console, not an app button.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Undo makes autonomy safe — 2026-11-06

**Decision:** One of three app features in the first release: a list of my writes with a one-click undo as a revert commit.

**Reason:** It supports the trust design. Undo makes autonomy safe.

## Tools and console, no app — 2026-11-11

**Decision:** `activity` and `undo` are tools. The console shows the same list on a server.

**Reason:** the app is dropped. Undo stays the safety net for agents that write freely.

## No `force`, a clean revert decides — 2026-11-14

**Decision:** `undo` has no `force`. If later commits changed the same files, the tool tries `git revert --no-commit`. A clean result is committed. A conflict ends with `stale_base` and names the later commits.

**Alternatives considered:** `force: true` after the person agreed (rejected: it overwrites newer work and the agent can set it alone). Always refuse when later commits exist (rejected: a later change in another part of the document is harmless, and Git can say this).

**Reason:** Git knows if the revert is safe. The person is protected without a flag the agent could use. Supersedes the `force` text of the earlier requirements. Decided autonomously.

## How far back the list goes — 2026-11-14

**Decision:** `activity` has `limit` (default 10, at most 50), `since` (a date), and `before` (a commit) for paging. There is no other limit.

**Alternatives considered:** A fixed window such as 30 days (rejected: a quiet knowledge base would show nothing).

**Reason:** The agent asks for what it needs. A small page keeps the context cost low. Decided autonomously.

## Only your own writes — 2026-11-14

**Decision:** `activity` shows and `undo` reverts only commits whose author email is the email of the person. A commit of another person gives `not_found`.

**Alternatives considered:** Let anyone undo anything (rejected: one person's agent could revert a colleague's work without a trace of intent).

**Reason:** This matches the user story and the wiki model: each person answers for their own writes. Decided autonomously.

## Moves are grouped by a message line — 2026-11-14

**Decision:** The link-fix commit of a move has the body line `Move: <hash>`. `activity` shows the pair as one entry, and `undo` reverts both in one commit. The undo commit has `Reverts: <hash> [<hash>]` in its body, and `activity` reads it to compute `undone`.

**Alternatives considered:** Git notes or a trailer file (rejected: they do not travel reliably with every clone). Time-based grouping (rejected: unreliable).

**Reason:** The message body travels with the commit and needs no extra state. Needs a message body in `gitsync::message`. Decided autonomously.

## The revert is not content-checked — 2026-11-14

**Decision:** `undo` runs the limits, the lock, and the approval, but not the content checks of `write-documents`.

**Alternatives considered:** Run the checks (rejected: an undo must always be able to restore an earlier valid state, and the state was valid when it was written).

**Reason:** Undo is the safety net, so it must not fail on a new duplicate title or a new broken link. `lint` reports them. Decided autonomously.

## `git` 2.30 or newer is enough — 2026-11-14

**Decision:** `undo` uses `git revert --no-commit` and `git reset --hard HEAD` under the lock, on a clean tree. It does not use `git merge-tree --write-tree`.

**Alternatives considered:** `merge-tree` for a dry run (rejected: needs Git 2.38 and does not handle a move with two commits).

**Reason:** Fewer requirements on the Git version of the person. Decided autonomously.

## Implementation notes — 2026-11-14

**Decided autonomously.**

- **One pass over the history.** `Repo::all_commits` reads all commits (no merges) with body and changed files in one `git log`. `activity::entries` builds the entries from it. A knowledge base has few thousand commits at most, so no cache.
- **`undone` is computed in time order.** An undo marks the commits it reverts. If it reverts another undo, the targets of that undo are live again (a redo).
- **`later_changes` ignores the undo commit** of the entry itself and the link-fix commit of a move.
- **Undo applies in two steps.** The tool first tries `git revert --no-commit` to find conflicts and to list the changes, then takes it back. `writes::execute` applies it again after the approval, inside the lock, and commits the staged paths. A conflict becomes `stale_base` and names the later commits. `git revert --abort` and `git reset --merge` keep unrelated local changes.
- **Hidden entries.** `activity` leaves out entries whose documents are all outside the `folders` limit. An `undo` for such a commit gives `not_found`.
- **`commit_change` filters paths.** A path that is gone from the disk and from the index (a revert removed it) is not given to `git add`.
- **A write that touched only non-documents** (for example `.mkbase/waymark.toml`) has no entry.
