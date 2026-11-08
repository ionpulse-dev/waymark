# Decisions: Notifications and review list

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Notifications come from the app — 2026-11-06

**Decision:** The CLI has no push. Notifications and dialogs come from the menu bar app. The first design had no popups at all.

**Reason:** The app closes the "no push" gap. Problems should be visible.

## One review list — 2026-11-06

**Decision:** One list for disputes and stale documents. The pull request reviewer item was dropped.

**Reason:** One place to look.
