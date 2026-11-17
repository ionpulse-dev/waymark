# Tests: Linux support

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_AI checks off `[auto]` items when tests pass. A human checks off `[manual]` items. The foundation item moves to `done` only when all items are checked._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| Clean Linux x86_64 machine: `make test-all` passes | CI Linux job (x86_64) |
| macOS Apple silicon: the same command passes | `make test-all` on the host |
| Pull request: CI runs Linux and macOS, both must pass | CI matrix check (workflow lint) |
| Mac with runtime: `make linux-test` builds the container and passes | `make linux-test` on the Mac |
| Same Mac: `make linux-check` runs lint, test, spec checks | `make linux-check` on the Mac |
| Attached to the container: `make build`, `make test-all`, `make check` work | `make linux-shell` run |
| After a container run, no file is owned by `root` | container ownership check |
| Container on Apple silicon passes on arm64; CI passes on x86_64 | `make linux-test`, CI Linux job |
| CI Linux job uses the same container definition | workflow check |
| Mac with no runtime: `make doctor` reports it and does not fail | doctor without runtime |
| Case-dependent tests run on the container file system | case ID tests, file system check |
| Linux setup text carries the "not yet tested" note until checked | docs note check |
| Clean small Linux machine: install steps give `waymark --version` | manual OrbStack check |
| `make doctor` on Linux names a missing tool and a Linux hint | doctor missing tool |
| `make doctor` on Linux x86_64 with all tools: no platform warning | doctor clean run |
| Homebrew install: update hint says `brew upgrade waymark` | `platform` unit tests |
| Other install on Linux: update hint has no `brew` | `platform` unit tests |
| `git` missing on Linux: text says package manager, no `xcode-select` | `platform` unit tests, `it_maintenance` |
| IDs `Notes/Plan` and `notes/plan`: same behavior on both systems | case ID tests |
| Release workflow for a tag: Linux x86_64 archive and checksum | release workflow check, manual first release |
| Downloaded Linux archive: `waymark --version` prints the version | manual OrbStack check |
| README and `docs/local-install.md` name both systems, the guide says how to install, the README links to it | docs check |

## Test Cases

- [x] [auto] The full suite (unit, conformance, integration) passes in the development container on Linux arm64 (`make linux-test`, and `make linux-check`). Checked 2026-11-15
- [x] [auto] The full suite passes on Linux x86_64 in CI (`.github/workflows/ci.yml`, run once enabled or by a one-time run of the container with `ARCH=amd64`). Checked 2026-11-15 with `ARCH=amd64 make check` in the container (emulated through OrbStack): all pass
- [x] [auto] The full suite passes on macOS Apple silicon (`make test-all`)
- [x] [auto] The update hint is `brew upgrade waymark` for paths with `/Cellar/`, `/homebrew/`, and `/linuxbrew/`, and the generic text for other paths (`src/platform.rs` unit tests)
- [x] [auto] The Git install hint is the `xcode-select` text on macOS and the package manager text on Linux, and the Linux text has no `xcode-select` (`src/platform.rs` unit tests)
- [x] [auto] The tests that check update and Git hint texts use the hint of the system they run on (`tests/it_updates_needed.rs`, `tests/it_memory_repository.rs`, `tests/it_maintenance.rs`)
- [x] [auto] Documents `Notes/Plan` and `notes/plan` are written and read with the behavior that the Waymark Memory Standard defines, in a folder of the temp directory (`tests/it_case_sensitivity.rs`)
- [x] [auto] The case test reports the file system it ran on, and fails if it ran on the shared Mac folder (`tests/it_case_sensitivity.rs`)
- [x] [auto] `scripts/doctor.sh` accepts Linux x86_64 and macOS arm64 with no platform warning (`scripts/tests/run.sh`)
- [x] [auto] `scripts/doctor.sh` names a missing `git` or Rust toolchain with a Linux hint that does not mention Homebrew as the only way (`scripts/tests/run.sh`)
- [x] [auto] `scripts/doctor.sh` reports a missing container runtime as a note and exits with success for it (`scripts/tests/run.sh`)
- [x] [auto] `scripts/doctor.sh` fails for Git older than 2.28 (`scripts/tests/run.sh`)
- [x] [auto] `scripts/linux.sh` stops with "Start OrbStack (or your container runtime) and try again." when the runtime is not running (`scripts/tests/run.sh`, with a fake `docker`)
- [x] [auto] After `make linux-test`, no file in the project folder is owned by `root` (`scripts/tests/run.sh`, needs a runtime; skipped with a clear message where none exists)
- [x] [auto] The `Makefile` takes the target from the host and has no fixed `aarch64-apple-darwin`; `rust-toolchain.toml` has no fixed macOS target (`scripts/tests/run.sh`)
- [x] [auto] The `Makefile` works with GNU Make 3.81 (`make -n` on the macOS system Make)
- [x] [auto] `scripts/package.sh` names the archive after the host target and makes a checksum (`scripts/tests/run.sh`)
- [x] [auto] The release binary of Linux links only to the system libraries of the supported systems, with a C library not newer than Ubuntu LTS and Debian stable (`scripts/tests/run.sh`)
- [x] [auto] The CI workflow has a Linux and a macOS job and the Linux job builds `.devcontainer/Dockerfile`; the release workflow builds a Linux x86_64 archive and a checksum (`scripts/tests/run.sh`, YAML check)
- [x] [auto] The Linux setup text in `docs/local-install.md` carries the "not yet tested on a real machine" note, until the manual check is done (`scripts/tests/run.sh`)
- [x] [auto] `README.md` and `docs/local-install.md` name Linux x86_64 and macOS Apple silicon, and `README.md` links to `docs/local-install.md` (`scripts/tests/run.sh`)
- [ ] [manual] Linux docs check on a clean machine.
  - Setup: in OrbStack, create a new Ubuntu LTS machine with nothing installed.
  - Steps: follow the Linux steps in `docs/local-install.md` exactly as written (before the first Linux release exists, follow the build-from-source steps). Run `waymark --version`.
  - Expected: it prints the version. Fix every mistake in the text, then remove the "not yet tested" note from `docs/local-install.md`.
  - Teardown: delete the machine.
- [x] [manual] Optional speed check of x86_64 on the Mac. Done 2026-11-15 by the AI: `ARCH=amd64` passes the whole suite (633 tests) in about 6 minutes with the first build.
  - Steps: run `ARCH=amd64 make linux-test`.
  - Expected: the suite passes. Note the run time in `todo.md`.
- [ ] [manual] First Linux release.
  - Steps: run `scripts/release.sh` for the next version (see the release skill) with the release workflow enabled.
  - Expected: the GitHub release has the macOS and the Linux x86_64 archives, with checksums. On the OrbStack machine, the downloaded Linux archive runs `waymark --version`.

## Notes

- The container tests need a running container runtime. The tested one is OrbStack. A test that needs a runtime prints a clear message and does not fail where none exists. This is the only allowed skip, and it is about a missing tool, not a missing system (see the constraint "One test suite").
- Tests that need `ruby` or `brew` (the Homebrew tests) run on macOS and print a message on Linux, as the plan says.
- `[auto]` items for CI run in CI only after the product owner enables the workflows. Until then, the Linux result comes from the container on the Mac (arm64) and from one `ARCH=amd64` run (x86_64).

- The `[auto]` items for CI (`.github/workflows/*.disabled`) are checked as text in `scripts/tests/run.sh`. The workflows themselves have not run, because they are disabled. Enabling CI is the decision of the product owner.
