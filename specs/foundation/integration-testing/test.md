# Tests: Integration testing

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:--|:--|
| A `World` has a remote and two clones with different authors and no access to the real home | `world_has_remote_and_two_clones`, `world_isolates_environment` |
| `tools/list` returns the tool list of the real binary | `client_lists_tools` |
| Two tests in parallel do not interfere | `worlds_run_in_parallel` |
| A failing test keeps its folder and prints the path | `failed_test_keeps_folder` |
| `make test-integration` builds the binary and runs all tests | `make_target_runs_tests` |
| The real Git config has no effect | `world_isolates_environment` |

## Test Cases

- [x] [auto] A world has a bare remote and two clones with different authors (`tests/it_harness.rs`)
- [x] [auto] A commit in clone A pushed, then pulled in clone B shows author A (`tests/it_harness.rs`)
- [x] [auto] The world ignores `~/.gitconfig` and the real `HOME` (`tests/it_harness.rs`)
- [x] [auto] The client starts the stub binary, initializes, and lists tools (`tests/it_harness.rs`)
- [ ] [auto] The client times out with a clear message when the server does not answer (`tests/it_harness.rs`)
- [x] [auto] Eight worlds run in parallel with no interference (`tests/it_harness.rs`)
- [x] [auto] A panic inside a world keeps the folder (`tests/it_harness.rs`)
- [x] [auto] `make test-integration` exits 0 (`scripts/tests/`)

## Notes

Until `mcp-server` exists, the tests use a stub binary that answers `initialize` and `tools/list`.
