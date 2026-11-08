# Decisions: Search and read

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## A small read budget — 2026-11-06

**Decision:** Defaults: at most 5 documents and about 800 tokens. If nothing matches, return nothing. Configurable per knowledge base and per call.

**Reason:** Protects the agent context. Learned from a similar project.

## No folder means everything — 2026-11-06

**Decision:** Every command with folders treats an empty list as the root. Followed folders were dropped.

**Reason:** With this rule followed folders have no job. A personal default can come later if people ask.

## Detail levels — 2026-11-07

**Decision:** `--level 0|1|2` and `--in` let the agent scan cheaply and open a body only when worth it.

**Reason:** Reading is the main cost. Path, title, and description cost little.

## Statistics for the budget — 2026-11-07

**Decision:** `stats` counts words, tokens, characters, or documents with a breakdown.

**Reason:** Helps the agent check whether a folder fits into its read budget, shows growth and noise, and can feed the app later.

## Pull before read — 2026-11-06

**Decision:** A read pulls first if the clone is older than a few minutes.

**Reason:** Agents never read an old clone, even if the background sync is off.
