---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - mac-app
  - updates-and-stale
---

# Notifications and review list

## User Story

> As a person, I want to hear about problems with my documents and my knowledge base, so that I can act without watching anything.

## Personas

- **Every person** — gets macOS notifications and one list of open items.

## UI / Visual Design

### Notifications

macOS notifications for:

- a document of mine was disputed or changed by others,
- sync fails (for example the login expired),
- writes are paused,
- a manual edit failed the checks (see `manual-edits`).

### Review list

One list in the app with:

- disputes on my documents,
- my stale documents.

Each item opens the document and offers the next step.

## Behaviour

- The app reads the data from `updates` and `stale`, and from the status file of the background sync.
- With verification on, documents waiting for verification show too (see `verification`).
- The same information also arrives as `notices` in CLI output.

## Interfaces & Contracts

The app calls `updates`, `stale`, and `status` through the CLI.

## Acceptance Criteria

- **Given** someone disputes my document, **When** the next sync completes, **Then** a notification appears.
- **Given** the login expired, **When** a sync fails, **Then** a notification explains what to do.
- **Given** a stale document of mine, **When** the review list opens, **Then** it shows with an action.
- **Given** no problems, **When** the list opens, **Then** it is empty and says so.

## Constraints & Edge Cases

- waymark cannot reach a person who uses neither an agent nor the app.

## Open Questions

- How often should notifications repeat for the same problem?
