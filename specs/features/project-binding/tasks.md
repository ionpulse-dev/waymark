# Tasks: Project binding

_`requires`: `mcp-server`._

- [x] `src/mcp/repo_url.rs`: `host`, `slug`, `same_remote`, `redact`, with unit tests for every example of `requirements.md` and `ssh://`, `https://` with a user and a password, `file://`, and plain paths.
- [x] No `find_root_in_or_above`: `binding::locate` walked up with its own marker check (see `decisions.md`). Removed by the rework below.
- [x] `binding::clone_checked` (temporary folder) and `binding::origin_url` use the existing `Repo::clone_from`.
- [x] `src/mcp/binding.rs`: `Binding`, `FoundBy`, `Problem`, and `resolve` for the four location rules.
- [x] `src/mcp/binding.rs`: the folder rules (worktree, origin match), the host check, the clone with the temporary folder, and the identity check.
- [x] `src/mcp/context.rs`: the `binding` field. `src/main.rs`: call `resolve` before serving.
- [x] `src/mcp/tool.rs`: tools declare if they need a memory repository. A missing binding gives the `Problem` as the error. `status` and `init` run without a root.
- [x] Repeat the identity check when the shared file is read after a pull.
- [x] `tests/it_project_binding.rs`: every test case of `test.md`.
- [x] README: the section "Project and default entry".
- [x] Run the full suite.

## Rework 2026-11-14: where the memory repository lives

_Prerequisite: the changed specs are approved._

- [x] `src/mcp/repo_url.rs`: replace `slug` by `name`, and make `same_remote` compare `host/path` (SSH and HTTPS forms). Unit tests.
- [x] `src/mcp/binding.rs`: remove the working-folder search, the scan of the home folder, and `default_clone_dir` by slug. Add `~/.waymark/<name>`, the `~` expansion, the empty-folder rule, and the new error sentences. `FoundBy` keeps `PathArgument` and `RepoArgument`.
- [x] `src/mcp/context.rs`, `src/mcp/session.rs`, `src/mcp/instructions.rs`: use the two `FoundBy` values and the new `locate`. `status` shows the folder in use.
- [x] `src/mcp/args.rs`: the help text and the doc comment of `--path` and `--repo`.
- [x] `src/memrepo/templates/settings.toml`: the comment of `slug`.
- [x] `tests/it_project_binding.rs`: one test per row of the table, and the other cases of `test.md`.
- [x] `tests/it_join_memory_repository.rs`: the clone-folder tests for the new rules.
- [x] Update the harness, `README.md`, `docs/install.md`, `CHANGELOG.md`, and the old path text in `docs/`.
- [x] Run `make check`.
