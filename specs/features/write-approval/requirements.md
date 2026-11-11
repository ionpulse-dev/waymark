---
type: feature
status: draft
created: 2026-11-11
updated: 2026-11-11
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

- Should the request show a diff for changes to existing documents?
- Do Claude Code, Copilot, and Codex support elicitation today? Verify.
- Is a wait of 2 minutes accepted by clients, or does the server need an asynchronous "pending" result?
