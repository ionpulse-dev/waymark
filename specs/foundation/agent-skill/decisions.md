# Decisions: Agent skill

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Write triggers A to D — 2026-11-06

**Decision:** The skill names four triggers: the person asks, a durable fact is learned, a decision is made, a contradiction is found. End of session and pull request merge are not in the first release. The earlier per-trigger levels (off, ask, auto) were replaced by the write level.

**Reason:** A sensible start that is safe against noise. The level decides how much the agent does alone.

## The skill is fixed and static — 2026-11-06

**Decision:** The skill text is compiled into the CLI, the same in every project. It only says where the agent finds the knowledge base id.

**Reason:** The skill stays version-matched and testable. Project differences live in `.waymark/`.

## Advice is not a control — 2026-11-07

**Decision:** `waymark session` informs the agent, but every command checks the same rules itself.

**Reason:** A skipped or ignored session call cannot allow a forbidden action.

## Noise rules — 2026-11-06

**Decision:** The skill asks the agent a fixed question before it writes: is this true in three months, and does someone outside the team need it?

**Reason:** The hard problem is write quality, not storage. Noise fills a knowledge base faster than anything else.
