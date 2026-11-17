---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-17
requires:
  - mcp-server
---

# Agent tool setup

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Entries for a remote server (HTTPS) are out of scope for now (see [project decisions](../../../docs/decisions.md)). This spec is documentation only.

## User Story

> As a person, I want a clear recipe for each of my agent tools, so that I add waymark with one entry and write my own rules, without waymark editing my files.

## Personas

- **Every person** — uses Claude Code, GitHub Copilot, or OpenAI Codex.
- **A team** — writes its own `AGENTS.md` and shares an MCP entry in the project.

## Behaviour

waymark does not install anything into agent tools. It ships **no skill** and **no instruction block**. It gives documentation in the README:

1. **MCP entry for each tool.** A copy-paste entry for Claude Code, GitHub Copilot (VS Code and CLI), and OpenAI Codex, for user scope and for project scope where the tool has one. The formats differ (JSON for some tools, TOML for Codex). The entry starts the binary over stdio with `--repo` (see `mcp-server`). Optional: the author in `env` (see `login-and-identity`).
2. **Permission advice for each tool.** How to allow the read tools, and two choices for the write tools: "Confirm every write" (the tool asks before each write tool) or "No confirmation" (all waymark tools are allowed, and `activity` and `undo` are the safety net). The person may also confirm only some write tools. The rules are in the permission settings of the tool, not in the MCP config. For Claude Code, they are `allow` and `ask` rules such as `mcp__waymark__search` in `settings.json`. Tool kind hints (read-only, write, destructive) support this. The README says that a mode without prompts ("YOLO") ignores `allow` rules, may still honor explicit `ask` rules (unverified), and that the built-in approval through elicitation is the other way (see `write-approval`).
3. **A choice of how the agent uses memory, with `AGENTS.md` lines.** The install guide splits this into two independent settings, and the person chooses each one:
   - *When the agent reads and writes* (a rule in `AGENTS.md`, followed by judgment). Reading: automatic (search at the start of each task) or on request. Writing: automatic (decide from context what to save, and tell the person in one line), ask in chat ("Save this?"), or on request.
   - *Which writes need a confirmation* (the permission rules of the agent tool, see item 2).
   The guide explains the effect of each choice, lists typical combinations, and names the combinations to watch for (for example "Ask in chat" with "Confirm every write" asks twice). It gives one `AGENTS.md` line for each reading and writing choice, and common lines to always add: check old or disputed documents, one topic per document, update before adding, dispute instead of deleting, write with the waymark tools and call `sync` only after a manual edit when the person asks, and never write secrets, tokens, or personal data. An agent that installs waymark asks the person about each setting, one question at a time, and shows the lines and rules before it writes them. All texts are suggestions only. waymark does not insert them and does not check them. The server enforces the hard rules. The texts only guide behavior.
4. **How to tighten limits.** Where the personal config, the shared file, and the system config live (see `trust-model`).

The entries use the same server name, `waymark`, so that a project entry can override a user entry (see `project-binding`).

## Interfaces & Contracts

There is no tool and no code. The deliverables are text:

| Where | What |
|---|---|
| `README.md`, section "Setup" | A link to `docs/local-install.md`. |
| `docs/local-install.md`, sections "Agent tools" and "Choose how the agent uses memory" | For each tool: the entry (user scope and project scope), the permission advice for both confirmation choices, and the removal step. Then the two settings, their effects, the combinations, and the `AGENTS.md` lines. `distribution` owns the file. This spec owns the sections. |
| `docs/agent-tools-research.md` | The facts and the sources behind each snippet, with `verified` or `unverified` for each. |

**Verification mark.** Every statement about a tool that is not in the verified column of `docs/agent-tools-research.md` carries the word "unverified" in the text, next to the statement. Each snippet names the tool version it was tested with. Before the first manual test the version reads "not yet tested".

**Required content for each tool**

| Tool | Entry format | Files |
|---|---|---|
| Claude Code | JSON, `mcpServers` | `.mcp.json` (project), `~/.claude.json` (user, local), or `claude mcp add` |
| GitHub Copilot, VS Code | JSON, `servers` | `.vscode/mcp.json` |
| GitHub Copilot, CLI | JSON, `mcpServers`, `type: "local"` | `~/.copilot/mcp-config.json`, `.mcp.json`, `.github/mcp.json` |
| OpenAI Codex | TOML, `[mcp_servers.waymark]` | `~/.codex/config.toml`, `.codex/config.toml` |

**Permission advice.** Claude Code: for "Confirm every write", `allow` rules for the read tools and `ask` rules for the write tools, as `mcp__waymark__<tool>`. For "No confirmation", one `allow` rule `mcp__waymark__*`, and no waymark tool in any `ask` list, because an `ask` rule wins. Codex: `default_tools_approval_mode = "writes"` is "Confirm every write". Which Codex mode runs waymark tools without a prompt is unverified. Copilot: unverified, no advice beyond "check the permission settings of the tool". The text names the read tools (`status`, `search`, `show`, `folder_tree`, `list_tags`, `history`, `stats`, `updates`, `lint`, `activity`) and the write tools.

**`AGENTS.md` examples.** One line for each reading and writing choice, and two example blocks: the common lines, and a full example (reading on request, writing on request). The text is marked as suggestions, and each block ends with "Never write secrets, tokens, or personal data."

**Removal step.** The section ends with the line for the tool: how to remove the entry (`claude mcp remove waymark`, delete the table in `config.toml`, delete the entry in the JSON file). The full removal steps are in `maintenance`.

## Acceptance Criteria

- **Given** the snippet for a tool, **When** a person adds it and starts a session, **Then** the tool lists the tools of waymark.
- **Given** each JSON and TOML snippet in the docs, **When** it is parsed, **Then** it is valid, uses the server name `waymark`, and names `command` `waymark`.
- **Given** a tool that is not listed in the install guide, **When** it speaks MCP over stdio, **Then** the generic entry (command and arguments) works.
- **Given** the install guide, **When** it is read, **Then** every `AGENTS.md` example is marked as a suggestion.
- **Given** an installed waymark, **When** the home and project folders are inspected, **Then** waymark wrote no file into any agent tool folder.
- **Given** a new release, **When** the snippets are checked, **Then** each one works with the current tool version.

## Constraints & Edge Cases

- No cloud agents. Only local tools that a person starts, or a login on a server.
- Formats and paths come from the current official documentation (read on 2026-11-14, see `docs/agent-tools-research.md`). A snippet is tested in the real tool in the manual tests. Until then, it says "not yet tested".
- Codex starts a server with a time-out of 10 seconds by default. waymark answers `initialize` at once and clones in the background (see `join-memory-repository`), so the default is enough.
- A tool that asks to approve a project server (Claude Code does) shows the prompt on the first start. The README says so.
- The person may write any rules in their own `AGENTS.md`. waymark does not edit them.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`). The facts that the documentation does not state stay marked "unverified" and are listed in `todo.md`.
