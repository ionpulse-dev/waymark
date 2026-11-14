# Plan: Verification

## Overview

Three small tool files on top of the helpers of `write-documents`. The setting `verification` of `.waymark/settings.toml` decides whether the registry lists them. Trust fields are changed through `format`, never by hand.

## Components Affected

- `kb` (`memory-repository`) — `settings.verification` (already in the schema).
- `mcp` (`mcp-server`) — the registry already hides `verify`, `review`, and `reject` unless verification is on. A direct call of a hidden tool is `access_denied`.
- `format` — the trust fields in the field table, and `render`.
- `writes` (`write-documents`) — `DocPlan`, `Repo::write` glue, preview and result helpers, the `folders` check.
- `src/tools/mod.rs` — three lines in the registry.

## New Files

- `src/tools/verify.rs` — the `verify` tool.
- `src/tools/review.rs` — the `review` tool.
- `src/tools/reject.rs` — the `reject` tool.
- `tests/it_verification.rs` — end-to-end tests with the `World` harness.

## Technical Approach

1. **Visibility.** Already done by the registry (see `mcp-server`, item 4). This feature only adds the three tools to the registry.
2. **`verify`.** Inside `Repo::write`: read the document, check `base`, check `status`, `disputed`, and `trust`, set the fields with `format`, run the format checks, return a `DocPlan` with the commit subject `waymark: update "<title>"` and the message body `Trust: verified`.
3. **`reject`.** The same path. Set `status: deprecated`, `trust: unverified`, remove `verified_by` and `verified`. The subject is `waymark: deprecate "<title>"`. The body is `Reason: <reason>` and `Trust: rejected`.
4. **`review`.** Read the local files (after a pull, like the read tools). Keep active documents with `trust` not `verified`, or `disputed: true`. Apply the `folders` limit and the optional `folder`. Sort: disputed first, then by `updated` ascending. Cut at `limit` and count `total`.
5. **Trust reset on change.** Lives in `update_document` (see `write-documents`). Its test is in this spec too.
6. **Result.** Plain `message`, `notices`, and the `data` of the requirements.

## Cross-Feature Dependencies

- `write-documents` — `add` sets `trust: unverified`, and `update_document` resets trust. Both read `kb.settings.verification`.
- `write-approval` — approval for `verify` and `reject` works like for any write tool.
- `activity-and-undo` — `undo` reverts a `verify` like any other `update` commit. Nothing special.
- `search-and-read` — `show` and `list` can show the `trust` fields. Not required here.

## Risks

- Agents call `verify` on their own documents. Mitigation: this is a known limit of local mode (see `trust-model`). Approval via elicitation is the option.
- A team turns verification on late, and `review` lists everything. Mitigation: the `limit` and the `folder` filter, and the `total` count.

## Open Questions

None.
