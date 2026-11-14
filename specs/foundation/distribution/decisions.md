# Decisions: Distribution

## First release: Homebrew builds from source — 2026-11-14

**Decision:** The formula builds from the source tarball of a tag with `cargo install`. A prebuilt binary workflow is prepared and not used for the first release.

**Alternatives considered:** A prebuilt signed binary (rejected for now: needs a developer identity and notarization). `cargo-dist` (rejected: another tool, and a hand-written formula is small).

**Reason:** No signing and no release files are needed, and Homebrew source installs are not quarantined by macOS. Decided autonomously.

## Apple silicon only — 2026-11-14

**Decision:** Only `aarch64-apple-darwin` is built and tested.

**Reason:** Scope (see `docs/decisions.md`).

## Outward steps are for the product owner — 2026-11-14

**Decision:** Creating the tap repository, tagging, and making this repository public are not done by the AI. They are written in `docs/next-steps/release-checklist.md` as exact steps.

**Reason:** Public releases cannot be undone, and the product owner asked for it (see `docs/decisions.md`).

## Implementation notes — 2026-11-14

**Decided autonomously:** The formula installs the prebuilt archive and has no `depends_on "git"`, because `brew audit` refuses it (macOS always has `git`). `docs/install.md` says that `git` must work. `scripts/package.sh` and the `sed` step of `scripts/brew-local.sh` replace the planned `make-formula.sh`. The release workflow is `.github/workflows/release.yml.disabled` (not run). `brew audit --strict --new` reports only that the homepage is not reachable (the repository is private). The local install test ran and was removed again.
