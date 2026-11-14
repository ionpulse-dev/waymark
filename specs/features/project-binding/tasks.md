# Tasks: Project binding

_`requires`: `mcp-server`._

- [ ] `src/mcp/repo_url.rs`: `host`, `slug`, `same_remote`, `redact`, with unit tests for every example of `requirements.md` and `ssh://`, `https://` with a user and a password, `file://`, and plain paths.
- [ ] `src/kb/layout.rs`: `find_root_in_or_above` (if missing), with unit tests.
- [ ] `src/gitsync/mod.rs`: `clone_to` and `origin_url` (if missing).
- [ ] `src/mcp/binding.rs`: `Binding`, `FoundBy`, `Problem`, and `resolve` for the four location rules.
- [ ] `src/mcp/binding.rs`: the folder rules (worktree, origin match), the host check, the clone with the temporary folder, and the identity check.
- [ ] `src/mcp/context.rs`: the `binding` field. `src/main.rs`: call `resolve` before serving.
- [ ] `src/mcp/tool.rs`: tools declare if they need a knowledge base. A missing binding gives the `Problem` as the error. `status` and `init` run without a root.
- [ ] Repeat the identity check when the shared file is read after a pull.
- [ ] `tests/it_project_binding.rs`: every test case of `test.md`.
- [ ] README: the section "Project and default entry".
- [ ] Run the full suite.
