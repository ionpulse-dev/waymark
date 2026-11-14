# Tasks: Pause and resume

_`requires`: `mcp-server`, `write-documents` (both must be `implemented` before this spec moves to `in-progress`)._

- [x] `src/config/pause.rs`: `WriteBlock`, `write_block`, and the texts. Add `mod pause;` to `src/config/mod.rs`.
- [x] Unit tests in `src/config/pause.rs`: read-only before paused, paused alone, neither.
- [x] `src/mcp/args.rs`: the `--paused` argument as a layer value.
- [x] `src/mcp/error.rs`: map `WriteBlock` to `access_denied` and `paused`.
- [x] `src/mcp/tool.rs`: hide write tools in the list while blocked. Check `write_block` first in a call of a write tool.
- [x] `src/mcp/result.rs`: add the paused notice.
- [x] `src/mcp/mod.rs`: advertise `tools.listChanged`. Send `notifications/tools/list_changed` after a call when the visible list changed.
- [x] `src/gitsync/write.rs`: step 1 uses `write_block`.
- [x] `tests/it_pause_and_resume.rs`: every test case of `test.md`.
- [x] `README.md`: the section "Pause writes".
- [x] Run the full suite.
