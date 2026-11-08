---
type: foundation
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - knowledge-base-repo
---

# Git sync

## Technical Goal

Define how waymark keeps the local clone and the remote in step and how every write reaches the remote safely, so that people never see a branch, a rebase, or a merge conflict.

## Dependents

- `write-documents`, `move-documents`, `verification`, `manual-edits` — use the write flow.
- `search-and-read` — reads pull first.
- `mac-app` — runs the background sync.
- `manual-edits` — commits under the lock.

## Deliverables

- The write flow with its lock.
- The sync command and the background sync run.
- The conflict rule.
- The commit message format and the author setup.

## Scope

**In scope:**
- Calls to the `git` binary, the lock, pull, commit, push, conflicts.

**Out of scope:**
- Login to the Git host (see `login-and-identity`).
- Offline mode and retries.

## Constraints

- waymark calls the `git` binary. `git` is a requirement. `/usr/bin/git` on macOS can be a stub, so setup checks that `git --version` works.
- waymark runs every Git command with its own settings for the repository: author name and email, and the credential helper. The global Git settings of the person do not matter.
- No offline mode and no retries. A failed write or sync stops with a clear error and nothing waits in a queue.
- Commit messages have a fixed format, for example `waymark: add "Payment API v3 uses idempotency keys"`. There is no agent tag.

### Write flow

Every write command runs these steps:

1. Take the repository lock (`.waymark/lock`). Background sync waits.
2. Fetch and fast-forward. Always, with no age check.
3. For a change to an existing document, compare it with the version the agent read (`--base <hash>`). If it differs, stop with exit code 5.
4. Check duplicates (same ID or title), size limits, links, and secrets.
5. Write the file, commit under the person's account, and push.
6. If the push is rejected, stop with exit code 3.
7. Release the lock.

### Sync

`waymark sync` runs `git pull --rebase`, commits with a standard message, and pushes. Because there is one topic per document, most changes touch different files and rebase cleanly. If a real conflict on the same file remains, waymark aborts the rebase, keeps both versions as two files, and reports it. The repository never stays in a conflict state.

### Background sync

- The menu bar app runs a sync at an interval, default 5 minutes. There is no separate background job.
- A run skips if another command holds the lock. Agent writes always go first.
- If the Git host does not answer, it logs the error and stops. The next scheduled run is a new run.
- A run writes a small status file that `doctor` and `status` read.
- A background run never shows a window. If sync fails for more than one day, the next command prints a short warning.
- After the Mac wakes, the app runs the next sync at once.
- Other systems later: Linux could use a `systemd` user timer. macOS is the only target for the app.
- Before every read, waymark does a fast pull if the last good sync is older than a few minutes.

## Acceptance Criteria

- **Given** two people who write different documents, **When** both push, **Then** both reach the remote without a manual step.
- **Given** a document changed on the remote after the agent read it, **When** the agent writes with the old base hash, **Then** the write fails with exit code 5.
- **Given** a real conflict on one file, **When** sync runs, **Then** both versions are kept as two files, the rebase is aborted, and the person gets a notice.
- **Given** a rejected push, **When** a write runs, **Then** it stops with exit code 3 and does not retry.
- **Given** a person's global Git author, **When** waymark commits, **Then** the commit uses the author from waymark's settings.
- **Given** a running write, **When** the background sync starts, **Then** it waits or skips.

## Open Questions

- What is the exact rule for naming the second file when both versions are kept?
