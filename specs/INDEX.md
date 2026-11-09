# Waymark · Spec Index

```
📦 Features      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0 / 19 active
🏗️ Foundation    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0 / 9 done+implemented
```

## Next Up

All specs are `draft`. The requirements are written. The next step is the review and approval of the requirements, foundation first:

1. `document-format`, `trust-model`, then `knowledge-base-repo`
2. `git-sync`, `cli-core`, `agent-skill`
3. `app-link`, `mac-app`, `distribution`
4. The feature specs, starting with `write-documents` and `search-and-read`

---

## Features

| Feature | Status | Updated |
|:--------|:-------|:--------|
| [activity-and-undo]() | `draft` | 2026-11-08 |
| [agent-tool-setup]() | `draft` | 2026-11-08 |
| [approval-dialog]() | `draft` | 2026-11-08 |
| [create-knowledge-base]() | `draft` | 2026-11-08 |
| [join-knowledge-base]() | `draft` | 2026-11-08 |
| [login-and-identity]() | `draft` | 2026-11-08 |
| [maintenance]() | `draft` | 2026-11-08 |
| [manual-edits]() | `draft` | 2026-11-08 |
| [move-documents]() | `draft` | 2026-11-09 |
| [notifications-and-review]() | `draft` | 2026-11-09 |
| [pause-and-resume]() | `draft` | 2026-11-08 |
| [plain-git-access]() | `draft` | 2026-11-08 |
| [project-binding]() | `draft` | 2026-11-08 |
| [quality-checks]() | `draft` | 2026-11-08 |
| [search-and-read]() | `draft` | 2026-11-09 |
| [session-start]() | `draft` | 2026-11-09 |
| [updates]() | `draft` | 2026-11-09 |
| [verification]() | `draft` | 2026-11-09 |
| [write-documents]() | `draft` | 2026-11-09 |

## Foundation

| Item | Status | Updated |
|:-----|:-------|:--------|
| [agent-skill]() | `draft` | 2026-11-09 |
| [app-link]() | `draft` | 2026-11-08 |
| [cli-core]() | `draft` | 2026-11-09 |
| [distribution]() | `draft` | 2026-11-08 |
| [document-format]() | `draft` | 2026-11-09 |
| [git-sync]() | `draft` | 2026-11-08 |
| [knowledge-base-repo]() | `draft` | 2026-11-09 |
| [mac-app]() | `draft` | 2026-11-08 |
| [trust-model]() | `draft` | 2026-11-09 |

---

## Status Legend

| Status          | Meaning                                                                     |
|:----------------|:----------------------------------------------------------------------------|
| `draft`         | `requirements.md` and `plan.md` are being drafted or reviewed — phase 1 not yet complete    |
| `planned`       | `plan.md` and `tasks.md` are complete and approved                          |
| `in-progress`   | Implementation is underway                                                  |
| `blocked`       | Cannot proceed — a `requires` dependency is not yet `implemented` or `done` |
| `implemented`   | All tasks in `tasks.md` are checked off                                     |
| `done`          | All items in `test.md` are checked off (auto + manual)                      |

---

## Post-MVP

_No specs for these items yet. Reminder only._

See the Post-MVP section in [PROJECT.md](PROJECT.md).

---

> AI: Keep all tables and the Next Up section up to date whenever a status changes. Must always match the frontmatter in each item's `requirements.md`. Sort order for both tables: `in-progress` → `implemented` → `planned` → `draft` → `done`. Re-sort whenever a status changes. Also keep the progress graph at the top in sync: Features bar = in-progress count / total features; Foundation bar = (done + implemented) count / total foundation items. Each bar is 40 blocks using `█` (filled) and `░` (empty), rounded to nearest block.
