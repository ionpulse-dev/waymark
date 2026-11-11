# Decisions: Multi-tenancy

## Tenancy is in the data model from the start — 2026-11-11

**Decision:** Every record and every call carries a tenant ID, even on a single-tenant server.

**Alternatives considered:**
- Add tenancy later. A later migration of the data model and of every check is costly and risky.

**Reason:** Isolation is easiest to build in at the start.

## Git is the source of truth for a tenant — 2026-11-11

**Decision:** The server stores no documents of its own. It holds a cache clone, credentials, settings, and logs.

**Reason:** The owner keeps the memory in their own repository, and deletion on the server never destroys their data.

## Commercial matters are out of scope — 2026-11-11

**Decision:** This spec covers technical isolation and administration only.

**Reason:** Commercial and legal terms are not part of the specification.
