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
