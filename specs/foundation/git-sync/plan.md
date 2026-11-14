# Plan: Git sync

## Overview

A `gitsync` module wraps the `git` binary. It gives the rest of the code one type, `Repo`, with a write transaction that holds a lock, brings the clone up to date, lets the caller change files, and commits and pushes under the right author. Callers never run `git` themselves.

## Components Affected

- Every write tool uses `Repo::write`.
- Read tools call `Repo::refresh_if_stale`.
- `sync`, `activity-and-undo`, and `manual-edits` call the other methods listed below.

## New Files

- `src/gitsync/mod.rs` — `Repo`, errors, and the public API.
- `src/gitsync/cmd.rs` — runs `git` with an isolated environment, captures output, maps failures to errors.
- `src/gitsync/lock.rs` — the lock file `.git/waymark.lock` with an operating system file lock (`File::lock`).
- `src/gitsync/write.rs` — the write transaction: lock, fetch, fast-forward, base check, change, commit, push.
- `src/gitsync/sync.rs` — the `sync` operation and the conflict rule.
- `src/gitsync/log.rs` — read history: last commit of a path, commits by author, show a file at a commit.
- `src/gitsync/message.rs` — the commit message format.
- `tests/it_git_sync.rs` — end-to-end tests with two clones.

## Technical Approach

1. **Commands.** Every call runs `git -C <repo>` with `-c user.name=... -c user.email=...` and the variables `GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL`, `GIT_COMMITTER_NAME`, `GIT_COMMITTER_EMAIL`, and `GIT_TERMINAL_PROMPT=0`, so a missing login fails fast and never waits for input. Output is read as UTF-8 (lossy only for error text).
2. **Author.** From `--author-name` and `--author-email`, or the environment variables, or `git config user.name` and `user.email` of the person. If none exists, `setup_problem`.
3. **Lock.** `.git/waymark.lock` is created if missing and locked with the file lock. A second process waits up to 30 seconds, then returns `sync_failed` with the message "another waymark call holds the lock".
4. **Update.** `git fetch` then, if there are no local commits ahead, `git merge --ff-only @{u}`. If the clone is ahead (an earlier push failed), `git pull --rebase`. A failed fetch in a read is a notice and in a write is `sync_failed`.
5. **No remote.** If the repository has no `origin`, fetch and push are skipped and every result carries the notice "this repository has no remote".
6. **Base check.** `show` returns `version` for a document: the short hash of the last commit that touched its path (`git log -1 --format=%h -- <path>`). A write sends that value as `base`. If the last commit now differs, the write fails with `stale_base`.
7. **Commit.** The caller stages exact paths. One commit per write. The message is `waymark: <verb> "<title>"` with the verbs `add`, `update`, `dispute`, `deprecate`, `delete`, `move`, `undo`, `sync`. No agent tag.
8. **Push.** `git push`. A rejection gives `sync_failed` and does not retry. The local commit stays and is pushed by the next write or sync.
9. **Preview.** With `preview: true` the transaction runs the checks and returns the planned change without writing.
10. **Sync.** `sync`: commit pending manual edits (owned by `manual-edits`), `git pull --rebase`, `git push`. On a conflict, abort the rebase. Then for each conflicted path: the remote version stays at the original path. The local version is saved as `<name>-conflict-<7 hex>.md` next to it, where the hex is the start of the SHA-256 of the local content. The saved copy gets the title suffix " (conflict copy <7 hex>)", `disputed: true`, and `dispute_reason: "Edited at the same time as <id>. Merge the two."`. Both files are committed (`waymark: sync "<title>"`), and a notice names both.
11. **Clone on first start.** `--repo` with a URL: `git clone` into `--path` if it does not exist. A clone failure is `sync_failed`.
12. **Pull freshness.** A read pulls when the last good fetch is older than 3 minutes (kept in memory for the session).

## Dependencies on Other Foundation Items

- `knowledge-base-repo` — the root and the repository path.
- `document-format` — the conflict copy edits frontmatter through the `format` module.

## Risks

- `/usr/bin/git` on macOS can be a stub that opens a dialog. Mitigation: `status` runs `git --version` and reports `setup_problem` with a `next` sentence.
- Credential prompts hang. Mitigation: `GIT_TERMINAL_PROMPT=0` and `GIT_ASKPASS` unset.
- A hook in the repository changes behavior. Mitigation: hooks of the owner are respected (see `decisions.md`), and a failing hook gives `sync_failed` with its output.

## Open Questions

None.
