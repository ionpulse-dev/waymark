---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - mac-app
  - quality-checks
  - git-sync
---

# Manual edits and Sync now

## User Story

> As a person, I want to edit or move documents in Obsidian or any editor and have them published for me, so that I never need Git.

## Personas

- **Every person** — edits documents by hand.
- **Obsidian user** — browses and edits the knowledge base as a vault.

## Behaviour

- The app sees uncommitted changes in the content repository. After about 1 minute without further changes, it runs `lint`, commits under the person with the message "Manual edit", and pushes.
- At write level `confirm` and confirm in `app`, the approval dialog asks first (see `approval-dialog`).
- If the checks fail, a notification says what is wrong, and nothing is pushed.
- Moves done by hand are committed the same way. `waymark show <old-id>` follows Git history after a move.
- The menu has **Sync now**. It runs a sync at once, including the commit of manual edits, and shows the result. The CLI equivalent is `waymark sync`.

## Interfaces & Contracts

The app calls `lint`, then a commit and push through the CLI. It uses the lock of the write flow.

## Acceptance Criteria

- **Given** an edit in Obsidian, **When** one minute passes without changes, **Then** a commit "Manual edit" under my account reaches the remote.
- **Given** an edit that fails `lint`, **When** the minute passes, **Then** a notification names the problem and nothing is pushed.
- **Given** "Sync now", **When** I choose it, **Then** pending manual edits are committed and the result shows.
- **Given** write level `confirm` and confirm in `app`, **When** a manual edit is ready, **Then** the dialog opens before the push.
- **Given** an agent write in progress, **When** the app wants to commit, **Then** it waits for the lock.

## Constraints & Edge Cases

- Edits made in Obsidian bypass the write checks until this step. `lint` is the safety net.
- Files that Obsidian creates for personal use are ignored by Git. Only `.obsidian/app.json`, with the link options of MKBase 1.0 section 11, is kept.

## Open Questions

- What happens when the person keeps editing for a long time? Does the one-minute timer restart?
