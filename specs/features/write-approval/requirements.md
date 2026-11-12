---
type: feature
status: draft
created: 2026-11-11
updated: 2026-11-12
requires:
  - mcp-server
---

# Write approval

## User Story

> As a person or a team owner, I want some writes to need a human yes before they run, so that nothing leaves the knowledge base by an agent's mistake.

## Personas

- **Knowledge base owner** — sets which tools need approval, in the shared file or on the server.
- **Every person** — sees what the agent wants to publish and answers.

## Behaviour

Approval is a limit (see `trust-model`): `approval.required_for` lists the tools, or `write` for all writes, and `approval.via` chooses the channel. The default is no approval.

The write flow of `git-sync` asks for approval after all checks passed and before the commit. The request shows the knowledge base, the folder, the title, and the full body. A move or a bulk change shows a summary instead, for example "Move 42 documents, update links in 17 documents".

| Channel | How it works | Strength |
|---|---|---|
| `elicitation` | The server asks the person through the MCP client (MCP elicitation). The person answers in the client. | Depends on the client. It guards against agent mistakes. |
| `console` | The request goes to the web console (see `web-console`). The person signs in there on their own and answers. The agent cannot answer. | Enforced. Server mode only. |

In local mode there are two ways to get a yes before a write:

1. **The agent tool asks.** The person sets the write tools to `ask` in the permission settings of the agent tool. This is outside waymark, and waymark builds nothing for it. It needs correct tool kind hints (read-only, write, destructive) and a README section (see `agent-tool-setup`). The settings are not in the MCP config. In Claude Code they are the `permissions` rules in `settings.json`, for example `mcp__waymark__add`. A person who runs the tool in a mode that skips prompts gets no question from this way, except from explicit `ask` rules, which the current Claude Code documentation says still prompt. Verify this.
2. **waymark asks** (`approval.via = "elicitation"`). Waymark runs all checks, then asks the person through the client, and then commits. This is the built-in way. It works the same in every client that supports elicitation.

Server mode adds the `console` channel, which is the only enforced one.

Client support (from the documentation and bug reports of 2026-11-12, not yet verified by a test): Claude Code CLI supports elicitation, and hooks can auto-answer it. The Claude Code VS Code extension and Claude Desktop do not show it, and some versions did not announce the capability. GitHub Copilot in VS Code lists elicitation as supported. Codex CLI supports it and controls it by an approval setting.

Rules:

- **Fail closed.** If approval is required and the channel is not available, the tool fails with `approval_unavailable` and writes nothing. For example, the client does not support elicitation.
- A "no", or no answer within the time-out (default 2 minutes), ends with `approval_denied`.
- A write that needs no approval never opens a request.
- The client may also ask the person before it calls a tool. That is the client's own prompt, outside waymark. The tool kind hints (read-only, write, destructive) help clients to do it.
- Manual edits use the same approval before they are committed (see `manual-edits`).

## Interfaces & Contracts

The tool call waits for the answer. While it waits, the server sends progress messages so that clients keep the call alive. Whether clients accept a wait of 2 minutes is an open question.

## Acceptance Criteria

- **Given** `approval.required_for = ["delete"]` and `via = "elicitation"`, **When** `delete` runs, **Then** the client shows the document and the question before the commit.
- **Given** the person says yes, **When** the answer arrives, **Then** the write continues.
- **Given** the person says no, **When** the answer arrives, **Then** the tool fails with `approval_denied` and nothing is written.
- **Given** a client without elicitation, **When** approval is required, **Then** the tool fails with `approval_unavailable`.
- **Given** no answer for 2 minutes, **When** the time ends, **Then** the tool fails with `approval_denied`.
- **Given** `via = "console"` in server mode, **When** a write needs approval, **Then** the request shows in the console and the agent has no way to answer it.
- **Given** `add` is not in `required_for`, **When** it runs, **Then** no request opens.

## Constraints & Edge Cases

- A person can approve without reading. An agent that controls the client screen could click an elicitation. This guards against mistakes, not deliberate bypass. Only `console` is enforced, and only on a server.
- A move shows a summary, so the person cannot see every changed link.

## Open Questions

- Does elicitation still ask when the agent tool runs without permission prompts? Test with Claude Code in a mode that skips prompts.
- Can a client answer elicitation automatically (a hook, or a setting)? If yes, say so in the docs.
- Should the request show a diff for changes to existing documents?
- Do Claude Code, Copilot, and Codex support elicitation today? Verify.
- Is a wait of 2 minutes accepted by clients, or does the server need an asynchronous "pending" result?
