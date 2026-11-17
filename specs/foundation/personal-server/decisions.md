# Decisions: Personal server

_Records why technical choices were made. Never delete entries — add new ones as decisions are made._

---

## A small single-owner slice, not the organization version — 2026-11-15

**Decision:** The first remote version serves one person with one memory repository. It does not have roles, tenancy, an audit log, or a web console. It is separate from `server-mode`.

**Alternatives considered:**
- Build `server-mode` as written — rejected: single sign-on, roles, and a console are too heavy for one person, and they delay a usable remote server.
- A hidden server on a private network — rejected: the apps of Claude, ChatGPT, and Gemini connect from the cloud of their vendor, so the server must be public.

**Reason:** The goal is the easiest remote use for a technical person, mainly from phone apps.

## Waymark is its own OAuth door, and GitHub only proves identity — 2026-11-15

**Decision:** The apps talk to the OAuth door of waymark. Waymark sends the person to GitHub only to learn who they are. It issues its own tokens. It never passes a GitHub token to a client.

**Alternatives considered:**
- Connect the apps straight to the GitHub login — rejected: the apps need automatic client registration, and GitHub's login probably does not offer it (not confirmed).
- Pass the GitHub token through — rejected: the client would hold a token for the repository, and waymark could not tell its sessions apart.
- A static token in a header only — rejected: it works for some clients and not for Gemini, which needs OAuth.

**Reason:** One sign-in method that works for every major app, with the repository protected.

## No stored state — 2026-11-15

**Decision:** The server stores nothing between runs. Codes and tokens are signed with a key from the environment, so they prove themselves. Git holds the memory. The only secrets live in the secret store of the host.

**Alternatives considered:**
- A database — rejected: extra cost and an extra service.
- A saved disk — rejected: not every host offers it, and it does not work for serverless hosts.
- A bucket (S3 or Cloudflare R2) — rejected for this item: its own key must still be pasted into the host, so it saves no step. It becomes useful when waymark must remember things after setup (see Deferred in `todo.md`).

**Reason:** It works on a normal host and on a serverless host, and it costs nothing extra.

**Consequence:** A single client cannot be revoked. A person changes the signing key and every client signs in again. A code cannot be made strictly single-use. A short expiry and PKCE limit the risk. This is acceptable for one person.

## Background Git access uses a GitHub App installation token — 2026-11-15

**Decision:** Waymark gets Git access from a GitHub App installed on the one memory repository. It makes a short installation token from the App private key when needed. It does not use the user token of the sign-in.

**Alternatives considered:**
- The user token of the sign-in — rejected: it lasts 8 hours and needs a renewal token that waymark must store and update.
- An OAuth App — rejected: its token reaches every private repository of the person.
- A deploy key — kept as a fallback: it never expires, but it cannot do the sign-in, so the GitHub App values are still needed for it.

**Reason:** Nothing expires on the side of the person, nothing needs storing, and a leak reaches one repository.

**Note:** The lifetimes of GitHub App tokens are checked only against Enterprise Server pages so far (open question in `requirements.md`).

## One setup guide for agents and people — 2026-11-15

**Decision:** Setup is one Markdown guide, written for an agent to follow. A person can read it as a recipe. Waymark ships no installer, no skill, and no setup page in this item.

**Alternatives considered:**
- A setup page on the server — deferred: it needs a one-time code from the log so that nobody else can claim a new server, and its own bucket keys still need a safe place.
- A command that runs the GitHub App manifest flow — deferred (a possible `waymark-doctor`).

**Reason:** `PROJECT.md` puts installers out of scope, most people will use an agent anyway, and one guide is the least to keep correct.

## Local clone only, in a fixed folder — 2026-11-15

**Decision:** The server works only on a local clone. It does not use the GitHub API for reads or writes. In HTTP mode the data folder is the temp folder of the system and is not a setting. `--path` is ignored. The memory repository comes from `WAYMARK_REPO`.

**Alternatives considered:**
- GitHub API only, with no clone — rejected: search, the quality checks, move, and undo are built on local Git, and the API would tie waymark to GitHub. It could return later as a second backend behind one interface.
- A data folder setting — rejected: every host has a writable temp folder, so the setting only adds a way to make a mistake.

**Reason:** One code path for local and HTTP mode, and nothing for the person to configure.

## First clone is reused from local mode — 2026-11-15

**Decision:** HTTP mode reuses the clone logic of local mode: a clone at start in the background, one lock, and a wait in `ctx.repo()` with `WAYMARK_CLONE_WAIT_SECONDS`. No tool checks for the clone on its own. The sign-in door does not need the clone.

**Alternatives considered:** A new lazy-start design for HTTP only — rejected: the existing design already covers it, and a second one adds concurrency to test.

**Reason:** The host decides how often a cold start happens. The person chooses a cheap serverless host (longer first call) or an always-on server. Waymark only has to work correctly in both. It cannot control the timeout of an AI app or of a host.

## `init` runs through the agent, and the author comes from the sign-in — 2026-11-15

**Decision:** A new memory repository is set up by the agent with the existing `init` tool, on an empty repository that the person created on GitHub. The tool does not change. In HTTP mode the commit author comes from the sign-in identity.

**Alternatives considered:** A separate setup step for `init` — rejected: the tool already handles the empty state and the same-session switch to `ready`.

**Reason:** Nothing new to build for first use, and the author rule stays the same: the commit belongs to the person.

## The author email must be verified by GitHub — 2026-11-15

**Decision:** At sign-in the server reads the email list of the GitHub account and accepts only an address that GitHub marks as verified and that passes the email check of `login-and-identity`. If none passes, the sign-in is refused. The no-reply address of GitHub stays blocked.

**Alternatives considered:**
- Use the no-reply address of GitHub when the email is hidden — rejected: `login-and-identity` blocks it on purpose, because a commit email must reach a person. An earlier draft of this spec said the opposite. This entry corrects it.
- Syntax and blocklist checks only, as in local mode — rejected for HTTP mode: the server has the GitHub identity at hand, so a stronger check costs nothing.
- An MX record check — not needed: a verified address is stronger.

**Reason:** In local mode the author comes from the Git config of a computer, and waymark can only check the form. In HTTP mode GitHub can say that the person confirmed the address. This builds the deferred item "the check against the verified emails of the Git host account" in `login-and-identity`.

**Open:** The permission that reads the email list, and whether the GitHub setting against private-email pushes applies to a push with an App token.

## One `--mode` argument: `local`, `personal-server`, `team-server` — 2026-11-15

**Decision:** One argument, `--mode` (or `WAYMARK_MODE`), selects both the transport and the kind of server. The values are `local` (the default, stdio, as today), `personal-server` (this spec), and `team-server` (later, the work of `server-mode`). A server mode needs `--mode`. Its settings are required or refused by the mode. Earlier entries of this file say "HTTP mode". They mean `personal-server` mode.

**Alternatives considered:**
- `--http <address>` plus a second option for the kind — rejected: two options that depend on each other, and invalid combinations.
- A mode named `personal` — rejected: it says who uses it, and nothing about being online.
- `remote` and `team`, or `online` and `shared` — rejected: they mix "where" and "who". `personal-server` and `team-server` name the audience in the same way, and "server" says it is online.
- `single server` — rejected: it reads as one server instance, not one user.
- A default of `personal-server` for a server — not needed: `local` is the default, and a server mode is always chosen on purpose, so a later `team-server` breaks nothing.

**Reason:** The mode decides which settings are needed and which are refused, so that a server on the open internet never starts with guessed settings. A later, more complex setup can be added as a new value with no change for the existing ones.

**Listen address and HTTPS:** The listen address is `0.0.0.0:$PORT` (default 8080). The connection from the app is HTTPS, with the certificate from the host. Waymark speaks plain HTTP behind it and does not end TLS. It refuses a public URL that does not start with `https://`. This follows `server-mode`.

## A small status page, a plain health answer, and a safe log — 2026-11-15

**Decision:** The address of the server shows a static status page: blue background, "Waymark" in white at the center, a line that the server is running, and a link to the project on GitHub. `/health` gives a plain `ok` for hosts. The server writes a safe log to standard output. It has no log page and stores no log.

**Alternatives considered:**
- Plain text for the page — rejected: the page is the first thing a person sees after setup, and a small design costs little.
- A richer public page (version, clone state, repository) — rejected: it is public, so it would tell a stranger what runs here. The `status` tool already tells the owner through the agent.
- An owner-only log page — deferred: it needs a browser sign-in, and with no stored state it shows only one instance.
- Webfonts, a script, or an image from outside — rejected: a page with no outside file never breaks, tracks no visitor, and needs no loosened security policy.

**Reason:** A page that looks cared for builds trust at no cost. A public page must stay dumb. The log is for the operator, and the host already shows it, so the work is in what the log may never contain. A test with known secret values proves it.

**Note:** The page names waymark. A visitor can learn that waymark runs at the address. This is accepted because the address is meant for apps, and the page gives no way in.

## Linux support comes first, as its own spec — 2026-11-15

**Decision:** This spec requires [linux-support](../linux-support/requirements.md). It does not start until that spec is `implemented`. The container image is built on the tested Linux build of that spec.

**Alternatives considered:** A Linux build only inside the `Dockerfile` of this spec — rejected: the code and the tests would not be proven on Linux, and Linux also matters for local mode.

**Reason:** Servers run Linux, and the project supports macOS only today. See the decisions of `linux-support`.

## Build on best knowledge and verify in real use — 2026-11-15

**Decision:** The server is built from the best current knowledge of the AI apps and of GitHub. It is tested in real use after the build. It is not tested against every vendor in advance. The parts that depend on an assumption are small and separate: three client registration methods from the start, all token lifetimes as settings in one place, and the identity step behind its interface. Automated tests use a fake identity provider and a fake Git host.

**Alternatives considered:**
- Test every app and the GitHub details before building — rejected: the project has one maintainer, and some tests need paid plans, admin access, and a real GitHub App.
- Build only the methods that are certain — rejected: a missing registration method locks out a whole app, and each one is small.

**Reason:** The unknowns concern vendor behavior and GitHub details. They touch the sign-in door and the Git credential code, not the MCP core. A wrong assumption is then a fix in one module (see the table in `plan.md`). A rewrite is unlikely.

**Remaining risk:** An assumption that breaks the sign-in design as a whole would change the door and the credential code. This is judged unlikely.

## Assumptions to test in real use — 2026-11-15

Research of 2026-11-15 was done from documentation only. Nothing was tested. Each line is an assumption, with its source and the module that changes if it is wrong. The list is the checklist of the real-use pass in `test.md`. When an assumption is tested, add a new entry with the result. Do not edit this one.

- **Registration.** The MCP standard (revision 2026-07-28) prefers client documents at a URL and keeps automatic registration for older clients. Assumed: Claude and ChatGPT use one of the two. Gemini Enterprise and the `agy` CLI need a manual client ID and secret. The consumer Gemini app is not documented. Module: `registration.rs`.
- **ChatGPT plans.** OpenAI names Business and Enterprise/Edu on the web. Third-party pages also name Plus and Pro. Assumed: the OpenAI page. Module: the guide.
- **Renewal tokens.** Assumed: every app accepts a signed renewal token that does not rotate. `agy` refreshes tokens by itself. ChatGPT and Gemini are not documented. Module: `tokens.rs`.
- **Lifetimes.** Assumed: access token about 1 hour, renewal token about 30 days (settings). GitHub user tokens last 8 hours and refresh tokens about 6 months (Enterprise Server pages only). The installation token is assumed to last about 1 hour, and the code renews on any `401`. Module: `tokens.rs`, `github_app.rs`.
- **Email list.** Assumed: the "Email addresses" user permission (read) lets a GitHub App user token call `GET /user/emails`. The documentation states it for fine-grained tokens. Module: `github.rs`, the guide.
- **Private-email push block.** Not researched. Assumed: it does not block a push with an App token. Module: `author.rs`, the guide.
- **Empty repository.** Not researched. Assumed: `init` can push the first commit to an empty GitHub repository with an App token. Module: `github_app.rs`.

## CLIs accept a static token, so a token for developers is deferred — 2026-11-15

**Decision:** The first version has no static or long-lived token for CLIs. The sign-in door is the only way in. A signed long-lived token for CLI clients is a deferred item.

**Alternatives considered:** A static token now — rejected: it adds a second way in to a server on the open internet, and every phone app needs the door anyway.

**Reason:** Codex CLI, the `agy` CLI, and Gemini CLI all accept a static `Authorization` header, so a token would suit developers. It must not weaken the one-account rule, so it needs its own design. See `todo.md`.

## An exception to "waymark never stores a Git secret" is not needed — 2026-11-15

**Decision:** Because the server stores nothing and the private key stays in the host's secret store, the principle "Secrets stay with Git — waymark never stores or passes a Git secret" holds unchanged.

**Reason:** A later version with storage would need a new decision.

## The build: crates and the shape of the server — 2026-11-15

**Decision:** The HTTP side uses `axum` (routes) and the `rmcp` Streamable HTTP service (the same `Server` handler as stdio, so the tools and the clone logic are shared with no change). Calls out of the server (GitHub, client documents) use `ureq`, a small blocking client, on blocking threads. The tokens are `payload.signature` with HMAC-SHA256 (`hmac`, `sha2`), built by hand. The statement of the GitHub App is an RS256 token made with `jsonwebtoken`.

**Alternatives considered:** `reqwest` (async, but a larger tree of dependencies for four small calls); a JWT library for our own tokens (more features than we need, and a format with known mistakes to avoid, such as the `alg` header); an HTTP framework other than `axum` (`rmcp` already builds on `tower`, so `axum` fits).

**Reason:** Few dependencies, one runtime, and the Git code of waymark is blocking already.

## The MCP service is stateless — 2026-11-15

**Decision:** The MCP service runs with `legacy_session_mode: false` and a session manager that keeps no sessions. Every request stands alone. The host check for local servers (DNS rebinding) is off, because the token protects the server and a public server must accept its own host name behind any proxy.

**Alternatives considered:** A session per client with the local session manager (rejected: it is state in the process, so a restart or a second instance breaks open sessions); the host check on with the public host name (rejected: a proxy that rewrites the `Host` header would block every call with a message that is hard to understand).

**Reason:** "No stored state" applies to MCP sessions too. A test restarts the process with an empty data folder and the same token works. **Consequence:** A client cannot receive messages that the server starts by itself (such as an approval question during a call), because there is no session to carry them. The first version uses no approval by default.

## Details of the door — 2026-11-15

**Decision:**
- The renewal token does not rotate. A renewal returns the same renewal token. The MCP standard says that an authorization server must rotate renewal tokens for public clients. We cannot, because rotation needs state. The lifetime is 30 days and a changed signing key cuts off everything. **Assumed, test in real use** (apps may refuse it).
- The access token carries the name and the verified email of the person. So the commit author comes from the token with no lookup. The server sets the author from the last valid token. This is right for one owner.
- The audience of every token is `<public URL>/mcp`. A request to the door with a `resource` that is neither the server nor its MCP address is refused.
- The client documents are fetched only from `https` addresses that do not point to a private network (a check against SSRF). A document address on `localhost` is allowed only when the server itself is on `localhost`, which is for tests. A check at fetch time still leaves a gap for a changed DNS answer between the check and the fetch. The risk is accepted: the fetch result is never shown, and it holds no secret.
- Automatic registration returns a client ID that is a signed value with the redirect addresses and a very long expiry. It has no secret.
- The manual client ID and secret come from the signing key. `waymark --print-client` prints them, so that nothing has to be stored or shown by the server.
- Authorization codes cannot be single-use with no state. The expiry is short and PKCE is required (the limit that this spec named).
- The sign-in checks the email domain list (`people.email_domains`) of the memory repository only when the clone is ready. The door must not wait for the clone. A domain check also runs at the first write (`checked_author`), so a wrong address never makes a commit.

**Reason:** The smallest door that follows the standard and keeps no state.

## Git credential through the environment — 2026-11-15

**Decision:** The installation token goes to `git` only through environment variables of the child process (`GIT_CONFIG_COUNT`, with `http.<url>.extraheader`). It is never on a command line, on disk, or in the log. It is scoped to the host of `WAYMARK_REPO`. If `git` says that the token was refused, waymark makes a new token and runs the command once more. This needs `git` 2.31 or newer (the image has 2.34).

**Alternatives considered:** A token in the URL (rejected: git writes it to `.git/config`); `-c http.extraheader=...` on the command line (rejected: visible in the process list); a credential helper script (rejected: more moving parts, and it needs a file).

**Reason:** No secret on disk or in a process list, and the retry covers any token lifetime (the assumed hour is not relied on).

## One retry after a rejected push, in every mode — 2026-11-15

**Decision:** When a push is rejected, the write pulls (rebase) and pushes once more, in local mode and in server mode. If the second push fails, the tool returns the error. Before, local mode returned the first error and kept the commit locally.

**Alternatives considered:** The retry only in server mode (rejected: the same race exists in local mode, and one code path is easier to test).

**Reason:** The server may stop after any request, so a write may not wait for a later push.

## The first guide is host-neutral — 2026-11-15

**Decision:** The setup guide names no hosting service. It lists the settings, the GitHub App steps, and what the host must provide (a container, secrets, HTTPS, a health check). A guide for one host follows the real-use pass.

**Alternatives considered:** A guide for a named host now (rejected: no host was tested, and an untested host guide would claim more than we know).

**Reason:** This answers the open question "which hosting service" without a claim that is not tested. The product owner may choose a host for the first real run.

## Real-use results of this build — 2026-11-15

All assumptions listed in "Assumptions to test in real use" are still assumptions. The build was tested against a fake GitHub only (see `test.md`, A1 to A34). Nothing in the real apps or in a real GitHub App was tested yet.


## The domain-list test does not race the clone — 2026-11-17

**Decision:** In `the_email_domain_list_of_the_memory_repository_is_applied`, the first sign-in uses an owner address in the allowed domain (`olivia@corp.example`). The second sign-in, with `olivia@example.org`, must still fail with 403.

**Reason:** Found in the first Linux run (2026-11-17). The test assumed that the first sign-in comes before the clone is ready, so that the domain list does not apply yet. In the Linux container the local clone was faster, the first sign-in already met the list, and it failed 3 times out of 3. The server behavior was right. Only the test changed.
