# Tasks: Trust model

`requires`: none. Needs `knowledge-base-repo` for the shared file schema (the types are defined in `src/kb`, see its plan).

- [ ] `src/config/limits.rs`: `Limits`, the defaults, and `strictest`.
- [ ] Unit and property tests for `strictest` (`tests/` and inline).
- [ ] `src/config/glob.rs`: folder glob matching and the intersection rule.
- [ ] `src/config/layers.rs`: load the system file, the personal file, and the MCP entry. Errors map to `setup_problem`.
- [ ] `src/config/mod.rs`: `EffectiveSettings` from all layers plus the shared section.
- [ ] Wire the shared layer after `knowledge-base-repo` provides its parser.
- [ ] `tests/it_trust_model.rs`: end-to-end tests of the acceptance criteria after `mcp-server` exists.
- [ ] Run the full suite.
