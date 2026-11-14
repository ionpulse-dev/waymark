# Todo: Agent tool setup

## Bugs

None.

## Deferred

- Entries for a remote server (HTTPS): out of scope for now (server mode).
- Managed MCP config for IT (`managedMcpServers` in Claude Code): details not read. Add a short note when it matters.
- Copilot in JetBrains, Xcode, and Visual Studio.
- Windows and Linux paths (out of scope for now).
- A check that compares the research file with the live documentation (a scheduled job).

## Open Questions

Facts that the documentation did not state (they stay "unverified" until the `[manual]` tests):

- Does an explicit `ask` rule on an MCP tool still prompt in Claude Code without prompts?
- Do the three tools use the kind hints, elicitation, and the server `instructions`?
- Does a project entry override a user entry of the same name in Codex and in Copilot?
- How does Copilot ask before a write tool, and where is the setting?
- What is the exact key for MCP servers in the personal `settings.json` of VS Code?

## Notes 2026-11-14

- Remove `updates` and `init` from `NOT_YET_BUILT` in `tests/it_agent_tool_docs.rs` when `updates` and `create-knowledge-base` are implemented (the test tells you).
- All snippets say "not yet tested" until the manual tests M1 to M6 ran.
