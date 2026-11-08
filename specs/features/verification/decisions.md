# Decisions: Verification

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Optional, off by default — 2026-11-06

**Decision:** Verification is a setting of the knowledge base. The default model is the wiki model without it.

**Alternatives considered:**
- Verification as the default (trust as a stored state) — needs a clear owner for every document and a review queue, and slows writes.

**Reason:** Organizations that need a stronger signal can turn it on. Others keep the simple model.
