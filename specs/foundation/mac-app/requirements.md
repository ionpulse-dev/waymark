---
type: foundation
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - app-link
---

# Mac app

## Technical Goal

Provide `Waymark.app`, the native macOS menu bar app for people. People use the app, agents use the CLI. The app holds no document logic. It calls the CLI and owns everything that needs native UI.

## Dependents

- `approval-dialog`, `pause-and-resume`, `activity-and-undo`, `notifications-and-review`, `manual-edits`, `project-binding`, `create-knowledge-base`, `join-knowledge-base`, `login-and-identity`, `distribution`.

## Deliverables

- A Swift menu bar app, macOS only, with the CLI inside its bundle.
- The shell: menu, icon states, settings window, login item, background sync, Keychain, localization.

## Scope

**In scope:**
- The app shell and the features that live in it (they have their own specs).

**Out of scope:**
- Document logic, Git logic, and checks (the CLI does these).
- A Dock icon. The app is menu-bar-only.
- Apps for other systems. They can come later.

## Constraints

- Menu-bar-only (`LSUIElement`): no Dock icon, no window in the way.
- The app starts at login (`SMAppService`).
- All window strings are in resource files. English first.
- The app is signed and notarized.
- It is not sold through the Mac App Store, because the sandbox blocks writing agent tool settings and calling `git`.

### Menu

- An icon that shows the state: running, paused, or error.
- Pause and resume.
- **Sync now**: runs a sync at once (including the commit of manual edits) and shows the result.
- Status: last good sync and last error.
- Entries to the activity list, the review list, and Settings.

### Settings window

Default knowledge base, linked folders (add with a folder picker, remove, opt out), access mode per knowledge base, read level, write level, confirm in, sync interval, agent tools. Only the person changes these.

### Windows

Setup and `init` windows, including the "Install Git" step, and the native dialogs specified in the feature specs.

## Acceptance Criteria

- **Given** a login, **When** the Mac starts, **Then** the app starts and shows its icon.
- **Given** the app is running, **When** the interval passes, **Then** a background sync runs.
- **Given** the Mac wakes from sleep, **When** the app notices, **Then** a sync runs at once.
- **Given** the menu, **When** "Sync now" is chosen, **Then** a sync runs and the result shows.
- **Given** the app code, **When** it is reviewed, **Then** it contains no document parsing or Git logic.
- **Given** a new window string, **When** a language file is added, **Then** the window shows the new language.

## Open Questions

- Which minimum macOS version?
- Which Xcode and Swift version?
