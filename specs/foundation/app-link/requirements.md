---
type: foundation
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - cli-core
---

# App link

## Technical Goal

Define the local channel between the CLI and the menu bar app, so the CLI can ask the app for things that only a native app can do, and the app can start CLI commands.

## Dependents

- `mac-app`, `approval-dialog`, `pause-and-resume`, `login-and-identity`, `project-binding`.

## Deliverables

- One message protocol, the same on every OS.
- The transport for macOS. Other transports come later.
- The list of message types.

## Scope

**In scope:**
- Messages, transport, caller checks, time-outs.

**Out of scope:**
- The windows themselves (see `mac-app`).

## Constraints

- Messages are small JSON documents. The protocol does not depend on the OS.
- Transport per OS: Unix domain socket on macOS and Linux, in the private folder of the user with permissions 0600. A named pipe on Windows, later. The first release builds only the macOS transport.
- The app accepts approval requests only from the `waymark` binary. Caller checks on Windows are to be verified.
- The agent cannot answer a request. It only receives the result: approved or canceled.
- No answer in 2 minutes means canceled.
- If the app is not running, requests that need it fail with exit code 7.
- The CLI never stores a secret received through the link.

### Message types

| Request from CLI | App answer |
|---|---|
| Approve a write (repo, folder, title, body, or a summary for moves and bulk changes) | approved, canceled |
| Confirm resume | approved, canceled |
| Confirm link or unlink of a project folder | approved, canceled |
| Offer the setup of an unknown knowledge base (name, URL) | confirmed, canceled |
| Get the credential for a Git host | token or none |
| Get the personal settings and the access mode | values |

The app starts CLI commands (sync, undo, lint) as normal processes.

## Acceptance Criteria

- **Given** a request from another program, **When** it connects to the socket, **Then** the app rejects it.
- **Given** a socket file, **When** another user lists it, **Then** its permissions deny access.
- **Given** no answer for 2 minutes, **When** an approval is open, **Then** it ends as canceled.
- **Given** the app is not running, **When** the CLI needs approval, **Then** the command exits with code 7 and tells the person to open the app.
- **Given** the same message, **When** it is sent on any OS, **Then** its JSON is identical.

## Open Questions

- What is the exact message format and version field?
- Should macOS use XPC for hardening later?
- Which crate hides the socket and pipe difference?
