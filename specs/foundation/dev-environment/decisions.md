# Decisions: Development environment

## Makefile over a task runner — 2026-11-14

**Decision:** The project uses a plain Makefile that calls shell scripts. It works with GNU Make 3.81.

**Alternatives considered:** `just` and `cargo xtask` (rejected: one more tool to install first, and a new Mac has `make` already).

**Reason:** A new machine needs nothing before `make setup`. Decided autonomously.

## Rust 1.100.0 pinned — 2026-11-14

**Decision:** `rust-toolchain.toml` pins Rust 1.100.0, the stable release on 2026-11-09, with `rustfmt` and `clippy`.

**Reason:** A pinned version gives the same build on every machine. The pin is raised on purpose. Decided autonomously.

## CI on macOS only — 2026-11-14

**Decision:** CI runs on a macOS runner on Apple silicon, on pushes to `main` and on pull requests. The workflow file is written, and it is pushed only when the plan says so.

**Reason:** The scope is macOS on Apple silicon (see `docs/decisions.md`). Decided autonomously.

## The CI file is shipped disabled — 2026-11-14

**Decision:** The workflow is committed as `.github/workflows/ci.yml.disabled`. GitHub reads only `.yml` and `.yaml` files, so it does nothing until someone renames it to `ci.yml`.

**Alternatives considered:** Commit `ci.yml` at once (rejected: it would start paid macOS runners on a private repository without the product owner's choice).

**Reason:** The plan says CI minutes are the product owner's call. Decided autonomously.
