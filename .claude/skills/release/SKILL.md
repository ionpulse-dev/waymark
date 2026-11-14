---
name: release
description: Release a new version of waymark. Writes the changelog entry, runs scripts/release.sh, and reports the result. Use when the product owner asks for a release or a new version.
---

# Release waymark

A release is one command, `scripts/release.sh <version>`. The rules are in `specs/foundation/distribution/requirements.md`, section "Release process". Read it first.

## Steps

1. **Pick the version.** Below 1.0, a fix or small change raises the patch number (0.1.5 to 0.1.6). A larger change raises the minor number. If the product owner gave a rule ("only the patch"), follow it. The latest tag is `git describe --tags --abbrev=0`.
2. **Write the changelog entry.** Read the commits since the last tag (`git log <tag>..HEAD --oneline`) and the specs that changed. Add `## <version> (<date>)` at the top of `CHANGELOG.md`, with a short description for people: what is new, and what to do (for example "Update with `brew upgrade waymark` and restart your agent tool"). No commit list, no internal jargon. Use the style of the existing entries.
3. **Show the entry to the product owner and wait for a yes.** It becomes public text on GitHub.
4. **Commit and push the entry** (`CHANGELOG.md` only), then run `bash scripts/release.sh <version> --check`. Fix what it names.
5. **Release:** `bash scripts/release.sh <version>`. It runs the checks, sets the version, builds, tags, publishes, updates the tap, and verifies the result. Do not chain it with other commands in a way that hides a failure. If it fails, report the step and the output. Do not retry blindly and do not work around a failed check.
6. **Report** the release link, that the checks passed, and the command for the person: `brew update && brew upgrade ionpulse-dev/tap/waymark`. Say that a running agent tool needs a restart.

## Never

- Never release without a changelog entry that the product owner has seen.
- Never skip or weaken a check to get a release through.
- Never edit a published release or tag without being asked.
