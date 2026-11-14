# Todo: Document format

## Bugs

None.

## Deferred

- **Align `mcp-server` with the YAML decision.** Its requirements say frontmatter uses `serde` with a maintained YAML crate. `document-format` uses `yaml-rust2` and no serde for frontmatter. Update that line when `mcp-server` is reviewed.
- ~~Check `yaml-rust2` release dates~~ Done on 2026-11-14 (see `decisions.md`).
- **Verify the Obsidian 1.4 floor at the first release.** Open `standard/conformance/vault/` in Obsidian 1.4. Check properties, list properties, the graph, and callouts. Check in the Obsidian changelog that 1.4 is the first version with the current property editor. Write the result in the release notes. Blocked by: the first release.
- **Align `quality-checks` with the broken-link severities.** Error for a link in a property, warning for a link in the body (see `decisions.md`).

## Open Questions

None.
