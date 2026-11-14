# Plan: Trust model

## Overview

A `config` module loads the limit layers, merges them with the rule "the strictest layer wins", and gives every tool one `EffectiveSettings` value. No tool reads a config file itself.

## Components Affected

- `mcp-server` — builds `EffectiveSettings` per call and puts a copy in every result as `settings`.
- `knowledge-base-repo` — owns the file `.mkbase/waymark.toml` that holds the shared layer. This module reads its `[limits]` and `[approval]` sections.

## New Files

- `src/config/mod.rs` — `Settings` loading and `EffectiveSettings`.
- `src/config/limits.rs` — the `Limits` struct, one field per limit, `Option` for "not set", and the `strictest` merge.
- `src/config/layers.rs` — load the system file, the personal file, the shared section, and the MCP entry (arguments and environment).
- `src/config/glob.rs` — folder glob matching (uses `globset`).
- `tests/it_trust_model.rs` — end-to-end tests with the real binary.

## Technical Approach

1. A `Limits` value holds every limit as `Option<T>`. `None` means the layer says nothing.
2. `strictest(a, b)`: `access` read-only wins. `paused` is true if either is true. `tools_denied` and `approval.required_for` are the union. `approval.via` takes `console` if either says so. `allowed_repo_hosts` and `folders` are the intersection of the layers that set them (an unset layer is ignored). Numeric caps take the smallest value.
3. Layers load in this order, and all are merged with `strictest`: system file, personal file, shared section (read after each sync), MCP entry. The server config layer is not built.
4. File locations: system `/Library/Application Support/waymark/config.toml`, personal `~/.config/waymark/config.toml` (or `$XDG_CONFIG_HOME/waymark/config.toml`).
5. `EffectiveSettings` is computed at every call, so a change of the pause flag in a file takes effect at once.
6. Built-in defaults apply for every limit that no layer sets.
7. `pause` is a field in the personal file. `pause-and-resume` owns the way the person sets it.
8. A syntax error in a layer file makes the call fail with `setup_problem` and the file path in `next`. A layer is never silently ignored, because that would loosen a limit.
9. Tool visibility: the registry filters tools by `tools_denied` and `access`.

## Dependencies on Other Foundation Items

- `knowledge-base-repo` — the shared file schema.
- `mcp-server` — calls this module. This module has no dependency on the MCP library.

## Risks

- A wrong merge loosens a limit. Mitigation: property tests for the merge (order does not matter, idempotent, never looser than any input).
- A broken file hides a limit. Mitigation: item 8.

## Open Questions

None.
