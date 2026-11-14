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

## Test Cases

- [ ] [auto] A1: `World::empty()`, a server on clone A, `init` with a name, folders, and `email_domains`: the remote has all files of the layout, one commit `waymark: init "Example"` by the author, `data.files` matches, `.mkbase/version` is `1.0\n`, and `waymark.toml` holds a valid UUID (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A2: a second server on clone B with only `--repo`: `status` is `ready` and `settings` show the limits that `init` stored (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A3: `init` twice: the second returns `validation_failed` with the "already a knowledge base" message, and the remote head did not move (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A4: a remote that holds `src/main.rs`: `validation_failed`, and the head did not move (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A5: a remote that holds an `AGENTS.md` with custom text: `validation_failed`, and the file is unchanged (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A6: a remote that holds only `README.md` and `LICENSE`: `init` works and both files are byte-equal to before (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A7: the files `.obsidian/app.json` (both keys) and `.gitignore` (both lines) have the exact content; after a file `.obsidian/workspace.json` is created in the clone, `git status --porcelain` does not list it (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A8: `access = "read-only"` in the personal config: `access_denied`, and with `tools_denied = ["init"]` the tool is not in `tools/list`. With `paused = true`: `paused` (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A9: `preview: true`: no file in the clone, no commit, the result has `commit: null` and `pushed: false` (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A10: after `init`, `add` a document and `search` it in the same session (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A11: bad slug, a folder `../x`, a folder with an uppercase letter, an unknown key in `limits`, a blank name: each is `invalid_arguments` with the field in `next` (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A12: two servers (clones A and B of one empty remote) call `init` at once: one succeeds, the other returns `validation_failed`, and `git log` shows one `init` commit (`tests/it_create_knowledge_base.rs`)
- [ ] [auto] A13: `derive_slug` for "Example Company", "  Ünï Corp ", "!!!" (`src/kb/init.rs`)
- [ ] [auto] A14: the `AGENTS.md` written for `personal` and for `team` carries the pinned skill version (`tests/it_create_knowledge_base.rs`; see `plain-git-access`)
- [ ] [auto] A15: the generated `AGENTS.md`, `.gitignore`, and `waymark.toml` pass `kb::settings` and the document checks (the root has no document, so `lint` finds nothing) (`tests/it_create_knowledge_base.rs`)
- [ ] [manual] M1: Obsidian. Steps: clone the repository that `init` made. Open the folder as a vault in Obsidian. Create a link between two documents through the link dialog. Expected: the vault opens, the link is a relative Markdown link with `.md`, and `git status` does not list `.obsidian/workspace.json`.
- [ ] [manual] M2: real Git host. Steps: create an empty private repository on a real host. Add the entry to an agent tool with the SSH URL. Start a session, call `status` (state `empty`), then ask the agent to call `init`. Expected: the files appear on the host in one commit, and a second person with only the URL can search.

## Notes

All `[auto]` tests use `file://` remotes of the harness. No real Git host is needed.
