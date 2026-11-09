# Decisions: Updates

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## `updates` with an explicit date — 2026-11-06

**Decision:** The command is named `updates` and takes `--since <date>`. Without it, it uses the last call.

**Reason:** A clearer name and an explicit date.

## `updates` is in the first release — 2026-11-06

**Decision:** The command is part of the first release. (The skill no longer runs it at session start, see below.)

**Reason:** It is cheap to build (reads Git history) and closes the "no push" gap.

## No startup report and no stale — 2026-11-09

**Decision:** The skill does not run `updates` at session start. The person asks when they want it. The `stale` command, `stale_after_days`, and every stale mark are removed. The skill says only: if `updated` is old, check the facts that matter before you rely on them, and say how old the document is.

**Reason:** The tested rules of the waymark-trail skills dropped both. A startup report costs context in every session. A stale mark means "somebody else should look at it": a person who doubts a document fixes it or disputes it right away.

**Supersedes:** `stale` has the same shape (2026-11-06); `updates` is in the first release (the part about session start)
