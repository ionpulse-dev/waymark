# Decisions: Quality checks

> **Note 2026-11-11:** All decisions stay valid. `lint` and `index` are tools, and a failed check is `validation_failed`.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Checks before every commit — 2026-11-08

**Decision:** The same checks run before every commit and on demand.

**Reason:** Format, size, links, and secrets are the main guard against noise and leaks.

## Generated indexes — 2026-11-06

**Decision:** No central index file that everyone edits. Indexes are generated.

**Reason:** A shared index file would cause merge conflicts.

## Company patterns — 2026-11-06

**Decision:** Secret patterns can be extended in the shared settings.

**Reason:** Some organizations have strict rules, for example banks with IBAN patterns.

## Checks are tools with a `next` sentence — 2026-11-11

**Decision:** A failed check returns a plain reason and a `next` sentence so that the agent can fix the input.

**Reason:** rejections should teach the agent.
