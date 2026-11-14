# Tests: Development environment

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:--|:--|
| New Mac, `make setup` installs the toolchain and `doctor` passes | Manual: run on a machine without Rust |
| `make setup` twice changes nothing | Setup idempotence test |
| Clean checkout builds and tests | Build and test run |
| Makefile parses with GNU Make 3.81 | Make parse test |
| `spec-check` fails on a missing file or a status mismatch | Spec-check tests |
| `make help` lists every target | Help test |
| `doctor` names a missing tool and its fix | Doctor test |

## Test Cases

- [x] [auto] `/usr/bin/make -n help build test lint check` parses and exits 0.
- [x] [auto] `make help` output contains every target name of the table in `requirements.md`.
- [x] [auto] `make setup` run twice: the second run reports nothing to install and exits 0.
- [x] [auto] `make doctor` exits 0 on this machine and prints the versions.
- [x] [auto] `scripts/doctor.sh` with `PATH` that has no `cargo` exits non-zero and names `cargo` and the fix.
- [x] [auto] `scripts/spec-check.sh` passes on the repository and fails on a temporary copy with one file removed, and on a copy with a changed status.
- [ ] [auto] `make build` and `make test` succeed on a clean `git clone` of the repository in a temporary folder.
- [x] [manual] `make setup` on a Mac that has no Rust. Setup: a clean user account or a fresh Mac with Xcode command line tools and Homebrew. Steps: clone the repository, run `make setup`, then `make check`. Expected: no error, `make doctor` passes, `make check` passes.

## Notes

The `[auto]` tests are shell tests in `scripts/tests/` run by `make test`.
