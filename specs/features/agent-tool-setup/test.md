# Tests: Agent tool setup

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** the snippet for a tool, **When** a person adds it and starts a session, **Then** the tool lists the tools of waymark. | M1, M2, M3, M4 |
| **Given** each JSON and TOML snippet in the docs, **When** it is parsed, **Then** it is valid, uses the server name `waymark`, and names `command` `waymark`. | A1 |
| **Given** a tool that is not listed in the install guide, **When** it speaks MCP over stdio, **Then** the generic entry (command and arguments) works. | M5 |
| **Given** the install guide, **When** it is read, **Then** every `AGENTS.md` example is marked as a suggestion. | A3 |
| **Given** an installed waymark, **When** the home and project folders are inspected, **Then** waymark wrote no file into any agent tool folder. | A4 |
| **Given** a new release, **When** the snippets are checked, **Then** each one works with the current tool version. | M1, M2, M3, M4 |

## Test Cases

- [x] [auto] A1: every snippet parses, has the server name `waymark`, the command `waymark`, and no secret-like key (`tests/it_agent_tool_docs.rs`)
- [x] [auto] A3: each `AGENTS.md` example starts with the suggestion line and ends with the secrets line (`tests/it_agent_tool_docs.rs`)
- [x] [auto] A4: after `status` and a read call, the temporary home has no folder or file of an agent tool (`tests/it_agent_tool_docs.rs`)
- [x] [auto] A5: every tool name in the permission advice is in `tools/list` of the real binary (`tests/it_agent_tool_docs.rs`)
- [x] [auto] A6: every row marked unverified in `docs/agent-tools-research.md` has the word "unverified" next to the statement in `docs/install.md` (`tests/it_agent_tool_docs.rs`)
- [ ] [manual] M1: Claude Code. Steps: install waymark with Homebrew. Run `claude mcp add --transport stdio waymark -- waymark --repo <test repository URL>`. Start `claude`. Run `/mcp`. Expected: `waymark` is connected and lists its tools. Repeat with the `.mcp.json` entry and approve the project server at the prompt. Add the permission rules and call `search` (no prompt) and `add` (prompt). Write the Claude Code version in the docs.
- [ ] [manual] M2: GitHub Copilot in VS Code. Steps: add `.vscode/mcp.json`, click **Start**, open Copilot Chat in Agent mode, open the tools icon. Expected: the tools of waymark are listed. Note what Copilot asks before a write tool (this answers an unverified item).
- [ ] [manual] M3: GitHub Copilot CLI. Steps: `copilot mcp add waymark -- waymark --repo <url>` or edit `~/.copilot/mcp-config.json`. Start `copilot`. Run `/mcp`. Expected: waymark is listed with its tools. Confirm the folder trust prompt for a project entry.
- [ ] [manual] M4: OpenAI Codex. Steps: add the table to `~/.codex/config.toml`. Run `codex mcp list`, then start `codex`. Expected: the tools of waymark are available, and a write tool asks before it runs (`writes` mode).
- [ ] [manual] M5: a fifth tool that speaks MCP over stdio (any). Steps: enter `waymark` as the command with the arguments of the generic entry in the install guide. Expected: it lists the tools.
- [ ] [manual] M6: for each tool, record whether it honors an explicit `ask` rule without prompts, the kind hints, elicitation, and server instructions. Expected: the answers replace the word "unverified" in `docs/install.md` and `docs/agent-tools-research.md`.

## Notes

A1 to A6 need only the real binary and the harness (`World`, `McpClient`). The `[manual]` tests need a test repository on a real Git host and a license for each tool.
