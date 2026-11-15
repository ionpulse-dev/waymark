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

## An exception to "waymark never stores a Git secret" is not needed — 2026-11-15

**Decision:** Because the server stores nothing and the private key stays in the host's secret store, the principle "Secrets stay with Git — waymark never stores or passes a Git secret" holds unchanged.

**Reason:** A later version with storage would need a new decision.
