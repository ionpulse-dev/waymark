---
type: foundation
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - mcp-server
---

# Distribution

> **Note 2026-11-15:** The spec [linux-support](../linux-support/requirements.md) (`implemented`) brings Linux on x86_64 back into scope. The release workflow builds a Linux archive and a checksum next to the macOS one. The Homebrew formula stays macOS only. The scope line below, "macOS only", no longer holds for the archive.

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Only macOS on Apple silicon and Homebrew are built. The container image and Linux are out of scope for now. The repository and the tap `ionpulse-dev/homebrew-tap` are public, and the current release is published. `brew install ionpulse-dev/tap/waymark` works (see `docs/next-steps/release-checklist.md`). Parts that need server mode stay in this text as a record, and they are not implemented now (see [project decisions](../../../docs/decisions.md)).

## Technical Goal

Define how waymark reaches people: one binary per platform, one container image for server mode, and clear documentation. A new person needs two steps: install the binary and add one MCP entry to the agent tool.

## Dependents

- `join-memory-repository`, `agent-tool-setup`, `maintenance`, `server-mode`.

## Deliverables

- A static binary for macOS and Linux. Windows later.
- A Homebrew tap and a download page with checksums (see "Homebrew").
- A container image for server mode, with `git` inside.
- The README sections: install, MCP entry for each agent tool, and example `AGENTS.md` snippets.

### Homebrew

- **Own tap first.** A public repository `homebrew-tap` in the GitHub account of the project holds one formula file for each program, in `Formula/`. People install with `brew install <account>/tap/waymark`. Homebrew finds the tap by its name. The tap repository is only for formulas.
- **The formula** holds the URL of the release file, its checksum, and the install step. The URL points to the release of the `waymark` repository. The tap can hold formulas for other projects of the same account.
- **Release flow.** CI builds the binaries, uploads them to the release, and updates the formula with the new URL and checksum. A tool such as `cargo-dist` may generate this. It is not decided.
- **Prebuilt or from source.** The formula installs the signed prebuilt binary. While the signing identity is missing, a formula that builds from source on the person's machine is the fallback. Homebrew installs are not quarantined by macOS, so no Gatekeeper warning shows.
- **Homebrew core later.** `brew install waymark` without a tap needs a pull request to `Homebrew/homebrew-core`. It needs a public repository, tagged releases, a build from source with Homebrew dependencies, and some real users. It is a goal after the first release, not part of it.
- **Linux.** Homebrew also runs on Linux, so the same tap serves both systems. The download page stays as a backup.
- The tap and the release files are public (since 2026-11-14).

## Scope

**In scope:**
- Packaging, signing, installing, updating, and the documentation of these.

**Out of scope:**
- An app, a `.dmg`, or an installer that writes into agent tools.
- Company rollout with device management (an idea for later). The system config file of `trust-model` is the hook for it.

## Constraints

- No Node, no Python. One binary per platform. `git` must be installed in local mode.
- The binary is signed, and on macOS notarized, with a developer identity, so that it starts without warnings.
- The release is built from the public repository and its CI. An organization can mirror the repository into its own Git host and build releases there.
- The container image is small, runs as a normal user, and holds no secret.
- Updates: the person updates with the package manager or a new download. The binary does not update itself. The container updates by a new image tag.
- Binary and tool descriptions always have the same version, because both are in one file.

### Release process

A release is one command: `scripts/release.sh <version>`. `scripts/release.sh <version> --check` runs only the checks and changes nothing. The script stops at the first step that fails and names it. A Claude Code skill (`.claude/skills/release/SKILL.md`) tells an agent how to prepare and run it.

- **Every release has a description.** `CHANGELOG.md` holds one entry for each version, newest first, with a short text for people: what is new and what to do, with no commit list. The text of the entry is the text of the GitHub release. A release without an entry of at least 40 characters is refused (`scripts/changelog-entry.sh`).
- **Steps of the script, in order:**
  1. The checks: the entry exists, the version is not older than `Cargo.toml`, the branch is `main`, the working tree is clean (`.DS_Store` aside), `main` equals `origin/main`, the tag does not exist, `gh` is signed in, the tap is reachable, and the Linux container works.
  2. `make check` and `scripts/public-check.sh`.
  3. The version in `Cargo.toml` (and `Cargo.lock`), in a commit `Release <version>` that is pushed. If `Cargo.toml` already has the version, no commit is made.
  4. The archive and its checksum (`scripts/package.sh`). Then the Linux x86_64 archive and its checksum, built in the development container (`ARCH=amd64 scripts/linux.sh bash scripts/package.sh`).
  5. The tag `v<version>`, pushed, and the GitHub release with both archives, the checksum files, and the entry of the changelog as notes.
  6. The formula in the tap, filled with the version and the checksum, in a commit `waymark <version>`.
  7. The verification: the published archives have their checksums, the binaries report the version (the Linux one runs in an Ubuntu container), `brew audit --strict --new` passes, and the tag points at `HEAD`.
- **The script never asks and never skips a failed check.** It does not install on the machine of the person. It prints the command: `brew update && brew upgrade <account>/tap/waymark`.
- **Version numbers.** While waymark is below 1.0, a fix or a small change raises the patch number (0.1.5 to 0.1.6) and a larger change raises the minor number. The product owner decides.
- **CI later.** The disabled workflow `.github/workflows/release.yml.disabled` may replace steps 4 to 6 when a token for the tap exists. The changelog rule stays.

## Acceptance Criteria

- **Given** a Mac with `git`, **When** the person runs `brew install <account>/tap/waymark`, **Then** `waymark` is on the `PATH`.
- **Given** a new release, **When** CI finishes, **Then** the formula in the tap points to the new release file with the right checksum.
- **Given** the README snippet for an agent tool, **When** the person adds it and starts a session, **Then** the tools of waymark show.
- **Given** a release, **When** Gatekeeper checks it, **Then** it passes as notarized.
- **Given** the container image, **When** it starts with the required settings, **Then** the server answers on HTTPS behind a proxy.
- **Given** a new release, **When** the person installs it, **Then** the tool descriptions match the binary.
- **Given** a version without an entry in `CHANGELOG.md`, **When** `scripts/release.sh` runs, **Then** it stops before it changes anything and says so.
- **Given** a version with an entry, **When** the release is made, **Then** the GitHub release has the text of the entry as its notes.
- **Given** a failed check or step, **When** `scripts/release.sh` runs, **Then** it stops and names the step, and no later step runs.
- **Given** a finished release, **When** the script verifies it, **Then** the published archive has the checksum of the formula and its binary reports the version.

## Open Questions

None for now. Answered on 2026-11-14 (see `decisions.md`). Still open for the product owner: the signing identity, and the registry of the container image (server mode).
