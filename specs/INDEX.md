# Waymark · Spec Index

```
📦 Features      ██████████████████████████████████████░░  20 / 21 active
🏗️ Foundation    █████████████████████████████████░░░░░░░  9 / 11 done+implemented
```

## Next Up

Scope: local mode on macOS (Apple silicon) only. Server mode, multi-tenancy, sign-in, the web console, and Linux are out of scope for now (see [project decisions](../docs/decisions.md)). The decisions of the autonomous work are tagged "decided autonomously".

1. All in-scope foundation specs are `implemented`. The server starts, lists tools, and has the `status` tool. The harness has a real MCP client (`tests/common/mcp.rs`).
2. The feature specs passed a consistency pass (see [project decisions](../docs/decisions.md)).
3. `search-and-read`, `quality-checks`, and `manual-edits`, and `move-documents`, and `activity-and-undo`, and `verification`, and `pause-and-resume`, and `write-approval`, and `project-binding`, and `session-start`, `agent-tool-setup`, `login-and-identity`, and `join-memory-repository`, `plain-git-access`, and `create-memory-repository`, and `updates`, `notifications-and-review`, and `maintenance` are `implemented`. All in-scope features were implemented. Phase G (end-to-end checks, reviews) is done. On 2026-11-14 the rules changed and are implemented: a write may not create a broken body link, `delete` refuses linked documents, and the folder tools `folders`, `add_empty_folder`, and `remove_folder` exist (spec `folders`).
4. `distribution` is `implemented` for the part that the AI can do: formula, local Homebrew install test, release workflow (disabled), release checklist (`docs/next-steps/release-checklist.md`). The tap, the tag, and the public switch are for the product owner.
5. The `[manual]` tests of the implemented specs are for the product owner (see each `test.md`).

---

## Features

| Feature | Status | Updated |
|:--------|:-------|:--------|
| [write-documents](features/write-documents/requirements.md) | `implemented` | 2026-11-14 |
| [activity-and-undo](features/activity-and-undo/requirements.md) | `implemented` | 2026-11-14 |
| [pause-and-resume](features/pause-and-resume/requirements.md) | `implemented` | 2026-11-14 |
| [write-approval](features/write-approval/requirements.md) | `implemented` | 2026-11-14 |
| [project-binding](features/project-binding/requirements.md) | `implemented` | 2026-11-14 |
| [session-start](features/session-start/requirements.md) | `implemented` | 2026-11-14 |
| [manual-edits](features/manual-edits/requirements.md) | `implemented` | 2026-11-14 |
| [move-documents](features/move-documents/requirements.md) | `implemented` | 2026-11-14 |
| [quality-checks](features/quality-checks/requirements.md) | `implemented` | 2026-11-14 |
| [verification](features/verification/requirements.md) | `implemented` | 2026-11-14 |
| [search-and-read](features/search-and-read/requirements.md) | `implemented` | 2026-11-14 |
| [agent-tool-setup](features/agent-tool-setup/requirements.md) | `implemented` | 2026-11-14 |
| [login-and-identity](features/login-and-identity/requirements.md) | `implemented` | 2026-11-14 |
| [join-memory-repository](features/join-memory-repository/requirements.md) | `implemented` | 2026-11-14 |
| [plain-git-access](features/plain-git-access/requirements.md) | `implemented` | 2026-11-14 |
| [create-memory-repository](features/create-memory-repository/requirements.md) | `implemented` | 2026-11-14 |
| [updates](features/updates/requirements.md) | `implemented` | 2026-11-14 |
| [notifications-and-review](features/notifications-and-review/requirements.md) | `implemented` | 2026-11-14 |
| [maintenance](features/maintenance/requirements.md) | `implemented` | 2026-11-14 |
| [folders](features/folders/requirements.md) | `implemented` | 2026-11-14 |
| [web-console](features/web-console/requirements.md) | `draft` | 2026-11-11 |

## Foundation

| Item | Status | Updated |
|:--------|:-------|:--------|
| [terminology](foundation/terminology/requirements.md) | `implemented` | 2026-11-14 |
| [dev-environment](foundation/dev-environment/requirements.md) | `implemented` | 2026-11-14 |
| [document-format](foundation/document-format/requirements.md) | `implemented` | 2026-11-14 |
| [git-sync](foundation/git-sync/requirements.md) | `implemented` | 2026-11-14 |
| [integration-testing](foundation/integration-testing/requirements.md) | `implemented` | 2026-11-14 |
| [memory-repository](foundation/memory-repository/requirements.md) | `implemented` | 2026-11-14 |
| [mcp-server](foundation/mcp-server/requirements.md) | `implemented` | 2026-11-14 |
| [trust-model](foundation/trust-model/requirements.md) | `implemented` | 2026-11-14 |
| [distribution](foundation/distribution/requirements.md) | `implemented` | 2026-11-14 |
| [multi-tenancy](foundation/multi-tenancy/requirements.md) | `draft` | 2026-11-11 |
| [server-mode](foundation/server-mode/requirements.md) | `draft` | 2026-11-11 |

## Superseded

Replaced on 2026-11-11 by the MCP-only design. The folders stay as a record. They are not part of the current specification.

| Spec | Superseded by |
|:-----|:--------------|
| [cli-core](foundation/cli-core/requirements.md) | `mcp-server` |
| [agent-skill](foundation/agent-skill/requirements.md) | `agent-tool-setup`, `mcp-server` |
| [mac-app](foundation/mac-app/requirements.md) | `web-console`, `distribution` |
| [app-link](foundation/app-link/requirements.md) | `write-approval`, `web-console` |
| [approval-dialog](features/approval-dialog/requirements.md) | `write-approval`, `web-console` |

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
| `superseded`    | Replaced by other specs. The folder stays as a record. Listed under Superseded only. |

---

## Post-MVP

_No specs for these items yet. Reminder only._

See the Post-MVP section in [PROJECT.md](PROJECT.md).

---

> AI: Keep all tables and the Next Up section up to date whenever a status changes. Must always match the frontmatter in each item's `requirements.md`. Sort order for both tables: `in-progress` → `implemented` → `planned` → `draft` → `done`. Re-sort whenever a status changes. Also keep the progress graph at the top in sync: Features bar = in-progress count / total features; Foundation bar = (done + implemented) count / total foundation items. Superseded specs do not count in the totals. Each bar is 40 blocks using `█` (filled) and `░` (empty), rounded to nearest block.
