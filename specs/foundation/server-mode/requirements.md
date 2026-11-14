---
type: foundation
status: draft
created: 2026-11-11
updated: 2026-11-11
requires:
  - mcp-server
---

# Server mode

## Technical Goal

Run the same binary as a shared MCP server over HTTPS, so that a whole organization, and chat apps and phones, can use one memory repository through one address. Anyone can host it: a company on its own infrastructure, or a provider for others. The core and the tools are the same as in local mode. This item adds the transport, the login, the roles, the Git credentials, the workspace, and the audit log, for one organization. `multi-tenancy` adds many organizations.

## Dependents

- `multi-tenancy`, `web-console`, and `write-approval` (console approval).
- `login-and-identity` (server sign-in) and `distribution` (container image).

## Deliverables

- `waymark --http <address>`: the MCP server over HTTPS (Streamable HTTP).
- Sign-in with OAuth 2.1 and single sign-on (OIDC).
- Roles and the limits per user or role.
- Git credentials for the server (GitHub App, deploy key, or token).
- The workspace: one clone per memory repository, a lock, and a background pull.
- The audit log.
- A health endpoint and logs for operators.
- A container image and a deploy guide.

## Scope

**In scope:**
- One organization with one or more memory repositories on one server.

**Out of scope:**
- Many organizations on one server (see `multi-tenancy`).
- The web pages for people (see `web-console`).
- Running a hosted service for others. The project ships the software and the guide. It does not decide whether anyone runs a service.

## Constraints

- **Same core.** Tool handlers, results, errors, and checks are the same as in local mode. Only the outer layer differs.
- **The client config is not a limit.** The user controls it. Limits come from the server config and the shared file (see `trust-model`).
- **No secret in the image, the repository, or the MCP entry.** Secrets come from the secret store of the host, as environment variables or mounted files.
- **TLS** is the job of a reverse proxy or the platform. The server speaks plain HTTP behind it and refuses to start without a configured public URL.
- **Git is the source of truth.** The server holds a cache clone. The data folder can be deleted and rebuilt from Git, except the audit log and stored secrets (see below).
- **Commit author is the signed-in person.** The push credential only proves that the server may write.

### Sign-in

- The server is an OAuth 2.1 resource server for MCP clients. It follows the MCP authorization flow (discovery metadata, PKCE, and client registration). The exact requirements are checked against the current MCP authorization specification when this item is planned.
- People sign in through an OIDC provider that the operator configures (a company directory or a Git host login). Chat apps start the same flow in a browser.
- The token identifies the person: stable ID, name, and email. The email goes through the check of `login-and-identity`.

### Roles

| Role | May |
|---|---|
| `reader` | Read tools |
| `writer` | Read and write tools |
| `owner` | Also verify documents, see all activity, and use the console settings |

The operator maps directory groups to roles for each memory repository. The role limits the tool list. A tool that the role does not allow is not listed.

**Option for later: separate connections for each permission level.** A server could offer more than one MCP connection, for example `read` and `write`, each with its own sign-in and its own OAuth scope. The person signs in to the level they want, in a browser, and the agent cannot change it. The agent tool can then also treat each connection differently (always allow the read one, ask for the write one). The hard limit comes from the sign-in and not from a file. This is not decided. It is recorded here so that it is not forgotten. Local mode does not need it (see `trust-model`).

### Git credentials of the server

| Credential | Notes |
|---|---|
| GitHub App installed on the organization | Preferred. Short-lived tokens, scoped to chosen repositories. |
| Deploy key (SSH) for one repository | Simple. Read and write for one repository. |
| Fine-grained token | Tied to one person. It expires. |

The credential comes from the secret store of the host. A later stage may offer a page where an owner connects a Git account. The server would then store tokens encrypted at rest, with a key from the environment. That stage is not part of the first release.

### Workspace

- One clone per memory repository, in the data folder.
- One write lock per clone (see `git-sync`).
- A background pull at an interval, default 5 minutes.
- The result of a write is pushed at once.

### Audit log

Every tool call writes one JSON line: time, person ID, memory repository, tool, outcome, and the document ID. It contains no document text and no secret. The operator chooses where it goes (standard output or a file).

### Config

The server reads one config file and environment variables: public URL, OIDC provider, memory repositories and their roles, credentials, server limits, and the data folder.

## Acceptance Criteria

- **Given** a started server, **When** an MCP client connects without a token, **Then** it gets a `401` with the discovery information.
- **Given** a signed-in `reader`, **When** the client lists tools, **Then** only read tools show.
- **Given** a signed-in `writer`, **When** `add` runs, **Then** the commit author is that person and the push uses the server credential.
- **Given** a server limit `access = "read-only"`, **When** a person sets `read-write` in their own client config, **Then** writes still fail.
- **Given** a restart with an empty data folder, **When** the first call arrives, **Then** the server clones again and works.
- **Given** any tool call, **When** it ends, **Then** one audit line exists and it holds no document text.
- **Given** two writers on one memory repository, **When** they write at the same time, **Then** both commits reach the remote and neither is lost.
- **Given** a secret in the config file, **When** the image is inspected, **Then** no secret is inside.

## Open Questions

- Should the server offer separate connections or scopes for read and write, besides roles? What do the chat apps of Claude and ChatGPT support for scopes?
- Which login flows do Claude and ChatGPT accept for a self-hosted MCP server today? Verify against their documentation.
- SAML or OIDC only for the first release?
- Does the server need its own small database for sessions and approvals, or is a file store enough?
- How long do approval requests wait (see `write-approval`)?
- Should the server offer a read-only public mode?
