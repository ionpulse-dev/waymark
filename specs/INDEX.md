# Waymark · Spec Index

```
📦 Features      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0 / 20 active
🏗️ Foundation    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0 / 10 done+implemented
```

## Next Up

On 2026-11-11 the design changed to MCP only: one binary, local and server mode, no skill, no app (see [architecture direction](../docs/architecture-direction.md)). On 2026-11-14 `document-format` was approved and is `planned`. All other specs are `draft`. The current scope is local mode on macOS only (see [project decisions](../docs/decisions.md)). Server mode, multi-tenancy, sign-in, the web console, and Linux are out of scope for now. The next step is the review and approval of the requirements and plans in scope, foundation first:

1. `trust-model`, `knowledge-base-repo`
2. `git-sync`, `mcp-server`
3. `distribution`, then `server-mode` and `multi-tenancy`
4. The feature specs, starting with `write-documents` and `search-and-read`

---

## Features

| Feature | Status | Updated |
|:--------|:-------|:--------|
| [activity-and-undo](features/activity-and-undo/requirements.md) | `draft` | 2026-11-11 |
| [agent-tool-setup](features/agent-tool-setup/requirements.md) | `draft` | 2026-11-12 |
| [create-knowledge-base](features/create-knowledge-base/requirements.md) | `draft` | 2026-11-11 |
| [join-knowledge-base](features/join-knowledge-base/requirements.md) | `draft` | 2026-11-11 |
| [login-and-identity](features/login-and-identity/requirements.md) | `draft` | 2026-11-11 |
| [maintenance](features/maintenance/requirements.md) | `draft` | 2026-11-11 |
| [manual-edits](features/manual-edits/requirements.md) | `draft` | 2026-11-11 |
| [move-documents](features/move-documents/requirements.md) | `draft` | 2026-11-11 |
| [notifications-and-review](features/notifications-and-review/requirements.md) | `draft` | 2026-11-11 |
| [pause-and-resume](features/pause-and-resume/requirements.md) | `draft` | 2026-11-11 |
| [plain-git-access](features/plain-git-access/requirements.md) | `draft` | 2026-11-11 |
| [project-binding](features/project-binding/requirements.md) | `draft` | 2026-11-11 |
| [quality-checks](features/quality-checks/requirements.md) | `draft` | 2026-11-11 |
| [search-and-read](features/search-and-read/requirements.md) | `draft` | 2026-11-11 |
| [session-start](features/session-start/requirements.md) | `draft` | 2026-11-11 |
| [updates](features/updates/requirements.md) | `draft` | 2026-11-11 |
| [verification](features/verification/requirements.md) | `draft` | 2026-11-11 |
| [web-console](features/web-console/requirements.md) | `draft` | 2026-11-11 |
| [write-approval](features/write-approval/requirements.md) | `draft` | 2026-11-12 |
| [write-documents](features/write-documents/requirements.md) | `draft` | 2026-11-11 |

## Foundation

| Item | Status | Updated |
|:-----|:-------|:--------|
| [dev-environment](foundation/dev-environment/requirements.md) | `planned` | 2026-11-14 |
| [distribution](foundation/distribution/requirements.md) | `planned` | 2026-11-14 |
| [document-format](foundation/document-format/requirements.md) | `planned` | 2026-11-14 |
| [git-sync](foundation/git-sync/requirements.md) | `planned` | 2026-11-14 |
| [integration-testing](foundation/integration-testing/requirements.md) | `planned` | 2026-11-14 |
| [knowledge-base-repo](foundation/knowledge-base-repo/requirements.md) | `planned` | 2026-11-14 |
| [mcp-server](foundation/mcp-server/requirements.md) | `planned` | 2026-11-14 |
| [trust-model](foundation/trust-model/requirements.md) | `planned` | 2026-11-14 |
| [multi-tenancy](foundation/multi-tenancy/requirements.md) | `draft` | 2026-11-11 |
| [server-mode](foundation/server-mode/requirements.md) | `draft` | 2026-11-12 |

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
