# Tests: Verification

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| Verification off: the three tools are not listed | `tools_hidden_when_off` |
| Verification on: `add_document` sets `trust: unverified` | `add_sets_unverified` |
| `verify_document` sets the fields and pushes one commit | `verify_sets_fields` |
| A body change resets trust | `update_resets_trust` |
| `verify_document` on a disputed document fails | `verify_disputed_fails` |
| `reject_document` keeps the file as deprecated | `reject_keeps_file` |
| Approval is asked for `verify_document` | `verify_asks_approval` |
| `review` lists disputed first with the reason | `review_order_and_reason` |
| A direct call with verification off gives `access_denied` | `direct_call_when_off` |

## Test Cases

- [x] [auto] All criteria above, through the real binary and the `World` harness (`tests/it_verification.rs`). The approval case is `#[ignore]` until `write-approval`.
- [x] [auto] The `verify_document` commit is `waymark: update "<title>"` with the body line `Trust: verified` (`tests/it_verification.rs`)
- [x] [auto] The `reject_document` commit is `waymark: deprecate "<title>"` with `Reason:` and `Trust: rejected`, and `verified_by` and `verified` are gone (`tests/it_verification.rs`)
- [x] [auto] `verify_document` twice, on a deprecated document, and with an old `base` each fail with the right code (`tests/it_verification.rs`)
- [x] [auto] A document without `trust` shows in `review` as unverified (`tests/it_verification.rs`)
- [x] [auto] `review` respects `folder`, `limit`, `total`, and the `folders` limit (`tests/it_verification.rs`)
- [x] [auto] `dispute_document` and `deprecate_document` keep a verified mark (`tests/it_verification.rs`)
- [x] [auto] With verification turned off again, trust fields stay in the files and `add_document` stops setting `trust` (`tests/it_verification.rs`)
- [x] [auto] `undo` of a `verify_document` restores the document (`tests/it_verification.rs`)
- [ ] [manual] Obsidian shows the trust fields. Steps: turn verification on in `.waymark/settings.toml`, run `verify_document` through a real agent tool, open the document in Obsidian. Expected: the Properties panel shows `trust`, `verified_by`, and `verified` without errors.

## Notes

The approval case needs `write-approval` and is `#[ignore]` until that spec is implemented. The tests turn verification on by committing `verification = true` to `.waymark/settings.toml` in the `World`.
