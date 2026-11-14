# Decisions: Updates

> **Note 2026-11-11:** All decisions stay valid. `updates` is a tool.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## `updates` with an explicit date — 2026-11-06

**Decision:** The command is named `updates` and takes `--since <date>`. Without it, it uses the last call.

**Reason:** A clearer name and an explicit date.

## `updates` is in the first release — 2026-11-06

**Decision:** The command is part of the first release. (The skill no longer runs it at session start, see below.)

**Reason:** It is cheap to build (reads Git history) and closes the "no push" gap.

## No startup report and no stale — 2026-11-09

**Decision:** The skill does not run `updates` at session start. The person asks when they want it. The `stale` command, `stale_after_days`, and every stale mark are removed. The skill says only: if `updated` is old, check the facts that matter before you rely on them, and say how old the document is.

**Reason:** The tested rules of the waymark-trail skills dropped both. A startup report costs context in every session. A stale mark means "somebody else should look at it": a person who doubts a document fixes it or disputes it right away.

**Supersedes:** `stale` has the same shape (2026-11-06); `updates` is in the first release (the part about session start)

## `updates` is a tool — 2026-11-11

**Decision:** The time of the last call is stored per person in the personal state folder, or on the server.

**Reason:** MCP is the only interface.

## The stored time is one file per person and knowledge base — 2026-11-14

**Decision:** The time of the last `updates` call without `since` is stored in `<state dir>/<kb id>.json`, key `last_updates`. The state dir is `$WAYMARK_STATE_DIR`, else `$XDG_STATE_HOME/waymark`, else `~/.local/state/waymark`. All agent tools of the person on the machine share it. The last call wins.

**Alternatives considered:**
- One file for each agent tool — rejected: the server cannot tell which tool started it, and the person wants one "since I last looked".
- A file in `~/.config/waymark/` — rejected: that folder holds limit layers, and state is not config.
- A file in the clone — rejected: it is per machine state, not part of the knowledge base.

**Reason:** Answers the open question. Decided autonomously.

## Only a call without `since` moves the time — 2026-11-14

**Decision:** A call with `since` is a pure query. A call without it lists since the stored time and stores now.

**Alternatives considered:** Every call moves the time (rejected: a look at an old date would hide newer changes from the next call).

**Reason:** The person controls what "seen" means. Decided autonomously.

## First call looks back 7 days — 2026-11-14

**Decision:** With no stored time and no `since`, the window starts 7 days ago.

**Alternatives considered:** The whole history (rejected: too long). The start of the session (rejected: always empty).

**Reason:** A week is a usual gap between two looks. Decided autonomously.

## Own changes are not listed — 2026-11-14

**Decision:** A path whose commits in the window are all by the person's email is left out. A path with at least one commit by someone else stays, with that person as `by`.

**Reason:** The person knows their own writes. `activity` lists them. Decided autonomously.

## Net change between two commits — 2026-11-14

**Decision:** `updates` compares the tree at the window start with `HEAD` and classifies the net change. It does not replay each commit.

**Alternatives considered:** Walk each commit (rejected: slower, and many entries per document).

**Reason:** One entry per document is what a reader needs. Decided autonomously.

## At most 50 entries, no token cap — 2026-11-14

**Decision:** The list is cut at 50 entries. The `max_results` and `max_tokens` caps of `search` do not apply.

**Reason:** The caps are small and fit a search. An update list needs to be complete for the window, and an entry is short. Decided autonomously.

## Foundation additions — 2026-11-14

**Decision:** This spec adds `src/gitsync/diff.rs` and `src/state/mod.rs`, and the dependency on `search-and-read` for `Store`. `requires` becomes `search-and-read` (it was already so).

**Reason:** Reported to `git-sync`, because the module `gitsync` gets a new file. Decided autonomously.

## Implementation notes — 2026-11-14

- Files: `src/tools/updates.rs`, `src/store/changes.rs` (`changes_since`, `Kind`, `email_of`), `src/gitsync/diff.rs` (`base_commit_before`, `changed_paths`, `window_commits`), `src/state/mod.rs` (`State`). As planned.
- A path counts as the person's own when every commit in the window has the person's email. Renames are followed (`git log --follow`).
- A new document stays `new` even if someone disputed it later in the window. To see a dispute, the window must start after the document was created (the tests do so with a first call that stores the time). This follows the rule "a new document stays new".
- `since` gives a base commit with `git rev-list -1 --before=<time> HEAD`. Commit times have a resolution of one second, so the tests pause 1.1 seconds between a stored time and the next commit.
- The harness sets `WAYMARK_STATE_DIR` to `<home>/state` for every client.
- There are no separate unit tests for `gitsync::diff`. The end-to-end tests cover rename, delete, and the empty-tree base. Decided autonomously.
- The test of the size of `tools/list` against the budget is part of phase G (task G5).
