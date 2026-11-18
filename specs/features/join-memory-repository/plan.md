# Plan: Join a memory repository

## Overview

A `join` module prepares the memory repository at the start of the server: it checks Git, decides the path, clones if needed, finds the root, and holds the state in a shared cell. `mcp::context` reads the state at every call. The clone runs in a background thread.

## Components Affected

- `mcp-server` — `main.rs` starts `join::start` and then serves. `mcp::context` asks `join` for the state and for the repository. Item 8 of its plan changes (see `decisions.md`).
- `git-sync` — `gitsync::clone` (item 11 of its plan) is used for the clone. It must take a time-out and a target folder.
- `memory-repository` — `memrepo::find_root` and `memrepo::settings` are used.
- `trust-model` — `allowed_repo_hosts` from `config::EffectiveSettings`.
- `login-and-identity` — `identity::redact_url` for every URL in a message.

## New Files

- `src/join/mod.rs` — `Startup` (state cell with a `Condvar`), `start(args) -> Startup`, `State`, and `wait_ready(timeout)`.
- `src/join/path.rs` — the default path from the URL (`repo_name`), the checks of an existing path, and the home expansion.
- `src/join/clone.rs` — the clone into a temporary folder, the rename, and the cleanup.
- `src/join/url.rs` — parse a Git URL (`https://`, `ssh://`, scp style `git@host:path`, `file://`) into host and repository name, and compare two URLs for the same repository.
- `tests/it_join_memory_repository.rs` — end-to-end tests.

## Technical Approach

1. **State cell.** `Startup` holds `Mutex<State>` and a `Condvar`. `State` is `Cloning`, `Ready(MemRepo)`, `Empty`, `NotAMemRepo`, `Broken(ToolError)`. `mcp::context` calls `wait_ready(60 s)` for every tool except `status`. `status` reads the state without waiting.
2. **Start.** `start` runs on a thread: `git --version` (5 second time-out), the host check, the path rules, the clone, then `memrepo::find_root` and `memrepo::settings`, then the ID check. On success it sets `Ready`. `memrepo::repo_is_empty` (see `create-memory-repository`) decides `Empty`: no commit, or only host-created files. Files but no `.waymark` give `NotAMemRepo`.
3. **URL parts.** `url.rs` gives the host for `allowed_repo_hosts` and the name for the default path. Two URLs are the same repository if the host and the path (without `.git` and a leading `/`) are equal, case-insensitive for the host. A `file://` URL has an empty host. When `allowed_repo_hosts` is set, an empty host is not in the list, so the URL is rejected.
4. **Path rules.** Use `std::fs` and `git -C <path> remote get-url origin`. A folder that is empty counts as a free path (the rename onto an empty folder works on macOS).
5. **Clone.** `git clone --quiet <url> <path>.cloning-<pid>`, with `GIT_TERMINAL_PROMPT=0` and a 5 minute kill time-out. Then `fs::rename`. On failure (`ENOTEMPTY`, `EEXIST`) delete the temporary folder, and go to the "path exists" branch. Any other failure deletes the temporary folder and sets `Broken(sync_failed)`. The error text passes `redact_text`.
6. **Retry.** A `Broken(sync_failed)` state from a failed clone is retried on the next non-`status` call, at most once per 30 seconds.
7. **Pull.** If the path was already a clone of the same repository, `gitsync::Repo::refresh_for_read` runs in the same thread. A failed pull is a notice, not a state change.
8. **No `--repo`.** `start` calls the discovery of `project-binding` and never clones.
9. **Personal config.** Read by `config::layers` (see `trust-model`). This spec adds no code for it. The tests check the merge in the join setting.

## Cross-Feature Dependencies

- `project-binding` — the discovery without `--repo`, and the meaning of `--memory-repository-id`.
- `create-memory-repository` — uses the `Empty` state.
- `maintenance` — `status` reads the state and shows it.
- `distribution` — the install step.

## Risks

- `/usr/bin/git` can be a stub on a Mac without the command line tools. It opens a dialog. Mitigation: the 5 second time-out kills the process, and the fix sentence says to install the command line tools.
- A background thread that clones when the agent tool exits early leaves a temporary folder. Mitigation: at start, delete folders `<path>.cloning-*` whose process no longer runs (`kill -0` check).
- The second start while the first still clones: it clones too and then loses the rename. Accepted, a rare case.

## Open Questions

None.
