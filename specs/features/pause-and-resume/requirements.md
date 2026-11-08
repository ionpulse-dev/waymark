---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - cli-core
---

# Pause and resume

## User Story

> As a person, I want one switch that stops all writes of my agents at once, so that I can stop a runaway agent in seconds.

## Personas

- **Every person** — uses the switch in the menu bar.

## Behaviour

- Two states only: running and paused. There is no automatic resume after a time.
- `waymark pause` and `waymark resume`, and the same in the menu bar.
- Paused: every write command stops with exit code 8. Reads and background sync continue.
- `resume` needs the native approval dialog, so an agent cannot switch it off. In the first phase, before the app exists, `resume` asks in the terminal.
- The menu bar icon shows the state.
- Pause is the global switch. Access modes per knowledge base stay.

## Interfaces & Contracts

```text
waymark pause
waymark resume      # asks the app; exit 6 if canceled
```

## Acceptance Criteria

- **Given** paused, **When** an agent writes, **Then** the command exits with code 8.
- **Given** paused, **When** an agent reads, **Then** it works.
- **Given** paused, **When** an agent runs `resume`, **Then** the dialog opens and the agent cannot answer it.
- **Given** paused, **When** a day passes, **Then** it is still paused.
- **Given** paused, **When** `session` runs, **Then** a notice names the state.

## Constraints & Edge Cases

- Background sync keeps pulling while paused, so the clone stays current.

## Open Questions

- Should a pause apply per knowledge base as an option?
