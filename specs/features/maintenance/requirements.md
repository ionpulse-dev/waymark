---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - login-and-identity
  - join-memory-repository
  - agent-tool-setup
---

# Maintenance

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. The `/health` endpoint, the update of a server image, and the checks of a server are out of scope for now (see [project decisions](../../../docs/decisions.md)). They are in `todo.md`. This spec covers health and diagnostics. The `stats` tool belongs to `search-and-read`.

## User Story

> As a person, I want to check, update, and remove waymark easily, so that I always know it works and I can leave without leftovers.

## Personas

- **Every person** — checks health, updates, removes.
- **IT** — runs the same checks through an agent, or reads them from a person.

## Behaviour

waymark has no commands for people. Health checks are a tool, and update and removal use the package manager and the README steps.

### The `status` tool

`status` is read-only. It runs in every limit state: `read-only`, `paused`, and also when the memory repository is not set up. `tools_denied` does not hide it. It takes no input. It returns the state of the memory repository, a list of `checks`, and the facts below. A failed check carries a plain-words `fix`.

Checks, in this fixed order and with these fixed names:

| Name | `ok` when | `fix` when not ok (examples) |
|---|---|---|
| `git` | `git --version` works (the `git` binary on macOS can be a stub) | "Install the Xcode command line tools: run `xcode-select --install`, or install Git with Homebrew." |
| `clone` | the state is `ready`, `empty`, or `not_a_memory_repository` and the folder is a repository | While cloning: "The first clone is still running. Call `status` again in a minute." For `broken`: the fix of the reason (see `join-memory-repository`). For `empty`: this check is ok and the `detail` says "The repository is empty. Call `init`." |
| `working_tree` | no Git operation is half done (no rebase or merge in progress). `detail` names the number of files with pending changes. | "A previous Git operation did not finish. Open the folder `<path>` in a terminal, run `git status`, and finish or abort it." |
| `remote` | the remote answers a read call within 10 seconds, or there is no remote (`detail`: "no remote") | "The remote is not reachable. Check the network and the URL `<url without password>`." |
| `git_login` | the same read call was not refused for a login reason | "Git cannot sign in to this repository. Sign in with Git first, for example with `gh auth login`, or set up an SSH key." |
| `memory_repository_id` | `--memory-repository-id` is not given, or it equals `id` of `.waymark/settings.toml` | "This repository has the ID `<id>`, and the entry expects `<expected>`. Check `--repo` and `--memory-repository-id`." |
| `settings` | every config layer file (system, personal, shared) parses | "The file `<path>` has a syntax error. Fix it. waymark does not ignore a broken file, because that would loosen a limit." |
| `author` | the author resolves and passes the email check (see `login-and-identity`) | The `next` sentence of `login-and-identity`. |

While the state is `cloning`, only `git` and `clone` run. If the state is `broken` because of `git`, only `git` runs. A check that cannot run is left out. It is not marked ok.

The network call (`remote` and `git_login`) is one read call to the remote with a time-out of 10 seconds. It reports "not reachable" and does not wait longer. `git` never asks a question.

### Result

`message` is "All checks are ok." or "<n> checks failed: <names>.". A failed check adds one `notices` entry. `notices` also holds the notices of `session-start` (for example "Writes are paused."). `settings` carries the effective limits. Besides these, `data` holds:

```json
{
  "healthy": false,
  "version": "0.1.0",
  "memory_repository": { "state": "ready", "name": "Example", "id": "0b1f3c9e-5a77-4d1e-9a41-2f6f0c8d7e11", "slug": "example", "path": "/Users/jane/Projects/waymark-agent-memory", "root": "knowledge" },
  "checks": [
    { "name": "git", "ok": true, "detail": "git version 2.50.1", "fix": null },
    { "name": "clone", "ok": true, "detail": "ready", "fix": null },
    { "name": "working_tree", "ok": true, "detail": "0 files with pending changes", "fix": null },
    { "name": "remote", "ok": true, "detail": "reachable", "fix": null },
    { "name": "git_login", "ok": false, "detail": "Permission denied", "fix": "Git cannot sign in to this repository. Sign in with Git first, for example with `gh auth login`, or set up an SSH key." },
    { "name": "memory_repository_id", "ok": true, "detail": "no ID expected", "fix": null },
    { "name": "settings", "ok": true, "detail": "3 files read", "fix": null },
    { "name": "author", "ok": true, "detail": "Jane Doe <jane@example.org>", "fix": null }
  ],
  "author": { "name": "Jane Doe", "email": "jane@example.org", "name_source": "argument", "email_source": "git_config" },
  "limit_sources": { "access": ["shared"], "max_results": ["default"], "tools_denied": ["system", "personal"] },
  "last_pull": "2026-11-14T10:20:00Z",
  "last_error": null
}
```

- `memory_repository.state` is one of `cloning`, `ready`, `empty`, `not_a_memory_repository`, `broken`. `name`, `id`, `slug`, and `root` are `null` if unknown.
- `limit_sources` names, for each limit in `settings`, the layers that set the effective value: `default`, `system`, `personal`, `shared`, `entry`. A list limit names all layers that add to the union.
- `last_pull` is the time of the last good fetch in this session, or `null`. `last_error` is the last error text of this session (without secrets), or `null`.
- `healthy` is true when every check that ran is ok.

### Update

- The person updates the binary with Homebrew (`brew upgrade waymark`). The tool descriptions come with the binary. The new version applies at the next start of the agent tool.
- waymark does not check for updates and does not call any server.

### Removal

The README (the section "Remove waymark" of `docs/install.md`) lists the steps. waymark does not remove itself:

1. Remove the MCP entry from the agent tool (the command for each tool is in the section "Agent tools").
2. Check the clone for changes that are not pushed (`git status` and `git log @{u}..` in the folder). Delete the folder if it is no longer needed. The remote repository stays.
3. Delete `~/.config/waymark/`. If IT set a system file, `/Library/Application Support/waymark/config.toml` belongs to IT.
4. Remove the binary: `brew uninstall waymark`.

waymark never writes into agent tools, so nothing else is left behind.

## Interfaces & Contracts

`status` takes no arguments (`{}`). Its result is the one above. It has the kind `read`. It is the only tool that is always listed.

## Acceptance Criteria

- **Given** a broken Git login, **When** `status` runs, **Then** it reports the problem and the fix.
- **Given** a healthy setup, **When** `status` runs, **Then** all checks are ok and `healthy` is true.
- **Given** `access = "read-only"` or `paused = true`, **When** `status` runs, **Then** it still works and names the state.
- **Given** `tools_denied = ["status"]`, **When** the client lists tools, **Then** `status` is still listed.
- **Given** a repository with another ID than expected, **When** `status` runs, **Then** it reports it.
- **Given** a remote that does not answer, **When** `status` runs, **Then** it ends within 12 seconds and reports "not reachable".
- **Given** a broken config file, **When** `status` runs, **Then** the `settings` check names the file.
- **Given** a limit set in two layers, **When** `status` runs, **Then** `limit_sources` names the layer of the effective value.
- **Given** the steps of the README, **When** a person follows them, **Then** no waymark file or entry remains, except the repository they chose to keep.
- **Given** any `status` result, **When** it is inspected, **Then** it holds no token or password.

## Constraints & Edge Cases

- `status` runs one network call. It has a short time-out and reports "not reachable" instead of waiting.
- `status` never changes the repository. It does not pull.
- A repository without a remote is ok. The result carries the notice "this repository has no remote".

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`): a `waymark doctor` command is not added.
