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
