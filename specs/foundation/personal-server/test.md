# Tests: Personal server

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_AI checks off `[auto]` items when tests pass. A human checks off `[manual]` items. The foundation item moves to `done` only when all items are checked._

## Acceptance Criteria Coverage

_Each BDD criterion from `requirements.md` must be covered by at least one test case below. The automated tests use a fake identity provider and a fake Git host (see "Notes"), so they need no vendor and no real GitHub._

| Criterion | Test Case |
|:----------|:----------|
| **Given** a started server, **When** an MCP client connects with no token, **Then** it gets a `401` with the discovery information of the standard. | A1 |
| **Given** an MCP client that identifies itself with a client document at a URL, **When** it starts the sign-in, **Then** the server reads the document and reaches the GitHub step with no manual input. | A2, M3 |
| **Given** an MCP client that supports automatic registration only, **When** it starts the sign-in, **Then** it registers and reaches the GitHub step with no manual input. | A3, M3 |
| **Given** an MCP client that needs a fixed client ID and secret, **When** the person enters the values that the server gives, **Then** the sign-in works. | A4, M4 |
| **Given** the owner signs in with the allowed GitHub account, **When** the sign-in ends, **Then** the client gets a token that works only on this server, and never a GitHub token. | A5 |
| **Given** any other GitHub account, **When** it signs in, **Then** the server refuses and no token is issued. | A6 |
| **Given** a valid token, **When** the server restarts with an empty data folder, **Then** the same token still works and the first call clones the memory repository again. | A7 |
| **Given** a token past its expiry, **When** a call arrives, **Then** it is refused, and the client can renew it with the renewal token. | A8, M5 |
| **Given** a changed signing key, **When** a call arrives with an old token, **Then** it is refused. | A9 |
| **Given** a write, **When** the tool returns, **Then** the commit is on the remote and its author is the signed-in person. | A10, M6 |
| **Given** a server limit `access = "read-only"`, **When** a client asks for a write, **Then** the write fails. | A11 |
| **Given** a stopped and later started process, **When** the first request arrives, **Then** it is served with no data from before. | A12 |
| **Given** the image, **When** it is inspected, **Then** it holds no secret. | A13 |
| **Given** a browser, **When** it opens the address of the server, **Then** it shows the blue status page with "Waymark" in white at the center, a line that says that the server is running, and a link to the project on GitHub. | A14, M2 |
| **Given** the status page, **When** its source is read, **Then** it holds no script, no external file, and no repository name, owner, version, or clone state. | A15 |
| **Given** a clone that is still running, **When** the status page or `/health` is requested, **Then** it answers at once. | A16 |
| **Given** a narrow phone screen, **When** the status page opens, **Then** the text is centered, readable, and does not scroll sideways. | A17, M2 |
| **Given** a run with known secret values (signing key, App key, client secret, a token, and a document text), **When** sign-ins and tool calls happen, **Then** the log contains none of these values. | A18 |
| **Given** any tool call, **When** it ends, **Then** the log has one line with the time, tool, outcome, and document ID. | A19 |
| **Given** an empty repository on GitHub with the App installed, **When** the server starts and an agent calls `init` with a name, **Then** the structure and the first commit are on the remote and the same session works with no restart. | A20, M7 |
| **Given** `--mode` with an unknown value, **When** the server starts, **Then** it stops and lists the values `local`, `personal-server`, and `team-server`. | A21 |
| **Given** `--mode team-server` before it exists, **When** the server starts, **Then** it stops and says that this mode is not available yet. | A22 |
| **Given** `--mode personal-server` and a public URL that does not start with `https://`, **When** the server starts, **Then** it stops with a clear message. | A23 |
| **Given** `--mode personal-server` and no `WAYMARK_OWNER`, **When** the server starts, **Then** it stops and names the missing setting. | A24 |
| **Given** no `--mode`, **When** the server starts, **Then** it runs as local mode over stdio, as before. | A25 |
| **Given** `PORT` is set, **When** the server starts, **Then** it listens on that port. **Given** it is not set, **Then** it listens on 8080. | A26 |
| **Given** a start in `personal-server` mode with `--path`, **When** the server starts, **Then** the option is ignored and the log says so. | A27 |
| **Given** a GitHub account with a verified email that passes the email check, **When** it signs in and a write runs, **Then** the commit author has that email. | A28, M6 |
| **Given** a GitHub account whose only emails are unverified, blocked, or the no-reply address, **When** it signs in, **Then** the sign-in is refused, no token is issued, and the message says to add and verify an email on GitHub. | A29, M6 |
| **Given** `people.email_domains` is set, **When** the verified email is in another domain, **Then** the sign-in is refused. | A30 |
| **Given** a clone that is still running, **When** a tool call arrives, **Then** it waits for the clone, and after the wait it returns a clear error that tells the agent to call again. | A31 |
| **Given** the setup guide, **When** an agent follows it step by step, **Then** it stops at each step that only a person can do and does not print a secret in the chat. | M1 |

## Test Cases

### Automated (`tests/personal_server.rs` unless noted)

- [x] [auto] A1: a request with no token gets `401` with a `WWW-Authenticate` header that points to the protected resource metadata, and the metadata documents name this server and its authorization server.
- [x] [auto] A2: the door accepts a client identified by a client document URL (served by a fake host). A document whose `client_id` differs from its URL, or whose redirect address does not match, is refused.
- [x] [auto] A3: automatic registration returns a client that completes the flow. The registered client is a signed value, and a second server instance with the same key accepts it.
- [x] [auto] A4: the manual client ID and secret are derived from the signing key. A call with a wrong secret is refused.
- [x] [auto] A5: the full flow (discovery, client, authorization code with PKCE, fake identity, token) ends with a token whose audience is this server. A token request with a wrong PKCE verifier or a missing `resource` is refused. The response holds no token of the identity provider.
- [x] [auto] A6: a fake identity other than `WAYMARK_OWNER` is refused at sign-in and no code or token is issued.
- [x] [auto] A7: start a server, get a token, stop it, start a new instance with an empty data folder and the same key. The token works and the first call clones again.
- [x] [auto] A8: a token past its expiry is refused. The renewal token gives a new access token. An expired renewal token is refused. Lifetimes come from the settings, and an environment override changes them.
- [x] [auto] A9: a token from before a key change is refused after the change.
- [x] [auto] A10: a write through the server is on the fake remote when the tool returns, and its author name and email are those of the signed-in identity. A rejected push is retried once after a pull and rebase. If the retry fails, the tool returns an error that says what happened.
- [x] [auto] A11: with `access = "read-only"` in the server config, a write tool fails and a read tool works. The client config cannot loosen the limit.
- [x] [auto] A12: stop the process, start it again, and send a request. It is served with no state from before.
- [x] [auto] A13: build the image and search its layers and its environment for the known test secrets. Skipped with a message when no container runtime is running (`scripts/tests/run.sh` style). The Linux check runs in the dev container.
- [x] [auto] A14: `GET /` returns `200`, `text/html`, the blue background, the word "Waymark", the running line, and the GitHub link. It carries `Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'` and `X-Content-Type-Options: nosniff`.
- [x] [auto] A15: the page source holds no `<script`, no `src=`, no `http` link other than the GitHub link, no `@import`, no `url(`, and none of the repository name, owner, version, or clone state. It sets no cookie. The white text on the blue passes a contrast ratio of 4.5 or more.
- [x] [auto] A16: start the server with a fake remote that blocks the clone. `GET /` and `GET /health` answer at once (`ok`, status 200).
- [x] [auto] A17: the page has a viewport tag and uses relative sizes, with no fixed width in pixels that exceeds a narrow screen.
- [x] [auto] A18: run the sign-in and several tool calls with known secret values (signing key, App key, client secret, a token, a document text). Search the whole output for each value. None may appear.
- [x] [auto] A19: every tool call, successful or failed, writes exactly one log line with the time, the tool, the outcome, and the document ID (when there is one).
- [x] [auto] A20: an empty fake remote with a fake App token. The agent calls `init` with a name, the structure and the first commit are on the remote, and the next tool call in the same session works.
- [x] [auto] A21: an unknown `--mode` stops the start and lists `local`, `personal-server`, and `team-server` (`tests/it_mode_and_settings.rs`).
- [x] [auto] A22: `--mode team-server` stops with "not available yet" (`tests/it_mode_and_settings.rs`).
- [x] [auto] A23: a public URL without `https://` stops the start, and a `localhost` address is allowed (`tests/it_mode_and_settings.rs`).
- [x] [auto] A24: a missing `WAYMARK_OWNER` stops the start and names the setting. The same test covers each other required setting (`tests/it_mode_and_settings.rs`). A setting of another mode, such as a roles file, also stops the start.
- [x] [auto] A25: with no `--mode`, the binary runs over stdio and all earlier tests still pass (the existing suite, plus one explicit test in `tests/it_mode_and_settings.rs`).
- [x] [auto] A26: the server listens on `PORT` when it is set, and on 8080 when it is not (`tests/it_mode_and_settings.rs`).
- [x] [auto] A27: `--path` in `personal-server` mode is ignored, the data folder is the temp folder, and the log has one line that says so.
- [x] [auto] A28: the identity step picks the primary verified address that passes the email check. A write has that address as its author.
- [x] [auto] A29: with only unverified, blocked, or no-reply addresses, the sign-in is refused, no token is issued, and the message says to add and verify an email on GitHub.
- [x] [auto] A30: with `people.email_domains` set, a verified address in another domain is refused.
- [x] [auto] A31: with `WAYMARK_CLONE_WAIT_SECONDS` set low and a fake remote that blocks the clone, a tool call waits and then returns the "call again" error. The next call works when the clone is done.
- [x] [auto] A32: the tool list in `personal-server` mode is the same as in `local` mode.
- [x] [auto] A33: when the Git host answers `401`, the server makes a new installation token and retries once.
- [x] [auto] A34: the whole suite passes on macOS (`make test-all`) and on Linux (`make linux-check`). Result of 2026-11-15: all tests pass on both systems. The only failing check of `make test` on this branch is a shell check about the changelog entry of the tag `v0.5.1`, which lives on `main` (not caused by this spec).

### Real-use pass (manual)

The real-use pass tests the assumptions in `decisions.md` ("Assumptions to test in real use"). It needs a deployed server, a real GitHub App, and the apps. When a check fails, find the module in the table of `plan.md` (section 11), fix it, and add a result entry to `decisions.md`.

- [ ] [manual] M1: the setup guide. **Steps:** give `docs/personal-server-setup.md` to an agent in a clean session and say "set this up for me". **Expected:** the agent stops at each step that only a person can do (create the GitHub App, install it, paste secrets into the host), never prints a secret in the chat, and ends with the address of a working server. **Setup:** an empty private repository on GitHub, an account on the chosen host. **Teardown:** delete the host service and the GitHub App.
- [ ] [manual] M2: the status page on a phone and a desktop. **Steps:** open the public address in a phone browser and a desktop browser. **Expected:** blue page, "Waymark" in white at the center, the running line, a link to the project, no sideways scroll, readable text. Open `/health` and expect `ok`.
- [ ] [manual] M3: registration by Claude and ChatGPT. **Steps:** add the server as a custom connector in the Claude app and in ChatGPT (use the plans named in the guide). **Expected:** the app reaches the GitHub sign-in with no manual input. Note which registration method each app used (client document or automatic) in `decisions.md`.
- [ ] [manual] M4: the manual client. **Steps:** add the server to Gemini (Enterprise, or the consumer app if it offers custom servers) and to the `agy` CLI with the client ID and secret from the guide. **Expected:** the sign-in works and a tool call returns a result. Note what each app accepts in `decisions.md`.
- [ ] [manual] M5: renewal. **Steps:** set a short access token lifetime in the host settings (for example 120 seconds), connect each app, wait past the lifetime, and ask a question. **Expected:** the app renews and answers with no new sign-in. If an app asks for a new sign-in, note it in `decisions.md`.
- [ ] [manual] M6: author email and push. **Steps:** sign in with a GitHub account that has a verified email, and ask an agent to write a document. **Expected:** the commit is on GitHub with that email as author and the push is accepted, also when the GitHub setting "Block command line pushes that expose my email" is on. Then sign in with an account that has no verified email and expect the refusal message. Note the result of the private-email setting in `decisions.md`.
- [ ] [manual] M7: `init` on an empty repository. **Steps:** create an empty private repository, install the App on it, start the server, and ask an agent to set up the memory repository with a name. **Expected:** the structure and the first commit appear on GitHub, and the same session can search and read.
- [ ] [manual] M8: cold start. **Steps:** on a serverless host, leave the server idle until it stops, then ask a question from a phone app. **Expected:** the first call works or returns the "call again" message, and the next call works. Note the time of the first call and the host timeout in the guide.
- [ ] [manual] M9: token lifetimes on GitHub. **Steps:** with a running server, wait longer than 1 hour after the first write and write again. **Expected:** the write works (the server made a new installation token). Note the real lifetime from the GitHub documentation or the token response in `decisions.md`.

## Notes

- **Fake identity provider and fake Git host.** The test harness starts a small local identity server that returns a configured account, name, and email list (verified and unverified). It also provides a bare Git repository as the fake remote, with an optional delay to block the clone, and an optional rejection of a push. The fake App token endpoint returns a short token. The tests set `WAYMARK_PUBLIC_URL` to a `localhost` address (allowed for tests).
- **A18** needs known values: set `WAYMARK_SIGNING_KEY`, `WAYMARK_GITHUB_APP_KEY`, and `WAYMARK_GITHUB_CLIENT_SECRET` to test values, sign in, call tools with a document that holds a marker text, and search the captured standard output and standard error.
- **A13** needs a container runtime. It skips with a message when none runs, as in `linux-support`.
- The `[manual]` items M3 to M9 are the real-use pass of the build-first approach. They need paid plans and admin access in some cases, so they run after the build, on the schedule of the product owner.
