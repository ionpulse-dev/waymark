# Decisions: Create a knowledge base

> **Note 2026-11-11:** "Interactive for humans, flags for scripts" and "`init` creates the id and runs `setup`" are superseded by "`init` is a tool" below.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Interactive for humans, flags for scripts — 2026-11-06

**Decision:** `init` asks questions only when a human runs it in a terminal or the app. Every question has a flag and a default.

**Reason:** Keeps the product generic and scriptable for company rollout, while agents never meet a prompt.

## `init` creates the id and runs `setup` — 2026-11-07

**Decision:** The UUID is generated once by `init` and stored in the shared file, so every clone knows it. The slug stays a human label.

**Reason:** Binding to a project folder needs a stable identifier.

## `init` is a tool — 2026-11-11

**Decision:** `init` is an MCP tool that runs only on an empty repository. There is no terminal flow and no `setup`.

**Reason:** MCP is the only interface. The first start of the server already clones the repository. Alternative considered: a terminal command for the owner. It would need a second interface.
