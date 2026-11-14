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

## A schema number in `waymark.toml` blocks older builds — 2026-11-14

**Decision (product owner):** `waymark.toml` has the key `schema` (default 1 when missing). A file with a higher schema than the build supports blocks every tool except `status` and says that a newer waymark is needed. Nothing raises the number yet.

**Alternatives considered:** Use `.mkbase/version` (it holds the version of the document standard, which changes for other reasons). Block only writes (rejected by the product owner: nobody should work on a file that an older waymark might read wrongly). Migrate automatically on read (rejected: a read must never change a shared file).

**Reason:** In a team, one person's update must not leave the others with a build that corrupts the file. The check has to ship before the first bump, because builds without it ignore the key. Releases up to 0.1.2 do. The key is added now, while no migration exists. Files written earlier have no key and count as schema 1.

## Repairs are automatic, migrations are manual — 2026-11-14

**Decision (product owner):** New versions of waymark repair the settings file by themselves when something is missing, without bothering the person. A change that older builds cannot read is a migration and stays a deliberate step of the owner (a tool `migrate`, designed and not built). All of it is described in the section "Versions and updates" of `requirements.md`.

**Alternatives considered:** Repair on a read, or at the start of the session (rejected: a person who only reads would cause a commit to a shared repository, and `initialize` must stay fast and free of writes). Let `init` complete an existing settings file (rejected: it is not what `init` is for, and it would be a second way to edit the file). Migrate automatically too (rejected: the first person on a new build would move the whole team's file and lock out everyone on an older build).

**Reason:** A repair only adds what is missing, so it is safe and the result stays readable for older builds. The repairs come from a list in the binary and never touch the limits that an owner set, so the trust model holds. The first repair adds the key `schema` to files that were written before it existed. A file with no `id` cannot be repaired this way, because it blocks every tool (see the section).
