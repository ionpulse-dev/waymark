# Plan: Linux support

_The technical approach. The requirements are in [requirements.md](requirements.md)._

## Overview

First find out what really fails on a clean Linux machine, then fix the real problems and replace the macOS-only text and tools. The Rust code already builds on Linux, so most of the work is tools, scripts, texts, CI, and the release.

## Components Affected

- `rust-toolchain.toml` — remove the fixed macOS target. Add targets only where a release build needs them.
- `Makefile` — take the target from the host (`rustc -vV`) instead of `aarch64-apple-darwin`. Keep it working with the old GNU Make 3.81 of macOS.
- `scripts/setup.sh`, `scripts/doctor.sh` — work on Linux and macOS. On Linux they check for tools and print how to get them. They do not install system packages.
- `scripts/package.sh`, `scripts/release.sh` — build for the host target and name the archive after it. Homebrew steps stay macOS-side and are skipped where Homebrew is not used.
- `scripts/brew-local.sh`, `scripts/tests/run.sh` — stay macOS-side. The tests for them skip tools that do not exist (`ruby`, `brew`) with a clear message and not a failure.
- `.github/workflows/ci.yml.disabled`, `release.yml.disabled` — a matrix of Linux and macOS. They stay disabled until the product owner enables them.
- `src/mcp/args.rs`, `src/memrepo/settings.rs` — the update hint comes from `src/platform.rs`.
- `src/maintenance/mod.rs` — the Git install hint comes from `src/platform.rs`.
- `tests/it_updates_needed.rs`, `tests/it_memory_repository.rs`, `tests/it_maintenance.rs` — the tests that check `brew` and `xcode-select` texts use the hint for the system that they run on.
- `README.md`, `docs/install.md`, `docs/development.md` — name both systems.
- `specs/foundation/distribution`, `specs/foundation/dev-environment` — notes that Linux x86_64 is now in scope.

## New Files

- `src/platform.rs` — the system-dependent texts: the update hint (Homebrew or generic) and the Git install hint, and the check that decides how waymark was installed.

## Technical Approach

1. **Measure first.** Run the whole suite on a clean Linux x86_64 machine (a CI runner or a container, not the sandbox with the odd `/dev/null`). Write the real failures into `todo.md`. Fix them one by one. Expected causes: case-sensitive file names, paths, the `kill` call and process handling, the temp folder, and the Git version.
2. **Remove the hard-coded target.** The toolchain file and `Makefile` follow the host. Release builds name their target on purpose.
3. **Platform texts.** `src/platform.rs` decides by the system at run time:
   - *Update hint:* `brew upgrade waymark` if the running binary is inside a Homebrew folder (the install path contains `/Cellar/`, `/homebrew/`, or `/linuxbrew/`). Otherwise: "Update waymark the way you installed it."
   - *Git hint:* on macOS, the `xcode-select --install` text. On Linux, "Install Git with the package manager of your system, for example `apt install git`."
   - Each function has a unit test for every branch.
4. **Developer scripts.** `doctor.sh` reads the system (`uname`) and accepts Linux x86_64 and macOS arm64. It lists missing tools with a Linux or macOS hint. `setup.sh` on Linux prints the rustup command and exits if the toolchain is missing. It does not run a package manager.
5. **CI.** One workflow with a matrix: `ubuntu-latest` and `macos-14`. It runs `make check` on both. It stays `.disabled`.
6. **Release.** The release workflow builds on both runners and uploads one archive and one checksum per target. The Linux archive is x86_64. The C library choice is an open question that the first release build decides.
7. **Docs.** The README badge and the setup text name both systems. `install.md` gets a Linux section: download the archive, check the checksum, put the binary on the path, and make sure `git` works.
8. **Specs.** Add a note to `distribution` and `dev-environment`. Record the scope change in `docs/decisions.md`.

## Dependencies on Other Foundation Items

- `dev-environment` — this item changes its scripts and its scope line. It is `implemented`.
- `distribution` — this item adds a Linux archive to its release flow. It is `implemented`.

## Risks

- **Unknown failures.** The integration tests are not yet judged on Linux. The measure step may find more work.
- **Case sensitivity.** The standard defines the ID rules. A bug in case handling is a bug on Linux, and it hides on macOS.
- **C library.** A binary built on a very new system may not run on an older one. Build on an old enough base, or link statically.
- **Homebrew on Linux.** If the tap also serves Linux, the formula needs a second archive and a test that this repository cannot run on macOS only.
- **CI cost and trust.** The workflows are disabled. Enabling them is a decision of the product owner.
- **Scope drift.** This item must not change a tool or a result. Only system texts and tools may differ.

## Open Questions

- See the open questions in [requirements.md](requirements.md).
- Which Linux machine runs the first full measurement before CI exists?
