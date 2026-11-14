# Tasks: Agent tool setup

`requires`: `mcp-server`.

- [x] Re-read the three sources of `docs/agent-tools-research.md`. Update the file if a format changed. Record the date.
- [x] Write the section "Agent tools" in `docs/install.md` (create the file with a heading if `distribution` has not yet): the Claude Code entry (command, `.mcp.json`) and its permission rules.
- [x] Add the Codex entry (`~/.codex/config.toml`, `.codex/config.toml`) and its approval mode.
- [x] Add the Copilot entries (VS Code and CLI). Mark the permission advice "unverified".
- [x] Add the three `AGENTS.md` examples with the suggestion lines, and the removal line for each tool.
- [x] Add the stable markers (`<!-- snippet: <tool> -->`, `<!-- example: agents-md -->`) around the blocks.
- [x] Update `README.md`: the generic entry, the link to `docs/install.md`, the note on the approval prompt of a project server.
- [x] `tests/it_agent_tool_docs.rs`: snippet parsing, tool names, suggestion marks, unverified marks.
- [x] `tests/it_agent_tool_docs.rs`: the test that the server writes nothing into agent tool folders.
- [x] Run the full suite.
