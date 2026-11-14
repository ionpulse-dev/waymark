# Decisions: Maintenance

> **Note 2026-11-11:** "Uninstall from a manifest" and "`doctor` and `status` stay in the CLI" are superseded by "`status` is a tool, removal is documented" below.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Uninstall from a manifest — 2026-11-06

**Decision:** `setup` writes a manifest. `uninstall` removes exactly its items, with a question and a check before the clone is deleted.

**Reason:** Users asked for a complete uninstall, including the background process, with no leftovers and no lost work.

## `doctor` and `status` stay in the CLI — 2026-11-06

**Decision:** They are useful in a terminal and for scripts, even though people mainly use the app.

**Reason:** Support and company rollout need them.

## `status` is a tool, removal is documented — 2026-11-11

**Decision:** `status` is a read-only tool with checks and fixes. Update and removal use the package manager and the README steps.

**Reason:** waymark writes nothing into agent tools, so there is no manifest to remove.

## Scope: health and diagnostics only — 2026-11-14

**Decision:** This spec covers the `status` tool, the update by Homebrew, and the removal steps. `/health` and the server image are cut (see `todo.md`). `stats` belongs to `search-and-read` and is not used here.

**Reason:** The scope is local mode, and the owner of `stats` is `search-and-read`. Decided autonomously.

## Fixed check names and order — 2026-11-14

**Decision:** The checks are `git`, `clone`, `working_tree`, `remote`, `git_login`, `kb_id`, `settings`, `author`, in this order. A check that cannot run is left out.

**Alternatives considered:** Mark a skipped check as ok (rejected: it would claim something that was not checked). Mark it as not ok (rejected: the real problem is another check).

**Reason:** Stable names let tests and agents rely on them. Honest output. Decided autonomously.

## One network call for `remote` and `git_login` — 2026-11-14

**Decision:** `git ls-remote origin` runs once with a 10 second time-out and `GIT_TERMINAL_PROMPT=0`. The output decides both checks: a login message (`Permission denied`, `Authentication failed`, `could not read Username`) fails `git_login` and passes `remote`. A host or time-out message fails `remote`.

**Alternatives considered:** Two calls (rejected: twice the time-out). `git fetch` (rejected: `status` must not change the repository).

**Reason:** Read-only, one wait, and the two causes have different fixes. The call goes into a new `gitsync::probe` function, which is an addition to `git-sync`. Decided autonomously.

## `status` ignores `tools_denied` — 2026-11-14

**Decision:** `status` is always listed and always runs. `tools_denied` cannot hide it.

**Supersedes:** the rule of step 9 of the plan of `trust-model` (the registry filters tools by `tools_denied`) for this one tool.

**Reason:** `status` is how a person finds out why other tools are missing. A broken setup must stay diagnosable. Decided autonomously.

## `limit_sources` uses the layers separately — 2026-11-14

**Decision:** `status` loads each layer through `config::layers` (public functions that return one `Limits` per layer) and compares the effective value with each layer. It does not need a change to `strictest`.

**Alternatives considered:** Track the source inside `EffectiveSettings` (rejected: a change to `trust-model` for a diagnostic).

**Reason:** No change to the merge code. Decided autonomously.

## No `waymark doctor` command — 2026-11-14

**Decision:** There is no command for people. A person without an agent can start the agent tool and ask it to call `status`.

**Alternatives considered:** A `doctor` subcommand (rejected: `mcp-server` allows only `--version` and `--help`, and a second interface doubles the checks to maintain).

**Reason:** MCP is the only interface. Decided autonomously.

## The binary does not check for updates — 2026-11-14

**Decision:** The update is `brew upgrade waymark`. The server makes no network call to find a new version.

**Reason:** No telemetry and no call home. The Homebrew tap is the update channel.

## Implementation notes — 2026-11-14

- Files: `src/maintenance/mod.rs` (`run_checks`, `limit_sources`), `src/maintenance/sources.rs`, `src/gitsync/probe.rs` (`probe_remote`), and the new `src/tools/status.rs`. The checks are in one file, not in a `checks.rs`. Decided autonomously.
- The check `knowledge_base` of the first `status` is gone. The state goes into `clone` (ok for `ready`, `empty`, and `not_a_knowledge_base`) and into `knowledge_base.state`. A broken knowledge base fails `clone`, with the fix of the reason.
- When `git` fails only `git` runs. While the state is `cloning` only `git` and `clone` run. If no repository is open, the rest is left out.
- `gitsync::cmd::run_timeout` now starts git in its own process group and kills the whole group at the time-out, because `git` starts `ssh`, and `ssh` holds the pipes. It does not wait for the readers after a kill.
- `limit_sources` reads each layer file again with `config::read_layer`. The names are `system`, `shared`, `personal`, `entry`, and `default`. Duplicates (two system files) are shown once.
- `last_error` is the last error text of any other tool in this session (kept in `Context`). `last_pull` is the time of the last good fetch (`Repo::mark_fetched`).
- The message is "All checks are ok." or "N checks failed: names.". Each failed check adds the notice "Check `name` failed. <fix>".
- `docs/install.md` has the sections "Update waymark", "Check that waymark works", and "Remove waymark". `distribution` keeps the section "Install the binary".
- The old test of `knowledge-base-repo` and `session-start` were changed to the new shape (`clone` check, the new message).
