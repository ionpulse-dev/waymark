# Tests: Trust model

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:--|:--|
| Commit author is the person | `commits_use_the_author_of_waymark_and_the_fixed_message` (`tests/it_git_sync.rs`) |
| Shared `read-only` beats personal `read-write` | `shared_read_only_wins` |
| `max_results` 5 and 3 gives 3 | `numeric_caps_take_smallest` |
| No tool changes a limit | `no_control_tool_listed` |
| `tools_denied = ["delete_document"]` hides `delete_document` and blocks a call | `delete_hidden_when_denied` (`tests/it_write_documents.rs`) |
| System file `read-only` beats personal | `system_file_wins` |
| `paused = true` gives `paused` | `paused_blocks_writes` (`tests/it_pause_and_resume.rs`) |
| No layer gives built-in defaults | `defaults_apply` |

## Test Cases

- [x] [auto] `strictest` is commutative and idempotent, and never looser than an input (property test, `src/config/limits.rs`)
- [x] [auto] Each limit merges as the requirements say: access, paused, tools_denied, approval, hosts, folders, caps (`src/config/limits.rs`)
- [x] [auto] Folder globs: `engineering/**` allows `engineering/a/b`, blocks `ops/x` (`src/config/glob.rs`)
- [x] [auto] A syntax error in a layer file gives `setup_problem` with the path (`src/config/layers.rs`)
- [x] [auto] The effective settings change when the personal file changes between two calls (`tests/it_trust_model.rs`)
- [x] [auto] All eight criteria above, through the real binary (`tests/it_trust_model.rs` and the tests named in the table)
- [x] [auto] The `settings` field of every result shows the effective limits (`tests/it_trust_model.rs`)

## Notes

Tests set `HOME` and `XDG_CONFIG_HOME` to a temporary folder. The system file path is overridden by the environment variable `WAYMARK_SYSTEM_CONFIG` in tests only. The binary honors this variable in release builds too, and the documentation says it is for tests (decision in `decisions.md`).
