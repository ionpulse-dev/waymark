# Plan: Agent tool setup

## Overview

Documentation only. The snippets go into `README.md` and the section "Agent tools" of `docs/install.md`. The facts and their sources are in `docs/agent-tools-research.md`. One integration test parses the snippets, so they cannot drift from the server. No installer, no code in the binary.

## Components Affected

- `README.md` — the section "Planned setup" keeps one generic entry and links to `docs/install.md`. The link to the spec is replaced by a link to the guide.
- `docs/install.md` — created by `distribution`. This spec adds the section "Agent tools" (see `decisions.md`).
- `mcp-server` — none. The test only calls `tools/list`.
- `login-and-identity` — the `env` of each entry shows `WAYMARK_AUTHOR_NAME` and `WAYMARK_AUTHOR_EMAIL`.

## New Files

- `docs/agent-tools-research.md` — facts, sources, and `verified` or `unverified` marks for each tool. Created with this spec.
- `tests/it_agent_tool_docs.rs` — end-to-end tests: parse the snippets of `docs/install.md`, check the rules, and check that no file is written into agent tool folders.

## Technical Approach

1. **Text.** For each tool: the entry for user scope and project scope, the permission advice for both confirmation choices, and the removal line. Once, shared by all tools: the section "Choose how the agent uses memory" with the two settings, the combinations, and the `AGENTS.md` lines. All text follows `docs/agent-tools-research.md`. A statement that the research marks "unverified" carries that word.
2. **Snippet test.** The test reads `docs/install.md`, takes every fenced block with the language `json` or `toml` that follows the marker comment `<!-- snippet: <tool> -->`, and parses it (`serde_json`, `toml`). It checks: valid syntax, the server name `waymark`, `command` equals `waymark`, `args` start with `--repo`, no key that looks like a secret (`token`, `password`, `secret`), and the test author addresses use `example.org`.
3. **Tool names.** The test extracts every `mcp__waymark__<tool>` and every tool name in the permission advice, starts the real binary with the harness (`McpClient`), calls `tools/list`, and checks that each name is a listed tool or the tool `init`/`verify_document`/`review`/`reject_document`. Names of tools that are present only with verification (`verify_document`, `review`, `reject_document`) must not be in the advice.
4. **Suggestion marks.** The test checks the line "Suggestions only. waymark does not insert or check these texts.", finds the two `AGENTS.md` example blocks (after the marker `<!-- example: agents-md -->`), and checks that each ends with the secrets line.
5. **No writes into tool folders.** The test starts the server with `HOME` set to a temporary folder, runs `status` and a read call, and checks that `HOME` holds no `.claude`, `.codex`, `.copilot`, `.vscode`, or `.mcp.json` (only `.config/waymark` may exist, and only if the server wrote it).
6. **Manual check.** The `[manual]` tests in `test.md` run each snippet in the real tool. The result and the tool version go into the table at the end of `docs/agent-tools-research.md` and into the snippet text.

## Cross-Feature Dependencies

- `distribution` — owns `docs/install.md`. Both add text. Mitigation: the section markers of this spec (`<!-- snippet: ... -->`, `<!-- example: agents-md -->`) are stable.
- `maintenance` — owns the removal steps. This spec only links to them.
- `project-binding` — uses the same server name so a project entry can override a user entry. This spec only keeps the name.

## Risks

- The tools change their formats. Mitigation: the dated sources in the research file, and a check before each release.
- A snippet works in the documentation but not in the tool. Mitigation: the manual tests.
- The sentence "unverified" is removed by accident. Mitigation: the snippet test checks that the unverified rows of the research file have the word in `docs/install.md`.

## Open Questions

None.
