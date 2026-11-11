# Decisions: Write documents

> **Note 2026-11-11:** All decisions stay valid. The commands are tools, and the advice about what deserves a document moved out of waymark.

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

**Superseded in part by:** Delete on request, and one process for wrong documents — 2026-11-09

## Delete on request, and one process for wrong documents — 2026-11-09

**Decision:** The agent never deletes on its own, but deletes when the person asks (`waymark delete`). Before it deletes, it fixes every link to the document in the same commit. The tools for wrong documents are, from light to heavy: fix, dispute, deprecate, replace (`add --supersedes`), delete. A dispute ends with `dispute --clear`. A reader of a deprecated document reads the replacement.

**Reason:** In a personal knowledge base the person owns the content and may want a document gone. Git keeps the history. The tested rules of the waymark-trail skills show that agents need the whole process in one place.

**Supersedes:** Dispute instead of delete (in part)

## Enforced checks, no behavior advice — 2026-11-11

**Decision:** The server rejects duplicates, near-duplicates, oversize, secrets, broken links, and forbidden folders, with a `next` sentence. When to write and what to write is for the team's `AGENTS.md`.

**Reason:** the server enforces, the team advises. `delete` can be switched off with `tools_denied`.
