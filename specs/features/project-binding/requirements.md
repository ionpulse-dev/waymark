---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - mcp-server
---

# Project binding

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Limits per memory repository are cut and stay in `todo.md`. Server mode has no binding by entry, because a server serves many memory repositories (see `server-mode`). It is out of scope for now (see [project decisions](../../../docs/decisions.md)).

## User Story

> As a person, I want each project to use exactly one memory repository, so that content from different memory repositories never mixes.

## Personas

- **Developer** — links code projects to a company memory repository.
- **Private person** — sets a default memory repository that works in every folder.
- **Team** — shares the binding through the project repository.

## Behaviour

A project uses a memory repository through an **MCP entry** of its agent tool. One server process serves exactly one memory repository. The server picks it once, at start. No tool can change it. Content of two memory repositories cannot mix.

- **Project entry.** The entry is in the project-level MCP config of the agent tool (for example `.mcp.json` of the project). It is committed into the project repository, so the whole team shares it. It holds the repository URL, and no secret and no personal path.
- **Personal default.** An entry in the user-level MCP config of the agent tool works in every folder. This is the personal default memory repository.
- **Both present.** Use the same server name in both entries. Agent tools let the project entry override the user entry of the same name. The manual tests check this for each tool. A project can opt out of the default with an entry of the same name that is disabled.
- **Identity.** If the entry has `--memory-repository-id`, the server checks that the `id` in `.waymark/settings.toml` is the same. A different ID is `setup_problem`, and the server serves nothing. The ID is checked at start and every time the shared file is read.
- **No hidden folder.** There is no `.waymark/` folder in the project and no `link` or `unlink` command. The person edits the MCP config of the agent tool.

### How the server finds the memory repository

The server does not need `--repo`. It looks in this order and uses the first match:

1. **`--path`** (with or without `--repo`). The folder of the clone.
2. **`--repo` without `--path`.** The default folder is `~/Projects/<slug>-agent-memory`. The slug comes from the last part of the URL: remove a trailing `/` and `.git`, remove a trailing `-agent-memory`, make it lower case, replace every run of characters other than `a-z` and `0-9` by `-`, and trim `-`. An empty result is `memory`. Example: `git@example.org:team/Example_Agent-Memory.git` gives `~/Projects/example-agent-memory`.
3. **The working folder.** The server looks in the working folder and in every folder above it. A folder matches if it contains `.waymark/`, or `agent-memory/.waymark/`.
4. **The default folder.** The folders `~/Projects/*-agent-memory` that hold a memory repository. If exactly one exists, the server uses it.

What happens at the folder:

- If the folder does not exist and `--repo` is set, the server clones the repository there, once, at the first start.
- If the folder exists and has a `origin` remote, and `--repo` is set, the remote must be the same URL. Another URL is `setup_problem`.
- If the folder is a Git repository without a memory repository marker, the server starts without a memory repository. Only `status` and `init` work (see `create-memory-repository`).
- If nothing is found, or more than one default folder exists, the server starts without a memory repository. Only `status` works. Its `next` sentence names what to add: `--repo`, or `--path`.
- If the folder is a linked Git worktree, or not a Git repository, it is `setup_problem`.

A server without a memory repository answers every other tool with `setup_problem`. The answer has `next`. `status` always works and tells how the memory repository was found (`found_by`, see `session-start`).

### Trust in a project entry

A project repository is untrusted input. A poisoned entry could point to another repository. Protections:

- Agent tools ask the person before they start a project-level MCP server for the first time. The manual tests check this for each tool.
- The limit `allowed_repo_hosts` (a list of hosts, personal or system layer, see `trust-model`) makes the server refuse to use a repository from another host. The default is no restriction. The check applies to the `--repo` URL before a clone, and to the `origin` of an existing folder. An existing folder without `origin` passes.
- A new clone happens only at the first start. The server never clones a second repository in a running session.
- A memory repository found in the working folder (step 3) is used only if its `origin` passes the host check.

### The host of a URL

The host is the part after `://` or after `@`, without a user, a password, a port, or a path, in lower case. `git@github.com:example/waymark-agent-memory.git` has the host `github.com`. `ssh://git@example.org:2222/team/k.git` has the host `example.org`. `file:///tmp/k.git` and a plain path have the host `localhost`. A host in `allowed_repo_hosts` must be equal to it (no globs). Waymark never prints a user or a password of a URL.

## Interfaces & Contracts

The entry of a project:

```json
{ "mcpServers": { "waymark": { "command": "waymark", "args": ["--repo", "git@github.com:example/waymark-agent-memory.git", "--memory-repository-id", "6f1c2a9e-3b7d-4c55-9a0e-1d2f3a4b5c6d"] } } }
```

The personal default needs no arguments if the memory repository is in `~/Projects/<slug>-agent-memory`:

```json
{ "mcpServers": { "waymark": { "command": "waymark" } } }
```

The arguments are those of `mcp-server`: `--repo`, `--memory-repository-id`, and `--path`. The tools take no repository argument. A call with an argument that names a repository, a path, or a memory repository fails with `invalid_arguments`.

A server without a memory repository answers a tool call like this:

```json
{
  "code": "setup_problem",
  "message": "No memory repository is set up. waymark found none in the working folder, and none or several in ~/Projects.",
  "next": "Add --repo <git url> or --path <folder> to the MCP entry.",
  "settings": { "access": "read-write", "paused": false }
}
```

The codes at start:

| Case | Code |
|---|---|
| The ID in the file differs from `--memory-repository-id` | `setup_problem` |
| The file has no `id` | `setup_problem` |
| The folder has another `origin` than `--repo` | `setup_problem` |
| The folder is not a Git repository, or is a linked worktree | `setup_problem` |
| The path does not exist and there is no `--repo` | `setup_problem` |
| The clone fails | `sync_failed` |
| The host is not in `allowed_repo_hosts` | `access_denied` |
| Nothing found, or several default folders | `setup_problem` |

## Acceptance Criteria

- **Given** a project entry with `--repo`, **When** a session starts in that project, **Then** only that memory repository is used.
- **Given** an entry with `--memory-repository-id` and a repository with another ID, **When** the server starts, **Then** it reports `setup_problem` and serves nothing.
- **Given** `allowed_repo_hosts = ["git.example.org"]`, **When** an entry names another host, **Then** the server refuses to clone and creates no folder.
- **Given** a user entry and a project entry with the same name, **When** a session starts in the project, **Then** the agent tool uses the project entry (manual, per tool).
- **Given** the server is running, **When** a tool asks for another repository, **Then** it is refused. The tools take no repository argument.
- **Given** an entry with `--repo` and no `--path`, **When** the server starts for the first time, **Then** it clones to `~/Projects/<slug>-agent-memory` and reuses the clone later.
- **Given** an entry without `--repo` and `--path`, **When** the working folder is inside a memory repository, **Then** the server uses it.
- **Given** an entry without `--repo` and `--path`, **When** the working folder has no memory repository and exactly one default folder has one, **Then** the server uses the default folder.
- **Given** an entry without `--repo` and `--path`, **When** no memory repository is found, or two default folders exist, **Then** `status` works and every other tool fails with `setup_problem` and a `next` sentence.
- **Given** a folder with another `origin` than `--repo`, **When** the server starts, **Then** it reports `setup_problem` and does not use the folder.
- **Given** two servers with different memory repositories, **When** both run, **Then** each tool call touches only the repository of its own server.
- **Given** a repository URL with a user and a password, **When** `status` shows it, **Then** it shows neither.

## Constraints & Edge Cases

- The ID is an identifier, not a secret.
- An agent that edits the MCP config of its tool could add an entry. This is a deliberate act and the same limit as raw Git use.
- A Git worktree of the code project has its own copy of the committed `.mcp.json`, so the project entry works there. A worktree of the memory repository repository is not supported.
- A first clone happens before the server answers `initialize`. A very large repository can exceed the start time-out of the client. The clone goes to a temporary folder next to the target and is renamed when it is complete, so a cut clone never counts as a memory repository. The next start clones again.
- The working folder of a server started by an agent tool is the project folder in the tools that were checked. The manual tests check this.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`). Which agent tools let a project entry override a user entry, and which ask before they start a project-level server, is checked by the manual tests in `test.md`.
