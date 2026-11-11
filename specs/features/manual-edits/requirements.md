---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - quality-checks
---

# Manual edits and sync

## User Story

> As a person, I want to edit or move documents in Obsidian or any editor and have them published for me, so that I never need Git.

## Personas

- **Every person** — edits documents by hand.
- **Obsidian user** — browses and edits the knowledge base as a vault.

## Behaviour

In local mode, the clone of the knowledge base is a normal folder. A person can open it as an Obsidian vault and edit it. waymark has no background job, so manual edits are published when the server runs:

- The `sync` tool commits pending manual edits, pulls, and pushes. The commit runs `lint` first, uses the message "Manual edit", and uses the author of the person.
- Every write tool commits pending manual edits first, in a separate commit, so that an agent write never carries foreign changes. The pending edits go through the same checks.
- If approval is required for writes (see `write-approval`), it is asked before the commit of manual edits too.
- If the checks fail, `sync` reports the findings as `validation_failed` and pushes nothing. The edits stay in the folder.
- Moves done by hand are committed the same way. `show` with an old ID follows Git history after a move.

In server mode, people do not edit the server's clone. They edit in their own clone and use Git or an Obsidian plugin to push. The server pulls in the background.

## Interfaces & Contracts

The `sync` tool takes no arguments except `preview: true`. The result lists what was committed and pushed.

## Acceptance Criteria

- **Given** an edit in Obsidian, **When** `sync` runs, **Then** a commit "Manual edit" under my account reaches the remote.
- **Given** an edit that fails `lint`, **When** `sync` runs, **Then** it fails with `validation_failed`, names the problem, and pushes nothing.
- **Given** pending manual edits, **When** an agent writes a document, **Then** the manual edits are committed first in their own commit.
- **Given** approval is required for writes, **When** manual edits are ready, **Then** the request opens before the commit.
- **Given** `access = "read-only"`, **When** `sync` runs, **Then** it pulls and does not commit or push.

## Constraints & Edge Cases

- Edits made in Obsidian bypass the write checks until they are committed. `lint` is the safety net.
- Files that Obsidian creates for personal use are ignored by Git. Only `.obsidian/app.json`, with the link options of MKBase 1.0 section 11, is kept.
- A person who edits and never runs a session leaves edits unpublished. This is a known limit of having no background job.

## Open Questions

- Should the server commit pending manual edits automatically after a quiet minute while it runs, or only on `sync` and before writes?
- Is a small optional helper for people without agents needed later?
