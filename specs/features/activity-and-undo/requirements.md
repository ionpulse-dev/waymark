---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - mac-app
  - write-documents
---

# Activity and undo

## User Story

> As a person, I want to see everything my agents wrote under my name and undo any write with one click, so that letting my agent write on its own feels safe.

## Personas

- **Every person who allows write level `auto`** — relies on undo as the safety net.

## UI / Visual Design

### "What my agent wrote"

A list in the app of the writes made under my account, newest first. Each entry shows the title, the folder, the time, and a button **Undo**. Undo makes a revert commit under my account and pushes it.

### Error States

- The document changed after the write: undo shows what would be reverted and asks. It does not overwrite newer changes silently.
- Sync fails: a notification says that the undo is not pushed yet.

## Behaviour

- The list comes from Git history of the knowledge base, filtered by the person's author email.
- An undo is a new commit. History stays.
- The CLI offers the same function for the app to call.

## Interfaces & Contracts

The app calls the CLI to list recent writes and to revert one commit.

## Acceptance Criteria

- **Given** an agent wrote a document, **When** the list opens, **Then** the document is first in the list.
- **Given** Undo is clicked, **When** the revert succeeds, **Then** a new commit under my account reverts the write and is pushed.
- **Given** a later change by someone else to the same document, **When** Undo is clicked, **Then** the app warns before reverting.
- **Given** writes by other people, **When** the list opens, **Then** they do not show.

## Constraints & Edge Cases

- Undo of a move reverts the move and the link rewrites together.

## Open Questions

- How far back does the list go?
