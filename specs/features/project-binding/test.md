# Tests: Project binding

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_The tests use the harness of `integration-testing`. The `HOME` of the `World` is a temporary folder, so `~/Projects` is inside it. The remote is a bare `file://` repository. The tests read the result through `status`._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** a project entry with `--repo`, **When** a session starts, **Then** only that knowledge base is used | `repo_argument_binds_one_knowledge_base` |
| **Given** `--kb-id` and another ID in the repository, **When** the server starts, **Then** `setup_problem` and nothing is served | `wrong_kb_id_serves_nothing` |
| **Given** `allowed_repo_hosts = ["git.example.org"]`, **When** the entry names another host, **Then** no clone | `host_not_allowed_refuses_clone` |
| **Given** a user entry and a project entry with the same name, **When** a session starts, **Then** the project entry is used | M1 to M3 (manual) |
| **Given** the server is running, **When** a tool asks for another repository, **Then** it is refused | `tools_take_no_repository_argument` |
| **Given** `--repo` and no `--path`, **When** the server starts the first time, **Then** it clones to `~/Projects/<slug>-knowledge-base` and reuses it | `default_folder_from_slug` |
| **Given** no `--repo` and `--path`, **When** the working folder is inside a knowledge base, **Then** it is used | `working_folder_is_found` |
| **Given** no `--repo` and `--path`, **When** one default folder has a knowledge base, **Then** it is used | `single_default_folder_is_used` |
| **Given** no `--repo` and `--path`, **When** none or two are found, **Then** `status` works and other tools give `setup_problem` | `nothing_or_many_found` |
| **Given** a folder with another `origin` than `--repo`, **When** the server starts, **Then** `setup_problem` | `origin_mismatch` |
| **Given** two servers with different knowledge bases, **When** both run, **Then** each call touches only its own repository | `two_servers_do_not_mix` |
| **Given** a URL with a user and a password, **When** `status` shows it, **Then** it shows neither | `url_is_redacted` |

## Test Cases

- [ ] [auto] `--repo` with a `file://` remote and an empty home: the server clones to the default folder, `status` shows the ID of that knowledge base and `found_by: "repo_argument"`. A second start does not clone again (`tests/it_project_binding.rs`)
- [ ] [auto] `--repo` and `--path`: the clone goes to the path (`tests/it_project_binding.rs`)
- [ ] [auto] `--kb-id` equal to the file: all tools work (`tests/it_project_binding.rs`)
- [ ] [auto] `--kb-id` different: `status` works and shows the problem, and a read tool and a write tool give `setup_problem`. The write makes no commit (`tests/it_project_binding.rs`)
- [ ] [auto] The shared file has no `id`: `setup_problem` (`tests/it_project_binding.rs`)
- [ ] [auto] The ID in the file changes after a pull while `--kb-id` is set: the next tool call gives `setup_problem` (`tests/it_project_binding.rs`)
- [ ] [auto] `allowed_repo_hosts = ["git.example.org"]` with a `file://` URL: `access_denied`, and the target folder and the temporary folder do not exist. With `["localhost"]` it works (`tests/it_project_binding.rs`)
- [ ] [auto] An existing folder whose `origin` has a refused host: `access_denied` (`tests/it_project_binding.rs`)
- [ ] [auto] A knowledge base without `origin` passes the host check (`tests/it_project_binding.rs`)
- [ ] [auto] Every schema in `tools/list` has `additionalProperties: false` and no property named `repo`, `path`, `kb`, or `kb_id`. A call with `repo` gives `invalid_arguments` (`tests/it_project_binding.rs`)
- [ ] [auto] The working folder is the root, a subfolder, and the parent of `knowledge/`: each finds the knowledge base (`tests/it_project_binding.rs`)
- [ ] [auto] The working folder has no knowledge base, and `~/Projects/a-knowledge-base` has one: it is used, with `found_by: "default_folder"` (`tests/it_project_binding.rs`)
- [ ] [auto] Two default folders with knowledge bases: `status` works, and every other tool gives `setup_problem` whose `message` names both folders and whose `next` mentions `--path` (`tests/it_project_binding.rs`)
- [ ] [auto] Nothing found: `setup_problem`, and the `next` sentence mentions `--repo` (`tests/it_project_binding.rs`)
- [ ] [auto] A Git repository without a marker: `status` works, other tools give `setup_problem` (`init` is tested in `create-knowledge-base`) (`tests/it_project_binding.rs`)
- [ ] [auto] The path exists with another `origin` than `--repo`: `setup_problem`, and the folder is not changed (`tests/it_project_binding.rs`)
- [ ] [auto] The path is a linked worktree of the knowledge base repository: `setup_problem` (`tests/it_project_binding.rs`)
- [ ] [auto] The path is not a Git repository: `setup_problem` (`tests/it_project_binding.rs`)
- [ ] [auto] A clone from a remote that does not exist: `sync_failed`, and no temporary folder remains. A leftover temporary folder from an earlier start is removed (`tests/it_project_binding.rs`)
- [ ] [auto] Two `McpClient`s with different `--path` in one `World`: a write through one leaves the other repository untouched (`tests/it_project_binding.rs`)
- [ ] [auto] A `--repo` URL with `user:secret@` never appears in a result, in `status`, or in stderr (`tests/it_project_binding.rs`)
- [ ] [auto] Unit tests of `host`, `slug`, `same_remote`, and `redact` (inline in `src/mcp/repo_url.rs`)
- [ ] [manual] M1, Claude Code: the project entry wins. Steps: (1) Add an entry named `waymark` with `--repo <A>` to the user-level MCP config. (2) In a project folder, commit a `.mcp.json` with an entry named `waymark` and `--repo <B>`. (3) Start Claude Code in the project and ask the agent to call `status`. Expected: `status` shows the knowledge base B. Outside the project it shows A. Record the exact config locations.
- [ ] [manual] M2, GitHub Copilot in VS Code: as M1. Expected and record: which entry wins.
- [ ] [manual] M3, OpenAI Codex CLI: as M1. Expected and record: which entry wins.
- [ ] [manual] M4, the first-start prompt. Steps: clone a project that has a committed project entry. Start each of the three tools in it. Expected: the tool asks the person before it starts the server. Record the result for each tool. If one does not ask, the README names it.
- [ ] [manual] M5, the working folder. Steps: use the entry `{ "command": "waymark" }` with no arguments, put the knowledge base at `~/Projects/example-knowledge-base`, and start each tool in an unrelated folder. Expected: `status` shows the knowledge base with `found_by: "default_folder"`. Then start each tool inside the knowledge base folder. Expected: `found_by: "working_folder"`. Record any tool that starts the server in another folder.
- [ ] [manual] M6, a Git worktree of the code project. Steps: `git worktree add ../wt` in a project with a committed `.mcp.json`. Start Claude Code in `../wt`. Expected: `status` shows the same knowledge base as in the main folder.
- [ ] [manual] M7, a large first clone. Steps: use a repository of at least 500 MB as the remote. Start the agent tool with a new `--repo`. Expected: either the server starts, or the tool reports a start time-out and the second start succeeds. Record the time-out of the tool.

## Notes

The manual tests answer the open questions of the old spec: which tools let a project entry override a user entry, which ask before they start a project-level server, and how a worktree behaves.
