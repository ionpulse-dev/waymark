# Tests: Session start

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_The tests use the harness of `integration-testing`. The `McpClient` returns the `instructions` string of the `initialize` result._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** a bound project, **When** a client connects, **Then** the instructions name the memory repository and carry the effective limits | `instructions_name_kb_and_limits` |
| **Given** the instructions, **When** they are read, **Then** they hold no advice | `instructions_have_no_advice` |
| **Given** writes are paused, **When** a client connects, **Then** the instructions and a notice say so | `paused_in_instructions_and_notice` |
| **Given** an agent that ignores the instructions, **When** it runs a write, **Then** the limits still apply | `limits_hold_without_instructions` |
| **Given** a memory repository that is already cloned, **When** a client connects, **Then** the instructions never say that a clone is running | A14 of `join-memory-repository` |
| **Given** no memory repository is set up, **When** a client connects, **Then** the instructions say so and `status` shows the fix | `unbound_instructions_and_status` |
| **Given** a limit changes during a session, **When** any tool runs, **Then** `settings` shows the new value | `settings_follow_changes` |
| **Given** a remote that is not reachable, **When** a client connects, **Then** `initialize` answers at once | `initialize_needs_no_network` |
| **Given** default limits, **When** the instructions are built, **Then** they have fewer than 900 characters, and never more than 1200 | `instructions_stay_short` |
| **Given** `status`, **When** it runs, **Then** its facts equal the facts in the instructions | `status_matches_instructions` |

## Test Cases

- [x] [auto] The instructions have the memory repository name, the ID, and the numbers of the limits from a shared file with `max_results = 3` and `folders = ["backend/*"]` (`tests/it_session_start.rs`)
- [x] [auto] The instructions contain none of `should`, `must`, `always`, `never`, `please`, `prefer`, `you`, in a table of ten setting combinations (inline in `src/mcp/instructions.rs`, and once end to end in `tests/it_session_start.rs`)
- [x] [auto] With `paused = true`, the instructions say `writes paused: yes` and the first notice of `status` is "Writes are paused." (`tests/it_session_start.rs`)
- [x] [auto] With `access = "read-only"` and an agent that never calls `status`, `add` gives `access_denied` (`tests/it_session_start.rs`)
- [x] [auto] With no memory repository, the instructions have the second form with the `message` of the problem, and `status` returns `memory_repository: null`, a `problem` with `next`, and no error flag (`tests/it_session_start.rs`)
- [x] [auto] The test changes `max_results` in the personal file after `initialize`. The `settings` of the next result show the new value, and the instructions do not change (`tests/it_session_start.rs`)
- [x] [auto] The remote is deleted after the clone. `initialize` answers in under 2 seconds (`tests/it_session_start.rs`)
- [x] [auto] Length: defaults give fewer than 900 characters. A name of 500 characters, a notice list of 10 entries, and a long `folders` list give at most 1200 (inline in `src/mcp/instructions.rs`)
- [x] [auto] A memory repository name with a line break and a fake line "Notices: none" does not add a line (`tests/it_session_start.rs`)
- [x] [auto] The facts of `status` (name, ID, access, paused, caps) equal the values in the instructions (`tests/it_session_start.rs`)
- [x] [auto] `status` has `found_by` and a `remote` without a user or a password (`tests/it_session_start.rs`)
- [x] [auto] The `settings` object has exactly the keys and the order of `requirements.md`, in a read result and in an error result (`tests/it_session_start.rs`)
- [x] [auto] A repository with no remote gives the notice "This repository has no remote." in the instructions and in `status` (`tests/it_session_start.rs`)
- [ ] [manual] M1, Claude Code uses the instructions. Steps: (1) Start Claude Code in a project with the MCP entry. (2) Without any tool call, ask: "Which memory repository does waymark use, and what are its limits?" Expected: the agent answers with the name and the limits, or says it needs to call `status`. Record which.
- [ ] [manual] M2, GitHub Copilot in VS Code: as M1. Record the result.
- [ ] [manual] M3, OpenAI Codex CLI: as M1. Record the result.

## Notes

The manual tests answer which agent tools show or use server instructions. The result goes into the README. The server does not depend on the answer, because `status` and `settings` in every result carry the same facts.

The test `limits_hold_without_instructions` calls `add` from `write-documents`. This spec does not require that spec, so the test ran as a normal test, because `write-documents` is `implemented`. The task "Run the full suite" runs ignored tests if the tool exists.
