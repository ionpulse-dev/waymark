# Todo: Verification

## Bugs

- [ ] None known.

## Deferred

- [ ] Server mode: `verify_document` and `reject_document` only for the role `owner`, and the answer in the console (see `server-mode` and `web-console`). Out of scope for now.
- [ ] A folder owner list: who verifies a folder. Today any role `owner` would do.
- [ ] A bulk verify.
- [ ] A rule that blocks a person from verifying their own document.
- [ ] Show `trust` in `search` and `list` results (belongs to `search-and-read`).

## Open Questions

- None for the product owner.

## Done in the implementation (2026-11-14)

- All `[auto]` cases pass in `tests/it_verification.rs`. The approval case (`verify_asks_approval`) waits for `write-approval` (`#[ignore]`).
- The `[manual]` Obsidian check is for the product owner.
