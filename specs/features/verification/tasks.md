# Tasks: Verification

`requires`: `write-documents`.

- [x] Confirm that `memrepo` exposes `verification` and that `format` renders and checks `trust`, `verified_by`, and `verified`.
- [x] `src/tools/verify.rs`: state checks, fields, commit message, preview, and result.
- [x] `src/tools/reject.rs`: state checks, fields, commit message with `Reason:` and `Trust:`, preview, and result.
- [x] `src/tools/review.rs`: filter, sort, `limit`, `total`, and the `folders` limit.
- [x] `src/tools/mod.rs`: register `verify`, `review`, and `reject`.
- [x] `tests/it_verification.rs`: the cases of `test.md`.
- [x] Run the full suite.
