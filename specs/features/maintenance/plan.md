# Plan: Maintenance

## Overview

The `status` tool is built in two steps. `mcp-server` creates `src/tools/status.rs` with the first fields (version, state, limits). This spec fills it with the checks. The checks live in a `maintenance` module, so each is a small function that tests can call alone. Removal and update are documentation.

## Components Affected

- `src/tools/status.rs` — created by `mcp-server`. This spec adds the call of `maintenance::run_checks` and the result fields.
- `mcp-server` — `Context` gives `status` the state, the settings, and the author. The registry lists `status` always.
- `git-sync` — new function `gitsync::probe_remote(timeout)` and the fields `last_pull` and `last_error` in `Repo`.
- `trust-model` — `config::layers` functions must be public so that `status` can load each layer (no change of the merge).
- `login-and-identity` — `identity::Author` and `check_email` for the `author` check.
- `join-knowledge-base` — `Startup::state()` without waiting.
- `docs/install.md` — the section "Remove waymark" (the file is created by `distribution`).

## New Files

- `src/maintenance/mod.rs` — `run_checks(&Context) -> Vec<Check>` and the `Check` type (`name`, `ok`, `detail`, `fix`).
- `src/maintenance/checks.rs` — one function for each check: `git`, `clone`, `working_tree`, `remote_and_login`, `kb_id`, `settings`, `author`.
- `src/maintenance/sources.rs` — `limit_sources(layers, effective)`.
- `src/gitsync/probe.rs` — `probe_remote(timeout) -> Probe` (`Reachable`, `NoRemote`, `LoginRefused(text)`, `Unreachable(text)`).
- `tests/it_maintenance.rs` — end-to-end tests.

## Technical Approach

1. **Order.** `run_checks` runs the checks in the fixed order and stops adding checks when the state forbids them (`cloning`: only `git` and `clone`; `broken` because of `git`: only `git`).
2. **`git`.** Runs `git --version` with a 5 second kill time-out. The output is the `detail`.
3. **`clone`.** Reads the state of `Startup` without waiting. Maps the state to `ok`, `detail`, and `fix`.
4. **`working_tree`.** Checks for `.git/rebase-merge`, `.git/rebase-apply`, and `.git/MERGE_HEAD`. Counts files with `git status --porcelain` (a read call).
5. **`remote` and `git_login`.** `gitsync::probe_remote(10 s)` runs `git ls-remote origin` with the isolated environment and a poll loop (`try_wait`, kill at the time-out). Classification of the output as in `decisions.md`. If there is no `origin`, `remote` is ok with the detail "no remote".
6. **`kb_id`.** Compares `--kb-id` with `id` of the settings.
7. **`settings`.** Parses each layer file separately with the loader of `config::layers`. The first error names the file and stops the list of layers. The same error makes every other tool fail with `setup_problem` (see `trust-model`), but `status` still runs and reports it. For that, `status` builds its own `Settings` view and falls back to the defaults of the strictest allowed state for the `settings` field of the result (`access = read-only`) when a layer is broken.
8. **`author`.** `identity::resolve` and `check_email`.
9. **Sources.** `limit_sources` compares the effective value of each limit with the value of each layer, in the order system, shared, personal, entry. A scalar limit lists the layers whose value equals the effective one. A list limit lists the layers that contribute. If no layer set it, the source is `default`.
10. **Result.** `message`, one notice for each failed check, `healthy`, and `data` as in the requirements. All text passes `identity::redact_text`.
11. **Time.** `last_pull` is a UTC time in the form `YYYY-MM-DDTHH:MM:SSZ` from the system clock. The `time` of the standard library is enough (`SystemTime`, formatted by a small function).
12. **Registry.** `status` is always in the list, also when `tools_denied` names it.
13. **Docs.** The section "Remove waymark" in `docs/install.md` has the four steps. The test checks that the headings and the four commands are there.

## Cross-Feature Dependencies

- `session-start` — the instructions and the notices share the facts of `status`. This spec owns the checks, `session-start` owns the instructions.
- `pause-and-resume` — `status` names the paused state through `settings` and a notice.
- `distribution` — owns `docs/install.md` and the Homebrew update.

## Risks

- A slow `git ls-remote` on a bad network makes `status` take 10 seconds. Accepted, and stated in the result.
- A fake `git` stub on macOS opens a dialog. The 5 second kill time-out limits the harm.
- A broken layer file makes `status` build its own settings view. Mitigation: one test for it (A7).

## Open Questions

None.
