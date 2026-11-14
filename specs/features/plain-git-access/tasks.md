# Tasks: Plain-Git access

`requires`: `knowledge-base-repo`.

- [ ] Copy `personal-knowledge-base/SKILL.md` and `team-knowledge-base/SKILL.md` from `waymark-trail` (version 1.5.1) to `plain-git/`. Check that both files have a `version` in the frontmatter.
- [ ] `src/kb/templates/agents-intro.md`: the introduction of the requirements.
- [ ] `src/kb/agents_file.rs`: `strip`, the version reader, and unit tests (version, update section removed, frontmatter removed).
- [ ] `src/kb/agents_file.rs`: `render` for both audiences and the team part, and unit tests.
- [ ] Unit tests: independence from waymark, heading drift, size limit.
- [ ] `knowledge-base-repo`: `kb::init` calls `render` (replaces the old `AGENTS.md` template).
- [ ] `tests/it_plain_git_access.rs`: the acceptance criteria.
- [ ] Run the full suite.
