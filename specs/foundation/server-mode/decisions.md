# Decisions: Server mode

## One binary for both modes — 2026-11-11

**Decision:** Server mode is `waymark --http`, the same binary as local mode. There is no separate server program.

**Alternatives considered:**
- A separate server project. It would copy the core and drift from it.

**Reason:** One core, one conformance suite, and a quick start that grows into the shared server.

## A shared server for organizations, not one server per person — 2026-11-11

**Decision:** Server mode serves an organization with many people. Each person signs in and is the commit author of their own writes.

**Alternatives considered:**
- One server per person. It does not fit a company, and each person would have to run a server.

**Reason:** Central limits, single sign-on, one Git credential for an organization, and an audit log need a shared server.

## Limits on the server, not in the client — 2026-11-11

**Decision:** In server mode the server config and the shared file hold the limits. The client config is never a limit.

**Reason:** The user controls the client config.

## The push credential and the author are separate — 2026-11-11

**Decision:** The server pushes with its own credential. The commit author is the signed-in person.

**Reason:** The history stays correct whatever the credential is. The host shows the credential as the pusher, which is an honest, small difference.

## Secrets from the host, no token page at first — 2026-11-11

**Decision:** The first release takes credentials from the secret store of the host. A page where an owner connects a Git account is a later stage.

**Alternatives considered:**
- A connect page at once. It needs encrypted storage, key handling, and a larger attack surface.

**Reason:** Keep the first release small and the stored secrets at zero.
