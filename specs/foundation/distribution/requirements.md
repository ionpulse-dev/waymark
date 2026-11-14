---
type: foundation
status: planned
created: 2026-11-08
updated: 2026-11-14
requires:
  - mcp-server
---

# Distribution

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Only macOS on Apple silicon and Homebrew are built. The container image and Linux are out of scope for now. The tap, the release, and the public repository are prepared, and they are created by the product owner (see `docs/release-checklist.md`). Parts that need server mode stay in this text as a record, and they are not implemented now (see [project decisions](../../../docs/decisions.md)).

## Technical Goal

Define how waymark reaches people: one binary per platform, one container image for server mode, and clear documentation. A new person needs two steps: install the binary and add one MCP entry to the agent tool.

## Dependents

- `join-knowledge-base`, `agent-tool-setup`, `maintenance`, `server-mode`.

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
- The tap and the release files are public. They are created at the public release (see the roadmap, phase 5).

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

## Acceptance Criteria

- **Given** a Mac with `git`, **When** the person runs `brew install <account>/tap/waymark`, **Then** `waymark` is on the `PATH`.
- **Given** a new release, **When** CI finishes, **Then** the formula in the tap points to the new release file with the right checksum.
- **Given** the README snippet for an agent tool, **When** the person adds it and starts a session, **Then** the tools of waymark show.
- **Given** a release, **When** Gatekeeper checks it, **Then** it passes as notarized.
- **Given** the container image, **When** it starts with the required settings, **Then** the server answers on HTTPS behind a proxy.
- **Given** a new release, **When** the person installs it, **Then** the tool descriptions match the binary.

## Open Questions

None for now. Answered on 2026-11-14 (see `decisions.md`). Still open for the product owner: the signing identity, and the registry of the container image (server mode).
