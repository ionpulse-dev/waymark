# Decisions: Distribution

## First release: Homebrew builds from source — 2026-11-14

**Decision:** The formula builds from the source tarball of a tag with `cargo install`. A prebuilt binary workflow is prepared and not used for the first release.

**Alternatives considered:** A prebuilt signed binary (rejected for now: needs a developer identity and notarization). `cargo-dist` (rejected: another tool, and a hand-written formula is small).

**Reason:** No signing and no release files are needed, and Homebrew source installs are not quarantined by macOS. Decided autonomously.

## Apple silicon only — 2026-11-14

**Decision:** Only `aarch64-apple-darwin` is built and tested.

**Reason:** Scope (see `docs/decisions.md`).

## Outward steps are for the product owner — 2026-11-14

**Decision:** Creating the tap repository, tagging, and making this repository public are not done by the AI. They are written in `docs/release-checklist.md` as exact steps.

**Reason:** Public releases cannot be undone, and the product owner asked for it (see `docs/decisions.md`).
