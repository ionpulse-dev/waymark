# Plan: Create a knowledge base

## Overview

The `init` tool is one file in `src/tools/`. It validates the input, then runs `gitsync::Repo::write`. Inside the write it checks that the repository is empty, writes the layout with `kb::init`, and commits and pushes. It uses the `kb` module for the layout and the settings, and `identity` for the author.

## Components Affected

- `knowledge-base-repo` (`kb`) — `kb::init` writes the layout. This spec adds `kb::repo_is_empty` to `src/kb/layout.rs` and a `Settings::to_toml` writer. The template `src/kb/templates/AGENTS.md` is replaced by the files of `plain-git-access` (see its plan).
- `git-sync` (`gitsync`) — `Repo::write` must handle a clone with no commit (no upstream, unborn branch, first push with `-u`). New verb `init` in `message.rs`.
- `mcp-server` — registers the tool. `Context` re-reads the state after `init`.
- `join-knowledge-base` — `Startup` gets a method `set_ready` that `init` calls after the push. It uses `repo_is_empty` for the state `empty`.
- `activity-and-undo` — `undo` refuses the verb `init`.
- `integration-testing` — `World::empty()`.

## New Files

- `src/tools/init.rs` — the `init` tool: schema, input validation, the write call, the result.
- `src/kb/init.rs` — `kb::init(root_repo, &InitInput, id) -> Vec<PathBuf>`: writes the files of the layout, refuses an existing file, returns the list. Also `derive_slug`.
- `tests/it_create_knowledge_base.rs` — end-to-end tests.

(`src/kb/layout.rs`, `src/kb/settings.rs`, and the templates exist in `knowledge-base-repo`. This spec changes `layout.rs` and `settings.rs`, as listed above.)

## Technical Approach

1. **Schema.** Hand-written JSON Schema (see `mcp-server`) with the fields of the requirements. The kind is `write`. The description: "Set up an empty repository as a knowledge base." Nothing more.
2. **Validation.** In `src/tools/init.rs`. `name` is trimmed and checked for length. `slug` is derived by `derive_slug` or checked against `^[a-z0-9]+(-[a-z0-9]+)*$` and 40 characters. `email_domains` entries are lowercased and checked for a dot and valid characters. `limits` and `approval` go through `kb::settings` (`serde` structs, unknown key is an error here, unlike a read of an existing file). `folders` entries are checked with the file-name rules of `format::id` for each segment, with no `..`, no leading `/`, and no empty segment. Each failure is `invalid_arguments`.
3. **State.** The tool waits for `Startup` like any tool. State `Ready` gives `validation_failed` "already a knowledge base". `NotAKnowledgeBase` is checked again inside the write. `Broken` returns its error. `Empty` goes on.
4. **Write.** `Repo::write(verb "init", title name, closure)`. The closure, after the fetch: (a) calls `kb::repo_is_empty` and returns `validation_failed` if not; (b) generates the UUID with the `uuid` crate (v4); (c) calls `kb::init`; (d) returns the paths to stage. `Repo::write` stages exact paths, commits `waymark: init "<name>"`, and pushes with `-u origin HEAD` if there is no upstream.
5. **Emptiness.** `kb::repo_is_empty` runs `git ls-files` (and `git rev-parse --verify HEAD` for "no commit"), plus a check of untracked files in the work tree: any file other than `.git` and the allowed names makes it not empty.
6. **No overwrite.** `kb::init` opens each file with `create_new(true)`. An existing file is an error, and the files written so far are removed. This is a second guard behind the emptiness check.
7. **Preview.** `Repo::write` with `preview` runs the closure with a flag that writes nothing and returns the planned list. `id` and `commit` are `null`.
8. **After.** On success, the tool calls `Startup::set_ready`, which finds the root and reads the new settings, so the next call in the session works. `settings` in the result is the effective limits after the write.
9. **Result.** `data` as in the requirements. `message`: "The knowledge base \"<name>\" is set up and pushed." For a preview: "This would set up the knowledge base \"<name>\"."
10. **Slug.** `derive_slug` lowercases, replaces each run of characters outside `a-z0-9` with `-`, trims `-`, cuts at 40 characters, and falls back to `knowledge-base`. Non-ASCII letters become `-` (accepted).

## Cross-Feature Dependencies

- `plain-git-access` — gives `kb::agents_file::render(audience, email_domains)`, which `kb::init` calls.
- `knowledge-base-repo` — the layout and the settings keys.
- `login-and-identity` — the author of the commit, checked before the write.
- `write-approval` — out of scope now. `Repo::write` step 6 already asks if configured.

## Risks

- The first push to a host with branch protection on an empty repository can fail. Mitigation: `sync_failed` with the output of Git. The person pushes the first commit by hand or changes the rule.
- A different default branch on the host and in the clone. Mitigation: push `HEAD` with `-u`, so the clone's branch is used.
- The uuid crate adds a dependency. Mitigation: it is small and widely used. Alternative: build a v4 UUID from `getrandom`.

## Open Questions

None.
