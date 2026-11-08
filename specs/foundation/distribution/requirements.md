---
type: foundation
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - mac-app
---

# Distribution

## Technical Goal

Define how waymark reaches people: one download, a signed and notarized app, a simple script for developers, and updates. A new person needs only two steps.

## Dependents

- `join-knowledge-base`, `maintenance`.

## Deliverables

- `Waymark.app` with the CLI inside, in a `.dmg`.
- `install.sh` and a Homebrew cask that install it and link the CLI into the `PATH`.
- Signed and notarized releases with an Apple Developer ID.
- Releases of the CLI alone for Linux and Windows, later.

## Scope

**In scope:**
- Packaging, signing, installing, updating, the bootstrap problem.

**Out of scope:**
- Company rollout with device management (an idea for later).

## Constraints

- No Node, no Python, no Docker. One binary per platform.
- The new person needs `waymark` before `waymark setup`. `install.sh` downloads the latest release and links the CLI. Then `waymark setup <url>` runs. Two commands in total.
- The release is built from the public repository and its CI. An organization can mirror the repository into its own Git host and build releases there.
- The skill and the templates are inside the binary, so their versions always match.

## Acceptance Criteria

- **Given** a Mac without developer tools, **When** the person drags the app to Applications and opens it, **Then** the setup window starts.
- **Given** a terminal, **When** `install.sh` runs, **Then** the app is installed and `waymark` is on the `PATH`.
- **Given** a release, **When** Gatekeeper checks it, **Then** it passes as notarized.
- **Given** a newer release, **When** `waymark update` runs, **Then** binary and skill are replaced together.

## Open Questions

- Is the Apple Developer account in place?
- Homebrew cask or only `install.sh` for the first release?
