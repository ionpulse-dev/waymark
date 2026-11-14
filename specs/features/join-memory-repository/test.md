# Tests: Join a knowledge base

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** a Mac without working `git`, **When** the server starts, **Then** `status` reports it with a fix and no clone starts. | A1, M2 |
| **Given** a valid URL and a Git login, **When** the first session starts, **Then** the repository is cloned and `status` is ok. | A2, M1 |
| **Given** a repository URL and no `--path`, **When** the first session starts, **Then** the clone is in `~/Projects/<repository-name>`. | A3 |
| **Given** a personal value looser than the shared limit, **When** the server reads both, **Then** the stricter value applies. | A4 |
| **Given** a person who adds a second entry for another knowledge base, **When** a session starts, **Then** the first one is unchanged. | A5 |
| **Given** a repository whose `.waymark/settings.toml` has no `id`, **When** the server reads it, **Then** it reports `setup_problem`. | A6 |
| **Given** the server started twice, **When** the second start ends, **Then** nothing is cloned twice. | A7 |
| **Given** a path that holds another repository or a non-empty folder, **When** the server starts, **Then** it changes nothing and reports `setup_problem`. | A8 |
| **Given** a clone that takes long, **When** the client connects, **Then** `initialize` answers at once and `status` shows `cloning`. | A9 |
| **Given** the Git login gives no usable email, **When** `status` runs, **Then** the `author` check is not ok with a `fix`, and reads still work. | A10 |
| **Given** `allowed_repo_hosts` without the host of `--repo`, **When** the server starts, **Then** it does not clone and reports `access_denied`. | A11 |

## Test Cases

- [x] [auto] A1: put a fake `git` that exits 1 first in `PATH`; start with `--repo`: `status` shows check `git` not ok with a `fix`, and the path does not exist (`tests/it_join_memory_repository.rs`)
- [x] [auto] A2: `World` remote with a knowledge base, `--repo file://<remote>` and `--path <new folder>`: after `status` the state is `ready` and the folder is a clone (`tests/it_join_memory_repository.rs`)
- [x] [auto] A3: `HOME` is a temporary folder, no `--path`, URL `.../team-knowledge.git`: the clone is in `$HOME/Projects/team-knowledge` (`tests/it_join_memory_repository.rs`)
- [x] [auto] A4: shared `access = "read-only"`, personal `read-write`: `settings.access` is `read-only` and `add` fails with `access_denied` (`tests/it_join_memory_repository.rs`)
- [x] [auto] A5: two servers with two `--repo` values and two paths: both are `ready`, and the files of the first clone do not change (`tests/it_join_memory_repository.rs`)
- [x] [auto] A6: a remote whose `settings.toml` has no `id`: the state is `broken` and `search` returns `setup_problem` (`tests/it_join_memory_repository.rs`)
- [x] [auto] A7: two processes start together on the same new path: the path holds one clone, no `.cloning-*` folder remains, and both answer `status` `ready` (`tests/it_join_memory_repository.rs`)
- [x] [auto] A8: an existing folder with a file; and a clone of another repository at the path: both give `setup_problem`, and the files are unchanged (`tests/it_join_memory_repository.rs`)
- [x] [auto] A9: a `git` wrapper that sleeps 3 seconds before the clone: `initialize` returns in under 1 second and `status` shows `cloning`, then `ready` (`tests/it_join_memory_repository.rs`)
- [x] [auto] A10: no author anywhere: `status` check `author` is not ok with a `fix`, and `search` works (`tests/it_join_memory_repository.rs`)
- [x] [auto] A11: personal `allowed_repo_hosts = ["github.com"]` and a `file://` URL: `access_denied`, and no folder is created (`tests/it_join_memory_repository.rs`)
- [x] [auto] A12: an empty remote (no commit): the state is `empty` and `search` returns `setup_problem` with the `init` hint (`tests/it_join_memory_repository.rs`)
- [x] [auto] A13: URL parsing of `https://`, `ssh://`, `git@host:path`, and `file://` forms (`src/join/url.rs`)
- [ ] [manual] M1: real Git host, new Mac user or clean path. Steps: install waymark with Homebrew. Add the entry with the SSH URL of a private test repository to Claude Code. Start a session, and call `status`. Expected: the clone is at `~/Projects/<name>`, `status` is ok, and the time from install to the first search is under 5 minutes.
- [ ] [manual] M2: a Mac without the command line tools (or a test user without them). Steps: start a session with the entry. Call `status`. Expected: the check `git` is not ok, the fix says to install the tools, and the Apple dialog does not block the agent tool.

## Notes

The fake `git` for A1 and A9 is a script in a temporary folder at the start of `PATH` that calls the real `git`.
