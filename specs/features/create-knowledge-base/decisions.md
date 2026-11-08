# Decisions: Create a knowledge base

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Interactive for humans, flags for scripts — 2026-11-06

**Decision:** `init` asks questions only when a human runs it in a terminal or the app. Every question has a flag and a default.

**Reason:** Keeps the product generic and scriptable for company rollout, while agents never meet a prompt.

## `init` creates the id and runs `setup` — 2026-11-07

**Decision:** The UUID is generated once by `init` and stored in the shared file, so every clone knows it. The slug stays a human label.

**Reason:** Binding to a project folder needs a stable identifier.
