# Security review of local mode

Date: 2026-11-14. Scope: the `waymark` binary in local mode on macOS. The review was done by reading the code and by tests (`tests/it_security.rs`). It is not an outside audit.

## What was checked, and the result

| Area | Result |
|---|---|
| Document IDs and folders that leave the root (`..`, absolute paths, hidden parts, backslash, NUL, `.md` suffix) | Refused by `MemRepo::path_of` for every tool. Test S1. |
| Symbolic links | A document that is a link is never read or written, also a dangling link. A link in a folder that leads out of the root is refused. **Found and fixed:** a dangling link to a file outside the root passed the old check, so a write would have created that file. Test S2. |
| Text from a caller in `git` calls | `git` runs with an argument list, never through a shell. Paths follow `--`. Commit hashes and dates are parsed before use. Test S3. |
| Repository URL | **Found and fixed:** a `--repo` that starts with a dash could be read as a `git clone` option. The URL is refused, and the call has `--`. |
| Git transports | **Found and fixed:** `git` now runs with `GIT_ALLOW_PROTOCOL=file:git:http:https:ssh`, so the `ext::` transport (it runs a command) is off. |
| Secret scan misses | Added Stripe live keys and Google API keys to the built-in patterns. A secret is never echoed in an error. Test S4. |
| Config that loosens a limit | No layer can loosen another. The strictest value wins, also against the entry and the personal file. A broken layer file makes the server read-only. Test S6. |
| Writes while paused or read-only | Every write tool is refused before it runs. `sync` only pulls and commits no manual edit. Test S5. |
| Author fields | A name with a line break does not forge a header. Test S7. |
| Secrets in results and logs | URLs pass `redact_text`. Test A9 of `login-and-identity` and A10 of `maintenance`. |

## Known limits (accepted)

- A race between the symlink check and the write (time of check, time of use) is not closed. An attacker needs write access to the clone on the same machine.
- Documents are data, not instructions. waymark cannot stop an agent that follows text in a document. The server instructions and the `AGENTS.md` rules say so, and the team decides what agents do.
- A hook in the Git configuration of the person runs with the rights of the person. waymark does not disable hooks.
- The approval by elicitation can be answered by a hook of the client (see `write-approval`).
- The scan for secrets finds patterns. It does not find every secret.

# Security review of the personal server

Date: 2026-11-15. Scope: the sign-in door, the tokens, and the Git credential of `--mode personal-server` (spec `personal-server`). It was done by reading the code and by the end-to-end tests (`tests/personal_server.rs`). It is not an outside audit, and the server was not yet tested on the open internet with the real apps.

## What was checked, and the result

| Area | Result |
|---|---|
| Access to the MCP path | Every request needs a valid access token: signature, type, expiry, and audience (`<public URL>/mcp`). A token of another type (code, renewal, client) is refused. Tests A1, A8, A9. |
| Token signature | HMAC-SHA256, compared in constant time. An altered payload, another key, and a malformed token are refused (unit tests in `src/auth/tokens.rs`). |
| Who may sign in | One GitHub account, compared with `WAYMARK_OWNER`. Any other account gets a refusal page and no code. Test A6. |
| GitHub token | Read once to get the account and the emails, then dropped. It is never in a response, a token, or the log. Tests A5, A18. |
| PKCE | `S256` is required. A wrong verifier is refused. A code is bound to its client and its redirect address. Test A5. |
| Redirect addresses | Only `https`, or `http` on localhost. A client document or a registration decides the allowed addresses by exact match. Tests A2, A3. |
| Client documents (SSRF) | Only `https` addresses with a path, with no user name, and not pointing to a private, loopback, or link-local address. The fetch has a size limit and a timeout. **Known gap:** the address is resolved once for the check and again for the fetch, so a DNS answer that changes between the two is not caught. The document is never shown and holds no secret, so the risk is accepted. |
| Manual client | The secret is derived from the signing key and compared in constant time. A wrong secret gets `401`. Test A4. |
| Secrets in the log | Tests A18 and A19 search the whole output for the signing key, the client secret, the tokens, the private key, and document text. The log is built from fixed fields and passes `redact_text`. |
| Secrets in the image | `scripts/tests/image.sh`: no private key in any layer, no secret in the environment of the image. |
| Git credential | Passed to `git` only through the environment of the child process. Never on a command line, on disk, or in the log. Scoped to the host of the repository. Test A33. |
| Wrong private key | The server stops at start and does not print the key. |
| Status page | Static. No script, no external file, no cookie, and no private fact. A strict `Content-Security-Policy`. Test A15. |

## Known limits (accepted)

- **Authorization codes are not single-use.** There is no state to remember them. The expiry is short (5 minutes), PKCE is required, and the code is bound to the client and the redirect address.
- **The renewal token does not rotate.** A stolen renewal token works until it expires (30 days). Changing the signing key cuts off every token. One client cannot be revoked alone.
- **The client ID of an automatic registration is a signed value with a very long expiry.** It holds only redirect addresses and no secret. A changed signing key invalidates it.
- **No rate limit on the door.** A guess of a code or a secret is limited by the signing key size and the short expiry. A host in front of the server can add a rate limit.
- **The host check of the MCP service is off** (it is for local servers). The token protects the server.
- **One owner.** The commit author is set from the last valid token. This is right for one person and wrong for many.
- **Not tested with the real apps and a real GitHub App.** See the real-use pass in `specs/foundation/personal-server/test.md`.
