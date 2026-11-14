# Tasks: Project binding

_`requires`: `mcp-server`._

- [x] `src/mcp/repo_url.rs`: `host`, `slug`, `same_remote`, `redact`, with unit tests for every example of `requirements.md` and `ssh://`, `https://` with a user and a password, `file://`, and plain paths.
- [x] No `find_root_in_or_above`: `binding::locate` walks up with its own marker check (see `decisions.md`).
- [x] `binding::clone_checked` (temporary folder) and `binding::origin_url` use the existing `Repo::clone_from`.
- [x] `src/mcp/binding.rs`: `Binding`, `FoundBy`, `Problem`, and `resolve` for the four location rules.
- [x] `src/mcp/binding.rs`: the folder rules (worktree, origin match), the host check, the clone with the temporary folder, and the identity check.
- [x] `src/mcp/context.rs`: the `binding` field. `src/main.rs`: call `resolve` before serving.
- [x] `src/mcp/tool.rs`: tools declare if they need a knowledge base. A missing binding gives the `Problem` as the error. `status` and `init` run without a root.
- [x] Repeat the identity check when the shared file is read after a pull.
- [x] `tests/it_project_binding.rs`: every test case of `test.md`.
- [x] README: the section "Project and default entry".
- [x] Run the full suite.
