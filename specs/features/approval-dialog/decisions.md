# Decisions: Approval dialog

> **Note 2026-11-11:** This spec is superseded by `write-approval` and `web-console`. The decisions below are kept as a record. They no longer apply.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## The dialog is the only enforced approval — 2026-11-06

**Decision:** Approval by a native dialog is enforced by waymark. Approval in the chat is advice only.

**Reason:** The CLI cannot prove who answered in a chat. The dialog is the human safeguard for confidential information.

## Defaults and a second reviewer — 2026-11-07

**Decision:** The first design had approval levels `none`, `author`, and `reviewer`, per type or folder. They became the write level and confirm in. The `reviewer` pull request is not in the first release.

**Reason:** One simple system. Per-type and per-folder settings were too complex.

## Time-out means cancel — 2026-11-06

**Decision:** No answer in 2 minutes cancels the write.

**Reason:** A silent dialog must not leave a write open.
