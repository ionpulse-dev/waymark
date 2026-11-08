# Decisions: Maintenance

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Uninstall from a manifest — 2026-11-06

**Decision:** `setup` writes a manifest. `uninstall` removes exactly its items, with a question and a check before the clone is deleted.

**Reason:** Users asked for a complete uninstall, including the background process, with no leftovers and no lost work.

## `doctor` and `status` stay in the CLI — 2026-11-06

**Decision:** They are useful in a terminal and for scripts, even though people mainly use the app.

**Reason:** Support and company rollout need them.
