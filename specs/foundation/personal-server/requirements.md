---
type: foundation
status: draft
created: 2026-11-15
updated: 2026-11-15
requires:
  - mcp-server
  - linux-support
---

# Personal server

## Technical Goal

Let one person run waymark as a remote MCP server for their own memory repository, so that chat apps on a phone (Claude, ChatGPT, Gemini) and any other MCP client can use it through one web address. The server keeps no database and no stored state. It works on a normal host and on a serverless host that starts and stops at any time. It is the small, single-owner case of [server-mode](../server-mode/requirements.md). It is not the organization version.

## Dependents

- `server-mode` — the organization version may reuse the sign-in door and the token design of this item.
- `distribution` — the container image of this item is the first part of the image that `distribution` plans.

## Deliverables

- `waymark --mode personal-server`: the MCP server over HTTPS (Streamable HTTP, with TLS provided by the host). One argument sets the transport and the kind of server. The values of `--mode` are `local` (the default, stdio), `personal-server`, and later `team-server`.
- A sign-in door that follows the MCP authorization standard, so that Claude, ChatGPT, and Gemini can connect with no help from their vendors.
- "Sign in with GitHub" as the first identity provider, with an allow-list of exactly one GitHub account. The identity step is a small interface, so other providers can be added later.
- Background Git access through a GitHub App installed on the one memory repository.
- A container image. It holds the Linux build of waymark (see [linux-support](../linux-support/requirements.md)), `git`, and certificates.
- One setup guide for the cloud host and for the GitHub App. It is written so that an agent can follow it and a person can read it as a recipe.
- A small status page at the address of the server: a blue page with the word "Waymark" in white at the center, a line that says that the server is running, and a link to the project on GitHub. It shows nothing about the memory repository or the owner. A separate plain `/health` answer is for the health checks of hosts.
- A safe log on standard output: one line for each start, sign-in, and tool call, with no secret and no document text.

## Scope

**In scope:**
- One owner, one memory repository, one server process.
- Local file storage only for the clone, which is a cache.
- Hosts that keep the process running, and hosts that start it for each request.

**Out of scope:**
- More than one person, roles, and an audit log (see `server-mode`).
- More than one organization (see `multi-tenancy`).
- The web console and any setup page (see Deferred in `todo.md`).
- More than one memory repository on one server.
- Stored state of any kind: a database, a saved disk, or a bucket (see Deferred in `todo.md`).
- A Git host other than GitHub.
- Running a hosted service for others. The project ships the software and the guide.

## Constraints

- **Same core.** Tool handlers, results, errors, and checks are the same as in local mode. Only the outer layer is new.
- **No stored state.** After a restart, a new start, or a second instance, the server works with no data from an earlier run. Git holds the memory. The clone is a cache that can be deleted.
- **Tokens prove themselves.** The tokens that the server gives to MCP clients are signed. The server checks the signature and the expiry and looks nothing up. A signing key comes from the secret store of the host.
- **Waymark issues its own tokens.** A GitHub token is never given to an MCP client. GitHub is used only to learn who the person is.
- **One allowed identity.** Anyone other than the configured GitHub account is refused at sign-in.
- **No Git secret in waymark.** The GitHub App private key and the other secrets come from the secret store of the host, as environment variables or mounted files. They are never in the image, the memory repository, or the MCP entry. Waymark does not write them anywhere.
- **Background Git access is separate from sign-in.** Waymark gets a short Git token from the GitHub App when it needs one. A lapsed client sign-in never stops a pull or a push.
- **The commit author is the signed-in person.** The App credential only proves that the server may write.
- **Writes are pushed at once.** The server may stop after any request, so a write may not wait for a later push.
- **Pull on request.** The server updates its clone when a request arrives and the clone is old. It has no background timer that a serverless host cannot run.
- **One argument for the mode.** `--mode` (or `WAYMARK_MODE`) is the only switch between local use and a server. A value that is not known stops the start with a message that lists the values. The mode selects which settings are required. Settings of another mode stop the start. They are not ignored. The tool list is the same in every mode.
- **HTTPS for clients.** The public URL must start with `https://` (a `localhost` address is allowed for tests). The host provides the certificate. Waymark does not end TLS and listens on `$PORT`, default 8080.
- **The status page is static and public.** It is the same for every instance and every visitor. It uses no script, no external file, no font from outside, and no cookie. It shows no repository name, owner, clone state, or version. It does not wait for the clone.
- **The log holds no secret and no document text.** Never in it: tokens, codes, the signing key, the GitHub App key, client secrets, `Authorization` headers, and document text. Allowed: start facts (mode), a sign-in result with the account name, one line per tool call (time, tool, outcome, document ID), and Git errors. The log goes to standard output, and the host shows it. The server stores no log and has no log page.
- **The data folder is not a setting.** In `personal-server` mode the server uses the temp folder of the system. `--path` is ignored, with one line in the log. The memory repository comes from `WAYMARK_REPO`.
- **First clone as in local mode.** The clone starts at start and the first tool call waits for it, as in `join-memory-repository`. A wait that is longer than the timeout of the app or of the host is the choice of the host and the plan of the person. It is not a rule of waymark.
- **The author needs a name and a verified email.** At sign-in the server accepts only an email that GitHub marks as verified and that passes the email check of `login-and-identity`. The no-reply address of GitHub does not pass. If no email passes, the sign-in is refused and no token is issued.
- **The client config is not a limit.** Limits come from the server config and the shared file, as in `trust-model`.
- **Revoke by key.** A person cuts off all clients by changing the signing key. A single client cannot be revoked in this item.
- **The setup guide is the interface for setup.** Waymark ships no installer and no skill (see `PROJECT.md`).

## Acceptance Criteria

- **Given** a started server, **When** an MCP client connects with no token, **Then** it gets a `401` with the discovery information of the standard.
- **Given** an MCP client that supports automatic registration, **When** it starts the sign-in, **Then** it registers and reaches the GitHub step with no manual input.
- **Given** an MCP client that needs a fixed client ID and secret, **When** the person enters the values that the server gives, **Then** the sign-in works.
- **Given** the owner signs in with the allowed GitHub account, **When** the sign-in ends, **Then** the client gets a token that works only on this server, and never a GitHub token.
- **Given** any other GitHub account, **When** it signs in, **Then** the server refuses and no token is issued.
- **Given** a valid token, **When** the server restarts with an empty data folder, **Then** the same token still works and the first call clones the memory repository again.
- **Given** a token past its expiry, **When** a call arrives, **Then** it is refused, and the client can renew it with the renewal token.
- **Given** a changed signing key, **When** a call arrives with an old token, **Then** it is refused.
- **Given** a write, **When** the tool returns, **Then** the commit is on the remote and its author is the signed-in person.
- **Given** a server limit `access = "read-only"`, **When** a client asks for a write, **Then** the write fails.
- **Given** a stopped and later started process, **When** the first request arrives, **Then** it is served with no data from before.
- **Given** the image, **When** it is inspected, **Then** it holds no secret.
- **Given** a browser, **When** it opens the address of the server, **Then** it shows the blue status page with "Waymark" in white at the center, a line that says that the server is running, and a link to the project on GitHub.
- **Given** the status page, **When** its source is read, **Then** it holds no script, no external file, and no repository name, owner, version, or clone state.
- **Given** a clone that is still running, **When** the status page or `/health` is requested, **Then** it answers at once.
- **Given** a narrow phone screen, **When** the status page opens, **Then** the text is centered, readable, and does not scroll sideways.
- **Given** a run with known secret values (signing key, App key, client secret, a token, and a document text), **When** sign-ins and tool calls happen, **Then** the log contains none of these values.
- **Given** any tool call, **When** it ends, **Then** the log has one line with the time, tool, outcome, and document ID.
- **Given** an empty repository on GitHub with the App installed, **When** the server starts and an agent calls `init` with a name, **Then** the structure and the first commit are on the remote and the same session works with no restart.
- **Given** `--mode` with an unknown value, **When** the server starts, **Then** it stops and lists the values `local`, `personal-server`, and `team-server`.
- **Given** `--mode team-server` before it exists, **When** the server starts, **Then** it stops and says that this mode is not available yet.
- **Given** `--mode personal-server` and a public URL that does not start with `https://`, **When** the server starts, **Then** it stops with a clear message.
- **Given** `--mode personal-server` and no `WAYMARK_OWNER`, **When** the server starts, **Then** it stops and names the missing setting.
- **Given** no `--mode`, **When** the server starts, **Then** it runs as local mode over stdio, as before.
- **Given** `PORT` is set, **When** the server starts, **Then** it listens on that port. **Given** it is not set, **Then** it listens on 8080.
- **Given** a start in `personal-server` mode with `--path`, **When** the server starts, **Then** the option is ignored and the log says so.
- **Given** a GitHub account with a verified email that passes the email check, **When** it signs in and a write runs, **Then** the commit author has that email.
- **Given** a GitHub account whose only emails are unverified, blocked, or the no-reply address, **When** it signs in, **Then** the sign-in is refused, no token is issued, and the message says to add and verify an email on GitHub.
- **Given** `people.email_domains` is set, **When** the verified email is in another domain, **Then** the sign-in is refused.
- **Given** a clone that is still running, **When** a tool call arrives, **Then** it waits for the clone, and after the wait it returns a clear error that tells the agent to call again.
- **Given** the setup guide, **When** an agent follows it step by step, **Then** it stops at each step that only a person can do and does not print a secret in the chat.

## Open Questions

- Which sign-in methods does the normal Gemini app accept for a custom MCP server? Verify against current Google help pages. Community guides say a client ID and a secret are entered by hand.
- Which ChatGPT plans have developer mode for custom MCP servers? The sources disagree.
- Does each app accept signed renewal tokens that do not rotate? ChatGPT needs a renewal token, or it asks the person to sign in again.
- Check on github.com (not only the Enterprise Server pages) the lifetimes of GitHub App user tokens and installation tokens.
- Can the signing key be derived from the GitHub App private key, so that one secret fewer is needed? Is this safe?
- Which hosting service does the first version of the guide cover? A second one can follow.
- How long should the client tokens last? A short access token with a renewal token of weeks is the starting idea.
- Which GitHub App permission reads the email list? Verify on github.com.
- Does the GitHub setting that blocks pushes with a private email also block a push with an App token?
- Does a push of `init` to an empty GitHub repository work with an App token?
- How does the server limit the damage when two instances push at the same time? The plan is pull, rebase, and retry once.
