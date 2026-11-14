# Tests: Create a knowledge base

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** an empty remote repository and a running server, **When** `init` runs with a name, **Then** the repository has the skeleton and the first commit is pushed. | A1 |
| **Given** the result, **When** another person adds the entry with the URL, **Then** they need not enter any shared value. | A2 |
| **Given** `init` ran, **When** it runs again, **Then** it fails with `validation_failed` and changes nothing. | A3 |
| **Given** a repository that is not empty, **When** `init` runs, **Then** it fails and changes nothing, and no existing file is overwritten. | A4, A5 |
| **Given** a repository with only a `README.md` from the Git host, **When** `init` runs, **Then** it succeeds and the `README.md` is unchanged. | A6 |
| **Given** the new repository, **When** it is opened in Obsidian, **Then** it works as a vault. | A7, M1 |
| **Given** `access = "read-only"`, **When** `init` runs, **Then** it fails with `access_denied`. | A8 |
| **Given** `preview: true`, **When** `init` runs, **Then** nothing is written or committed. | A9 |
| **Given** `init` succeeded, **When** the same session calls `search` or `add`, **Then** the tools work with no restart. | A10 |
| **Given** a bad input, **When** `init` runs, **Then** it fails with `invalid_arguments`, names the field, and changes nothing. | A11 |
| **Given** two people who call `init` at the same time, **When** both end, **Then** exactly one commit of `init` exists. | A12 |
| **Given** a repository with a `.waymark` marker and documents but no `settings.toml`, **When** `init` runs with a name, **Then** only the `settings.toml` is added and the state is `ready` in the same session. | A16, A17 |
| **Given** that repository, **When** `init` runs with `folders` or `audience`, **Then** it fails with `invalid_arguments` and changes nothing. | A18 |

## Test Cases

- [x] [auto] A1: `World::empty()`, a server on clone A, `init` with a name, folders, and `email_domains`: the remote has all files of the layout, one commit `waymark: init "Example"` by the author, `data.files` matches, `.waymark/version` is `1.0\n`, and `settings.toml` holds a valid UUID (`tests/it_create_memory_repository.rs`)
- [x] [auto] A2: a second server on clone B with only `--repo`: `status` is `ready` and `settings` show the limits that `init` stored (`tests/it_create_memory_repository.rs`)
- [x] [auto] A3: `init` twice: the second returns `validation_failed` with the "already a knowledge base" message, and the remote head did not move (`tests/it_create_memory_repository.rs`)
- [x] [auto] A4: a remote that holds `src/main.rs`: `validation_failed`, and the head did not move (`tests/it_create_memory_repository.rs`)
- [x] [auto] A5: a remote that holds an `AGENTS.md` with custom text: `validation_failed`, and the file is unchanged (`tests/it_create_memory_repository.rs`)
- [x] [auto] A6: a remote that holds only `README.md` and `LICENSE`: `init` works and both files are byte-equal to before (`tests/it_create_memory_repository.rs`)
- [x] [auto] A7: the files `.obsidian/app.json` (both keys) and `.gitignore` (both lines) have the exact content; after a file `.obsidian/workspace.json` is created in the clone, `git status --porcelain` does not list it (`tests/it_create_memory_repository.rs`)
- [x] [auto] A8: `access = "read-only"` in the personal config: `access_denied`, and with `tools_denied = ["init"]` the tool is not in `tools/list`. With `paused = true`: `paused` (`tests/it_create_memory_repository.rs`)
- [x] [auto] A9: `preview: true`: no file in the clone, no commit, the result has `commit: null` and `pushed: false` (`tests/it_create_memory_repository.rs`)
- [x] [auto] A10: after `init`, `add` a document and `search` it in the same session (`tests/it_create_memory_repository.rs`)
- [x] [auto] A11: bad slug, a folder `../x`, a folder with an uppercase letter, an unknown key in `limits`, a blank name: each is `invalid_arguments` with the field in `next` (`tests/it_create_memory_repository.rs`)
- [x] [auto] A12: two servers (clones A and B of one empty remote) call `init` at once: one succeeds, the other returns `validation_failed`, and `git log` shows one `init` commit (`tests/it_create_memory_repository.rs`)
- [x] [auto] A13: `derive_slug` for "Example Company", "  Ünï Corp ", "!!!" (`src/kb/init.rs`)
- [x] [auto] A14: the `AGENTS.md` written for `personal` and for `team` carries the pinned skill version (`tests/it_create_memory_repository.rs`; see `plain-git-access`)
- [x] [auto] A15: the generated `AGENTS.md`, `.gitignore`, and `settings.toml` pass `kb::settings` and the document checks (the root has no document, so `lint` finds nothing) (`tests/it_create_memory_repository.rs`)
- [x] [auto] A16: a remote with `.waymark/version`, `.waymark/exclude`, a document, and a `README.md` but no `settings.toml`: `status` is `broken`, `init` adds exactly `.waymark/settings.toml` (`adopted: true`), the other files are byte-equal, the state is `ready` in the same session, and a second `init` is `validation_failed` (`tests/it_create_memory_repository.rs`)
- [x] [auto] A17: the same with the marker in `agent-memory/`: the file is `agent-memory/.waymark/settings.toml` (`tests/it_create_memory_repository.rs`)
- [x] [auto] A18: `folders` or `audience` for such a repository: `invalid_arguments`, and the head did not move (`tests/it_create_memory_repository.rs`)
- [ ] [manual] M1: Obsidian. Steps: clone the repository that `init` made. Open the folder as a vault in Obsidian. Create a link between two documents through the link dialog. Expected: the vault opens, the link is a relative Markdown link with `.md`, and `git status` does not list `.obsidian/workspace.json`.
- [ ] [manual] M2: real Git host. Steps: create an empty private repository on a real host. Add the entry to an agent tool with the SSH URL. Start a session, call `status` (state `empty`), then ask the agent to call `init`. Expected: the files appear on the host in one commit, and a second person with only the URL can search.

## Notes

All `[auto]` tests use `file://` remotes of the harness. No real Git host is needed.
