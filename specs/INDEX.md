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
| [activity-and-undo](features/activity-and-undo/requirements.md) | `draft` | 2026-11-08 |
| [agent-tool-setup](features/agent-tool-setup/requirements.md) | `draft` | 2026-11-08 |
| [approval-dialog](features/approval-dialog/requirements.md) | `draft` | 2026-11-08 |
| [create-knowledge-base](features/create-knowledge-base/requirements.md) | `draft` | 2026-11-08 |
| [join-knowledge-base](features/join-knowledge-base/requirements.md) | `draft` | 2026-11-08 |
| [login-and-identity](features/login-and-identity/requirements.md) | `draft` | 2026-11-08 |
| [maintenance](features/maintenance/requirements.md) | `draft` | 2026-11-08 |
| [manual-edits](features/manual-edits/requirements.md) | `draft` | 2026-11-08 |
| [move-documents](features/move-documents/requirements.md) | `draft` | 2026-11-08 |
| [notifications-and-review](features/notifications-and-review/requirements.md) | `draft` | 2026-11-08 |
| [pause-and-resume](features/pause-and-resume/requirements.md) | `draft` | 2026-11-08 |
| [plain-git-access](features/plain-git-access/requirements.md) | `draft` | 2026-11-08 |
| [project-binding](features/project-binding/requirements.md) | `draft` | 2026-11-08 |
| [quality-checks](features/quality-checks/requirements.md) | `draft` | 2026-11-08 |
| [search-and-read](features/search-and-read/requirements.md) | `draft` | 2026-11-08 |
| [session-start](features/session-start/requirements.md) | `draft` | 2026-11-08 |
| [updates-and-stale](features/updates-and-stale/requirements.md) | `draft` | 2026-11-08 |
| [verification](features/verification/requirements.md) | `draft` | 2026-11-08 |
| [write-documents](features/write-documents/requirements.md) | `draft` | 2026-11-08 |

## Foundation

| Item | Status | Updated |
|:-----|:-------|:--------|
| [agent-skill](foundation/agent-skill/requirements.md) | `draft` | 2026-11-08 |
| [app-link](foundation/app-link/requirements.md) | `draft` | 2026-11-08 |
| [cli-core](foundation/cli-core/requirements.md) | `draft` | 2026-11-08 |
| [distribution](foundation/distribution/requirements.md) | `draft` | 2026-11-08 |
| [document-format](foundation/document-format/requirements.md) | `draft` | 2026-11-08 |
| [git-sync](foundation/git-sync/requirements.md) | `draft` | 2026-11-08 |
| [knowledge-base-repo](foundation/knowledge-base-repo/requirements.md) | `draft` | 2026-11-08 |
| [mac-app](foundation/mac-app/requirements.md) | `draft` | 2026-11-08 |
| [trust-model](foundation/trust-model/requirements.md) | `draft` | 2026-11-08 |

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
