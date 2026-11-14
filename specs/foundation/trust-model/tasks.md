# Tasks: Trust model

`requires`: none. Needs `memory-repository` for the shared file schema (the types are defined in `src/kb`, see its plan).

- [x] `src/config/limits.rs`: `Limits`, the defaults, and `strictest`.
- [x] Unit and property tests for `strictest` (`tests/` and inline).
- [x] `src/config/glob.rs`: folder glob matching and the intersection rule.
- [x] `src/config/layers.rs`: load the system file, the personal file, and the MCP entry. Errors map to `setup_problem`.
- [x] `src/config/mod.rs`: `EffectiveSettings` from all layers plus the shared section.
- [x] Wire the shared layer after `memory-repository` provides its parser.
- [x] `tests/it_trust_model.rs`: end-to-end tests of the acceptance criteria after `mcp-server` exists.
- [x] Run the full suite.
