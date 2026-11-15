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

## An exception to "waymark never stores a Git secret" is not needed — 2026-11-15

**Decision:** Because the server stores nothing and the private key stays in the host's secret store, the principle "Secrets stay with Git — waymark never stores or passes a Git secret" holds unchanged.

**Reason:** A later version with storage would need a new decision.
