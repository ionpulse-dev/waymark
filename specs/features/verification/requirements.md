---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - write-documents
---

# Verification (optional)

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Roles, the owner-only rule, and the console are out of scope for now and stay in `todo.md` (see [project decisions](../../../docs/decisions.md)).

## User Story

> As a knowledge base owner, I want owners to verify important documents, so that readers can tell checked knowledge from agent guesses when my organization needs it.

## Personas

- **Knowledge base owner** — turns verification on in `.waymark/settings.toml`.
- **Document owner** — verifies documents.

## Behaviour

Verification is off by default. It is on when `.waymark/settings.toml` has `verification = true`. With it off, `verify`, `review`, and `reject` are not listed, and a call fails with `access_denied`. No tool changes the setting.

With verification on:

- `add` sets `trust: unverified` (see `write-documents`).
- A change of `title`, `description`, or `body` of a verified document sets it back to `unverified` and removes `verified_by` and `verified` (see `write-documents`).
- A document without a `trust` field counts as unverified.

| Tool | Kind | Task |
|---|---|---|
| `verify` | write | Sets `trust: verified`, `verified_by`, and `verified` (today). |
| `review` | read | Lists the documents that wait for review. |
| `reject` | write | Deprecates a document and clears its trust. The file stays. |

`verify` and `reject` follow the write flow of `git-sync`. They accept `preview: true`. A team can ask for approval with `approval.required_for = ["verify", "reject"]` (see `write-approval`).

### `verify`

Input: `id` (string, required), `base` (string, required, `version` from `show`), `preview` (boolean, default `false`).

- The document must be `active`, not disputed, and not already verified. Otherwise `validation_failed`, with a `next` sentence: for a disputed document "Fix it with `update_document`, or end the dispute with `dispute` and `clear: true`."
- It sets `trust: verified`, `verified_by: "Name <email>"` of the person, `verified` to today, and `updated` and `updated_by`.
- The commit message is `waymark: update "<title>"`, then an empty line and `Trust: verified`.

Result `data`: `{ "id", "title", "version", "commit", "pushed", "trust": "verified", "verified_by", "verified", "preview" }`.

### `reject`

Input: `id` (string, required), `base` (string, required), `reason` (string, required, one line, at most 50 words), `preview` (boolean).

- It sets `status: deprecated` and `trust: unverified`, and removes `verified_by` and `verified`. It sets `updated` and `updated_by`. The file stays.
- The commit message is `waymark: deprecate "<title>"`, then an empty line, `Reason: <reason>`, and `Trust: rejected`.
- A document that is already deprecated fails with `validation_failed`.

Result `data`: `{ "id", "title", "version", "commit", "pushed", "status": "deprecated", "preview" }`.

### `review`

Input: `folder` (string, optional, default all), `limit` (integer, optional, default 10, at most 50).

It lists active documents that are unverified or disputed, within the `folders` limit. Disputed documents come first. Inside each group the oldest `updated` comes first. It does not use the read budget.

Result `data`:

```json
{
  "entries": [
    {
      "id": "backend/payments/idempotency-keys",
      "title": "Payment API v3 uses idempotency keys",
      "trust": "unverified",
      "disputed": false,
      "dispute_reason": null,
      "updated": "2026-11-10",
      "updated_by": "Jane Doe <jane@example.org>",
      "version": "a1b2c3d"
    }
  ],
  "total": 14
}
```

`total` is the full count. For each entry the person can fix it (`update_document`), verify it, or reject it. The tool gives no advice.

### Who may verify

In local mode `verify` is a normal write tool. waymark cannot prove that a human, not an agent, called it. The commit author is the audit trail. A team that wants a human answer sets `approval.required_for = ["verify", "reject"]` with `via = "elicitation"`. This is a policy, not a hard control (see `trust-model`).

How readers treat the trust state is advice for the team's `AGENTS.md`. A suggested wording for the README: use a verified document as a fact and cite it, use an unverified one as a hint and say so, and do not rely on a disputed one.

### Errors

| Code | When |
|---|---|
| `validation_failed` | The state does not allow the action (see above). |
| `stale_base` | `base` is not the current `version`. |
| `not_found` | The ID does not exist, or is outside the `folders` limit. |
| `invalid_arguments` | A required field is missing or wrong. |
| `access_denied` | Verification is off, or a limit blocks the call. |
| `paused`, `approval_denied`, `approval_unavailable`, `sync_failed` | As in `mcp-server` and `git-sync`. |

## Acceptance Criteria

- **Given** verification is off, **When** the client lists tools, **Then** `verify`, `review`, and `reject` are not listed.
- **Given** verification is on, **When** `add` runs, **Then** the document has `trust: unverified`.
- **Given** an active document, **When** `verify` runs, **Then** `trust`, `verified_by`, and `verified` are set and one commit reaches the remote.
- **Given** a verified document, **When** `update_document` changes the body, **Then** `trust` is `unverified` and `verified_by` and `verified` are gone.
- **Given** a disputed document, **When** `verify` runs, **Then** it fails with `validation_failed`.
- **Given** `reject`, **When** it runs, **Then** the file stays with `status: deprecated`.
- **Given** `approval.required_for = ["verify"]`, **When** `verify` runs, **Then** the approval is asked before the commit (see `write-approval`).
- **Given** unverified and disputed documents, **When** `review` runs, **Then** disputed ones come first and each entry shows the dispute reason.
- **Given** verification is off and a direct call of `verify`, **When** it runs, **Then** it fails with `access_denied`.

## Constraints & Edge Cases

- In local mode, `verify` is a policy with an audit trail (the commit author), not a hard control.
- When a team turns verification off, the trust fields stay in the files and are kept as they are.
- When a team turns verification on, existing documents have no `trust` field. They count as unverified, and `review` lists them. No bulk write happens.
- The standard says a producer must not set trust fields unless the knowledge base uses verification. These tools set them only when it does.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`). The server-mode question (which role verifies a folder) is out of scope for now and is in `todo.md` under Deferred.
