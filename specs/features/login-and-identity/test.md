# Tests: Login and identity

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** an argument, an environment variable, and a Git config that all set the name, **When** waymark resolves the author, **Then** the argument wins, then the variable, then the Git config. | A1 |
| **Given** no author anywhere, **When** a write runs, **Then** it fails with `setup_problem` and a `next` sentence. | A2 |
| **Given** a person's global Git author and different `--author-*` arguments, **When** waymark commits, **Then** the commit uses the arguments. | A3 |
| **Given** a Git login that works, **When** `status` runs, **Then** the Git login check is ok and no secret was read. | A4, M1 |
| **Given** a login that fails, **When** a sync runs, **Then** it fails with `sync_failed` and a plain `next` sentence, and Git did not wait for input. | A5, M1 |
| **Given** the address `jane@users.noreply.github.com`, **When** the check runs, **Then** it fails on the blocklist. | A6 |
| **Given** `people.email_domains = ["corp.example.org"]`, **When** the address uses another domain, **Then** the check fails. | A7 |
| **Given** a commit by waymark, **When** it is created, **Then** the author email passed the check. | A3, A8 |
| **Given** any result or log, **When** it is inspected, **Then** it holds no token or password. | A9 |

## Test Cases

- [x] [auto] A1: each source order, name and email on their own, with the harness `World` (`tests/it_login_and_identity.rs`)
- [x] [auto] A2: no argument, no variable, `GIT_CONFIG_GLOBAL=/dev/null`: `add_document` returns `setup_problem` with `next`, and the remote has no new commit (`tests/it_login_and_identity.rs`)
- [x] [auto] A3: the Git config says `Other <other@example.org>`, the arguments say `Jane Doe <jane@example.org>`: the pushed commit has the argument author and the same committer (`tests/it_login_and_identity.rs`)
- [x] [auto] A4: `status` on a `file://` remote: `git_login` ok, and the result has no secret (`tests/it_login_and_identity.rs`)
- [x] [auto] A5: a remote that rejects the login (a fake `GIT_SSH_COMMAND` script that prints `Permission denied (publickey)` and exits 255): `sync` returns `sync_failed` with the fixed `next` sentence in under 10 seconds (`tests/it_login_and_identity.rs`)
- [x] [auto] A6: unit and end-to-end: every default blocklist part is blocked, a normal address passes (`tests/it_login_and_identity.rs`, `src/identity/email.rs`)
- [x] [auto] A7: `email_domains` set in the shared file: a matching address passes, another one fails, a subdomain fails (`tests/it_login_and_identity.rs`)
- [x] [auto] A8: after any write, `git log -1 --format=%ae` passes `check_email` (`tests/it_login_and_identity.rs`)
- [x] [auto] A9: start with `--repo https://jane:s3cret-value@example.org/x.git` and the variable `GH_TOKEN=ghp_testtoken`; collect every result, error, and the stderr log. None holds `s3cret-value` or `ghp_testtoken` (`tests/it_login_and_identity.rs`)
- [x] [auto] A10: a syntax error or blank name counts as missing (`src/identity/author.rs`)
- [ ] [manual] M1: real Git host. Steps: with a test repository on a real host, sign out of Git (remove the credential helper entry or use a key without access). Start a session and call `status`, then `sync`. Expected: `git_login` is not ok with the plain `fix` sentence, and nothing waits for input. Sign in with `gh auth login`. Call `status` again. Expected: all ok.

## Notes

A5 uses a script as `GIT_SSH_COMMAND` so no network is needed.
