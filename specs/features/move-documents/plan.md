# Plan: Move documents

## Overview

`move` is one tool file. It lists the documents that move, runs `git mv`, and then uses `linkfix` from `write-documents` to rewrite links. Both commits happen inside one write transaction of `gitsync::Repo::write` with one lock and one push.

## Components Affected

- `gitsync` (`git-sync`) — `Repo::write` must allow an ordered list of commits in one transaction, and a check step that runs before the push (see `decisions.md`).
- `writes` (`write-documents`) — `linkfix` (retarget), the `folders` check, preview and result helpers.
- `format` — `parse`, `render`, `links` (spans), `id` (file name rules).
- `kb` — the root and the list of documents. `config` — the `folders` globs.
- `src/tools/mod.rs` — one line in the registry.

## New Files

- `src/tools/move_documents.rs` — the `move` tool (the file has this name because `move` is a keyword in Rust; the tool name stays `move`).
- `src/writes/moveplan.rs` — builds the move plan: the list of renames, the new IDs, and the new relative paths.
- `tests/it_move_documents.rs` — end-to-end tests with the `World` harness.

## Technical Approach

1. **Parse input.** A trailing `/` means a folder. Check the kind rules, the names, and `to` against the existing paths. Reject a move into itself and a move of the root or `.mkbase/`.
2. **List documents.** For a folder, all `.md` files below it that are documents (not excluded). Other files move with them by `git mv` and are not listed.
3. **Plan** (`moveplan.rs`). A map from old ID to new ID. A function that gives the new relative path from document X (at its new place) to document Y (at its new place).
4. **Links.** `linkfix` scans all documents (moved and not moved), using `format::links` spans. It computes the rewrites of the table in the requirements. Rewrites of documents that move are applied at their new path in commit 2.
5. **Limits.** The `folders` glob must match the old and new IDs of all moved documents and the IDs of all documents that need a link fix. Any miss is `access_denied`.
6. **Transaction.** Inside `Repo::write`: (a) `git mv`, commit 1 (`waymark: move "<title>"`); (b) apply rewrites, set `updated` and `updated_by` on changed documents, commit 2 (`waymark: update "<title>"`, body `Move: <hash>`); skipped if there is no rewrite; (c) the check step: for each moved document, `git log --follow --format=%h <new path>` must include a commit that existed before the move. It checks at most 500 documents (the first 500 by path) and adds a notice if there are more; (d) push. A failed check rolls back the unpushed commits of this transaction.
7. **Pure rename.** Commit 1 is created from `git mv` only. The tool checks `git diff --cached -M100% --name-status` shows only `R100` lines before it commits. Any other line aborts the call.
8. **Preview.** The plan, with `moved` and `links_updated`, and no write.
9. **Result.** As in the requirements. The approval summary is "Move N documents, update links in M documents".

## Cross-Feature Dependencies

- `write-documents` — `linkfix`, helpers, tests.
- `activity-and-undo` — reads the `Move:` line to revert the two commits together.
- `search-and-read` — the redirect of an old ID uses the rename history.
- `write-approval` — the summary of the request.

## Risks

- `git mv` of a folder on a case-insensitive file system. Mitigation: names are lowercase, so a case-only rename does not occur.
- A rewrite that damages text. Mitigation: the same as in `write-documents` (spans only, no code).
- Many documents slow the history check. Mitigation: the cap above.
- Transaction rollback needs support in `git-sync`. Mitigation: the first task checks this.

## Open Questions

None.
