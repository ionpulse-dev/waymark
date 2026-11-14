# Tasks: Verification

`requires`: `write-documents`.

- [ ] Confirm that `kb` exposes `verification` and that `format` renders and checks `trust`, `verified_by`, and `verified`.
- [ ] `src/tools/verify.rs`: state checks, fields, commit message, preview, and result.
- [ ] `src/tools/reject.rs`: state checks, fields, commit message with `Reason:` and `Trust:`, preview, and result.
- [ ] `src/tools/review.rs`: filter, sort, `limit`, `total`, and the `folders` limit.
- [ ] `src/tools/mod.rs`: register `verify`, `review`, and `reject`.
- [ ] `tests/it_verification.rs`: the cases of `test.md`.
- [ ] Run the full suite.
