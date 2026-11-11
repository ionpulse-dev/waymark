---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - write-documents
---

# Verification (optional)

## User Story

> As a knowledge base owner, I want owners to verify important documents, so that readers can tell checked knowledge from agent guesses when my organization needs it.

## Personas

- **Knowledge base owner** — turns verification on in `.mkbase/waymark.toml`.
- **Document owner** — verifies documents of their team's folder.

## Behaviour

Verification is off by default. With `verification = true`, three more tools exist:

- `add` sets `trust: unverified`.
- `verify` sets `trust: verified`, `verified_by`, and `verified` to today.
- `review` shows the review queue of the current person: unverified and disputed documents, for each: verify, fix, or reject.
- `reject` marks the document as deprecated with the reason and keeps the file.

Who may verify:

- **Local mode.** `verify` is a normal write tool. waymark cannot prove that a human, not an agent, called it. A team that wants this proof sets `approval.required_for = ["verify"]` with `via = "elicitation"`, or works on a server.
- **Server mode.** `verify` and `reject` are listed only for the role `owner`. With `approval.via = "console"` they need the owner's answer in the console. This is enforced.

How readers treat the trust state is advice for the team's `AGENTS.md`. A suggested wording for the README: use a verified document as a fact and cite it, use an unverified one as a hint and say so, and do not rely on a disputed one.

## Interfaces & Contracts

```json
{ "tool": "verify", "arguments": { "id": "backend/payments/idempotency-keys" } }
{ "tool": "reject", "arguments": { "id": "...", "reason": "..." } }
```

## Acceptance Criteria

- **Given** verification is off, **When** the client lists tools, **Then** `verify`, `review`, and `reject` are not listed.
- **Given** verification is on, **When** `add` runs, **Then** the document has `trust: unverified`.
- **Given** a server and a `writer`, **When** the client lists tools, **Then** `verify` and `reject` are not listed.
- **Given** `approval.required_for = ["verify"]` with `via = "console"`, **When** `verify` runs, **Then** the owner must answer in the console first.
- **Given** `reject`, **When** it runs, **Then** the file stays with `status: deprecated`.

## Constraints & Edge Cases

- In local mode, `verify` is a policy with an audit trail (the commit author), not a hard control.

## Open Questions

- Which role or group verifies a folder in server mode: the folder owner, or any `owner`?
