---
type: foundation
status: draft
created: 2026-11-15
updated: 2026-11-15
requires:
  - dev-environment
  - distribution
---

# Linux support

## Technical Goal

Make waymark build, pass all its tests, and run on Linux, as it does on macOS today. Today the project says "macOS on Apple silicon only" in the README, the docs, the scripts, the toolchain file, and CI. Servers run Linux, so a remote server (see [personal-server](../personal-server/requirements.md)) needs this first. Linux users of local mode also gain from it.

## Dependents

- `personal-server` — its container image holds the Linux build of this item.
- `distribution` — it gets a Linux release archive, next to the macOS one.
- `dev-environment` — its out-of-scope line "Linux and Intel Macs" changes for Linux.

## Current state

Checked on 2026-11-15 on Linux x86_64 (Rust 1.100.0, Git 2.43), in a sandbox:
- **Build:** a debug build succeeds. No code change was needed.
- **Unit tests:** all 214 tests of the library pass.
- **Integration tests:** not judged. In that sandbox `/dev/null` was a regular file with text in it, so the tests that use `/dev/null` as an empty Git config failed before they ran (`bad config line 1 in file /dev/null`). The cause is probably the sandbox. A clean Linux machine must confirm it.
- **Platform text and tools that say macOS only:**
  - `rust-toolchain.toml` lists the target `aarch64-apple-darwin`.
  - `Makefile`: `TARGET := aarch64-apple-darwin`.
  - `scripts/setup.sh` (needs Homebrew), `scripts/doctor.sh` (warns unless macOS arm64), `scripts/package.sh`, `scripts/release.sh`, and `scripts/brew-local.sh` (macOS target and Homebrew).
  - `.github/workflows/ci.yml.disabled` and `release.yml.disabled` run on macOS.
  - The code tells the person `brew upgrade waymark` (`src/mcp/args.rs`, `src/memrepo/settings.rs`) and `xcode-select --install` (`src/maintenance/mod.rs`). Several tests check these texts.
  - `README.md` (badge, "Setup (macOS only)"), `docs/install.md`, and `docs/development.md`.

## Deliverables

- All tests (unit, conformance, integration) pass on Linux x86_64 and on macOS, from one test suite.
- A CI workflow that runs the checks on Linux and on macOS.
- Messages that fit the system: the update hint and the Git install hint depend on the operating system and on how waymark was installed.
- Scripts for the developer that work on Linux and macOS: `make setup`, `make doctor`, `make build`, `make test`, and the release build. The toolchain file and the `Makefile` no longer name only the macOS target.
- A release archive for Linux x86_64 with a checksum, made by the release workflow.
- README and docs that name both systems and how to install on each.

## Scope

**In scope:**
- Linux on x86_64, tested on a current Ubuntu LTS and on Debian (the systems of the CI runner and of common container images).
- Local mode, as today. The server modes are the work of other specs.
- The developer tools and CI.

**Out of scope:**
- Linux on arm64 (see `todo.md`).
- Windows and Intel Macs.
- Linux packages other than the release archive (`apt`, `rpm`, Snap, and so on).
- The container image (see `personal-server`).
- Server mode, sign-in, and any change in the tools or the behavior of waymark.

## Constraints

- **macOS keeps working.** Nothing may get worse on macOS on Apple silicon. The same tests run on both systems.
- **No change in behavior.** A tool returns the same result on both systems. Only texts about the system (install and update hints) differ.
- **Decide at run time, not with `cfg` where possible.** The hint text depends on the system and the install path. Code that needs a system-specific branch is small, in one place, and tested.
- **One test suite.** There is no list of tests that run on one system only. A test that cannot run on a system is a bug to fix, not a skip.
- **Case-sensitive file system.** macOS is usually case-insensitive and Linux is case-sensitive. Document IDs, links, and folder names must behave the same, and the tests must show it.
- **`git` is required.** waymark calls the `git` program (`src/gitsync/cmd.rs`). The minimum version is the one that supports the options waymark uses (for example `--initial-branch`, which needs Git 2.28 or newer). The doctor checks it.
- **The release binary runs on common Linux systems.** It does not need a newer C library than the one in the supported systems.

## Acceptance Criteria

- **Given** a clean Linux x86_64 machine with Git and the Rust toolchain, **When** `make test-all` runs, **Then** all unit, conformance, and integration tests pass.
- **Given** macOS on Apple silicon, **When** the same command runs, **Then** all tests still pass.
- **Given** a pull request, **When** CI runs, **Then** it runs the checks on Linux and on macOS, and both must pass.
- **Given** `make doctor` on Linux, **When** a tool is missing (for example `git` or the Rust toolchain), **Then** it names the missing tool and how to get it on Linux, and it does not mention Homebrew as the only way.
- **Given** `make doctor` on Linux x86_64, **When** all tools exist, **Then** it reports no warning about the platform.
- **Given** waymark was installed with Homebrew, **When** a text tells the person to update, **Then** it says `brew upgrade waymark`.
- **Given** waymark was installed any other way on Linux, **When** a text tells the person to update, **Then** it does not say `brew` and it says to update waymark the way it was installed.
- **Given** `git` is missing on Linux, **When** a check reports it, **Then** the text says to install Git with the package manager of the system, and does not say `xcode-select`.
- **Given** a document with the ID `Notes/Plan` and another with `notes/plan`, **When** both are written and read on Linux and on macOS, **Then** the behavior is the same on both, and it is the behavior that the Waymark Memory Standard defines.
- **Given** the release workflow, **When** it runs for a tag, **Then** it produces a Linux x86_64 archive and a checksum, next to the macOS one.
- **Given** the downloaded Linux archive on a supported system, **When** `waymark --version` runs, **Then** it prints the version.
- **Given** the README and `docs/install.md`, **When** they are read, **Then** they name Linux x86_64 and macOS Apple silicon, and say how to install on each.

## Open Questions

- Which C library does the release binary use: the standard one (glibc), built on an old system so that it runs widely, or a static one (musl)? The container image of `personal-server` needs a choice that works with `git` inside the image.
- Does the Homebrew tap serve Linux too? `distribution` says Homebrew also runs on Linux. Does the formula need a Linux block and a second archive?
- Which Linux systems do we promise? The proposal is "tested on Ubuntu LTS and Debian, others best effort".
- Is the `/dev/null` problem only in that sandbox? Run the suite on a clean Linux machine first.
- Where does the system config file live on Linux? `docs/install.md` says `/etc/waymark/config.toml` "or the macOS equivalent". Check what the code reads on each system.
- When does the product owner enable CI? The two workflow files are `.disabled` today.
