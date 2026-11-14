# Todo: Knowledge base repository

## Bugs

None.

## Deferred

- Per-folder settings in `settings.toml` (a later version).

## Open Questions

None.

- `.waymark/version` (the version of the document standard) is not read by any code. Add a check like the one for `schema` before the standard changes in a way that older builds cannot read (2026-11-14).
- Migrations and the tool `migrate`: build them with the first schema 2.
