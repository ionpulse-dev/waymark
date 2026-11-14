# Plan: Development environment

## Overview

A Makefile calls small shell scripts. The scripts do the work, so the Makefile stays short and works with GNU Make 3.81. `rust-toolchain.toml` pins Rust.

## Components Affected

- `docs/development.md` — the development guide. The README links to it.
- `.gitignore` — already ignores `tmp/`. Adds `target/`.

## New Files

- `Makefile` — the targets of the requirements.
- `rust-toolchain.toml` — Rust 1.100.0, profile minimal, components `rustfmt` and `clippy`, target `aarch64-apple-darwin`.
- `scripts/doctor.sh` — checks tools and versions.
- `scripts/setup.sh` — installs what is missing.
- `scripts/spec-check.sh` — checks the spec folders.
- `.github/workflows/ci.yml.disabled` — build (inert until renamed to `ci.yml`, so a push does not start macOS runners), test, and lint on a macOS runner on pushes to `main` and on pull requests.

## Technical Approach

1. `rust-toolchain.toml` is the single source of the Rust version. `cargo` installs it on first use.
2. `scripts/doctor.sh` prints each tool with its version and exits non-zero on the first missing tool, with the fix command.
3. `scripts/setup.sh` runs the checks of `doctor`, installs what is missing in this order: Homebrew check, `rustup` (from the official installer if missing), the toolchain, extra tools. It only prints advice when a step needs `sudo` or a GUI (for example the Xcode command line tools).
4. The Makefile defines `.PHONY` for every target, uses `@` with an `echo` line per target, and avoids `$(file ...)`, `.ONESHELL`, and `$(shell ...)` tricks that Make 3.81 does not know.
5. `scripts/spec-check.sh` uses plain `bash`, `grep`, `sed`, and `find`.
6. CI runs `make check` on `macos-latest` (Apple silicon).

## Dependencies on Other Foundation Items

None.

## Risks

- A new Make feature slips in. Mitigation: a test that parses the Makefile with `/usr/bin/make -n`.
- CI minutes on macOS cost money. Mitigation: CI runs only on `main` and pull requests.

## Open Questions

None.
