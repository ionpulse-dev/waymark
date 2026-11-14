# Decisions: Knowledge base repository

## Unknown settings keys are kept and reported — 2026-11-14

**Decision:** An unknown key in `waymark.toml` gives a notice and is kept. It is not an error.

**Alternatives considered:** Reject unknown keys (rejected: a team that uses a newer waymark would break everyone with an older binary).

**Reason:** Forward compatibility. Decided autonomously.

## `init` never overwrites — 2026-11-14

**Decision:** `init` fails with `validation_failed` if a file of the layout already exists, and writes nothing.

**Reason:** Data loss on a second run is worse than a clear error. Decided autonomously.

## The root is found by the marker folder — 2026-11-14

**Decision:** The root is the folder that contains `.mkbase/`. The default layout places it at `knowledge/`, but any depth works.

**Reason:** This follows the MKBase standard and allows repositories that already have other content. Decided autonomously.

## Where the code ended up — 2026-11-14

**Decision:** `init` is the function `kb::init(repo_dir, name, slug)` and `KnowledgeBase::open(repo_dir)` opens a repository. The settings of the shared file are read again at every tool call (see `mcp-server`). End-to-end tests are in `tests/it_knowledge_base_repo.rs`.

**Reason:** A free function is enough because `init` does not need an open knowledge base. Decided autonomously.
