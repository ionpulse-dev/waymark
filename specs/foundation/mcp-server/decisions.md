# Decisions: MCP server

## MCP is the only interface for agents — 2026-11-11

**Decision:** The binary is an MCP server. Agents use it only through MCP. There is no CLI for agents and no skill.

**Alternatives considered:**
- A CLI plus a skill (the earlier design). The skill can fail to load, needs an installer per agent tool, and the CLI cannot serve chat apps.
- A CLI and an MCP server side by side. Two interfaces to keep in step.

**Reason:** One interface and one behavior to test. All three target agent tools support MCP (to be verified). Tool descriptions are always in the context of the agent.

**Supersedes:** `cli-core` and `agent-skill`.

## One binary with all logic — 2026-11-11

**Decision:** The server is not a wrapper around another program. It holds the logic itself. The core is independent of the transport.

**Alternatives considered:**
- A thin MCP adapter that runs a CLI per call. It needs two programs and turns results into text again.

**Reason:** One install, one version, and the same core for local mode and server mode.

## The server enforces, the team advises — 2026-11-11

**Decision:** Hard rules are limits in the server. Behavior advice is not built in. Teams write their own `AGENTS.md`.

**Reason:** Text can be ignored, code cannot. A rejection message tells the agent what to do next.

## No control tools — 2026-11-11

**Decision:** No tool changes limits, pause, approval settings, or credentials.

**Reason:** An agent must never loosen its own limits.

## Configuration in the MCP entry — 2026-11-11

**Decision:** The MCP entry names the repository, the identity, and the limits. The server clones on first start. There is no setup command.

**Alternatives considered:**
- A `setup` command. It would be a second interface and needs a terminal.

**Reason:** One place to configure, in a format that every agent tool already has.

## Error codes, not exit codes — 2026-11-11

**Decision:** Failures carry a named `code`. The old exit codes 2 to 8 map to `validation_failed`, `sync_failed`, `setup_problem`, `stale_base`, `approval_denied`, `approval_unavailable`, and `paused`. Exit code 4 also covered a blocking limit, which now has its own code `access_denied`.

**Reason:** MCP has no exit codes. Names are clearer for agents.

## Earlier decisions of `cli-core` — 2026-11-11

The decisions "A portable Rust CLI and a separate Swift app", "The agent is the user interface", "Settings in every result", "One entry point", "Skill and templates are compiled in", "English only", "No MCP server first", and "Conformance tests" live on in `foundation/cli-core/decisions.md`. "Settings in every result", "English only", and "Conformance tests" carry over to this item. "No MCP server first" is superseded by "MCP is the only interface for agents".
