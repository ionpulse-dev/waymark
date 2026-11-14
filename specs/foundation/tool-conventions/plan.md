# Plan: Tool conventions

## Overview

One table of renames in `src/config/limits.rs` serves the registry (alias calls) and the limits (alias names). Each tool file gets the new name, a description in the common form, and a description for every parameter. Tests walk the tool list and check the rules.

## Components Affected

- `src/config/limits.rs`: `RENAMED_TOOLS`, `canonical_tool(name)`. `Effective::from_layers` maps the names in `tools_denied` and `approval_required_for`.
- `src/mcp/tool.rs`: `Registry::call` resolves an old name and adds the notice. `list` shows new names only.
- `src/tools/*.rs`: new names and descriptions. Every `writes::execute(ctx, env, "<tool>", ...)` and every check by name uses the new name.
- `src/tools/read_common.rs`: `parse_when` (one parser for `since`), shared by `updates` and `activity`.
- `src/mcp/args.rs`: the help text of `--no-delete` and `--tools-denied`.
- `src/memrepo/templates/settings.toml`, `src/mcp/instructions.rs`, `src/maintenance/sources.rs`, `src/mcp/approval_text.rs`: names in text.
- `tests/*.rs`: calls use the new names.
- Docs: `README.md`, `docs/install.md`, `docs/agent-tools-research.md`, `CHANGELOG.md`, and all specs that name a tool.

## New Files

- `tests/it_tool_conventions.rs`: the checks of the rules over the tool list, the aliases, and the dates.

## Technical Approach

1. **Table.** `RENAMED_TOOLS: [(&str, &str); 9]` holds (old, new). `canonical_tool` returns the new name for an old one, and the name itself otherwise.
2. **Registry.** `call` looks up the tool by `canonical_tool(name)`. When the name was old, the result gets the notice `The tool "<old>" is now "<new>". The old name stops working in 0.5.0.`. A call with an old name is not an unknown tool.
3. **Limits.** `from_layers` maps each name of `tools_denied` and `approval_required_for` with `canonical_tool`. The `write` keyword and other words stay as they are. The `settings` field of a result shows new names.
4. **Descriptions.** Written by hand, one per tool, in the form "what it does. Returns ... Commits and pushes. Changes nothing." Every `properties` entry gets a `description`.
5. **Dates.** `read_common::parse_when(name, text)` parses a date or an RFC 3339 time and refuses a time in the future. `updates` uses it. `activity` uses it and compares with the parsed commit time.
6. **Rename in code and tests.** A script replaces the old names in quoted and backticked forms. The compiler and the tests find what is left. The commit verbs (`add`, `delete`, ...) do not change.
7. **Budget.** The two budget tests get the new limits.

## Cross-Feature Dependencies

- `search-filters`, `restore-and-replace`, `document-history`, and `manage-tags` add tools that follow these rules from the start.

## Risks

- A limit that names an old tool must still work. The test `an_old_name_in_a_limit_still_blocks_the_new_tool` covers it.
- A blind rename also hits words that are not tool names (`folders` is also a parameter and a limit). The rename is done by hand for `folders`, and reviewed.

## Open Questions

None.
