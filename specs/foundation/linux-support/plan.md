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
- `Makefile` — new targets `linux-build`, `linux-test`, `linux-check`, and `linux-shell`. Each one calls `scripts/linux.sh`.
- `scripts/doctor.sh` — also reports whether a container runtime is available. A missing runtime is a note, not a failure.
- `AGENTS.md` and `docs/development.md` — tell an agent on a Mac how to run the suite in the container.
- `README.md`, `docs/install.md`, `docs/development.md` — name both systems.
- `specs/foundation/distribution`, `specs/foundation/dev-environment` — notes that Linux x86_64 is now in scope.

## New Files

- `src/platform.rs` — the system-dependent texts: the update hint (Homebrew or generic) and the Git install hint, and the check that decides how waymark was installed.
- `.devcontainer/Dockerfile` — the development container: a Debian or Ubuntu base, `git`, build tools, and Rust at the version of `rust-toolchain.toml`. Runs as a user with the host user ID.
- `.devcontainer/devcontainer.json` — the configuration that lets an editor attach to the container (see the open question). It names the volumes and the user.
- `scripts/linux.sh` — builds the image if needed, starts the container with the source mounted and the named volumes, and runs the command that it gets. Used by the `linux-*` targets and by CI.

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
7. **Docs.** The README badge and the setup text name both systems. `install.md` gets a Linux section: download the archive, check the checksum, put the binary on the path, and make sure `git` works. The Linux text is written first from the commands, and it carries a visible note ("The Linux steps are not yet tested on a real machine"). Then the maintainer makes a small Ubuntu machine in OrbStack, follows the text step by step with no other knowledge, fixes every mistake, deletes the machine, and removes the note. This is one manual check and has no tooling. It needs the first Linux release archive. Before that exists, the build-from-source steps can be followed on the same kind of machine.
8. **Specs.** Add a note to `distribution` and `dev-environment`. Record the scope change in `docs/decisions.md`.

### The development container

- **Image.** One `Dockerfile` for development. It has no project files inside. The Rust version comes from `rust-toolchain.toml`, so the container and the host use the same compiler. `git` is installed because waymark calls it.
- **Run.** `scripts/linux.sh <command>` does `docker`-compatible calls only (`build`, `run`). It mounts the source folder, and uses named volumes for `target/` and for the Cargo registry and git caches. The first run builds the image and fetches the crates. Later runs reuse them.
- **User.** The container user gets the user and group ID of the host user at run time. Files that the container writes into the mounted folder then belong to the person on the Mac.
- **Architecture.** On an Apple silicon Mac the container is Linux arm64, which runs natively and fast. CI runs x86_64. The suite runs on both, so that neither is a surprise. `scripts/linux.sh` takes an optional architecture (for example `ARCH=amd64 make linux-test`) that runs the image as x86_64. OrbStack can run x86_64 containers on Apple silicon, and as far as I know it uses Rosetta for this, which is faster than a general emulator. This is not verified here, so the first task checks the speed and whether the whole suite passes that way.
- **Runtime.** The tested runtime is OrbStack. The script uses only the Docker-compatible command (`docker build`, `docker run`), which OrbStack provides, so other runtimes can work. When the runtime is not running, the script stops with the message "Start OrbStack (or your container runtime) and try again."
- **Case sensitivity.** The source folder is mounted from the Mac, and its file system is usually case-insensitive. The tests make their folders in the temp folder of the container, which is case-sensitive. The case test must use that folder, and a check shows which file system it ran on.
- **CI.** The Linux job builds the same `Dockerfile` and runs `make check` in it. So the developer and CI see one environment.
- **Agents.** An AI agent on a Mac calls `make linux-test` and `make linux-check`. The doctor tells it if no runtime is running.
- **Not a release.** The image of `personal-server` is a separate file with a separate job. The two share the same Linux base so that results match.

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
- **Docs not yet checked.** The Linux setup text may hold a mistake that only a real system shows (a package name, a path, a missing library). The visible note and the one manual check on a clean OrbStack machine limit the damage. If the check is skipped, the first Linux user finds the first error.
- **File sharing speed.** A mounted folder from a Mac can be slow. The named volumes for `target/` and the caches are the main protection. Measure a full run on the Mac.
- **Architecture surprise.** A bug may show only on x86_64 and a Mac tests arm64. CI is the second check, so CI must be enabled before this item is called done.
- **A runtime on every Mac.** The container needs a running runtime. A runtime that is stopped makes `make linux-test` fail, so the message must say what to start.
- **License.** The license terms of OrbStack depend on how a person uses it. The project does not require OrbStack. It only tests with it.

## Open Questions

- See the open questions in [requirements.md](requirements.md).
- The first full measurement runs in the development container on the Mac (arm64), then in CI (x86_64). Is that enough, or does a one-time run on a clean Linux x86_64 machine help before CI is enabled?
