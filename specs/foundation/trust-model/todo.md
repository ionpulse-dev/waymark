# Todo: Trust model

## Bugs

None.

## Deferred

- The server config layer, roles, and separate connections (server mode).
- Per-folder access levels.
- Research: which agent tools support a managed MCP config, and is the system file enough for IT? (Unverified, see `docs/agent-tools-research.md`.)

## Open Questions

None.

## Notes from the implementation (2026-11-14)

- Two criteria need a write tool and are tested end to end in `write-documents`: "the commit author is the person" and "a denied or paused tool blocks a write". The registry unit tests in `src/mcp/tool.rs` cover the blocking rule with fake tools. The author rule is tested in `tests/it_git_sync.rs`.
