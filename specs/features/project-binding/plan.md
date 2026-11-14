# Plan: Project binding

## Overview

A `binding` module in `mcp` runs once at start. It turns the arguments and the home folder into a `Binding` (the repository path, the root of the memory repository, the ID, and how it was found) or into a `Problem` (a code, a message, and a `next` sentence). The `Context` keeps the result for the whole process.

## Components Affected

- `mcp` (`src/mcp/args.rs`) — the arguments `--repo`, `--memory-repository-id`, and `--path` already exist. No change.
- `mcp` (`src/mcp/context.rs`) — holds `binding: Result<Binding, Problem>`. A tool that needs the memory repository gets it from here, and gets the `Problem` as an error when there is none.
- `mcp` (`src/mcp/tool.rs`) — a tool declares if it needs a memory repository. `status` does not. `init` needs a Git repository without a marker (see `create-memory-repository`). All others need a `Binding`.
- `mcp` (`src/main.rs`) — runs `binding::resolve` before the server starts, and the clone of step 4.
- `memrepo` (`src/memrepo/settings.rs`) — gives the `id` for the check against `--memory-repository-id`.
- `gitsync` (`src/gitsync/mod.rs`) — `clone_to(url, path)` (already planned for the first start) and `origin_url(path)`.
- `config` — `allowed_repo_hosts` from `EffectiveSettings`.
- `README.md` and `docs/install.md` — the section "Where the memory repository lives" and the entry examples.

## New Files

- `src/mcp/binding.rs` — `Binding`, `FoundBy` (`PathArgument`, `RepoArgument`), `Problem`, and `resolve(&Args, &Env) -> Result<Binding, Problem>`. `Env` holds the home folder, and a function that reads the effective settings. This makes it testable with no process.
- `src/mcp/repo_url.rs` — pure functions: `host(url) -> String`, `name(url) -> Option<String>`, `same_remote(a, b) -> bool`, and `redact(url) -> String` (removes the user and password).
- `tests/it_project_binding.rs` — end-to-end tests.

## Technical Approach

1. **Location.** `resolve` has no search. With `--path`: the folder is the path (`~` expanded, relative paths resolved against the working folder). With `--repo` only: the folder is `~/.waymark/<name>`, where `repo_url::name` gives `<name>`. With neither: `setup_problem` ("The MCP entry names no memory repository", `next` names `--repo` and `--path`). `FoundBy` is `PathArgument` when `--path` is set, otherwise `RepoArgument`.
2. **Folder rules** (the table of `requirements.md`).
   - Missing or empty folder with `--repo`: clone (step 4). Missing or empty folder without `--repo`: `setup_problem` that says to add `--repo`.
   - Existing folder that is not empty: it must be a Git repository (`.git` is a directory). A file `.git` is a linked worktree: `setup_problem`. Not a Git repository: `setup_problem`.
   - With `--repo` and an `origin`: `same_remote` or `setup_problem`. The sentence says "name collision, set `--path`" when the folder is the default folder, and "remote mismatch" when `--path` was given.
3. **Host check.** Compute `host(url)` for the `--repo` URL, or for the `origin` of the folder. If `allowed_repo_hosts` is not empty and does not contain the host, the result is `access_denied` and nothing is cloned.
4. **Clone.** `gitsync::clone_to(url, "<path>.waymark-clone")`, then rename to `<path>`. A failure removes the temporary folder and gives `sync_failed` with the redacted URL. The parent folder (`~/.waymark`) is created if it is missing.
5. **Root.** `memrepo::layout::find_root(repo_path)`. If none exists, the binding is "repository without a memory repository": `Binding` has `root = None`, and only `init` and `status` run.
6. **Identity.** Read `.waymark/settings.toml` through `memrepo::settings`. No `id` is `setup_problem`. If `--memory-repository-id` is set and differs, `setup_problem`. The check runs again every time the shared file is read after a pull, so an ID that changes later stops the server's tools.
7. **Same remote.** `repo_url::same_remote` turns a URL into `host/path` (user, port, scheme, a trailing `/`, and `.git` removed, host in lower case), so that `git@github.com:acme/k.git` and `https://github.com/acme/k` match. Only `origin` is read.
8. **Host check for found folders.** Rule 3 applies to the `origin` of an existing folder.
9. **No second memory repository.** `Binding` is immutable. No tool schema has a repository, path, or memory repository property. `additionalProperties: false` in every schema gives `invalid_arguments` for such an argument.
10. **Redaction.** `repo_url::redact` is the only way a URL goes into a message, a log, or a result.
11. **Cost.** `resolve` runs a few `git` calls and file reads. It does no network call, except the first-start clone.

## Cross-Feature Dependencies

- `mcp-server` — owns `args.rs`, `context.rs`, `main.rs`. This feature adds `binding.rs` and the `Context` field. The `--path` text of `args.rs` changes with this feature.
- `memory-repository` — the root and the `id`.
- `git-sync` — `clone_to` and `origin_url`. This feature may add them to `src/gitsync/mod.rs`.
- `trust-model` — `allowed_repo_hosts`.
- `create-memory-repository` — the `init` tool uses the "repository without a memory repository" state.
- `session-start` — reads `Binding` for the instructions and `status`.
- `agent-tool-setup` — the README examples for the project entry and the personal default.

## Risks

- A slow first clone exceeds the start time-out of the client. Mitigation: the temporary folder, and the note in the README that a large repository can be cloned by hand to the folder first.
- A person has a different repository with the same name from another owner. Mitigation: the name collision error names `--path`.
- A URL form that `same_remote` does not know (for example a mirror) does not match. Mitigation: a clear `setup_problem` that names `--path`.

## Open Questions

None.
