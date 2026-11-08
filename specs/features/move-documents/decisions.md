# Decisions: Move documents

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
