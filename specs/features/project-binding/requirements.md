---
type: feature
status: draft
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

### Where the memory repository lives

The server does not look for a memory repository. The entry names it, with `--repo` (the Git URL), `--path` (a folder), or both. One of them is required. `<name>` is the repository name from the URL: the last part, without a trailing `/` and without `.git`. An SSH URL and an HTTPS URL of the same repository give the same name. The owner is not part of the name.

| `--repo` | `--path` | Folder state | Result |
|---|---|---|---|
| given | none | `~/.waymark/<name>` is missing or empty | Clone into it |
| given | none | holds a repository with the same remote | Use it |
| given | none | holds a repository with another remote | Error: name collision, set `--path` |
| given | given | `--path` is missing or empty | Clone into `--path` |
| given | given | holds a repository with the same remote | Use it |
| given | given | holds a repository with another remote | Error: remote mismatch |
| none | given | holds a memory repository | Use it |
| none | given | missing or empty | Error: add `--repo` to clone it |
| none | none | not applicable | Error: give `--repo` or `--path` |

- **Path wins.** When both are given, `--path` sets the location and `--repo` is only the clone source.
- **`~` and relative paths.** A leading `~` in `--path` is the home folder. A relative path is allowed. The server resolves it against its working folder.
- **No guessing.** The server does not scan any folder. It does not look in the working folder or the folders above it. It never uses a repository that the entry did not name.
- **Same remote.** The server compares the `--repo` URL with the `origin` remote of the folder. A trailing `/`, a trailing `.git`, and the form (SSH or HTTPS) do not count. The `origin` remote is the only remote that the server reads. A folder without an `origin` remote passes the check.
- **Errors are plain sentences.** Each error says what is wrong and names the fix (`--path`, `--repo`, or the folder to check). The server never changes a folder that it refuses.
- **First start.** The first clone runs in the background (see `join-memory-repository`). `status` shows the state and the folder in use.

More cases of an existing folder, with `--repo` or `--path`:

- A Git repository without a memory repository marker: the server starts without a memory repository. Only `status` and `init` work (see `create-memory-repository`).
- A linked Git worktree, or a folder that is not empty and not a Git repository: `setup_problem`.
- A URL that gives no name (for example `https://example.org/.git`) and no `--path`: `setup_problem` that asks for `--path`.

A server that has an error answers every tool except `status` with `setup_problem`. The answer has `next`. `status` always works and tells how the folder was chosen (`found_by`: `path_argument` or `repo_argument`, see `session-start`).

### Trust in a project entry

A project repository is untrusted input. A poisoned entry could point to another repository. Protections:

- Agent tools ask the person before they start a project-level MCP server for the first time. The manual tests check this for each tool.
- The limit `allowed_repo_hosts` (a list of hosts, personal or system layer, see `trust-model`) makes the server refuse to use a repository from another host. The default is no restriction. The check applies to the `--repo` URL before a clone, and to the `origin` of an existing folder. An existing folder without `origin` passes.
- A new clone happens only at the first start. The server never clones a second repository in a running session.

### The host of a URL

The host is the part after `://` or after `@`, without a user, a password, a port, or a path, in lower case. `git@github.com:example/waymark-agent-memory.git` has the host `github.com`. `ssh://git@example.org:2222/team/k.git` has the host `example.org`. `file:///tmp/k.git` and a plain path have the host `localhost`. A host in `allowed_repo_hosts` must be equal to it (no globs). Waymark never prints a user or a password of a URL.

## Interfaces & Contracts

The entry of a project:

```json
{ "mcpServers": { "waymark": { "command": "waymark", "args": ["--repo", "git@github.com:example/waymark-agent-memory.git", "--memory-repository-id", "6f1c2a9e-3b7d-4c55-9a0e-1d2f3a4b5c6d"] } } }
```

The personal default is the same entry in the user-level config of the agent tool. A person who wants a visible folder, for example an Obsidian vault, adds `--path`:

```json
{ "mcpServers": { "waymark": { "command": "waymark", "args": ["--repo", "git@github.com:example/waymark-agent-memory.git", "--path", "~/vaults/memory"] } } }
```

An entry without `--repo` and `--path` (`{ "command": "waymark" }`) is not valid.

The arguments are those of `mcp-server`: `--repo`, `--memory-repository-id`, and `--path`. The tools take no repository argument. A call with an argument that names a repository, a path, or a memory repository fails with `invalid_arguments`.

A server without a memory repository answers a tool call like this:

```json
{
  "code": "setup_problem",
  "message": "The MCP entry names no memory repository.",
  "next": "Add --repo <git url> or --path <folder> to the MCP entry.",
  "settings": { "access": "read-write", "paused": false }
}
```

The codes at start:

| Case | Code |
|---|---|
| The ID in the file differs from `--memory-repository-id` | `setup_problem` |
| The file has no `id` | `setup_problem` |
| The folder has another `origin` than `--repo` (name collision or remote mismatch) | `setup_problem` |
| The folder is not a Git repository, or is a linked worktree | `setup_problem` |
| `--path` is missing or empty and there is no `--repo` | `setup_problem` |
| The clone fails | `sync_failed` |
| The host is not in `allowed_repo_hosts` | `access_denied` |
| Neither `--repo` nor `--path` is given | `setup_problem` |

## Acceptance Criteria

- **Given** a project entry with `--repo`, **When** a session starts in that project, **Then** only that memory repository is used.
- **Given** an entry with `--memory-repository-id` and a repository with another ID, **When** the server starts, **Then** it reports `setup_problem` and serves nothing.
- **Given** `allowed_repo_hosts = ["git.example.org"]`, **When** an entry names another host, **Then** the server refuses to clone and creates no folder.
- **Given** a user entry and a project entry with the same name, **When** a session starts in the project, **Then** the agent tool uses the project entry (manual, per tool).
- **Given** the server is running, **When** a tool asks for another repository, **Then** it is refused. The tools take no repository argument.
- **Given** `--repo` and no `--path`, **When** `~/.waymark/<name>` is missing or empty, **Then** the server clones into it.
- **Given** `--repo` and no `--path`, **When** `~/.waymark/<name>` holds a repository with the same remote (also in the other URL form), **Then** the server uses it and does not clone again.
- **Given** `--repo` and no `--path`, **When** `~/.waymark/<name>` holds a repository with another remote, **Then** the server reports a name collision, asks for `--path`, and changes nothing.
- **Given** `--repo` and `--path`, **When** the path is missing or empty, **Then** the server clones into the path.
- **Given** `--repo` and `--path`, **When** the path holds a repository with the same remote, **Then** the server uses it.
- **Given** `--repo` and `--path`, **When** the path holds a repository with another remote, **Then** the server reports a remote mismatch and changes nothing.
- **Given** `--path` and no `--repo`, **When** the path holds a memory repository, **Then** the server uses it.
- **Given** `--path` and no `--repo`, **When** the path is missing or empty, **Then** the server asks for `--repo` to clone it.
- **Given** neither `--repo` nor `--path`, **When** the server starts, **Then** `status` works, every other tool fails with `setup_problem`, and the `next` sentence names `--repo` and `--path`. A memory repository in the working folder or in any other folder is not used.
- **Given** a `~` in `--path`, **When** the server starts, **Then** it means the home folder.
- **Given** two servers with different memory repositories, **When** both run, **Then** each tool call touches only the repository of its own server.
- **Given** a repository URL with a user and a password, **When** `status` shows it, **Then** it shows neither.

## Constraints & Edge Cases

- The ID is an identifier, not a secret.
- An agent that edits the MCP config of its tool could add an entry. This is a deliberate act and the same limit as raw Git use.
- A Git worktree of the code project has its own copy of the committed `.mcp.json`, so the project entry works there. A worktree of the memory repository repository is not supported.
- A first clone happens before the server answers `initialize`. A very large repository can exceed the start time-out of the client. The clone goes to a temporary folder next to the target and is renamed when it is complete, so a cut clone never counts as a memory repository. The next start clones again.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`). Which agent tools let a project entry override a user entry, and which ask before they start a project-level server, is checked by the manual tests in `test.md`.
