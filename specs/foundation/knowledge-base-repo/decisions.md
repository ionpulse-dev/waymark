# Decisions: Knowledge base repository

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Content and tooling live in separate repositories — 2026-11-06

**Decision:** The tooling (CLI, app, skill, schema) is one project. Each knowledge base has its own content repository. The default local path is `~/Projects/<slug>-knowledge-base`.

**Alternatives considered:**
- A fixed path such as `~/knowledge-base` — conflicts with the tooling folder and only fits one knowledge base.
- Content inside each code repository — no cross-team view.

**Reason:** A fixed, predictable path lets agents in other repositories find the knowledge base. A person can join several knowledge bases.

## All shared settings in `waymark.toml` — 2026-11-06

**Decision:** A file in the repository root holds all shared settings. Joining needs only the repository URL. Settings files are TOML, not YAML. Document frontmatter stays YAML.

**Alternatives considered:**
- YAML settings — hard to read, no comments, indentation rules.

**Reason:** TOML allows comments, needs no indentation rules, and is well supported in Rust. Obsidian reads properties only from YAML, so documents keep YAML.

## Teams are folders — 2026-11-06

**Superseded by:** No `_team.md` convention

**Decision:** The folder tree mirrors the teams or areas of an organization, and the folder of a document is its owning team. waymark has no built-in folders, no `owners` field, and no list of followed folders. A `_team.md` document per folder is an optional convention.

**Alternatives considered:**
- A team list in the repository or from the Git host API — maintenance and a host dependency.
- Followed folders — with "no folder means everything" they have no job.

**Reason:** Nothing to maintain. The organization decides its own structure.

## One audience per repository — 2026-11-06

**Decision:** The skill has no topic rules for confidentiality and waymark has no list of confidential markers. Everyone with read access sees every document. Several knowledge bases per person are the way to separate audiences.

**Alternatives considered:**
- Topic rules in the skill — they fail for many organizations, for example a law firm where nearly everything is legal.

**Reason:** What is confidential depends on the organization. Git has no read permission per folder.

## Obsidian vault settings are written by `init` — 2026-11-06

**Decision:** `init` writes `.obsidian/app.json` with Markdown links and relative paths as defaults, and Git ignore rules for personal Obsidian files.

**Reason:** Links that people create in Obsidian match waymark's style, and personal files never cause conflicts.

## No `_team.md` convention — 2026-11-08

**Supersedes:** Teams are folders — 2026-11-06

**Decision:** There is no `_team.md` document and no convention for team documents. The folder tree may mirror the team structure if the organization decides so, and waymark does not care. The rest of the earlier decision stands: waymark has no built-in folders, no `owners` field, and no list of followed folders.

**Alternatives considered:**
- A `_team.md` document per folder with contacts — suggested during the design phase and rejected. It is outdated.

**Reason:** The folder structure is the organization's own choice. waymark needs no special file for it.
