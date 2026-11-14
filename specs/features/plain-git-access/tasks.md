# Tasks: Plain-Git access

`requires`: `memory-repository`.

- [x] Copy `personal-agent-memory/SKILL.md` and `team-agent-memory/SKILL.md` from `waymark-trail` (version 1.5.1) to `plain-git/`. Check that both files have a `version` in the frontmatter.
- [x] `src/kb/templates/agents-intro.md`: the introduction of the requirements.
- [x] `src/kb/agents_file.rs`: `strip`, the version reader, and unit tests (version, update section removed, frontmatter removed).
- [x] `src/kb/agents_file.rs`: `render` for both audiences and the team part, and unit tests.
- [x] Unit tests: independence from waymark, heading drift, size limit.
- [x] `memory-repository`: `kb::init` calls `render` (replaces the old `AGENTS.md` template).
- [x] `tests/it_plain_git_access.rs`: the acceptance criteria.
- [x] Run the full suite.
