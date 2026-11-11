# Decisions: Activity and undo

> **Note 2026-11-11:** The decision "Undo makes autonomy safe" stays valid. Undo is now the `undo` tool and the console, not an app button.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Undo makes autonomy safe — 2026-11-06

**Decision:** One of three app features in the first release: a list of my writes with a one-click undo as a revert commit.

**Reason:** It supports the trust design. Undo makes autonomy safe.

## Tools and console, no app — 2026-11-11

**Decision:** `activity` and `undo` are tools. The console shows the same list on a server.

**Reason:** the app is dropped. Undo stays the safety net for agents that write freely.
