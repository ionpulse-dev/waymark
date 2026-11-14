# Tasks: Maintenance

`requires`: `login-and-identity`, `join-knowledge-base`, `agent-tool-setup`.

- [ ] `src/gitsync/probe.rs`: `probe_remote` with the time-out and the classification of the output. Add `last_pull` and `last_error` to `Repo`.
- [ ] `config::layers`: make the per-layer loaders public.
- [ ] `src/maintenance/checks.rs`: `git`, `clone`, `working_tree`.
- [ ] `src/maintenance/checks.rs`: `remote_and_login`, `kb_id`, `settings`, `author`.
- [ ] `src/maintenance/sources.rs`: `limit_sources`.
- [ ] `src/maintenance/mod.rs`: `run_checks` with the order and the skip rules.
- [ ] `src/tools/status.rs`: build the result with `healthy`, `checks`, `author`, `limit_sources`, `last_pull`, `last_error`, and the notices.
- [ ] Registry: list `status` always, also when `tools_denied` names it.
- [ ] `docs/install.md`: the section "Remove waymark".
- [ ] `tests/it_maintenance.rs`: the acceptance criteria.
- [ ] Run the full suite.
