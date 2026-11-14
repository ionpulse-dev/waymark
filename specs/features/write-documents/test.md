# Tests: Write documents

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| Valid `add`: one commit under the author reaches the remote | `add_commits_and_pushes` |
| Secret in the body fails and writes nothing | `add_rejects_secret` |
| Existing title fails and names the document | `add_rejects_duplicate_title` |
| 80 percent word overlap fails | `add_rejects_near_duplicate` |
| Old `base` gives `stale_base` | `update_stale_base` |
| Only `description` sent: body unchanged | `update_partial_keeps_body` |
| Title change keeps file name and ID | `update_title_keeps_id` |
| `preview: true` writes nothing | `preview_writes_nothing` |
| `dispute` sets the flag and the reason | `dispute_sets_fields` |
| `deprecate` keeps the file | `deprecate_keeps_file` |
| `dispute` with `clear: true` | `dispute_clear` |
| `add` with `supersedes`: one commit, old document deprecated | `add_supersedes_one_commit` |
| `delete` fixes links | `delete_fixes_links` |
| `tools_denied` hides `delete` | `delete_hidden_when_denied` |
| `read-only` gives `access_denied` and no commit | `read_only_blocks_writes` |
| `approval.required_for = ["add"]` asks first | `approval_asked_before_commit` |
| Body link to a missing document is a notice | `add_warns_on_body_link` |
| `folders` limit gives `access_denied` | `add_outside_folders` |
| Verified document goes back to `unverified` | `update_resets_trust` |

## Test Cases

- [ ] [auto] All criteria above, through the real binary and the `World` harness (`tests/it_write_documents.rs`)
- [ ] [auto] The commit subject is `waymark: add "<title>"` and has no agent tag (`tests/it_write_documents.rs`)
- [ ] [auto] `deprecate` puts `Reason:` in the commit message body and not in the file (`tests/it_write_documents.rs`)
- [ ] [auto] Property link to a missing document fails with `validation_failed`, and every finding is in `message` (`tests/it_write_documents.rs`)
- [ ] [auto] Two people write different documents at the same time and both reach the remote (`tests/it_write_documents.rs`)
- [ ] [auto] The same `add` twice leaves the repository as after the first call (`tests/it_write_documents.rs`)
- [ ] [auto] `delete` outside `folders` for a linking document gives `access_denied` and no commit (`tests/it_write_documents.rs`)
- [ ] [auto] An id outside `folders` gives `not_found` (`tests/it_write_documents.rs`)
- [ ] [auto] `dispute` twice gives `validation_failed` with the old reason in `message` (`tests/it_write_documents.rs`)
- [ ] [auto] File name rules: slug, 60-character cut, empty title, collision suffix (unit tests in `src/writes/slug.rs`)
- [ ] [auto] Near-duplicate boundary: 4 of 5 shared words fails, 3 of 5 passes, a 2-word title only fails on an exact match (unit tests in `src/writes/titles.rs`)
- [ ] [auto] Every built-in secret pattern matches its sample and the result never contains the secret (unit tests in `src/writes/secrets.rs`)
- [ ] [auto] Link removal and retarget keep every other byte and skip code fences (unit tests in `src/writes/linkfix.rs`)
- [ ] [manual] A document written by `add` opens in Obsidian with its properties, links, and tags shown. Steps: run `add` through a real agent tool with `tags`, `related`, and a body link, open the clone as a vault in Obsidian. Expected: the Properties panel shows every field, the links are clickable, and no error shows.

## Notes

Cases about approval use the elicitation hook of the harness. They need `write-approval`. Until it is implemented they are `#[ignore]`, and the marker is removed with that spec. Every test builds its own `World` (see `integration-testing`).
