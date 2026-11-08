---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - write-documents
---

# Verification (optional)

## User Story

> As a knowledge base owner, I want owners to verify important documents, so that readers can tell checked knowledge from agent guesses when my organization needs it.

## Personas

- **Knowledge base owner** — turns verification on in `waymark.toml`.
- **Document owner** — verifies documents of their team's folder.

## Behaviour

Verification is off by default. With `verification = true`:

- `add` sets `trust: unverified`.
- `waymark verify <id>` sets `trust: verified`, `verified_by`, and `verified` to today. It runs only after an explicit "yes" from a human owner.
- `waymark review [<folder>...]` shows the review queue of the current user: unverified, disputed, and stale documents, for each: verify, fix, or reject.
- `waymark reject <id> --reason` marks the document as deprecated with the reason and keeps the file.
- After `stale_after_days`, a verified document becomes stale again and returns to the review queue.

How agents treat the trust state:

| State | Agent behavior |
|---|---|
| `verified` | Use as a fact. Cite the document. |
| `unverified` | Use as a hint. Say that it is not verified. Check it against the code if possible. |
| `disputed` | Do not rely on it. Mention the dispute. |
| stale | Treat like `unverified`. |

## Interfaces & Contracts

```text
waymark verify <id> --kb <uuid>
waymark review [<folder>...] --kb <uuid>
waymark reject <id> --reason <text> --kb <uuid>
```

## Acceptance Criteria

- **Given** verification is off, **When** `verify` runs, **Then** it fails with a clear message.
- **Given** verification is on, **When** `add` runs, **Then** the document has `trust: unverified`.
- **Given** an agent without a human "yes", **When** the skill is followed, **Then** it does not run `verify`.
- **Given** a verified document older than `stale_after_days`, **When** `review` runs, **Then** it is in the queue.
- **Given** `reject`, **When** it runs, **Then** the file stays with `status: deprecated`.

## Constraints & Edge Cases

- The CLI cannot prove that a human, not an agent, ran `verify`. The skill forbids it, and the commit author shows who did it. This is a policy with an audit trail, not a hard control.

## Open Questions

- Should verification be allowed for some document types only?
