# Decisions: Distribution

## First release: Homebrew builds from source — 2026-11-14

**Decision:** The formula builds from the source tarball of a tag with `cargo install`. A prebuilt binary workflow is prepared and not used for the first release.

**Alternatives considered:** A prebuilt signed binary (rejected for now: needs a developer identity and notarization). `cargo-dist` (rejected: another tool, and a hand-written formula is small).

**Reason:** No signing and no release files are needed, and Homebrew source installs are not quarantined by macOS. Decided autonomously.

## Apple silicon only — 2026-11-14

**Decision:** Only `aarch64-apple-darwin` is built and tested.

**Reason:** Scope (see `docs/decisions.md`).

## Outward steps are for the product owner — 2026-11-14

**Superseded by (in part):** The release checklist is removed

**Decision:** Creating the tap repository, tagging, and making this repository public are not done by the AI. They are written in `docs/next-steps/release-checklist.md` as exact steps.

**Reason:** Public releases cannot be undone, and the product owner asked for it (see `docs/decisions.md`).

## Implementation notes — 2026-11-14

**Decided autonomously:** The formula installs the prebuilt archive and has no `depends_on "git"`, because `brew audit` refuses it (macOS always has `git`). `docs/install.md` says that `git` must work. `scripts/package.sh` and the `sed` step of `scripts/brew-local.sh` replace the planned `make-formula.sh`. The release workflow is `.github/workflows/release.yml.disabled` (not run). `brew audit --strict --new` reports only that the homepage is not reachable (the repository is private). The local install test ran and was removed again.

## One release script and a changelog with a description for each version — 2026-11-14

**Decision (product owner):** Releases are made by `scripts/release.sh`. Each release has a short description for people in `CHANGELOG.md`, and that text is the text of the GitHub release. A release without a description is refused.

**Alternatives considered:** `--generate-notes` of GitHub (rejected: for this repository it gives one line with a link and says nothing about the release). A separate notes file for each release (rejected: a second place that goes stale, and no single history to read). Only the CI workflow (rejected: it is disabled, and it needs a token for the tap that does not exist yet).

**Reason:** Six releases in one day were made by hand with the same commands, and one commit went out with failing tests because a command did not stop on the failure. A script that stops at the first failure and checks the result removes both problems. The descriptions were missing from every release page.

## The tap and the public release are done — 2026-11-14

**Decision (product owner):** The repository `ionpulse-dev/waymark` and the tap `ionpulse-dev/homebrew-tap` are public. Release 0.2.0 is published, and the install with Homebrew works. The README and `docs/install.md` now tell people to install with `brew install ionpulse-dev/tap/waymark`.

**Reason:** The install path of the README (an agent reads the README and follows the install guide) needs a working Homebrew install. Checked: the tap is public, `brew info` shows 0.2.0, and `waymark --version` prints `waymark 0.2.0`.

## The release checklist is removed — 2026-11-15

**Decision (product owner):** The folder `docs/next-steps/` is removed, with `release-checklist.md` in it. The open steps of the checklist are now in `todo.md`. The release steps are in `requirements.md`, section "Release process", and in the skill `.claude/skills/release/SKILL.md`.

**Supersedes (in part):** "Outward steps are for the product owner". The rule stays. Only the place of the steps changes.

**Alternatives considered:** Keep the checklist as a document in `docs/` (rejected: it repeats the release process of `requirements.md` and goes stale, as the release list at its top did).

**Reason:** One place for each fact. The spec is the source of truth, and `todo.md` is the place for open work.

