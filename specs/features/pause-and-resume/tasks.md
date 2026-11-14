# Tasks: Pause and resume

_`requires`: `mcp-server`, `write-documents` (both must be `implemented` before this spec moves to `in-progress`)._

- [ ] `src/config/pause.rs`: `WriteBlock`, `write_block`, and the texts. Add `mod pause;` to `src/config/mod.rs`.
- [ ] Unit tests in `src/config/pause.rs`: read-only before paused, paused alone, neither.
- [ ] `src/mcp/args.rs`: the `--paused` argument as a layer value.
- [ ] `src/mcp/error.rs`: map `WriteBlock` to `access_denied` and `paused`.
- [ ] `src/mcp/tool.rs`: hide write tools in the list while blocked. Check `write_block` first in a call of a write tool.
- [ ] `src/mcp/result.rs`: add the paused notice.
- [ ] `src/mcp/mod.rs`: advertise `tools.listChanged`. Send `notifications/tools/list_changed` after a call when the visible list changed.
- [ ] `src/gitsync/write.rs`: step 1 uses `write_block`.
- [ ] `tests/it_pause_and_resume.rs`: every test case of `test.md`.
- [ ] `README.md`: the section "Pause writes".
- [ ] Run the full suite.
