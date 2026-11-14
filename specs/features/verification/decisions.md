# Decisions: Verification

> **Note 2026-11-11:** "Optional, off by default" stays valid.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Optional, off by default — 2026-11-06

**Decision:** Verification is a setting of the knowledge base. The default model is the wiki model without it.

**Alternatives considered:**
- Verification as the default (trust as a stored state) — needs a clear owner for every document and a review queue, and slows writes.

**Reason:** Organizations that need a stronger signal can turn it on. Others keep the simple model.

## Verification tools and roles — 2026-11-11

**Decision:** `verify`, `review`, and `reject` exist only with verification on. On a server, `verify` is for the role `owner`, and the console can enforce the answer.

**Reason:** in local mode waymark cannot prove who called `verify`. A server and the console can.

## Verification tools in local mode — 2026-11-14

**Decision:** `verify`, `review`, and `reject` are normal tools in local mode. They are listed only when `verification = true` is in `.mkbase/waymark.toml`. A direct call with verification off is `access_denied`. A team can ask for approval with `approval.required_for`.

**Alternatives considered:** Always list the tools (rejected: they cost context in every session of every team that does not use verification). Block verification of your own document (rejected: in a one-person knowledge base nobody could verify).

**Reason:** The set stays small for most teams, and the policy is honest about local mode. Decided autonomously.

## Commit verbs of `verify` and `reject` — 2026-11-14

**Decision:** `verify` uses the verb `update` and `reject` uses `deprecate`. Both add a message body line (`Trust: verified`, or `Reason:` and `Trust: rejected`).

**Alternatives considered:** New verbs `verify` and `reject` (rejected: the verbs are fixed in `git-sync`, and `activity` parses them).

**Reason:** No change to the shared message format. The body line tells apart the cases. Decided autonomously.

## `reject` clears the trust — 2026-11-14

**Decision:** `reject` sets `status: deprecated` and `trust: unverified` and removes `verified_by` and `verified`. A reason is required and goes into the commit message body, like in `deprecate`.

**Alternatives considered:** Keep the verified mark on a rejected document (rejected: it contradicts the rejection). A new trust value `rejected` (rejected: the standard has two values).

**Reason:** The state stays valid under the standard. Decided autonomously.

## Content changes reset trust — 2026-11-14

**Decision:** With verification on, `update_document` of a verified document sets `trust: unverified` and removes `verified_by` and `verified` when `title`, `description`, or `body` changes. `dispute` and `deprecate` keep the fields.

**Alternatives considered:** Keep the mark (rejected: it would vouch for text nobody checked).

**Reason:** The standard allows a verification process to change these fields. The code lives in `update_document`. Same decision in `write-documents/decisions.md`. Decided autonomously.

## `review` is a global queue — 2026-11-14

**Decision:** `review` lists all active documents that are unverified or disputed, in the `folders` limit. Disputed first, then oldest `updated`. The original text said "the queue of the current person". In local mode there are no owners, so the queue is not per person.

**Alternatives considered:** Only documents created by other people (rejected: breaks one-person knowledge bases). An `owners` field (rejected: the standard and the repository spec have none).

**Reason:** No ownership data exists in local mode. A `folder` filter lets a person narrow the queue. Decided autonomously.

## Which role verifies a folder — 2026-11-14

**Decision:** Out of scope for now. When server mode comes back, the role `owner` verifies any folder. Folder owners are deferred.

**Alternatives considered:** Folder owners now (rejected: needs an owner list that does not exist, and server mode is out of scope).

**Reason:** Answers the open question without building server-mode parts. Decided autonomously.

## Implementation notes — 2026-11-14

**Decided autonomously.**

- **Direct calls when verification is off** give `access_denied` from the registry, because the three tools have `needs_verification() = true`. See `mcp-server` `decisions.md` (this replaces "unknown tool" for these tools).
- **`verify` and `reject`** use `writes::execute` like the other write tools. The commit body lines are `Trust: verified`, and `Reason: <reason>` with `Trust: rejected`. The reason of `reject` is scanned for secrets.
- **`review` is a read tool** without the read budget. It needs one `git log` per listed entry for `version` (at most 50).
- **Undo of a verify** works through `undo` (tested). The undo of a verify that a later update changed needs the later change undone first.
