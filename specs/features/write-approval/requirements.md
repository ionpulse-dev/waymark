---
type: feature
status: implemented
created: 2026-11-11
updated: 2026-11-14
requires:
  - mcp-server
  - write-documents
---

# Write approval

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. The only channel is MCP elicitation. The `console` channel and the web console are out of scope for now (see [project decisions](../../../docs/decisions.md)). Parts that need server mode stay in `todo.md`.

## User Story

> As a person or a team owner, I want some writes to need a human yes before they run, so that nothing leaves the knowledge base by an agent's mistake.

## Personas

- **Knowledge base owner** — sets which tools need approval, in the shared file.
- **Every person** — sees what the agent wants to publish and answers in the agent tool.

## Behaviour

Approval is a limit (see `trust-model`): `approval.required_for` lists the tools, or `write` for all write tools. The default is no approval. The keys are in `[approval]` of any config layer. The union of all layers applies.

- **Channel.** `approval.via` is `elicitation`. The server asks the person through the MCP client (MCP elicitation, form mode). If any layer says `console`, the channel is not available in local mode (see "Errors").
- **When it asks.** The write flow of `git-sync` asks after all checks passed and before the commit. A call asks only if the tool is in `required_for`, or `write` is in `required_for` and the tool is a write tool. A call with `preview: true` never asks. A call that commits nothing never asks. A read tool never asks. A person is asked once for each write. An earlier yes is never reused.
- **Fail closed.** If approval is required and cannot be asked, the tool fails with `approval_unavailable` and writes nothing.
- **Other ways to get a yes.** The agent tool can ask before it calls a write tool (its own permission prompt). This is outside waymark. The tool kind hints of `mcp-server` help clients to do it. The README explains it (see `agent-tool-setup`).
- Manual edits use the same approval before they are committed (see `manual-edits`).

### The request

waymark sends an `elicitation/create` request with mode `form`:

```json
{
  "method": "elicitation/create",
  "params": {
    "message": "waymark asks to add a document to \"Example Company\".\nTool: add\nFolder: backend/payments\nTitle: Payment API v3 uses idempotency keys\nDocument: backend/payments/idempotency-keys\nAuthor: Jane Doe <jane@example.org>\n\n--- Body ---\n<the full body>",
    "requestedSchema": {
      "type": "object",
      "properties": {
        "approve": { "type": "boolean", "title": "Approve this write", "default": false }
      },
      "required": ["approve"]
    }
  }
}
```

The `message` is plain text. Its first line says what the tool does. The next lines name the tool, the folder, the title, the document ID, and the author. Then comes a line `--- Body ---` and the full body of the new document. The rules for each tool:

| Tool | The message shows |
|---|---|
| `add` | The fields above and the full body |
| `update_document`, `dispute`, `deprecate` | The fields above, the full new body, and a line `Words: <before> -> <after>` |
| `delete` | The title, the ID, and the full body of the document that goes away |
| `move` | A summary, for example `Move 42 documents, update links in 17 documents`, and the first 10 moves |
| `undo` | The commit message and the hash of the write that goes away |
| `sync`, `index`, `init`, `verify`, `review`, `reject` | A one-line summary and the list of changed paths (at most 20) |

The body is cut at 20000 characters. A cut message ends with `[Cut: <n> more characters. The document has <w> words.]`. The `message` never holds a secret that the checks found, because a document with a secret never reaches this step.

### What each answer means

| Client answer | Result |
|---|---|
| `action: "accept"` with `content.approve` equal to `true` | Yes. The write continues. |
| `action: "accept"` with `approve` missing, `false`, or not a boolean | `approval_denied`. Message: "The person did not approve." |
| `action: "decline"` | `approval_denied`. Message: "The person said no." |
| `action: "cancel"` | `approval_denied`. Message: "The person closed the question." |
| No answer in 120 seconds | `approval_denied`. Message: "No answer in 2 minutes." The server cancels the request. |

After a yes, the server reads the limits again. A pause or `read-only` set during the wait ends the write (see `pause-and-resume`). Then the write commits and pushes. Nothing is written in every other case, and the lock is released.

### Errors

| Case | Code | `message` | `next` |
|---|---|---|---|
| The client did not announce the `elicitation` capability with form mode | `approval_unavailable` | "This write needs the approval of the person, and the client cannot ask for it." | "Use a client that supports elicitation, or ask the owner to remove the approval rule." |
| The client announced elicitation but answered the request with an error | `approval_unavailable` | "The client could not show the approval request." | same as above |
| `approval.via` is `console` in a layer | `approval_unavailable` | "The approval channel `console` is not available in local mode." | "Set `via` to `elicitation` where it was set." |
| The answer is no, or the time ends | `approval_denied` | see the table above | "Nothing was written." |

### Result of a write

A write tool adds one field to its `data` when approval applied:

```json
{ "approval": "approved" }
```

With `preview: true` and a rule that applies, the field is `"required"`. If no rule applies, the field is missing.

## Acceptance Criteria

- **Given** `approval.required_for = ["delete"]` and `via = "elicitation"`, **When** `delete` runs, **Then** the client gets a request that shows the document and the question before the commit.
- **Given** the person says yes, **When** the answer arrives, **Then** the write continues and the result has `approval: "approved"`.
- **Given** the person says no, **When** the answer arrives, **Then** the tool fails with `approval_denied` and nothing is written.
- **Given** a client without elicitation, **When** approval is required, **Then** the tool fails with `approval_unavailable` and nothing is written.
- **Given** no answer for 2 minutes, **When** the time ends, **Then** the tool fails with `approval_denied`.
- **Given** `add` is not in `required_for`, **When** it runs, **Then** no request opens.
- **Given** `required_for = ["write"]`, **When** any write tool runs, **Then** a request opens.
- **Given** `preview: true`, **When** a write tool runs, **Then** no request opens and the result says `approval: "required"`.
- **Given** `accept` with `approve` false or missing, **When** the answer arrives, **Then** the tool fails with `approval_denied`.
- **Given** a pause set while the request waits, **When** the person says yes, **Then** the tool fails with `paused` and nothing is written.
- **Given** `via = "console"`, **When** a write needs approval, **Then** the tool fails with `approval_unavailable` and no request opens.
- **Given** a long body, **When** the request is built, **Then** the message is cut at 20000 characters and says how much is missing.

## Constraints & Edge Cases

- A person can approve without reading. An agent that controls the client screen, or a client hook that answers automatically, can say yes. Approval by elicitation guards against mistakes, not against a deliberate bypass. Only a separate channel that the agent cannot reach is enforced. That channel needs server mode and is out of scope.
- A move shows a summary, so the person cannot see every changed link.
- The server holds the write lock while it waits (see `git-sync`). Another waymark process that wants the lock waits up to 30 seconds and then fails with `sync_failed`.
- The tool call waits for the answer. If the caller sent a progress token, the server sends `notifications/progress` every 10 seconds with the message "Waiting for approval", so clients keep the call alive.
- The server never asks a question that is not an approval.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`). Which clients support elicitation is checked by the manual tests in `test.md`.
