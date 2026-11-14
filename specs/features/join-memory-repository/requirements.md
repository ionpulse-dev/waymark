---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-14
requires:
  - login-and-identity
  - agent-tool-setup
---

# Join a memory repository

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. A server workspace for each memory repository, the HTTPS connection of a client, and the choice of a memory repository by sign-in are out of scope for now (see [project decisions](../../../docs/decisions.md)). They are in `todo.md`.

## User Story

> As a new person, I want to install waymark and add one entry to my agent tool, so that my agents can use the memory repository without any Git knowledge.

## Personas

- **Developer** — installs the binary and adds the entry.
- **Anyone with a coding agent** — follows the README steps. IT can prepare the machine (Git login, system config).

## Behaviour

Steps for a new person:

1. Install the binary (see `distribution`).
2. Make sure Git can sign in to the repository (see `login-and-identity`).
3. Add the MCP entry for the agent tool, from the snippet, with the repository URL `--repo` (see `agent-tool-setup`).
4. Start a session. The server clones the repository on first start. `status` shows the state.

There are no questions and no setup window. Everything the person must decide is an argument of the entry or a key of the personal config.

### First start

The server answers `initialize` and `tools/list` at once. It does the first-start work in the background. In this order:

1. **Git.** Check that `git --version` works. If not, stop with the state `broken` (`setup_problem`). No clone starts.
2. **Host.** If `allowed_repo_hosts` is set in any limit layer (see `trust-model`), the host of `--repo` must be in it. If not, `access_denied`.
3. **Path.** `--path`, or the default `~/.waymark/<name>`. `<name>` is the last part of the URL without `.git`. The full table of cases is in `project-binding`.
   - The path is missing or empty: clone into it (step 4).
   - The path is a Git repository whose `origin` is the same repository: use it, and pull.
   - The path is a Git repository with another `origin`, or a folder that is not empty and not a repository: stop with `setup_problem`. The server never overwrites anything.
4. **Clone.** `git clone <url>` into a temporary folder next to the path, then rename it to the path. Two servers that start at the same time clone once: the second one finds the path and uses it. The clone has a time-out of 5 minutes.
5. **Read the memory repository.** Find the root (the folder with `.waymark`) and read `.waymark/settings.toml` with the shared limits.
6. **ID.** If `--memory-repository-id` is given, it must equal `id` of the file. A different ID is `setup_problem` (see `project-binding`).
7. **Author.** Resolve and check the author (see `login-and-identity`). A problem does not stop the start. It stops writes and shows in `status`.

The **state** of the server after the start is one of:

| State | Meaning | What works |
|---|---|---|
| `cloning` | The first clone runs. | `status`. Other tools wait up to 60 seconds, then return `setup_problem`: "The first clone is still running. Call again in a minute." |
| `ready` | A memory repository was found. | All tools. |
| `empty` | The repository has no commit, or holds only files that a Git host creates (`README.md`, `README`, `LICENSE`, `LICENSE.md`). See `create-memory-repository`. | `status`, and `init` (see `create-memory-repository`). Other tools return `setup_problem` with the `next` sentence "This repository is empty. Call `init`." |
| `not_a_memory_repository` | The repository has files but no `.waymark` folder. | `status`. Others return `setup_problem`. |
| `broken` | Git, host, path, ID, or a settings file failed. | `status` with the reason and the fix. Others return the same `setup_problem`. |

A server without `--repo` and `--path` has no memory repository and clones nothing (see `project-binding`). waymark never clones without an entry that the person (or the project and the person's agent tool) approved.

### Shared and personal settings

- **Shared** settings are in `.waymark/settings.toml` in the repository. People who join never type them again.
- **Personal** settings are in `~/.config/waymark/config.toml`. The file is optional. The server reads it at every call (see `trust-model`).

```toml
# ~/.config/waymark/config.toml  (all keys optional)
[limits]
access = "read-write"            # read-only | read-write
paused = false
tools_denied = []
allowed_repo_hosts = []          # empty = no restriction

[approval]
required_for = []
via = "elicitation"
```

A personal value can only tighten the shared one (see `trust-model`). Other state is a small folder `~/.config/waymark/state/`. Its files belong to the specs that use them (for example `updates`).

## Interfaces & Contracts

The MCP entry is the interface (see `mcp-server`). There is no `setup` command and no tool. The result of `status` carries the state (see `maintenance`):

```json
{ "memory_repository": { "state": "ready", "name": "Example", "id": "0b1f3c9e-5a77-4d1e-9a41-2f6f0c8d7e11", "path": "/Users/jane/.waymark/waymark-agent-memory" } }
```

## Acceptance Criteria

- **Given** a Mac without working `git`, **When** the server starts, **Then** `status` reports it with a fix and no clone starts.
- **Given** a valid URL and a Git login, **When** the first session starts, **Then** the repository is cloned and `status` is ok.
- **Given** a repository URL and no `--path`, **When** the first session starts, **Then** the clone is in `~/.waymark/<name>`.
- **Given** a personal value looser than the shared limit, **When** the server reads both, **Then** the stricter value applies.
- **Given** a person who adds a second entry for another memory repository, **When** a session starts, **Then** the first one is unchanged.
- **Given** a repository whose `.waymark/settings.toml` has no `id`, **When** the server reads it, **Then** it reports `setup_problem`.
- **Given** the server started twice, **When** the second start ends, **Then** nothing is cloned twice.
- **Given** a path that holds another repository or a non-empty folder, **When** the server starts, **Then** it changes nothing and reports `setup_problem`.
- **Given** a clone that takes long, **When** the client connects, **Then** `initialize` answers at once and `status` shows `cloning`.
- **Given** the Git login gives no usable email, **When** `status` runs, **Then** the `author` check is not ok with a `fix`, and reads still work.
- **Given** `allowed_repo_hosts` without the host of `--repo`, **When** the server starts, **Then** it does not clone and reports `access_denied`.

## Constraints & Edge Cases

- A new person must reach a working setup in under 5 minutes with two steps: install the binary and add one entry.
- The clone is a normal Git clone. The person may open it in Obsidian.
- No offline mode. If the clone fails (no network, no login), the state is `broken` with `sync_failed`, and the next call retries the clone.
- A clone of an empty remote works in Git (with a warning). The state is `empty`.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`).
