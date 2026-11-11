---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - mcp-server
---

# Pause and resume

## User Story

> As a person, I want one switch that stops all writes of my agents at once, so that I can stop a runaway agent in seconds.

## Personas

- **Every person** — uses the switch.
- **Knowledge base owner** — in server mode, can pause a whole knowledge base.

## Behaviour

- Two states only: running and paused. There is no automatic resume after a time.
- **Local mode.** The person sets `paused = true` in the personal config (`~/.config/waymark/config.toml`). The server reads that file at every call, so the next write fails at once. IT can set it in the system config file. The person can also stop the MCP entry in the agent tool.
- **Server mode.** The person uses the switch in the console (see `web-console`). An owner can pause a whole knowledge base. An operator can pause the server.
- Paused: every write tool fails with `paused`. Reads continue.
- Pause is a limit like any other (see `trust-model`). The strictest layer wins, so a pause set in a higher layer cannot be cleared in a lower one.
- Resume means clearing the flag in the layer that set it. No tool can do this.
- `status` and a notice name the state.

## Interfaces & Contracts

```toml
# ~/.config/waymark/config.toml
[limits]
paused = true
```

## Acceptance Criteria

- **Given** `paused = true`, **When** an agent calls a write tool, **Then** it fails with `paused`.
- **Given** `paused = true`, **When** an agent reads, **Then** it works.
- **Given** `paused = true` in the system config, **When** the person sets `false` in the personal config, **Then** writes still fail.
- **Given** paused, **When** a day passes, **Then** it is still paused.
- **Given** paused, **When** `status` runs, **Then** it names the state.
- **Given** an agent, **When** it lists tools, **Then** no tool resumes writes.

## Constraints & Edge Cases

- In local mode, an agent with file access could edit the personal config. This is the honest limit of a personal layer. The system config file and a server are hard.
- Pulls continue while paused, so the clone stays current.

## Open Questions

- Should a pause apply per knowledge base as an option in local mode?
