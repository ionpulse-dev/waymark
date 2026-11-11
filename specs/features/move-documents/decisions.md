# Decisions: Move documents

> **Note 2026-11-11:** All decisions stay valid. `move` is now a tool.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Moves keep history — 2026-11-07

**Decision:** `git mv`, no content change in the move, rewrites and move in one commit, and a check with `git log --follow`.

**Alternatives considered:**
- Delete and re-create — loses the history of the document.

**Reason:** Users do not want to lose the history of a document. Git detects a rename only if the files are similar.

## One command for reorganizations — 2026-11-06

**Decision:** `move` handles folders and documents and rewrites links.

**Reason:** Folders are teams, and teams reorganize.

## Two commits and rename — 2026-11-09

**Decision:** A move is two commits: first a pure `git mv`, then the link fixes. A rename is a move.

**Reason:** A pure rename commit makes Git detect the rename for sure (similarity 100%). The link fixes in a second commit keep the history of the moved file. This is the rule of the tested waymark-trail skills.

## `move` is a tool — 2026-11-11

**Decision:** The move rules and the two-commit rule stay. A team can switch the tool off with `tools_denied`.

**Reason:** MCP is the only interface.
