# Tests: Terminology

_`[auto]` items are verified by running the automated test suite or a script. `[manual]` items require human verification._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| README calls the thing "memory repository"; "knowledge base" only as a use case | README read-through |
| Every "knowledge base" hit is a use case, another tool, or a dated record | final search |
| A new memory repository has the `-agent-memory` folder and the new skill names | `tests/it_create_memory_repository.rs` |
| No tool description or message names the thing "knowledge base" | tool list and `status` test |
| The full suite passes with no change of behavior | full test suite |
| The conventions hold the three terms and the "repository alone" rule | conventions check |

## Test Cases

- [x] [auto] `init` writes the marker line with `personal-agent-memory` or
      `team-agent-memory`, and the default clone folder ends in `-agent-memory`.
- [x] [auto] The conformance fixtures pass against the renamed standard
      (marker folder `.waymark/`).
- [x] [auto] The full test suite passes.
- [x] [auto] The `status` result and the "needs a newer waymark" message use
      "memory repository".
- [x] [auto] `AGENTS.md` contains "memory repository" and the rule that "repository"
      alone means the code repository.
- [ ] [manual] Read the README from top to bottom. Steps: open `README.md`. Expected:
      the first sentence is the tagline; "What you can build with it" lists the
      team knowledge base first; no other place calls the thing a knowledge base;
      no sentence reads wrongly (for example "memory repository of the memory
      repository").
- [ ] [manual] Start a session in a test project and read the agent's first
      messages. Expected: the agent calls the thing "memory repository" or
      "memory".

## Notes

Checked on 2026-11-14: the full suite passes (586 tests). The `status` text, the "needs a newer waymark" message, and the rules in `AGENTS.md` were checked by search, not by a new test, because the spec asks for no automated guard against the old phrase.

The final search is manual: `grep -ri "knowledge base"`, then read each hit.
The manual items exist because a script cannot judge whether a sentence reads
well.
