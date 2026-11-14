# Plan: Manual edits and sync

## Overview

A `manual` module finds pending edits in the working tree, checks them with `lint`, and commits the ones that pass. The `sync` tool calls it inside `gitsync::Repo::sync`. Every write flow calls it first through a hook that `Context` installs on the `Repo`.

## Components Affected

- `gitsync` — `Repo::sync` takes a hook that runs after the lock and before the pull. `Repo::write` calls an installed pending-edit hook after the fetch and before the base check, and refuses with `stale_base` if a path that it changes has uncommitted edits. Both are additions (see `decisions.md`).
- `lint` — `lint::run_on(paths)` checks pending files with the whole store, so duplicates and links resolve.
- `format` — `render` sets `updated` and `updated_by` without changing the body.
- `store` — the document set and `Scope`.
- `mcp` — `Context` installs the hook. The registry lists `sync` also in read-only and paused states.
- `config` — `access`, `paused`, `tools_denied`, `approval`.

## New Files

- `src/manual/mod.rs` — `commit_pending(tx, ctx) -> PendingReport`: the steps of the requirements.
- `src/manual/pending.rs` — runs `git status --porcelain=v1 -z --untracked-files=all` for the root and maps each entry to `PendingEdit { path, id, kind }` or a skip with a reason.
- `src/manual/touch.rs` — sets `updated` and `updated_by` on a passing changed document.
- `src/tools/sync.rs` — the `sync` tool.
- `tests/it_manual_edits.rs` — end-to-end tests.

## Technical Approach

1. **Pending.** Parse `git status` output. Keep `.md` paths below the root that are documents (no dot path, not in `exclude`). `??` and `A` are `new`, `M` is `changed`, `D` is `deleted`. Other paths are skipped with the reason "Not a document." or "Excluded.". The file `.mkbase/waymark.toml` and other files in `.mkbase/` are skipped and never committed.
2. **Lint.** Load the `Store` from the working tree. Run `lint::run_on` for the pending `new` and `changed` documents. An error finding blocks the document. A deleted document has nothing to check and is committed.
3. **Touch.** For each passing `changed` document, if `updated` is not today or `updated_by` is not the author, call `format::render` with the new values and write the file. Nothing else changes. A `new` document keeps what the person wrote.
4. **Approval.** Call the approval step of `gitsync` (see `write-approval`) with the tool name `sync` and the list of IDs. A refusal returns `approval_denied` or `approval_unavailable` before anything is committed.
5. **Commit.** Stage exact paths (`git add -- <paths>`, and `git rm` for deleted ones). Make one commit with `gitsync::message` (verb `sync`, title as the requirements say).
6. **Sync tool.** `sync` runs `Repo::sync(hook)`. The hook is `manual::commit_pending`. After it, `Repo::sync` does `pull --rebase`, the conflict rule, and `push`. The result of the hook and of the sync fill `committed`, `blocked`, `skipped`, `pulled`, `pushed`, `conflicts`.
7. **All blocked.** If the hook committed nothing and `blocked` is not empty, return `validation_failed` before the pull. The `message` names the first document and finding. The `next` sentence says to fix the document and run `sync` again.
8. **Read-only and paused.** The registry lists `sync` always (unless `tools_denied` names it). The tool checks `EffectiveSettings`: if writes are not allowed, it skips the hook and the push, runs only the fetch and fast-forward, and adds the notice "Writes are not allowed, so nothing was committed or pushed."
9. **Preview.** The hook runs steps 1 and 2 only. It returns the plan. The tool does not pull or push.
10. **Before other writes.** `Context` calls `Repo::install_pending_hook(commit_pending_quiet)`. `Repo::write` calls it after the fetch. The quiet variant commits passing documents and, for blocked ones, adds the notice "N manual edits are not published. Run `sync`." It does not return an error. A `preview: true` write does not run the hook. After the hook, `Repo::write` checks that each path of the write has no uncommitted change. If it has, it returns `stale_base` with the `next` sentence of the requirements.
11. **Hand moves.** Git sees a delete and a new file. The one commit holds both, so `git log --follow` and rename detection find the move. `search-and-read` follows it.
12. **No timer.** There is no watcher and no background job.

## Cross-Feature Dependencies

- `quality-checks` — `lint::run_on`. Listed in `requires`.
- `git-sync` — the hooks on `Repo::sync` and `Repo::write`, and the conflict rule. Reported as additions.
- `write-approval` — the approval step. If it is not built yet, the step is a no-op that allows, and the test for approval is marked as waiting for it in `todo.md`.
- `mcp-server` — `sync` is listed in read-only and paused states. The listing rule of the plan of `mcp-server` (step 4) needs this exception.
- `search-and-read` — `show` follows a hand move.

## Risks

- A hook that commits before every write adds time to a write. Mitigation: `git status` for one root is fast, and the hook does nothing when the tree is clean.
- A person edits while the agent writes. Mitigation: the lock covers the commit. An edit during the lock may be missed and is picked up by the next call.
- `updated` and `updated_by` change a file that the person just saved in Obsidian, which may show a "file changed on disk" prompt. Accepted, because the standard requires the update.
- Approval may not exist yet. Mitigation: see above.

## Open Questions

None.
