# Tests: Pause and resume

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_The tests use the harness of `integration-testing`. The personal config file is `$XDG_CONFIG_HOME/waymark/config.toml` in the config home of the `World`. The system file is set with `WAYMARK_SYSTEM_CONFIG`._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** `paused = true`, **When** an agent calls a write tool, **Then** it fails with `paused` and writes nothing | `paused_blocks_writes` |
| **Given** `paused = true`, **When** an agent reads, **Then** it works | `paused_allows_reads` |
| **Given** `paused = true` in the system config, **When** the person sets `false` in the personal config, **Then** writes still fail | `lower_layer_cannot_resume` |
| **Given** paused, **When** a day passes, **Then** it is still paused | `pause_has_no_timer` |
| **Given** paused, **When** `status` runs, **Then** it names the state | `status_names_pause` |
| **Given** an agent, **When** it lists tools, **Then** no tool resumes writes | `no_resume_tool` |
| **Given** a running session, **When** the person sets `paused = true` and then `false`, **Then** the next write fails and the one after the resume works | `pause_and_resume_live` |
| **Given** a write that waits for approval, **When** a pause is set before the answer, **Then** it fails with `paused` | `pause_during_approval` (in `tests/it_write_approval.rs`) |
| **Given** `access = "read-only"` and `paused = true`, **When** a write runs, **Then** it fails with `access_denied` | `read_only_wins_over_paused` |
| **Given** paused, **When** the client lists tools, **Then** the write tools are missing and a `list_changed` arrived | `paused_hides_write_tools` |
| **Given** a broken personal config file, **When** any tool runs, **Then** `setup_problem` names the file | `broken_file_is_setup_problem` |

## Test Cases

- [x] [auto] A paused write gives code `paused`, a `next` sentence, and no new commit in the clone or the remote (`tests/it_pause_and_resume.rs`)
- [x] [auto] `status` and a read tool work while paused (`tests/it_pause_and_resume.rs`)
- [x] [auto] System file `paused = true` and personal `paused = false`: a write fails (`tests/it_pause_and_resume.rs`)
- [x] [auto] The config file has a modification time two days old, and a write still fails. Two calls with a pause between give the same answer (`tests/it_pause_and_resume.rs`)
- [x] [auto] `status` has `settings.paused = true` and the notice "Writes are paused." (`tests/it_pause_and_resume.rs`)
- [x] [auto] `tools/list` has no tool with a name that contains `pause`, `resume`, or `limit` (`tests/it_pause_and_resume.rs`)
- [x] [auto] One server process: write works, the test writes `paused = true`, the write fails, the test writes `paused = false`, the write works (`tests/it_pause_and_resume.rs`)
- [x] [auto] `access = "read-only"` in the shared file and `paused = true`: a write gives `access_denied` (`tests/it_pause_and_resume.rs`)
- [x] [auto] While paused, `tools/list` has no write tool, and the client received `notifications/tools/list_changed` once. After the resume, it received it again (`tests/it_pause_and_resume.rs`)
- [x] [auto] A direct call of a hidden write tool while paused gives `paused`, not "unknown tool" (`tests/it_pause_and_resume.rs`)
- [x] [auto] A syntax error in the personal file gives `setup_problem` with the path, for a read and for a write (`tests/it_pause_and_resume.rs`)
- [x] [auto] Every result while paused has the notice "Writes are paused." as the first notice (`tests/it_pause_and_resume.rs`)
- [x] [auto] `--paused` in the arguments pauses from the start (`tests/it_pause_and_resume.rs`)
- [x] [auto] Person B pushes a document while A is paused. `Repo::refresh_if_stale` on the clone of A brings it in (`tests/it_pause_and_resume.rs`)
- [x] [auto] Unit tests of the pause rules: the existing tests in `src/config/limits.rs` (`paused_if_any_layer_says_so`, check order) replace `write_block`
- [ ] [manual] The kill switch with a real agent tool. Steps: (1) Add the MCP entry to Claude Code and start a session. (2) Ask the agent to add a test document. It works. (3) In a terminal, add `paused = true` under `[limits]` in `~/.config/waymark/config.toml`. (4) Ask the agent to add another document. Expected: the agent reports that writes are paused, and no commit appears in the clone. (5) Set `paused = false` and ask again. Expected: the write works.
- [ ] [manual] The README steps work as written, on a Mac with no config file and on a Mac with a config file that has `[limits]`. Expected: no `setup_problem`.

## Notes

The last `[auto]` item for pulls uses the library, because the read tools belong to `search-and-read`, which this spec does not require.
