# Decisions: Join a knowledge base

> **Note 2026-11-11:** "Install Git for the person" and "Two steps to join" are superseded by "One MCP entry" below. "Shared and personal settings are split" stays valid.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Shared and personal settings are split — 2026-11-06

**Decision:** The shared file holds everything the same for all. Personal values go to `~/.config/waymark/config.toml`, keyed by UUID.

**Reason:** People who join never type shared values again, and each person controls their own levels.

## Two steps to join — 2026-11-06

**Decision:** Target: under 5 minutes. Download and open the app, answer defaults. For terminals: `install.sh`, then `waymark setup <url>`.

**Reason:** Least friction for non-developers.

## Install Git for the person — 2026-11-06

**Decision:** If `git` is missing, setup starts Apple's installer (one dialog, a download of a few hundred MB, once).

**Reason:** People need no Git knowledge, and developers already have it.

## One MCP entry — 2026-11-11

**Decision:** Joining is: install the binary and add one MCP entry. The server clones on first start. The personal config is optional.

**Reason:** no app and no setup command. IT handles the Git installation and login, which was the assumption of the design. Alternative considered: a setup tool in the server. It would clone without a person's entry.

## Scope: first-start clone through `--repo` — 2026-11-14

**Decision:** This spec covers the first-start clone, the path rules, the state of the server, and the personal config. It cuts the server workspace, the HTTPS client entry, and the choice of the knowledge base by sign-in. They are in `todo.md`. The key `[kb."<uuid>".limits]` of the personal config is cut too, because `trust-model` has no such layer. Decided autonomously.

**Reason:** The scope is local mode. The per-agent-memory key is not in the plan of `trust-model`, so building it here would contradict that spec.

## Default folder: `~/Projects/<repository-name>` — 2026-11-14

**Decision:** The default clone folder is `~/Projects/<repository-name>`, where the name is the last URL segment without `.git`. `--path` overrides it.

**Supersedes:** the `<slug>-agent-memory` default in the text of `mcp-server` and in the layout comment of `memory-repository`. The slug is known only after the clone, so it cannot name the folder.

**Alternatives considered:** `~/Projects/<slug>-agent-memory` (rejected: needs the slug before the clone). A folder in `~/.local/share/waymark` (rejected: people want to find the clone and open it in Obsidian).

**Reason:** The URL gives a stable name before the clone. A person who does not use `~/Projects` passes `--path`. Decided autonomously.

## Clone in the background, with a state — 2026-11-14

**Decision:** The server answers `initialize` at once and clones in a background thread. The state is `cloning`, `ready`, `empty`, `not_a_memory_repository`, or `broken`. Tools other than `status` wait up to 60 seconds for the clone, then return `setup_problem`.

**Supersedes:** item 8 of the approach of `mcp-server` ("clone it" at the first start, before serving).

**Alternatives considered:** Clone before serving (rejected: Codex stops a server that does not start within 10 seconds by default, and a large repository takes longer). A very long wait in the first call (rejected: the agent tool times out a call after 60 seconds in Codex by default).

**Reason:** The agent tools have start time-outs. The state also gives `status` and `init` one clear model. Decided autonomously.

## Clone into a temporary folder, then rename — 2026-11-14

**Decision:** The clone goes to `<path>.cloning-<pid>` and is renamed to `<path>`. If the rename fails because the path exists, the temporary folder is deleted and the existing clone is used.

**Alternatives considered:** A lock file in the parent folder (rejected: a crash leaves a stale file, and the rename is atomic).

**Reason:** Two servers that start together clone at most twice but leave one result, and a half clone never sits at the final path. Decided autonomously.

## The server never overwrites a path — 2026-11-14

**Decision:** A path that holds another repository, or a non-empty folder that is not a repository, is `setup_problem`. waymark does not touch it.

**Reason:** Data loss is worse than a clear error.

## The probe of the Git login in `status` replaces a check at clone time — 2026-11-14

**Decision:** The clone itself shows a login problem (`sync_failed`). There is no separate pre-check before the clone.

**Reason:** One network call fewer, and the error text of Git is mapped to the plain `next` sentence (see `login-and-identity`).

## Implementation notes — 2026-11-14

- No `src/join/` module. The code sits where the earlier specs put it: `mcp::binding` (path, host check, clone), `mcp::repo_url` (URL parts, A13), `mcp::context` (state), `gitsync::cmd::run_timeout` (time-out). The plan's file list was a proposal. Decided autonomously.
- **The default folder stays `~/Projects/<slug>-agent-memory`** (the form of `project-binding`, which was implemented first and is documented in the README). The entry "Default folder: `~/Projects/<repository-name>`" above is not applied. A3 checks the implemented form. Reopen it if the product owner wants the other form. Decided autonomously.
- The state lives in `Context`: `repo()` holds a mutex during the clone. Other calls poll `try_lock` for up to 60 seconds (`WAYMARK_CLONE_WAIT_SECONDS` for tests), then return `setup_problem`. `status`, `initialize`, and `tools/list` never wait (`is_cloning`, `kb_if_ready`). Decided autonomously.
- The background clone starts at process start only when `--repo` is set (`Server::start_join`). Without `--repo` nothing is cloned, so the discovery of `project-binding` still happens at the first tool call.
- A failed clone with `sync_failed` is not tried again for 30 seconds. Other failures are tried again at once.
- The clone time-out is 5 minutes (`WAYMARK_CLONE_TIMEOUT_SECONDS` for tests). `git --version` has no separate 5 second time-out: a stub that opens a dialog is a known risk (M2 checks it by hand).
- The temporary clone folder is `<name>.waymark-clone-<pid>`. At each clone, folders of dead processes and the old form `<name>.waymark-clone` are removed. If the rename fails because another server won, the existing clone is used when its origin is the same.
- `status` shows `memory_repository.state` always (also `cloning` and `broken`, with `reason`). The registry computes the limits at the start of a call, so the first `status` after a clone can show limits without the shared layer. The next call is right.
- `repo_is_empty` is in `kb::layout`. `Context::kb()` returns `setup_problem` with the `init` sentence for an empty repository.
