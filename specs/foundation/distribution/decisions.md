# Decisions: Distribution

> **Note 2026-11-11:** "One deliverable" and "Direct download" described the app and its `.dmg`. They are superseded by "One binary and one image" below. "Free, open source, MIT" stays valid.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## One deliverable — 2026-11-06

**Decision:** The release is `Waymark.app` with the CLI inside. The skill and templates are embedded in the CLI.

**Reason:** Binary and skill versions always match and there is one file to install.

## Free, open source, MIT — 2026-11-06

**Decision:** The whole product is open source under the MIT license and free for everybody.

**Reason:** Public releases make install easy for everyone.

## Direct download — 2026-11-06

**Decision:** The Mac app is a direct, notarized download. No Mac App Store.

**Reason:** The sandbox blocks writing agent tool settings and calling `git`.

## One binary and one image — 2026-11-11

**Decision:** Distribution is one binary per platform and one container image for server mode. There is no app, no `.dmg`, and no installer that writes into agent tools. The README holds the MCP entries and example `AGENTS.md` snippets.

**Supersedes:** "One deliverable" and "Direct download".

**Reason:** The app and the skill installer are dropped (see `docs/architecture-direction.md`).
