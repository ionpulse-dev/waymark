# Decisions: Mac app

> **Note 2026-11-11:** This spec is superseded by `web-console` and `distribution`. The decisions below are kept as a record. They no longer apply.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## People use the app, agents use the CLI — 2026-11-06

**Decision:** Everything a person does has native UI. The CLI keeps what makes sense in a terminal anyway: `doctor`, `status`, and non-interactive `init` and `setup` with flags for scripts and rollout tools.

**Alternatives considered:**
- A CLI-only product — managers would not use it.
- A web UI — needs a server.

**Reason:** Comfort for non-developers, while agents keep a stable CLI.

## Menu-bar-only app — 2026-11-06

**Decision:** No Dock icon. One icon shows whether everything is fine.

**Reason:** No window in the way, one visible process instead of a hidden job.

## Native Swift app, no Mac App Store — 2026-11-06

**Decision:** Swift, AppKit, and ServiceManagement. Direct download, signed and notarized.

**Alternatives considered:**
- Go with a systray library (no update since 2024, needs cgo).
- Mac App Store — the sandbox blocks writing agent tool settings and calling `git`.

**Reason:** Menu bar, dialog, Keychain, and login item are standard APIs in Swift.

## Settings can be changed only in the app — 2026-11-06

**Decision:** The app has the settings. `link` and `unlink` in a terminal need a native confirmation from the app.

**Reason:** An agent must not change its own limits or bind a folder.

## No rules editor in the app — 2026-11-06

**Decision:** The skill text is fixed. Extra rules go into the `AGENTS.md` files that exist anyway.

**Reason:** Rules can always be added with `AGENTS.md`, so an editor has no benefit.
