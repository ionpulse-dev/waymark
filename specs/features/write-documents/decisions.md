# Decisions: Write documents

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Every write is checked — 2026-11-08

**Decision:** Schema, Obsidian compatibility, location, size, duplicates, links, and secrets are checked before every commit. Companies add their own secret patterns. Plain-Git mode has only a text rule.

**Reason:** This supports the claim of a safe knowledge base. A failed check stops the write with exit code 2.

## Preview — 2026-11-06

**Decision:** `--preview` shows what would be written.

**Reason:** At write level `confirm` with confirmation in the chat, the agent needs something to show.

## Dispute instead of delete — 2026-11-06

**Decision:** Agents mark wrong documents as disputed or deprecated and keep the file.

**Reason:** History stays, and the author sees the problem in the review list.
