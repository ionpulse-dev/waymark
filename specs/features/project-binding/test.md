# Tests: Project binding

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_The tests use the harness of `integration-testing`. The `HOME` of the `World` is a temporary folder, so `~/.waymark` is inside it. The remote is a bare `file://` repository. The tests read the result through `status`._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** a project entry with `--repo`, **When** a session starts, **Then** only that memory repository is used | `repo_argument_binds_one_memory_repository` |
| **Given** `--memory-repository-id` and another ID in the repository, **When** the server starts, **Then** `setup_problem` and nothing is served | `wrong_memory_repository_id_serves_nothing` |
| **Given** `allowed_repo_hosts = ["git.example.org"]`, **When** the entry names another host, **Then** no clone | `host_not_allowed_refuses_clone` |
| **Given** a user entry and a project entry with the same name, **When** a session starts, **Then** the project entry is used | M1 to M3 (manual) |
| **Given** the server is running, **When** a tool asks for another repository, **Then** it is refused | `tools_take_no_repository_argument` |
| **Given** `--repo`, no `--path`, and `~/.waymark/<name>` missing or empty, **Then** it clones into it (row 1) | `row1_repo_only_clones_to_default` |
| **Given** `--repo`, no `--path`, and the folder holds the same remote (also in the other URL form), **Then** it is used and not cloned again (row 2) | `row2_repo_only_uses_same_remote` |
| **Given** `--repo`, no `--path`, and the folder holds another remote, **Then** name collision, set `--path` (row 3) | `row3_repo_only_name_collision` |
| **Given** `--repo` and `--path`, and the path missing or empty, **Then** it clones into the path (row 4) | `row4_both_clones_to_path` |
| **Given** `--repo` and `--path`, and the path holds the same remote, **Then** it is used (row 5) | `row5_both_uses_same_remote` |
| **Given** `--repo` and `--path`, and the path holds another remote, **Then** remote mismatch (row 6) | `row6_both_remote_mismatch` |
| **Given** `--path` only, and the path holds a memory repository, **Then** it is used (row 7) | `row7_path_only_uses_memory_repository` |
| **Given** `--path` only, and the path missing or empty, **Then** the error says to add `--repo` (row 8) | `row8_path_only_missing_or_empty` |
| **Given** neither, **Then** `status` works, other tools give `setup_problem` naming `--repo` and `--path` (row 9) | `row9_neither_given` |
| **Given** `~` in `--path`, **Then** it is the home folder | `tilde_in_path` |
| **Given** a folder with another `origin` than `--repo`, **When** the server starts, **Then** `setup_problem` | `row3_repo_only_name_collision`, `row6_both_remote_mismatch` |
| **Given** two servers with different memory repositories, **When** both run, **Then** each call touches only its own repository | `two_servers_do_not_mix` |
| **Given** a URL with a user and a password, **When** `status` shows it, **Then** it shows neither | `url_is_redacted` |

## Test Cases

- [x] [auto] Row 1: `--repo` with a `file://` remote and an empty home: the server clones to `~/.waymark/<name>`, `status` shows the ID, the path, and `found_by: "repo_argument"`. An existing empty `~/.waymark/<name>` is used as the target too. A second start does not clone again (`tests/it_project_binding.rs`)
- [x] [auto] Row 2: `~/.waymark/<name>` holds a clone of the same remote, and `--repo` uses the other URL form (with and without `.git`): the folder is used and no clone starts (`tests/it_project_binding.rs`)
- [x] [auto] Row 3: `~/.waymark/<name>` holds a repository with another remote: `setup_problem` whose message says name collision and whose `next` names `--path`. The folder is not changed (`tests/it_project_binding.rs`)
- [x] [auto] Row 4: `--repo` and `--path` to a missing folder, and to an empty folder: the clone goes to the path, and `~/.waymark` is not created (`tests/it_project_binding.rs`)
- [x] [auto] Row 5: `--repo` and `--path` with a clone of the same remote: it is used (`tests/it_project_binding.rs`)
- [x] [auto] Row 6: `--repo` and `--path` with another remote: `setup_problem` that says remote mismatch. The folder is not changed (`tests/it_project_binding.rs`)
- [x] [auto] Row 7: `--path` only, with a memory repository: it is used, with `found_by: "path_argument"` (`tests/it_project_binding.rs`)
- [x] [auto] Row 8: `--path` only, missing and empty: `setup_problem` whose `next` says to add `--repo` (`tests/it_project_binding.rs`)
- [x] [auto] Row 9: no arguments: `status` works, another tool gives `setup_problem` whose `next` names `--repo` and `--path`. The working folder holds a memory repository and `$HOME/Projects/x-agent-memory` holds one: neither is used (`tests/it_project_binding.rs`)
- [x] [auto] `--path ~/x` expands to `$HOME/x`, and a relative `--path` is resolved against the working folder (`tests/it_project_binding.rs`)
- [x] [auto] A URL without a name and no `--path`: `setup_problem` that asks for `--path` (unit test in `src/mcp/repo_url.rs`, and `tests/it_project_binding.rs`)
- [x] [auto] `--memory-repository-id` equal to the file: all tools work (`tests/it_project_binding.rs`)
- [x] [auto] `--memory-repository-id` different: `status` works and shows the problem, and a read tool and a write tool give `setup_problem`. The write makes no commit (`tests/it_project_binding.rs`)
- [x] [auto] The shared file has no `id`: `setup_problem` (`tests/it_project_binding.rs`)
- [x] [auto] The ID in the file changes after a pull while `--memory-repository-id` is set: the next tool call gives `setup_problem` (`tests/it_project_binding.rs`)
- [x] [auto] `allowed_repo_hosts = ["git.example.org"]` with a `file://` URL: `access_denied`, and the target folder and the temporary folder do not exist. With `["localhost"]` it works (`tests/it_project_binding.rs`)
- [x] [auto] An existing folder whose `origin` has a refused host: `access_denied` (`tests/it_project_binding.rs`)
- [x] [auto] A memory repository without `origin` passes the host check (`tests/it_project_binding.rs`)
- [x] [auto] Every schema in `tools/list` has `additionalProperties: false` and no property named `repo`, `path`, `memrepo`, or `memory_repository_id`. A call with `repo` gives `invalid_arguments` (`tests/it_project_binding.rs`)
- [x] [auto] A Git repository without a marker: `status` works, other tools give `setup_problem` (`init` is tested in `create-memory-repository`) (`tests/it_project_binding.rs`)
- [x] [auto] The path is a linked worktree of the memory repository repository: `setup_problem` (`tests/it_project_binding.rs`)
- [x] [auto] The path is not a Git repository: `setup_problem` (`tests/it_project_binding.rs`)
- [x] [auto] A clone from a remote that does not exist: `sync_failed`, and no temporary folder remains. A leftover temporary folder from an earlier start is removed (`tests/it_project_binding.rs`)
- [x] [auto] Two `McpClient`s with different `--path` in one `World`: a write through one leaves the other repository untouched (`tests/it_project_binding.rs`)
- [x] [auto] A `--repo` URL with `user:secret@` never appears in a result, in `status`, or in stderr (`tests/it_project_binding.rs`)
- [x] [auto] Unit tests of `host`, `name`, `same_remote` (SSH and HTTPS forms), and `redact` (inline in `src/mcp/repo_url.rs`)
- [ ] [manual] M1, Claude Code: the project entry wins. Steps: (1) Add an entry named `waymark` with `--repo <A>` to the user-level MCP config. (2) In a project folder, commit a `.mcp.json` with an entry named `waymark` and `--repo <B>`. (3) Start Claude Code in the project and ask the agent to call `status`. Expected: `status` shows the memory repository B. Outside the project it shows A. Record the exact config locations.
- [ ] [manual] M2, GitHub Copilot in VS Code: as M1. Expected and record: which entry wins.
- [ ] [manual] M3, OpenAI Codex CLI: as M1. Expected and record: which entry wins.
- [ ] [manual] M4, the first-start prompt. Steps: clone a project that has a committed project entry. Start each of the three tools in it. Expected: the tool asks the person before it starts the server. Record the result for each tool. If one does not ask, the README names it.
- [ ] [manual] M5, the visible folder. Steps: add the entry with `--repo <url>` and `--path ~/vaults/memory` to each tool, and start a session. Expected: the clone is in `~/vaults/memory`, `status` shows that folder with `found_by: "path_argument"`, and Obsidian opens it as a vault.
- [ ] [manual] M6, a Git worktree of the code project. Steps: `git worktree add ../wt` in a project with a committed `.mcp.json`. Start Claude Code in `../wt`. Expected: `status` shows the same memory repository as in the main folder.
- [ ] [manual] M7, a large first clone. Steps: use a repository of at least 500 MB as the remote. Start the agent tool with a new `--repo`. Expected: either the server starts, or the tool reports a start time-out and the second start succeeds. Record the time-out of the tool.

## Notes

The manual tests answer the open questions of the old spec: which tools let a project entry override a user entry, which ask before they start a project-level server, and how a worktree behaves.
