# Plan: Project binding

## Overview

A `binding` module in `mcp` runs once at start. It turns the arguments, the environment, the working folder, and the home folder into a `Binding` (the repository path, the root of the knowledge base, the ID, and how it was found) or into a `Problem` (a code, a message, and a `next` sentence). The `Context` keeps the result for the whole process.

## Components Affected

- `mcp` (`src/mcp/args.rs`) — the arguments `--repo`, `--memory-repository-id`, and `--path` already exist. No change.
- `mcp` (`src/mcp/context.rs`) — holds `binding: Result<Binding, Problem>`. A tool that needs the knowledge base gets it from here, and gets the `Problem` as an error when there is none.
- `mcp` (`src/mcp/tool.rs`) — a tool declares if it needs a knowledge base. `status` does not. `init` needs a Git repository without a marker (see `create-memory-repository`). All others need a `Binding`.
- `mcp` (`src/main.rs`) — runs `binding::resolve` before the server starts, and the clone of step 4.
- `kb` (`src/kb/layout.rs`) — gets `find_root_in_or_above(dir)`, which checks a folder and its parents for `.waymark/` and `agent-memory/.waymark/`. If the plan of `memory-repository` already has a function that does this, use it.
- `kb` (`src/kb/settings.rs`) — gives the `id` for the check against `--memory-repository-id`.
- `gitsync` (`src/gitsync/mod.rs`) — `clone_to(url, path)` (already planned for the first start) and `origin_url(path)`.
- `config` — `allowed_repo_hosts` from `EffectiveSettings`.
- `README.md` — a section "Project and default entry" with the two JSON examples and the result of the manual tests.

## New Files

- `src/mcp/binding.rs` — `Binding`, `FoundBy` (`PathArgument`, `RepoArgument`, `WorkingFolder`, `DefaultFolder`), `Problem`, and `resolve(&Args, &Env) -> Result<Binding, Problem>`. `Env` holds the working folder, the home folder, and a function that reads the effective settings. This makes it testable with no process.
- `src/mcp/repo_url.rs` — pure functions: `host(url) -> String`, `slug(url) -> String`, `same_remote(a, b) -> bool`, and `redact(url) -> String` (removes the user and password).
- `tests/it_project_binding.rs` — end-to-end tests.

## Technical Approach

1. **Location.** `resolve` applies the four rules of `requirements.md` in order. It checks the existence of folders and reads no document.
2. **Folder rules.**
   - Missing folder with `--repo`: clone (step 4). Missing folder without `--repo`: `setup_problem`.
   - Existing folder: it must be a Git repository (`git rev-parse --show-toplevel` equals the folder, and `.git` is a directory). A file `.git` is a linked worktree: `setup_problem`.
   - With `--repo` and an `origin`: `same_remote` or `setup_problem`.
3. **Host check.** Compute `host(url)` for the `--repo` URL, or for the `origin` of the folder. If `allowed_repo_hosts` is not empty and does not contain the host, the result is `access_denied` and nothing is cloned.
4. **Clone.** `gitsync::clone_to(url, "<path>.waymark-clone")`, then rename to `<path>`. A failure removes the temporary folder and gives `sync_failed` with the redacted URL.
5. **Root.** `kb::layout::find_root(repo_path)`. If none exists, the binding is "repository without a knowledge base": `Binding` has `root = None`, and only `init` and `status` run.
6. **Identity.** Read `.waymark/settings.toml` through `kb::settings`. No `id` is `setup_problem`. If `--memory-repository-id` is set and differs, `setup_problem`. The check runs again every time the shared file is read after a pull, so an ID that changes later stops the server's tools.
7. **Without `--repo` and `--path`.** Walk from the working folder up to `/`, using `find_root_in_or_above`. Then list `~/Projects/*-agent-memory`, keep those that hold a knowledge base, and use the one if the count is 1. For a count above 1 the `message` names the folders and the `next` sentence says to add `--path`.
8. **Host check for found folders.** Rule 3 applies to the `origin` of a folder from the working folder rule and the default rule.
9. **No second knowledge base.** `Binding` is immutable. No tool schema has a repository, path, or knowledge base property. `additionalProperties: false` in every schema gives `invalid_arguments` for such an argument.
10. **Redaction.** `repo_url::redact` is the only way a URL goes into a message, a log, or a result.
11. **Cost.** `resolve` runs a few `git` calls and file reads. It does no network call, except the first-start clone.

## Cross-Feature Dependencies

- `mcp-server` — owns `args.rs`, `context.rs`, `main.rs`. This feature adds `binding.rs` and the `Context` field. The decision "One knowledge base per process" of `mcp-server` is the source of the discovery rule.
- `memory-repository` — the root and the `id`. This feature may add `find_root_in_or_above` to `src/kb/layout.rs`.
- `git-sync` — `clone_to` and `origin_url`. This feature may add them to `src/gitsync/mod.rs`.
- `trust-model` — `allowed_repo_hosts`.
- `create-memory-repository` — the `init` tool uses the "repository without a knowledge base" state.
- `session-start` — reads `Binding` for the instructions and `status`.
- `agent-tool-setup` — the README examples for the project entry and the personal default.

## Risks

- A slow first clone exceeds the start time-out of the client. Mitigation: the temporary folder, and the note in the README that a large repository can be cloned by hand to the default path first.
- The working folder of the server is not the project folder in some agent tool. Mitigation: rule 3 is only a convenience. The manual tests check it, and the README tells to use `--repo` or `--path` otherwise.
- Two forms of one URL (`https` and `ssh`) do not match. Mitigation: a clear `setup_problem`.

## Open Questions

None.
