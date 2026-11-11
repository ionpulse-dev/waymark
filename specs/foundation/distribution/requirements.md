---
type: foundation
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - mcp-server
---

# Distribution

## Technical Goal

Define how waymark reaches people: one binary per platform, one container image for server mode, and clear documentation. A new person needs two steps: install the binary and add one MCP entry to the agent tool.

## Dependents

- `join-knowledge-base`, `agent-tool-setup`, `maintenance`, `server-mode`.

## Deliverables

- A static binary for macOS and Linux. Windows later.
- A package for Homebrew and a download page with checksums.
- A container image for server mode, with `git` inside.
- The README sections: install, MCP entry for each agent tool, and example `AGENTS.md` snippets.

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

- **Given** a Mac with `git`, **When** the person installs with Homebrew, **Then** `waymark` is on the `PATH`.
- **Given** the README snippet for an agent tool, **When** the person adds it and starts a session, **Then** the tools of waymark show.
- **Given** a release, **When** Gatekeeper checks it, **Then** it passes as notarized.
- **Given** the container image, **When** it starts with the required settings, **Then** the server answers on HTTPS behind a proxy.
- **Given** a new release, **When** the person installs it, **Then** the tool descriptions match the binary.

## Open Questions

- Is the developer account for signing in place?
- Homebrew only, or also a download script for the first release?
- Which registry hosts the container image?
