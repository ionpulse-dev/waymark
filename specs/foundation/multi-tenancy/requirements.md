---
type: foundation
status: draft
created: 2026-11-11
updated: 2026-11-11
requires:
  - server-mode
---

# Multi-tenancy

## Technical Goal

Let one running server serve many separate organizations (tenants) and private people, with strict isolation, so that the same software can run as a self-hosted server for one organization or as a shared server for many. This is the second stage of server mode.

## Dependents

- A provider who hosts many organizations. No other spec depends on it.

## Deliverables

- The tenant model: tenant, user, knowledge base, role.
- Isolation of data, credentials, limits, and logs per tenant.
- Provisioning of a tenant.
- Per-tenant sign-in.
- Per-tenant limits and quotas.
- Export and deletion of a tenant.

## Scope

**In scope:**
- Isolation and administration of many tenants on one server.

**Out of scope:**
- Commercial and legal matters. This spec covers technical isolation and administration only.
- Sharing a knowledge base across tenants.

## Constraints

- **Isolation first.** No tool call, log line, cache, or error may show data of another tenant. Every call carries the tenant ID from the token.
- **Git stays the source of truth.** A tenant's data is its Git repository. The server holds only a cache clone, and the stored credentials and settings of the tenant.
- **Separate clones.** Each tenant has its own folder. A path outside it is refused.
- **Separate credentials.** Each tenant has its own Git credential. The server never uses one tenant's credential for another.
- **Stored secrets are encrypted at rest.** The key comes from the environment or a key service, never from the data folder.
- **A single-tenant server is a tenant count of one.** The tenant model is in the data model from the first release of `server-mode`, so no later migration changes it.
- **Limits per tenant and per user** follow `trust-model`. A server limit applies to all tenants and wins.
- **Sign-in per tenant.** A tenant can bring its own OIDC provider. A private person can use a Git host login.

### Provisioning

A server administrator creates a tenant through the web console or an admin API. Steps: name, sign-in provider, knowledge bases (repository URL), Git credential (GitHub App installation), roles, limits.

### Quotas

Per tenant: knowledge bases, users, calls per minute, and clone size. A quota failure is `access_denied` with a plain message.

### Export and deletion

- Export: the tenant's Git repository is already theirs. The export adds the audit log and the settings.
- Deletion: removes the clone, stored credentials, settings, and logs of the tenant. The Git repository of the tenant is not touched.

## Acceptance Criteria

- **Given** two tenants, **When** a user of tenant A calls a tool with an ID from tenant B, **Then** the call fails and reveals nothing about B.
- **Given** two tenants with documents of the same ID, **When** both search, **Then** each sees only their own results.
- **Given** a path that leaves the tenant folder, **When** a tool receives it, **Then** it is refused.
- **Given** a tenant, **When** it is deleted, **Then** no file, secret, or log of it remains on the server, and its Git repository is unchanged.
- **Given** an audit log, **When** a tenant owner reads it, **Then** it holds only that tenant's lines.
- **Given** a server limit, **When** a tenant sets a looser limit, **Then** the server limit applies.
- **Given** a data folder backup, **When** it is inspected, **Then** no stored secret is readable without the key.

## Open Questions

- Is one process per tenant needed for stronger isolation, or is one process with strict checks enough?
- Which key service options does the first release support?
- How are rate limits shared when one provider runs many tenants?
