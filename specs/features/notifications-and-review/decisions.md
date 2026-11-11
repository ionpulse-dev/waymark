# Decisions: Notifications and review list

> **Note 2026-11-11:** "Notifications come from the app" is superseded by "Notices in results" below. "One review list" moves to the console.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Notifications come from the app — 2026-11-06

**Decision:** The CLI has no push. Notifications and dialogs come from the menu bar app. The first design had no popups at all.

**Reason:** The app closes the "no push" gap. Problems should be visible.

## One review list — 2026-11-06

**Decision:** One list for disputes. The pull request reviewer item was dropped.

**Reason:** One place to look.

## Notices in results — 2026-11-11

**Decision:** The server puts notices into tool results. The review list is in the console on a server.

**Reason:** no app and no background process in local mode. Alternative considered: email or webhook messages. They are a later option.
