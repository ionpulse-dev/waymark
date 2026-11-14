# Tasks: Remove Index

_Written after `plan.md` is approved. AI checks off items as work is completed._

- [x] Specs: remove `index` and `index-stale` from `quality-checks` (requirements, plan, tasks, test)
- [x] Specs: remove the index-file rule from `search-and-read`, `updates`, `folders`, `write-documents`
- [x] Specs: remove `index` from the tool lists of `pause-and-resume`, `write-approval`, `mcp-server`
- [x] Specs: add a decision entry that points here, in `quality-checks`, `search-and-read`, and `docs/decisions.md`
- [x] Code: delete `src/tools/index.rs` and `src/indexgen`, and their registration
- [x] Code: delete the `index-stale` check
- [x] Code: remove `is_index` from `store`, `changes`, `backlinks`, `folders`, `delete`, `remove_folder`
- [x] Tests: remove the tests of the tool and the rule, and add the normal-document test
- [x] README, `docs/install.md`, open questions: remove `index`
- [x] `CHANGELOG.md`: add the 0.3.0 entry
- [x] Run the full suite, clippy, fmt, `spec-check`, `public-check`
- [x] Set the status in `requirements.md` and `INDEX.md`
