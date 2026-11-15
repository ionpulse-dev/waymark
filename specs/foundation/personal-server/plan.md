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
- `src/mcp/context.rs` and `src/mcp/binding.rs` — in HTTP mode the data folder is fixed (the temp folder of the system) and `--path` is ignored. The clone on first use and the background clone at start stay as they are (see `join-memory-repository`).
- `src/identity/author.rs` — in HTTP mode the commit author comes from the sign-in identity and not from the Git config of a person's computer.
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
- The clone is a cache in the data folder. The data folder is fixed: the temp folder of the system, which is the writable place on every host. `--path` is ignored in HTTP mode, with one line in the log.
- The clone logic of local mode is reused as it is. The server starts to clone in the background at start. The first tool call that needs the repository waits for the same clone through the existing lock (`ctx.repo()`), up to `WAYMARK_CLONE_WAIT_SECONDS` (default 60). After that it returns "The first clone is still running. Call again in a minute." No tool checks for the clone on its own. The first clone is shallow by default.
- The sign-in door never needs the clone, so sign-in is fast on a cold start.
- The wait of the AI app and the maximum time of the host are not under our control. A host or an app that waits less than the clone needs will fail the first call. The guide says so.
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

### 7. First use: an empty repository and `init`
The person creates an empty private repository on GitHub and installs the GitHub App on it. After the server starts, the clone of the empty repository gives the state `empty`. `status` says that `init` can set it up (see `create-memory-repository`). The person asks the agent to set up the memory repository. The agent calls `init`, which creates the structure, commits, and pushes with the App token. The same session then works with no restart. `init` is a write tool, so the limits and the approval of the client apply as for any write. The tool does not change.

### 8. Commit author
Every write, and `init` too, needs an author name and an email. In HTTP mode both come from the identity step. If the GitHub account hides its email, the identity step returns the no-reply address of GitHub (`<id>+<login>@users.noreply.github.com`). The email check of `login-and-identity` must accept it. This is checked in the task phase.

### 9. Setup guide
One Markdown file for a cloud host and for the GitHub App. The order is: create the empty repository, deploy the server, create the GitHub App and install it on that repository, set the secrets, connect an app, and ask the agent to run `init`. Each step has three parts: what to do, what you should see, and what to do if you do not. Steps that only a person can do are marked. The guide lists the exact names of every secret, says never to print a secret in the chat, and ends with a check: open the address, add the connector, ask one question. A guide for each app (Claude, ChatGPT, Gemini) lists the plan and the steps to add the connector.

## Dependencies on Other Foundation Items

- `mcp-server` — provides the core and the tools. It is `implemented`.
- `git-sync` — provides the clone, the lock, and the push. This item adds a credential source and a pull-on-request rule. Needs a check that its local-mode text allows this.
- `trust-model` — provides the limit layers. This item adds the server config as a layer.
- `distribution` — owns the release of the image. Needs a short update to its text.

## Risks

- **Vendor behavior.** The apps change often. Gemini and the ChatGPT plans are not verified. The plan includes a test of each app against a dummy door before the code is final.
- **Signed tokens without revoke.** A stolen renewal token works until it expires. The lifetime and the one-key revoke limit the damage. The decision is in `decisions.md`.
- **Two instances.** Two writers at once can reject each other's push. The retry covers the normal case.
- **Cold start.** A large memory repository clones slowly. A shallow clone helps. A very large one is a problem for the serverless case. A wait that is longer than the timeout of the app or of the host makes the first call fail, and the next call works.
- **Empty remote.** The push of `init` to an empty GitHub repository, with an App token, must be tested. The spec of `init` says the branch is whatever the clone has.
- **Hidden email.** A GitHub account with a hidden email needs the no-reply address as the author.
- **Public exposure.** The server is on the open internet with one allowed account. The door and the token checks are the only protection, so they need a security review (see `docs/security-review.md`).
- **Scope.** Server mode is out of scope for the current task (see [project decisions](../../../docs/decisions.md)). This item stays `draft` until the product owner changes that.

## Open Questions

- Which Rust crates fit for HTTP, token signing, and the GitHub App call? Prefer few dependencies.
- Does the MCP library of `mcp-server` already support Streamable HTTP?
- What is the exact current MCP authorization standard that the door must follow? Check it when the task phase starts.
- Does the email check of `login-and-identity` accept the no-reply address of GitHub?
- Does the MCP library of `mcp-server` serve both stdio and HTTP with the same `Server` handler, so that the clone logic is shared with no change?
