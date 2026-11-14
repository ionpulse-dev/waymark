# Tests: Maintenance

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** a broken Git login, **When** `status` runs, **Then** it reports the problem and the fix. | A1, M1 |
| **Given** a healthy setup, **When** `status` runs, **Then** all checks are ok and `healthy` is true. | A2 |
| **Given** `access = "read-only"` or `paused = true`, **When** `status` runs, **Then** it still works and names the state. | A3 |
| **Given** `tools_denied = ["status"]`, **When** the client lists tools, **Then** `status` is still listed. | A4 |
| **Given** a repository with another ID than expected, **When** `status` runs, **Then** it reports it. | A5 |
| **Given** a remote that does not answer, **When** `status` runs, **Then** it ends within 12 seconds and reports "not reachable". | A6 |
| **Given** a broken config file, **When** `status` runs, **Then** the `settings` check names the file. | A7 |
| **Given** a limit set in two layers, **When** `status` runs, **Then** `limit_sources` names the layer of the effective value. | A8 |
| **Given** the steps of the README, **When** a person follows them, **Then** no waymark file or entry remains. | A9, M2 |
| **Given** any `status` result, **When** it is inspected, **Then** it holds no token or password. | A10 |

## Test Cases

- [ ] [auto] A1: a fake `GIT_SSH_COMMAND` that prints `Permission denied (publickey)` and exits 255, with an `ssh://` remote: `git_login` is not ok with the fixed `fix` sentence, and `remote` is ok (`tests/it_maintenance.rs`)
- [ ] [auto] A2: `World` with a knowledge base and an author: every check is ok, `healthy` is true, the check names and order match the requirements, and `version` equals the crate version (`tests/it_maintenance.rs`)
- [ ] [auto] A3: personal `access = "read-only"`, then `paused = true`: `status` works, `settings` shows the state, and a notice says writes are paused (`tests/it_maintenance.rs`)
- [ ] [auto] A4: `tools_denied = ["status"]`: `tools/list` holds `status` and a call works (`tests/it_maintenance.rs`)
- [ ] [auto] A5: `--kb-id` with a different UUID: `kb_id` is not ok and the `fix` names both IDs (`tests/it_maintenance.rs`)
- [ ] [auto] A6: a fake `GIT_SSH_COMMAND` that sleeps 60 seconds: `status` returns in under 12 seconds with `remote` not ok, "not reachable" in the detail, and the process is killed (`tests/it_maintenance.rs`)
- [ ] [auto] A7: a syntax error in the personal config: `settings` is not ok and names the file, `status` still returns, and `search` fails with `setup_problem` (`tests/it_maintenance.rs`)
- [ ] [auto] A8: shared `max_results = 3` and personal `max_results = 5`: `limit_sources.max_results` is `["shared"]`. A limit set nowhere gives `["default"]`. `tools_denied` in two layers lists both (`tests/it_maintenance.rs`, `src/maintenance/sources.rs`)
- [ ] [auto] A9: `docs/install.md` has the section "Remove waymark" with the four steps and the command for each tool (`tests/it_maintenance.rs`)
- [ ] [auto] A10: a URL with a password in `--repo` and a token in the environment: the whole `status` result holds neither (`tests/it_maintenance.rs`)
- [ ] [auto] A11: state `cloning` (slow fake `git`): only `git` and `clone` are in `checks`, and `clone` is not ok with the wait fix (`tests/it_maintenance.rs`)
- [ ] [auto] A12: a repository with no remote: `remote` is ok with "no remote" and the notice is present (`tests/it_maintenance.rs`)
- [ ] [auto] A13: a half-done rebase (`.git/rebase-merge` exists): `working_tree` is not ok with the fix (`tests/it_maintenance.rs`)
- [ ] [auto] A14: `status` makes no commit and no fetch: the clone and the remote are unchanged (`tests/it_maintenance.rs`)
- [ ] [manual] M1: real Git host. Steps: with a private test repository, sign out of Git (or use a key without access). Start a session and call `status`. Expected: `git_login` is not ok with the sign-in fix, in under 15 seconds. Sign in with `gh auth login`, call `status` again. Expected: `healthy` is true.
- [ ] [manual] M2: a real Mac. Steps: install waymark with Homebrew, add the entry to Claude Code, start a session, call `search`. Then follow the "Remove waymark" steps. Then run `find ~ /opt/homebrew -iname '*waymark*' -not -path '*/Projects/*' 2>/dev/null`. Expected: no result, except the knowledge base folder that you chose to keep, and `~/.claude.json` has no `waymark` entry.

## Notes

A1 and A6 use a script as `GIT_SSH_COMMAND`, so no network is needed. A6 takes about 10 seconds.
