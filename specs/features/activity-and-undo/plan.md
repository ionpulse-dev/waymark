# Plan: Activity and undo

## Overview

Two tool files. Both read the Git history through `gitsync::log`. `undo` runs inside `Repo::write`, uses `git revert --no-commit` for the revert, and makes one commit with the verb `undo`.

## Components Affected

- `gitsync` (`git-sync`) — `log` gets two helpers (see below), and `message` gets the message body (also needed by `write-documents`). `Repo::write` runs `undo` in its closure.
- `config` — `folders` globs. `kb` — the root and the document test (a path below the root that ends in `.md`, not excluded).
- `writes` (`write-documents`) — preview and result helpers.
- `mcp` — the `Tool` trait. `src/tools/mod.rs` — two lines in the registry.

## New Files

- `src/tools/activity.rs` — the `activity` tool.
- `src/tools/undo.rs` — the `undo` tool.
- `src/activity/mod.rs` — the model of an entry and the functions that build the list from the log: filter by author and subject, group a move with its link-fix commit, compute `undone` and `later_changes`.
- `tests/it_activity_and_undo.rs` — end-to-end tests with the `World` harness (two clones for the conflict cases).

## Technical Approach

1. **Author.** The email comes from `git-sync` author settings. Comparison ignores case.
2. **List** (`activity.rs`). Run `git log --format=%H%x1f%an%x1f%ae%x1f%aI%x1f%s%x1f%b%x1e` over the history of the current branch, newest first, without merges. Keep commits whose author email matches and whose subject matches `^waymark: (add|update|dispute|deprecate|delete|move|undo|sync) "(.*)"$`. Read the changed paths with `git diff-tree --no-commit-id --name-status -r -M <hash>`. Keep commits that changed at least one document file.
3. **Move grouping.** A commit with the body line `Move: <hash>` is the link-fix part of that move. It is not an entry. Its files are added to the entry of the move commit.
4. **`undone`.** Scan all `undo` commits (by anyone) for `Reverts:` lines. A commit is undone if an undo names it and that undo is not itself undone. The function is recursive and cached for one call.
5. **`later_changes`.** `git log --format=%h <hash>..HEAD -- <files>` is not empty. At most 50 entries per call, so at most 50 such calls.
6. **Paging.** `since` filters by author date. `before` skips entries until it passes the given commit. `next_before` is the hash of the last returned entry if older entries remain.
7. **`undo` flow.** Inside `Repo::write`, after fetch and fast-forward: find the entry (rule 1), check `undone` (rule 2), check that the tree is clean for the files (rule 3), check the `folders` limit (rule 5). Then run `git revert --no-commit <link-fix hash> <move hash>` (or the one hash). Exit code 0 means clean. A non-zero code means a conflict: restore the tree with `git reset --hard HEAD` (safe: the tree was clean at the start and the lock is held) and return `stale_base` with the later commits named (rule 4). For a preview, restore the tree after a clean result too.
8. **Commit.** The message is `waymark: undo "<title>"`, an empty line, and `Reverts: <hashes>`. `gitsync` commits under the author and pushes. `changes` come from `git diff --cached --name-status` before the commit.
9. **Needs from `git-sync`.** In `gitsync::log`: `changed_files(commit)` and `later_commits_touching(commit, paths)`. In `Repo`: a way to run a revert inside the write closure (`Repo::revert_no_commit(hashes)` and `Repo::reset_clean()`).

## Cross-Feature Dependencies

- `move-documents` — the `Move:` body line. It must stay as written there.
- `write-documents` — the message body format (`Reason:`) and the `writes` helpers.
- `manual-edits` — pending edits are committed before a write. Rule 3 only applies if edits appeared between that commit and the revert.
- `write-approval` — approval is asked after the checks, before the commit. The summary is "Undo: <title>".

## Risks

- `git revert` leaves a half state after a crash. Mitigation: the lock is released by the operating system, and the next call starts with a clean-tree check. A dirty tree with a `REVERT_HEAD` file is cleaned with `git revert --abort` at the start of every write by `git-sync`.
- A history of tens of thousands of commits slows `activity`. Mitigation: `git log` stops at `limit` entries, and `--author` is passed to Git to narrow the scan.
- A rebase changes commit hashes after a `sync`. Mitigation: `activity` always reads the current history, so hashes in an old result may no longer exist. `undo` then fails with `not_found` and `next` says to run `activity` again.

## Open Questions

None.
