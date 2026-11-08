# Decisions: Distribution

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## One deliverable — 2026-11-06

**Decision:** The release is `Waymark.app` with the CLI inside. The skill and templates are embedded in the CLI.

**Reason:** Binary and skill versions always match and there is one file to install.

## Free, open source, MIT — 2026-11-06

**Decision:** The whole product is open source under the MIT license and free for everybody.

**Alternatives considered:**
- Paid apps, paid features, or open core — against adoption and against the open source goal.

**Reason:** Public releases make install easy for everyone.

## Direct download — 2026-11-06

**Decision:** The Mac app is a direct, notarized download. No Mac App Store.

**Reason:** The sandbox blocks writing agent tool settings and calling `git`.
