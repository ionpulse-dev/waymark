---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - mac-app
---

# Approval dialog

## User Story

> As a person, I want to see exactly what my agent wants to publish and approve it myself, so that nothing leaves my knowledge base without my yes.

## Personas

- **Every person who sets confirm in to `app`** — sees the dialog.
- **Knowledge base owner** — can set the weakest allowed value.

## UI / Visual Design

### Approval dialog

A native macOS dialog from the menu bar app. It shows the repository name, the folder, the title, and the full body, with the line "Visible to everyone with access to this knowledge base" and the buttons **Publish** and **Cancel**. Moves and bulk changes show a summary instead ("Move 42 documents, update links in 17 documents").

### Error States

- No answer in 2 minutes: the write is canceled. The agent gets "Canceled by the person" with exit code 6.
- The app is not running: the write fails with exit code 7 and "Approval needed. Open the waymark app."

## Behaviour

- It applies at write level `confirm` with confirm in `app`. At that level it also covers moves and bulk changes. Levels `auto` and `manual` never open a dialog.
- The dialog belongs to the app. The agent cannot answer it.
- Manual edits use the same dialog (see `manual-edits`).
- The skill tells the agent to allow enough time, because some tools stop a command after about 2 minutes.
- Confirm in `chat` is advice: the agent asks in the conversation, and waymark cannot prove that a human answered.

## Interfaces & Contracts

The CLI sends an approval request over the app link and waits for approved or canceled (see `app-link`).

## Acceptance Criteria

- **Given** write level `confirm` and confirm in `app`, **When** an agent writes, **Then** the dialog shows the full document.
- **Given** the person clicks Publish, **When** the answer arrives, **Then** the write continues.
- **Given** no answer for 2 minutes, **When** the time ends, **Then** the write is canceled with exit code 6.
- **Given** the app is not running, **When** a write needs approval, **Then** it fails with exit code 7.
- **Given** write level `auto`, **When** an agent writes, **Then** no dialog opens.
- **Given** an agent that controls only the terminal, **When** it tries to answer, **Then** it has no way to do so.

## Constraints & Edge Cases

- A person can click Publish without reading. An agent that controls the screen could click the dialog. This guards against agent mistakes, not against deliberate bypass.

## Open Questions

- Should the dialog show a diff for changes to existing documents?
