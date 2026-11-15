# Plan: Personal server

_The technical approach. The requirements are in [requirements.md](requirements.md)._

## Overview

Add an HTTP transport to the existing MCP core, put a small OAuth sign-in door in front of it, and reach Git through a GitHub App. The door signs its own tokens, so the server keeps no state. GitHub only answers "who is this?".

```text
MCP client ⇄ HTTPS (host) ⇄ [ door: discovery · sign-in · tokens ] ⇄ MCP core ⇄ clone (cache) ⇄ GitHub App token ⇄ memory repository
                                    │
                                    └─ sign in with GitHub (identity only)
```

## Components Affected

- `src/main.rs` and the argument parser — a new `--http <address>` option, next to the stdio mode of `mcp-server`.
- The MCP transport layer — serve the same tools over Streamable HTTP.
- `git-sync` — an access mode that gets its credential from the GitHub App, a pull when a request arrives and the clone is old, and a pull, rebase, and retry once after a rejected push.
- `trust-model` — the server config and the environment variables become a limit layer. The client config stays advice.
- `distribution` — the container image. This item adds it, and `distribution` records it as its own deliverable.
- `docs/` and the README — the setup guide and a pointer to it.

## New Files

The module names are a first idea. The task phase fixes them.

- `src/http/mod.rs` — the HTTP server and the routes.
- `src/http/health.rs` — the health page.
- `src/auth/mod.rs` — the sign-in door and its routes.
- `src/auth/discovery.rs` — the metadata that tells a client where and how to sign in.
- `src/auth/registration.rs` — client registration: a client document by URL, automatic registration, and one manual client.
- `src/auth/tokens.rs` — create and check signed codes and tokens.
- `src/auth/identity.rs` — the small interface "who is this person?".
- `src/auth/github.rs` — the GitHub implementation of that interface, with the one-account allow-list.
- `src/git/github_app.rs` — make a short Git token from the App key, and use it for pull and push.
- `Dockerfile` — the container image.
- `docs/personal-server-setup.md` — the setup guide for the cloud host and the GitHub App.
- `tests/personal_server.rs` — end-to-end tests with a fake identity provider.

## Technical Approach

### 1. Transport
`--http` serves MCP over Streamable HTTP on plain HTTP. The host provides TLS. The server needs the public URL as a setting and refuses to start without it, as in `server-mode`. The MCP core does not change.

### 2. Sign-in door
The door follows the current MCP authorization standard. It publishes discovery metadata and answers an unsigned request with `401` and a pointer to that metadata.
- **Registration.** A client may identify itself with a client document at a URL, or register automatically, or use one manual client ID and secret. The manual client is derived from the signing key, so nothing is stored.
- **Flow.** Authorization code with PKCE. The `resource` parameter is carried into the token.
- **Identity step.** The door sends the person to the identity provider. The interface returns a stable ID, a name, and an email. The first provider is GitHub.
- **Allow-list.** One GitHub user. Any other result ends with a refusal page.

### 3. Tokens without stored state
All codes and tokens are signed with a key from the environment.
- **Authorization code.** Short expiry (minutes). It holds the client, the PKCE challenge, and the identity.
- **Access token.** Short expiry (about an hour). It holds the person, the audience (this server), and the limits that apply.
- **Renewal token.** Long expiry (weeks). It does not rotate in this item. A stolen one works until it expires, so the lifetime stays moderate.
- **Revoke.** Change the signing key. Every token stops working.
- **Replay of a code.** A code is single-use in principle. With no state, a short expiry and the PKCE check limit the risk. This is a known limit and it is noted in `decisions.md`.

### 4. Git access
A GitHub App is installed on the one memory repository.
- The server signs a short statement with the App private key. GitHub returns an installation token for that repository. The token lasts about an hour and is made again when needed. Nothing is stored.
- Pull and push use that token. The commit author comes from the signed-in identity.
- The sign-in uses the same App for its login step (client ID and client secret). The GitHub user token is read once for the identity and then dropped.

### 5. No state, and serverless
- The clone is a cache in the data folder. On a start with an empty folder, the first request clones (shallow by default).
- A request pulls first when the clone is older than a short limit.
- A write commits and pushes before it returns. If the push is rejected, the server pulls, rebases, and tries once more. If that fails, the tool returns an error that tells the agent what happened.
- The per-session read budget lives in the process and resets on a new start. This is accepted for a personal server.

### 6. Settings
All values come from environment variables or mounted files. The names are proposals.

| Value | Variable |
|---|---|
| Public URL | `WAYMARK_PUBLIC_URL` |
| Memory repository | `WAYMARK_REPO` |
| Allowed GitHub account | `WAYMARK_OWNER` |
| Signing key | `WAYMARK_SIGNING_KEY` |
| GitHub App ID, installation number, private key | `WAYMARK_GITHUB_APP_ID`, `WAYMARK_GITHUB_INSTALLATION_ID`, `WAYMARK_GITHUB_APP_KEY` |
| GitHub App client ID and secret (sign-in) | `WAYMARK_GITHUB_CLIENT_ID`, `WAYMARK_GITHUB_CLIENT_SECRET` |

### 7. Setup guide
One Markdown file for a cloud host and for the GitHub App. Each step has three parts: what to do, what you should see, and what to do if you do not. Steps that only a person can do are marked. The guide lists the exact names of every secret, says never to print a secret in the chat, and ends with a check: open the address, add the connector, ask one question. A guide for each app (Claude, ChatGPT, Gemini) lists the plan and the steps to add the connector.

## Dependencies on Other Foundation Items

- `mcp-server` — provides the core and the tools. It is `implemented`.
- `git-sync` — provides the clone, the lock, and the push. This item adds a credential source and a pull-on-request rule. Needs a check that its local-mode text allows this.
- `trust-model` — provides the limit layers. This item adds the server config as a layer.
- `distribution` — owns the release of the image. Needs a short update to its text.

## Risks

- **Vendor behavior.** The apps change often. Gemini and the ChatGPT plans are not verified. The plan includes a test of each app against a dummy door before the code is final.
- **Signed tokens without revoke.** A stolen renewal token works until it expires. The lifetime and the one-key revoke limit the damage. The decision is in `decisions.md`.
- **Two instances.** Two writers at once can reject each other's push. The retry covers the normal case.
- **Cold start.** A large memory repository clones slowly. A shallow clone helps. A very large one is a problem for the serverless case.
- **Public exposure.** The server is on the open internet with one allowed account. The door and the token checks are the only protection, so they need a security review (see `docs/security-review.md`).
- **Scope.** Server mode is out of scope for the current task (see [project decisions](../../../docs/decisions.md)). This item stays `draft` until the product owner changes that.

## Open Questions

- Which Rust crates fit for HTTP, token signing, and the GitHub App call? Prefer few dependencies.
- Does the MCP library of `mcp-server` already support Streamable HTTP?
- What is the exact current MCP authorization standard that the door must follow? Check it when the task phase starts.
