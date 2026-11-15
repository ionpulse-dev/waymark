# Tasks: Linux support

_Written after `plan.md` is approved. Bite-sized implementation checklist. AI checks off items as work is completed._

`requires`: `dev-environment`, `distribution` (both `implemented`).

## A. Dev container (first, so all later work can run on Linux)

- [x] Write `.devcontainer/Dockerfile`: Debian or Ubuntu base, `git`, build tools, Rust at the version of `rust-toolchain.toml`, user with the host user ID at run time.
- [x] Write `scripts/linux.sh`: check the runtime, build the image if needed, mount the source, use named volumes for `target/` and the Cargo caches, take optional `ARCH`, and stop with the "Start OrbStack" message when the runtime is not running.
- [x] Add the `Makefile` targets `linux-build`, `linux-test`, `linux-check`, and `linux-shell` that call `scripts/linux.sh`.
- [x] Write `.devcontainer/devcontainer.json` (attach with the named volumes and the user).

## B. Measure

- [x] Run the whole suite in the container (arm64) with `make linux-test`. Write every real failure into `todo.md`. Confirm or close the `/dev/null` bug.
- [x] Run the suite once as x86_64 (`ARCH=amd64`). Note the speed and the result in `todo.md`.

## C. Fix and tools

- [x] Fix each failure found in B, one by one, with the test that shows it. Record non-obvious choices in `decisions.md`.
- [x] Remove the fixed macOS target from `rust-toolchain.toml`. Take the target from the host (`rustc -vV`) in the `Makefile`, and keep it working with GNU Make 3.81.
- [x] Update `scripts/setup.sh` and `scripts/doctor.sh` for Linux x86_64 and arm64 and for macOS: tool checks, Git 2.28 minimum, Linux hints, and the runtime note.
- [x] Update `scripts/package.sh` and `scripts/release.sh` for the host target. Keep the Homebrew steps on macOS. Update `scripts/brew-local.sh` and `scripts/tests/run.sh` to skip `ruby` and `brew` tests with a message.

## D. Platform texts (prerequisite: C builds on Linux)

- [x] Write `src/platform.rs` (update hint, Git install hint, install path check) with a unit test for each branch, and register the module.
- [x] Use `src/platform.rs` in `src/mcp/args.rs`, `src/memrepo/settings.rs`, and `src/maintenance/mod.rs`.
- [x] Update `tests/it_updates_needed.rs`, `tests/it_memory_repository.rs`, and `tests/it_join_memory_repository.rs` to check the hint of the running system (`it_maintenance.rs` needed no change).
- [x] Write `tests/it_case_sensitivity.rs` (IDs `Notes/Plan` and `notes/plan`, in the temp folder, with a file system check).
- [x] Check what the code reads for the system config file on each system, and fix `docs/install.md` to match.

## E. CI and release (prerequisite: A, C)

- [x] Update `.github/workflows/ci.yml.disabled`: matrix of Linux (builds `.devcontainer/Dockerfile`, runs `make check`) and macOS. Keep it disabled.
- [x] Update `.github/workflows/release.yml.disabled`: build the Linux x86_64 archive and checksum next to the macOS one. Decide glibc or musl with the first release build (see `todo.md`). Keep it disabled.
- [x] Add the script tests for the `Makefile`, the doctor, `scripts/linux.sh`, the packages, and the workflows to `scripts/tests/run.sh`.

## F. Docs and specs (prerequisite: C, D, E)

- [x] Update `README.md`: badge and setup for both systems, with the "not yet tested on a real machine" note on the Linux steps.
- [x] Update `docs/install.md` with the Linux section (download, checksum, path, `git`) and the same note.
- [x] Update `docs/development.md` and `AGENTS.md`: the container commands and how an agent runs the suite on a Mac.
- [x] Add notes to `specs/foundation/distribution` and `specs/foundation/dev-environment`. Record the scope change in `docs/decisions.md`.
- [x] Update `CHANGELOG.md` for the release that carries this item.

## G. Verify

- [x] Run the full suite on macOS (`make test-all`) and in the container (`make linux-check`). Tick the `[auto]` items in `test.md`.
- [x] Set the status to `implemented` in `requirements.md` and in `specs/INDEX.md`, and update `## Next Up`.
- [x] Hand the `[manual]` items of `test.md` to the product owner (OrbStack docs check, x86_64 speed check, first Linux release).
