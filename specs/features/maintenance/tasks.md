# Tasks: Maintenance

`requires`: `login-and-identity`, `join-knowledge-base`, `agent-tool-setup`.

- [x] `src/gitsync/probe.rs`: `probe_remote` with the time-out and the classification of the output. Add `last_pull` and `last_error` to `Repo`.
- [x] `config::layers`: make the per-layer loaders public.
- [x] `src/maintenance/checks.rs`: `git`, `clone`, `working_tree`.
- [x] `src/maintenance/checks.rs`: `remote_and_login`, `kb_id`, `settings`, `author`.
- [x] `src/maintenance/sources.rs`: `limit_sources`.
- [x] `src/maintenance/mod.rs`: `run_checks` with the order and the skip rules.
- [x] `src/tools/status.rs`: build the result with `healthy`, `checks`, `author`, `limit_sources`, `last_pull`, `last_error`, and the notices.
- [x] Registry: list `status` always, also when `tools_denied` names it.
- [x] `docs/install.md`: the section "Remove waymark".
- [x] `tests/it_maintenance.rs`: the acceptance criteria.
- [x] Run the full suite.
