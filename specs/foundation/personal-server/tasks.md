# Tasks: Personal server

_Written after `plan.md` is approved. Bite-sized implementation checklist. AI checks off items as work is completed._
_List tasks in strict prerequisite-first order. A task may only appear after all tasks it depends on._

`requires`: `mcp-server`, `linux-support` (both `implemented`).

After every task, run the full suite (`make test-all`, and `make linux-check` for tasks that touch the platform or the image). The module names in `plan.md` are a first idea. Task A2 fixes them. If a task needs a file that is not in `plan.md`, update the list in `plan.md` first.

## A. Preparation

- [x] A1. Verify that `mcp-server` and `linux-support` are `implemented` in `specs/INDEX.md`. Set the status of this spec to `in-progress` in `requirements.md` and in `specs/INDEX.md`, and update `## Next Up`.
- [x] A2. Read the full text of the MCP authorization standard (revision 2026-07-28) and fix the exact well-known paths, the `WWW-Authenticate` format, and the token request fields. Fix the module names and update the "New Files" list in `plan.md`. Record the result in `decisions.md`.
- [x] A3. Choose the crates (HTTP server and router, HTTP client for GitHub and client documents, HMAC signing for tokens, RS256 signing for the GitHub App statement, base64). Prefer few dependencies. `rmcp` 3.10 has the features `transport-streamable-http-server` and `-session` (found in its `Cargo.toml`). Check that the same `Server` handler serves stdio and HTTP. Record the choice in `decisions.md`.
- [x] A4. Add the dependencies and `rmcp` features to `Cargo.toml`. Check that the build and the suite still pass on macOS and on Linux, and note the change in binary size.

## B. Mode and settings (prerequisite: A)

- [x] B1. Add `--mode` and `WAYMARK_MODE` to `src/mcp/args.rs` with the values `local` (default), `personal-server`, and `team-server`. An unknown value stops the start and lists the values. `team-server` stops with "not available yet". Write the tests A21, A22, and A25 in `tests/it_mode_and_settings.rs`.
- [x] B2. Write the settings of `personal-server` mode (table in `plan.md`, section 6): read from the environment and from mounted files, with the required and the refused settings per mode. Stop the start with a message that names the missing or refused setting. Require a public URL that starts with `https://` (a `localhost` address is allowed). Read `PORT` (default 8080). Write the tests A23, A24, and A26.
- [x] B3. Add the server config as a limit layer in `src/config` (`trust-model`): the environment and the server config are limits, and the client config stays advice. Write the test A11 for the layer.
- [x] B4. In `personal-server` mode use the temp folder as the data folder, ignore `--path`, and set the memory repository from `WAYMARK_REPO` (`src/mcp/context.rs`, `src/mcp/binding.rs`). Write the test A27 (the log line comes in C1).

## C. Log (prerequisite: B)

- [x] C1. Write `src/log.rs`: one line for each start, sign-in, and tool call, with fixed fields (time, event, tool, outcome, document ID, account name for a sign-in), written to standard output. Run the error text through `src/identity/redact.rs`. Free text from clients and documents never goes in a line. Use it for the `--path` line of B4.
- [x] C2. Log one line for each tool call from the tool layer in `src/mcp`, successful or failed. Write the tests A19 and A27.

## D. HTTP server and status page (prerequisite: A3, B, C)

- [x] D1. Write `src/http/mod.rs`: listen on `0.0.0.0:$PORT`, route the MCP path, the discovery paths, `/`, and `/health`, and keep the status page and health answer from shadowing the MCP and discovery paths. Add `--listen <address>` as an optional override.
- [x] D2. Write `src/http/status.html` and `src/http/health.rs`: the blue page with "Waymark" in white at the center, the running line, the GitHub link, a viewport tag, no script, no external file, no cookie, and the headers of `plan.md` section 9. `/health` returns `ok` and never touches the clone. Write the tests A14, A15, A16, and A17 (check the contrast ratio in the test).
- [x] D3. Serve the MCP tools over Streamable HTTP in `personal-server` mode, with the same handler and registry as stdio. Start the clone in the background at start (`server.start_join()`). Without a token the MCP path answers `401` (the full door comes in G). Write the test A32.

## E. Tokens (prerequisite: B2)

- [x] E1. Write `src/auth/tokens.rs`: create and check signed authorization codes, access tokens, and renewal tokens with the signing key. Every lifetime is a setting with an environment override (defaults: code in minutes, access token about 1 hour, renewal token about 30 days). A token holds the person, the audience (this server), and the limits that apply. Check the signature, the expiry, and the audience. Look nothing up.
- [x] E2. Write unit tests for E1: valid, expired, wrong audience, wrong key, altered payload, and a lifetime changed by the environment. These cover the core of A8 and A9.

## F. Identity (prerequisite: B2, E1)

- [x] F1. Write `src/auth/identity.rs`: the small interface "who is this person?" that returns a stable ID, a name, and a list of emails with a verified flag. Add a fake implementation for tests that returns a configured account.
- [x] F2. Write the email choice: accept only a verified address that passes the check of `src/identity/email.rs` (syntax, blocklist, no-reply, `people.email_domains`), prefer the primary one, and refuse with the message "Add and verify an email address on your GitHub account." when none passes. Write the tests A28, A29, and A30 against the fake identity.
- [x] F3. Write `src/auth/github.rs`: the GitHub implementation. It sends the person to GitHub with the client ID and secret of the App, reads the user and `GET /user/emails` once with the user token, drops the token, and applies the allow-list of exactly one account (`WAYMARK_OWNER`). Any other account ends with a refusal page. Keep the GitHub calls behind a base address setting so the tests can use a fake GitHub.

## G. Sign-in door (prerequisite: D3, E, F)

- [x] G1. Write `src/auth/discovery.rs`: the protected resource metadata and the authorization server metadata, and the `401` with `WWW-Authenticate` on the MCP path. Write the test A1.
- [x] G2. Write `src/auth/registration.rs` with the three methods as separate functions: client document at a URL (fetch, check that `client_id` equals the URL, check the redirect address), automatic registration (the client is a signed value), and the manual client (ID and secret derived from the signing key). Write the tests A2, A3, and A4.
- [x] G3. Write `src/auth/mod.rs`: the routes of the door (authorize, identity callback, token, renew). Authorization code with PKCE, and the `resource` parameter carried into the token. Never put a GitHub token in a response. Write the tests A5 and A6.
- [x] G4. Check the token on every request to the MCP path (signature, expiry, audience) and pass the identity to the request, so that the author and the log line use it. Write the tests A7, A8, and A9.

## H. Git access (prerequisite: B4, F, G4)

- [x] H1. Write `src/git/github_app.rs`: sign a short statement with the App private key, get an installation token from GitHub, and make a new one when needed and after any `401`. Nothing is stored. Keep the GitHub address as a setting so the tests can use a fake. Write the test A33.
- [x] H2. Use the token for clone, pull, and push in `src/gitsync` through an access mode that takes its credential from `github_app.rs`. The credential is never written to disk or to the log (see `redact.rs`). Do not change the local-mode path.
- [x] H3. Pull on request when the clone is older than a short limit. Make a write push before it returns. After a rejected push, pull, rebase, and retry once, and return an error that says what happened if the retry fails. Write the tests A10 (the push part), A12, and A31.
- [x] H4. Take the commit author from the identity of the request in `src/identity/author.rs` in `personal-server` mode (name and verified email). Write the test A10 (the author part).
- [x] H5. Check that `init` works on an empty remote with the fake App token and that the same session works with no restart. Write the test A20.

## I. End-to-end tests (prerequisite: G, H)

- [x] I1. Write the harness in `tests/common`: a fake identity server, a fake GitHub (user, emails, App token), and a bare Git repository as the fake remote with options to delay the clone and to reject a push. Start the real binary in `personal-server` mode against them.
- [x] I2. Write `tests/personal_server.rs` with the end-to-end tests of `test.md` that are not yet covered by a unit test: A1 to A12, A16, A18, A20, A31, and A33.
- [x] I3. Write the secret test A18 with known values and search the whole output.

## J. Container and guide (prerequisite: D, linux-support)

- [x] J1. Write the `Dockerfile`: the Linux build of waymark from `linux-support`, `git`, and certificates, run as a normal user, with no secret and no memory repository. Listen on `$PORT`. Write the test A13 (skips with a message when no container runtime runs).
- [x] J2. Write `docs/personal-server-setup.md` as `plan.md` section 10 says: the order of steps, three parts for each step (what to do, what you should see, what to do if you do not), person-only steps marked, the GitHub App permissions (including "Email addresses: read"), the exact secret names, the registration and plan notes for Claude, ChatGPT, and Gemini, the `agy` CLI note, the cold start timeout note, and the final check. State the unverified assumptions as "to test in real use".
- [x] J3. Update `README.md` and `docs/` with a pointer to the guide and the `--mode` argument. Update the notes of `specs/foundation/distribution` for the image and of `specs/foundation/trust-model` for the server config layer.

## K. Verify and hand over

- [x] K1. Run the full suite on macOS (`make test-all`) and on Linux (`make linux-check`). Tick the `[auto]` items in `test.md` (A1 to A34).
- [x] K2. Review the door and the token checks against `docs/security-review.md` (see "Public exposure" in `plan.md`). Fix the findings or record them in `todo.md`.
- [x] K3. Set the status to `implemented` in `requirements.md` and in `specs/INDEX.md` and update `## Next Up`. Update `CHANGELOG.md` for the release that carries this item.
- [ ] K4. Hand the `[manual]` items of `test.md` (M1 to M9, the real-use pass) to the product owner, with the list of assumptions in `decisions.md`.
