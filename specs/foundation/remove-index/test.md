# Tests: Remove Index

_`[auto]` items are verified by running the automated test suite or a script. `[manual]` items require human verification._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| `index` is not listed, with any `access` setting | tool list test |
| A call of `index` fails as an unknown tool | unknown-tool test |
| `index.md` with the tag `index` is a normal document | normal-document test |
| `delete` refuses a linked `index.md` | delete test |
| No `index-stale` result | lint test |
| Only dated records and this spec mention the tool | final search |
| The full suite passes | full test suite |

## Test Cases

- [x] [auto] `tools/list` has no `index` with `read-write` and with `read-only`.
- [x] [auto] A call of `index` returns an unknown-tool error.
- [x] [auto] A document `ops/index.md` with the tag `index` appears in `search`, `list`, `stats`, and `updates`.
- [x] [auto] `delete` of a document that `index.md` links to refuses and names `index`.
- [x] [auto] `lint` never returns `index-stale`.
- [x] [auto] A search of the project for the `index` tool finds only dated records and this spec.
- [x] [auto] The full test suite passes.
