# Decisions: Trust model

_Earlier decisions of this item are in the history of this file in Git and in `docs/decisions.md`._

## The default of `access` stays `read-write` — 2026-11-14

**Decision:** The built-in default of `access` is `read-write`. A team or a person who wants less sets `read-only` in a layer.

**Alternatives considered:** `read-only` by default (rejected: reads are always allowed, and a person who installs waymark expects agents to write; a strict team sets the shared limit).

**Reason:** The first-run experience matters, and every write is attributed, reversible, and checked. Decided autonomously.

## Layer files are never ignored when broken — 2026-11-14

**Decision:** A layer file that exists but cannot be read or parsed fails the call with `setup_problem`.

**Alternatives considered:** Ignore the broken file (rejected: it would loosen a limit by accident).

**Reason:** A limit must never get looser by error. Decided autonomously.

## The system file path can be overridden in tests — 2026-11-14

**Decision:** The environment variable `WAYMARK_SYSTEM_CONFIG` replaces the path of the system file.

**Alternatives considered:** A build flag for tests only (rejected: tests must run the release binary too).

**Reason:** Tests cannot write to `/Library`. The variable is a layer under the control of the person, which is the same strength as the personal file, so it never loosens a stricter system file: if the real system file exists, both are read. Decided autonomously.
