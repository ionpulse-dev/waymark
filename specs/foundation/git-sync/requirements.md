---
type: foundation
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - knowledge-base-repo
---

# Git sync

## Technical Goal

Define how waymark keeps the local clone and the remote in step and how every write reaches the remote safely, so that people never see a branch, a rebase, or a merge conflict.

## Dependents

- `mcp-server` — every write tool uses the write flow.
- `write-documents`, `move-documents`, `verification`, `manual-edits`, `activity-and-undo` — use the write flow.
- `search-and-read` — reads pull first.
- `server-mode` — adds the background pull and the workspace per knowledge base.

## Deliverables

- The write flow with its lock.
- The `sync` tool.
- The conflict rule.
- The commit message format and the author setup.

## Scope

**In scope:**
- Calls to the `git` binary, the lock, pull, commit, push, conflicts.

**Out of scope:**
- The Git login (see `login-and-identity`).
- Offline mode and retries.
- The workspace layout of a shared server (see `server-mode`).

## Constraints

- waymark calls the `git` binary. `git` is a requirement. `/usr/bin/git` on macOS can be a stub, so `status` checks that `git --version` works.
- **Git handles the login.** In local mode, `git` uses the credentials that the person set up: an SSH key or a credential helper. waymark never reads, stores, or passes a secret. In server mode, the host provides them (see `server-mode`).
- waymark sets only the commit author, for each Git command, from its own settings. The global author of the person does not matter.
- No offline mode and no retries. A failed write or sync stops with a clear error and nothing waits in a queue.
- Commit messages have a fixed format, for example `waymark: add "Payment API v3 uses idempotency keys"`. There is no agent tag.
- The lock is a file in `.git/` of the clone. Only one write runs at a time per clone.

### Write flow

Every write tool runs these steps:

1. Check the limits (see `trust-model`). A failure ends the call with `access_denied` or `paused`.
2. Take the repository lock. `sync` and a background pull wait.
3. Fetch and fast-forward. Always, with no age check.
4. For a change to an existing document, compare it with the version the agent read (`base`). If it differs, stop with `stale_base`.
5. Check duplicates (same ID or title), size limits, links, and secrets. A failure ends with `validation_failed`.
6. If approval is required, ask for it (see `write-approval`). A refusal ends with `approval_denied` or `approval_unavailable`.
7. Write the file, commit under the author, and push.
8. If the push is rejected, stop with `sync_failed`.
9. Release the lock.

A tool takes `preview: true`, which shows what would be written and writes nothing.

### The `sync` tool

`sync` commits pending manual edits (see `manual-edits`), runs `git pull --rebase`, and pushes. Because there is one topic per document, most changes touch different files and rebase cleanly. If a real conflict on the same file remains, waymark aborts the rebase, keeps both versions as two files, and reports it in `notices`. The repository never stays in a conflict state.

### When the clone is updated

- **Local mode.** The server runs for one session. It pulls at start, before a read if the last good pull is older than a few minutes, and in every write flow. There is no background job and no app.
- **Server mode.** The server also pulls in the background at an interval, default 5 minutes (see `server-mode`).
- A pull that fails never blocks a read. The result carries a notice that the clone may be behind.

## Acceptance Criteria

- **Given** two people who write different documents, **When** both push, **Then** both reach the remote without a manual step.
- **Given** a document changed on the remote after the agent read it, **When** the agent writes with the old base, **Then** the write fails with `stale_base`.
- **Given** a real conflict on one file, **When** `sync` runs, **Then** both versions are kept as two files, the rebase is aborted, and a notice says so.
- **Given** a rejected push, **When** a write runs, **Then** it stops with `sync_failed` and does not retry.
- **Given** a person's global Git author, **When** waymark commits, **Then** the commit uses the author from waymark's settings.
- **Given** a running write, **When** `sync` starts, **Then** it waits for the lock.
- **Given** `preview: true`, **When** a write tool runs, **Then** nothing is written or committed.
- **Given** no network, **When** `search` runs, **Then** it answers from the local clone with a notice that the clone may be behind.

## Open Questions

- What is the exact rule for naming the second file when both versions are kept?
- Should the lock also protect against two servers that share one clone?
