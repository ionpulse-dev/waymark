# Decisions: Manual edits and Sync now

> **Note 2026-11-11:** "The app commits manual edits" is superseded by "The server commits manual edits" below. "Sync now" is now the `sync` tool.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## The app commits manual edits — 2026-11-06

**Decision:** The app waits about a minute, runs `lint`, commits under the person, and pushes.

**Reason:** It makes "edits in Obsidian are allowed" work without Git knowledge.

## Sync now — 2026-11-06

**Decision:** The menu has a "Sync now" entry, with `waymark sync` as the CLI equivalent.

**Reason:** A person wants to see sync run at once, also after manual edits.

## The server commits manual edits — 2026-11-11

**Decision:** The `sync` tool and every write flow commit pending manual edits. There is no background watcher.

**Reason:** no app and no background job in local mode. Alternative considered: a file watcher while the server runs. It is an open question.
