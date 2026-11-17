# Waymark · Spec Index

```
📦 Features      ███████████████████████████████████████░  27 / 28 active
🏗️ Foundation    ███████████████████████████████████░░░░░  13 / 15 done+implemented
```

## Next Up

Scope: local mode on macOS (Apple silicon) and Linux, plus the personal server. Server mode, multi-tenancy, sign-in, and the web console are out of scope for now (see [project decisions](../docs/decisions.md)). The decisions of the autonomous work are tagged "decided autonomously".

1. **`personal-server` real-use pass (product owner).** The spec is `implemented` and tested against a fake GitHub only. Run the `[manual]` items M1 to M9 in its [test.md](foundation/personal-server/test.md) with a real GitHub App, a host, and the Claude, ChatGPT, and Gemini apps.
2. **`clean-git-status` is `implemented`: the Obsidian check (product owner).** It stops `.DS_Store` and the Obsidian save of `.obsidian/app.json` from showing in `git status` of a memory repository. Run the `[manual]` item M1 in its [test.md](features/clean-git-status/test.md). It is part of the 0.6.0 release.
3. **`[manual]` tests and open release steps (product owner).** A spec moves to `done` only when its `[manual]` items pass (see each `test.md`), including the Linux install check. The release steps are in [distribution/todo.md](foundation/distribution/todo.md).
4. **Next specs.** All in-scope specs are `implemented`. The candidates are the drafts (`web-console`, `server-mode`, `multi-tenancy`) and the Post-MVP list in [PROJECT.md](PROJECT.md).

---

## Features

| Feature | Status | Updated |
|:--------|:-------|:--------|
| [clean-git-status](features/clean-git-status/requirements.md) | `implemented` | 2026-11-17 |
| [document-names](features/document-names/requirements.md) | `implemented` | 2026-11-17 |
| [lean-tools](features/lean-tools/requirements.md) | `implemented` | 2026-11-15 |
| [write-documents](features/write-documents/requirements.md) | `implemented` | 2026-11-16 |
| [activity-and-undo](features/activity-and-undo/requirements.md) | `implemented` | 2026-11-16 |
| [pause-and-resume](features/pause-and-resume/requirements.md) | `implemented` | 2026-11-14 |
| [write-approval](features/write-approval/requirements.md) | `implemented` | 2026-11-16 |
| [manual-edits](features/manual-edits/requirements.md) | `implemented` | 2026-11-16 |
| [move-documents](features/move-documents/requirements.md) | `implemented` | 2026-11-16 |
| [quality-checks](features/quality-checks/requirements.md) | `implemented` | 2026-11-16 |
| [verification](features/verification/requirements.md) | `implemented` | 2026-11-16 |
| [search-and-read](features/search-and-read/requirements.md) | `implemented` | 2026-11-17 |
| [agent-tool-setup](features/agent-tool-setup/requirements.md) | `implemented` | 2026-11-17 |
| [login-and-identity](features/login-and-identity/requirements.md) | `implemented` | 2026-11-14 |
| [plain-git-access](features/plain-git-access/requirements.md) | `implemented` | 2026-11-14 |
| [create-memory-repository](features/create-memory-repository/requirements.md) | `implemented` | 2026-11-16 |
| [updates](features/updates/requirements.md) | `implemented` | 2026-11-14 |
| [notifications-and-review](features/notifications-and-review/requirements.md) | `implemented` | 2026-11-14 |
| [maintenance](features/maintenance/requirements.md) | `implemented` | 2026-11-14 |
| [folders](features/folders/requirements.md) | `implemented` | 2026-11-14 |
| [project-binding](features/project-binding/requirements.md) | `implemented` | 2026-11-14 |
| [session-start](features/session-start/requirements.md) | `implemented` | 2026-11-14 |
| [join-memory-repository](features/join-memory-repository/requirements.md) | `implemented` | 2026-11-14 |
| [search-filters](features/search-filters/requirements.md) | `implemented` | 2026-11-15 |
| [restore-and-replace](features/restore-and-replace/requirements.md) | `implemented` | 2026-11-15 |
| [document-history](features/document-history/requirements.md) | `implemented` | 2026-11-15 |
| [manage-tags](features/manage-tags/requirements.md) | `implemented` | 2026-11-15 |
| [web-console](features/web-console/requirements.md) | `draft` | 2026-11-11 |

## Foundation

| Item | Status | Updated |
|:--------|:-------|:--------|
| [terminology](foundation/terminology/requirements.md) | `implemented` | 2026-11-14 |
| [dev-environment](foundation/dev-environment/requirements.md) | `implemented` | 2026-11-14 |
| [document-format](foundation/document-format/requirements.md) | `implemented` | 2026-11-16 |
| [git-sync](foundation/git-sync/requirements.md) | `implemented` | 2026-11-16 |
| [integration-testing](foundation/integration-testing/requirements.md) | `implemented` | 2026-11-14 |
| [memory-repository](foundation/memory-repository/requirements.md) | `implemented` | 2026-11-16 |
| [trust-model](foundation/trust-model/requirements.md) | `implemented` | 2026-11-14 |
| [distribution](foundation/distribution/requirements.md) | `implemented` | 2026-11-16 |
| [remove-index](foundation/remove-index/requirements.md) | `implemented` | 2026-11-14 |
| [mcp-server](foundation/mcp-server/requirements.md) | `implemented` | 2026-11-14 |
| [tool-conventions](foundation/tool-conventions/requirements.md) | `implemented` | 2026-11-15 |
| [linux-support](foundation/linux-support/requirements.md) | `implemented` | 2026-11-17 |
| [personal-server](foundation/personal-server/requirements.md) | `implemented` | 2026-11-17 |
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
