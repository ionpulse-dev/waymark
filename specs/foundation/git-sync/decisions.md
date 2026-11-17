# Decisions: Git sync

## Name and content of the second file in a conflict — 2026-11-14

**Decision:** The remote version stays at the original path. The local version is saved as `<name>-conflict-<7 hex>.md` (hex from the SHA-256 of the local content). It gets a title suffix, `disputed: true`, and a `dispute_reason` that says to merge the two.

**Alternatives considered:** Keep Git conflict markers (rejected: breaks the document format and the person sees Git words). Keep the local version at the original path (rejected: it would hide the newer remote version).

**Reason:** Both versions stay valid documents. The dispute flag shows in every read. The rule needs no Git knowledge. Decided autonomously.

## The lock is an operating system file lock — 2026-11-14

**Decision:** `.git/waymark.lock` uses `std::fs::File::lock`. It is released when the process ends, even after a crash.

**Alternatives considered:** A lock file that is created and removed (rejected: a crash leaves a stale lock).

**Reason:** Two servers on one clone are safe, and no stale lock exists. Decided autonomously.

## A repository without a remote is supported — 2026-11-14

**Decision:** If `origin` is missing, waymark skips fetch and push and adds a notice to each result.

**Alternatives considered:** Require a remote (rejected: a private person may start local, and tests need it).

**Reason:** Local use costs nothing and the notice keeps the person informed. Decided autonomously.

## Commit message verbs — 2026-11-14

**Decision:** The verbs are `add`, `update`, `dispute`, `deprecate`, `delete`, `move`, `undo`, `sync`. The format is `waymark: <verb> "<title>"`.

**Reason:** One format lets `activity` list and group writes by reading `git log`. Decided autonomously.

## Hooks are respected — 2026-11-14

**Decision:** waymark does not skip Git hooks. A failing hook gives `sync_failed` with the hook output in `message`.

**Reason:** The owner of the repository set the hook on purpose. Decided autonomously.

## Update at the start of every call — 2026-11-17

**Decision:** Every read call brings the clone up to date at its start, as every write already does. The 3 minute freshness window and its variable `WAYMARK_FETCH_MAX_AGE` are removed. Calls in one process that start while an update runs share that update.

**Supersedes:** "Pull before read — 2026-11-06" and "The fetch age is a variable that tests can set — 2026-11-14" in [search-and-read](../../features/search-and-read/decisions.md).

**Alternatives considered:**
- Keep the window, with a default of 0. Rejected: a setting that nobody changes is code, spec, and tests with no use.
- A shorter window (10 to 30 seconds). Rejected: a read can still miss a change, and the window is hard to explain.
- Fetch in every call without sharing. Rejected: parallel calls would each fetch and wait for the lock.

**Reason:** The product owner wants every call to see the latest state. The window existed to make bursts of reads cheap. The shared update covers parallel calls. A burst of calls one after another pays one fetch per call, about 0.2 to 2 seconds. The old window also had a gap: the timestamp was set after the fetch and before the fast-forward or rebase, so a failed rebase was not tried again for 3 minutes.

## Approved — 2026-11-17

**Decision:** The product owner approved the change "Update at the start of every call" on 2026-11-17. The spec moves to `planned`.

## The shared update is built; `status` does not update — 2026-11-17

**Decision:** `Repo::refresh_for_read` has the update gate of plan step 13. The test of the shared fetch runs eight threads on one `Repo` and accepts one or two updates: a thread that starts after the first update ended runs its own, which is correct. `status` does not call `refresh_for_read`. It checks the remote with its own probe and reports `last_pull`, and it must answer even when the update hangs.

**Reason:** Decided autonomously during the build. The rule "every read call" is about the tools that read documents.

