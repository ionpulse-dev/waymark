# Tasks: Agent tool setup

`requires`: `mcp-server`.

- [ ] Re-read the three sources of `docs/agent-tools-research.md`. Update the file if a format changed. Record the date.
- [ ] Write the section "Agent tools" in `docs/install.md` (create the file with a heading if `distribution` has not yet): the Claude Code entry (command, `.mcp.json`) and its permission rules.
- [ ] Add the Codex entry (`~/.codex/config.toml`, `.codex/config.toml`) and its approval mode.
- [ ] Add the Copilot entries (VS Code and CLI). Mark the permission advice "unverified".
- [ ] Add the three `AGENTS.md` examples with the suggestion lines, and the removal line for each tool.
- [ ] Add the stable markers (`<!-- snippet: <tool> -->`, `<!-- example: agents-md -->`) around the blocks.
- [ ] Update `README.md`: the generic entry, the link to `docs/install.md`, the note on the approval prompt of a project server.
- [ ] `tests/it_agent_tool_docs.rs`: snippet parsing, tool names, suggestion marks, unverified marks.
- [ ] `tests/it_agent_tool_docs.rs`: the test that the server writes nothing into agent tool folders.
- [ ] Run the full suite.
