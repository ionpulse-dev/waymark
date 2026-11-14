# Decisions: Write documents

> **Note 2026-11-11:** All decisions stay valid. The commands are tools, and the advice about what deserves a document moved out of waymark.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Every write is checked — 2026-11-08

**Decision:** Schema, Obsidian compatibility, location, size, duplicates, links, and secrets are checked before every commit. Companies add their own secret patterns. Plain-Git mode has only a text rule.

**Reason:** This supports the claim of a safe knowledge base. A failed check stops the write with exit code 2.

## Preview — 2026-11-06

**Decision:** `--preview` shows what would be written.

**Reason:** At write level `confirm` with confirmation in the chat, the agent needs something to show.

## Dispute instead of delete — 2026-11-06

**Decision:** Agents mark wrong documents as disputed or deprecated and keep the file.

**Reason:** History stays, and the author sees the problem in the review list.

**Superseded in part by:** Delete on request, and one process for wrong documents — 2026-11-09

## Delete on request, and one process for wrong documents — 2026-11-09

**Decision:** The agent never deletes on its own, but deletes when the person asks (`waymark delete`). Before it deletes, it fixes every link to the document in the same commit. The tools for wrong documents are, from light to heavy: fix, dispute, deprecate, replace (`add --supersedes`), delete. A dispute ends with `dispute --clear`. A reader of a deprecated document reads the replacement.

**Reason:** In a personal knowledge base the person owns the content and may want a document gone. Git keeps the history. The tested rules of the waymark-trail skills show that agents need the whole process in one place.

**Supersedes:** Dispute instead of delete (in part)

## Enforced checks, no behavior advice — 2026-11-11

**Decision:** The server rejects duplicates, near-duplicates, oversize, secrets, broken links, and forbidden folders, with a `next` sentence. When to write and what to write is for the team's `AGENTS.md`.

**Reason:** the server enforces, the team advises. `delete` can be switched off with `tools_denied`.

## `update_document` accepts a partial update — 2026-11-14

**Decision:** `update_document` takes `id`, `base`, and any of `title`, `description`, `body`, `sources`, `tags`, `related`, `anchors`. Fields that are not sent stay as they are. A list replaces the old list, and an empty list removes the property. The status, dispute, authorship, and trust fields cannot be sent.

**Alternatives considered:** Always send the whole document (rejected: it costs context and invites accidental changes to the body). A merge of lists (rejected: the agent cannot remove an entry).

**Reason:** A small change should cost a small call. Status and dispute have their own tools, so one tool is one meaning. Decided autonomously.

## Near-duplicate titles by word overlap — 2026-11-14

**Decision:** Titles are split into lowercase words (runs of letters and digits). The overlap is the distinct shared words divided by the distinct words of the shorter title. At 0.8 or more the title is a near-duplicate. If the shorter title has fewer than 3 words, only an exact match counts. Deprecated documents and the target of `supersedes` are ignored.

**Alternatives considered:** Edit distance or embeddings (rejected: more code, hard to explain in a `next` sentence). No minimum length (rejected: "Deploy" would block "Deploy to production"). Include deprecated documents (rejected: a replacement often has almost the same title).

**Reason:** A simple rule that an agent can understand and fix. The 3-word minimum and the deprecated rule are additions to the rule given by the product owner. Decided autonomously.

## The reason of `deprecate` goes into the commit message — 2026-11-14

**Decision:** `reason` is required. It is written as `Reason: <reason>` in the body of the commit message. The file gets only `status: deprecated`, `updated`, and `updated_by`.

**Alternatives considered:** A new frontmatter field (rejected: the standard has none, and the format is fixed). A callout in the body (rejected: it changes the content). Drop the reason (rejected: the next reader needs it).

**Reason:** The format stays unchanged, and Git history keeps the reason. `show` can read it later. Decided autonomously.

## `dispute` refuses a second dispute — 2026-11-14

**Decision:** `dispute` on a disputed document fails with `validation_failed` and shows the old reason. `clear: true` on a document that is not disputed also fails.

**Alternatives considered:** Replace the reason (rejected: the first reason is lost). Append (rejected: the field would grow).

**Reason:** The agent learns what is already known and can fix the document or end the dispute. Decided autonomously.

## Delete fixes links as plain text — 2026-11-14

**Decision:** `delete` replaces a Markdown link with its text, a wikilink with its alias or the title of the deleted document, removes an embed, and removes a property entry. It fails with `access_denied` if a document to fix is outside `folders`.

**Alternatives considered:** Refuse to delete while links exist (rejected: the rules of the knowledge base say to fix them in the same commit). Leave the link broken (rejected: breaks `lint`).

**Reason:** This follows the delete rule of the knowledge base standard. Decided autonomously.

## A verified document goes back to unverified on a content change — 2026-11-14

**Decision:** With verification on, `update_document` sets `trust: unverified` and removes `verified_by` and `verified` when `title`, `description`, or `body` changes. `dispute` and `deprecate` keep the trust fields.

**Alternatives considered:** Keep the trust fields (rejected: a verified mark on changed text is false).

**Reason:** The standard allows a verification process to change these fields. Decided autonomously. Also in `verification/decisions.md`.

## Needs from the foundation specs — 2026-11-14

**Decision:** This feature needs three small additions: `Repo::write` takes a closure that runs inside the lock and returns the plan, `gitsync::message` supports an optional message body, and `format::links` returns the span and place (body or property) of each link. The secret scan lives in `src/writes/secrets.rs`, because no spec owns it yet, and `quality-checks` reuses it.

**Alternatives considered:** Put the logic in the tools (rejected: three specs need it).

**Reason:** The foundation plans name the parts but not these details. Decided autonomously. Reported to the product owner.

## The folder limit gives `access_denied` for writes and `not_found` for reads of an id — 2026-11-14

**Decision:** A `folder` outside `folders` in `add` is `access_denied`. An `id` outside `folders` in the other tools is `not_found`.

**Reason:** `mcp-server` lists the folder under `access_denied`. `search-and-read` says a document outside the limit is not found. Decided autonomously.

## Implementation notes — 2026-11-14

**Decided autonomously.**

- **Retarget of links moves to `move-documents`.** `src/writes/linkfix.rs` has `fix_body` and `fix_properties` (remove a link). The retarget code is written with `move-documents`, because only that tool needs it. Task and test item are changed to match.
- **Approval fails closed.** `writes::approve` returns `approval_unavailable` when the limits ask for approval for the tool, because `write-approval` is not implemented yet. Nothing is written without the approval that the limits ask for. A preview never asks. `write-approval` replaces the body of this one function. The test `approval_asked_before_commit` stays `#[ignore]` until then.
- **One execution path.** Every tool builds a `Plan` (files to write, remove, or keep, the commit message, the result fields). `writes::execute` runs it through `Repo::write`. A call that finds nothing to change returns a `Keep` entry, so that `git add` never runs with an empty path list.
- **The knowledge base is read again inside the lock**, after the fetch, so settings and documents are current.
- **Secret rule for assignments.** `password`, `passwd`, `secret`, `token`, and `api_key` followed by `=` or `:` match only when the value is at least 4 characters and does not start with `<`, `$`, `{`, `[`, `*`, or `.`. This keeps placeholders such as `password: <your password>` out of the findings.
- **`stale_base` next sentence** now says "Read the document again with show, then redo the change." (`git-sync` `Tx::check_base`).
- **Excluded folders.** `KnowledgeBase::is_excluded(id)` is new. A write into a path of `.mkbase/exclude` fails with `validation_failed`.
- **The `Reason:` of `deprecate`** is scanned for secrets, because it goes into the Git history.

## A write never creates a broken link, and `delete` refuses linked documents — 2026-11-14

**Supersedes:** "Delete fixes links as plain text" (2026-11-14), and the sentence of the earlier delete decision that `delete` fixes every link in the same commit.

**Decision:** `add` and `update_document` fail with `validation_failed` when the body gets a link to a document that does not exist. For `update_document`, only new links count. `delete` fails with `validation_failed` while another document links to the target, and lists the linking documents in `data.linked_from`. It edits no other document. The agent removes the links with `update_document`, or uses `deprecate`. `move` keeps rewriting links, because a move keeps the target alive. The standard says the same in section 8.

**Alternatives considered:** (1) Keep the automatic fix in `delete` (rejected: it silently edits other documents and loses the information that they were linked). (2) An opt-in switch to fix links in `delete` (rejected: it brings back the silent edit, and one more parameter costs context). (3) Keep body links as a warning (rejected: the knowledge base should hold a consistent state at all times).

**Reason:** The product owner wants a consistent state, prevented at the write and not repaired later. Old broken links, and links broken by changes outside waymark, stay visible in `lint` as warnings. An agent that writes a document with links must write the linked document first. Concurrent writers are out of scope: a conflict is found at the next check.
