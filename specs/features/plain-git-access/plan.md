# Plan: Plain-Git access

## Overview

The rules are text. The two standalone skills move into the folder `plain-git/` of this repository. A small function in the `kb` module builds `AGENTS.md` from a fixed introduction, an optional team part, and the text of the skill, and `init` calls it. Tests check the text, the pin, and the independence of the skill from waymark.

## Components Affected

- `knowledge-base-repo` — the template `src/kb/templates/AGENTS.md` is replaced by `src/kb/templates/agents-intro.md`. `kb::init` calls `kb::agents_file::render`.
- `create-knowledge-base` — passes `audience` and `email_domains` to `render`.
- `docs/next-steps/open-questions.md` — the question about the home of the skills is answered (a later edit by the owner of that file, see the report).

## New Files

- `plain-git/personal-knowledge-base/SKILL.md` — the standalone skill for one person. Copied from `waymark-trail` at version 1.5.1.
- `plain-git/team-knowledge-base/SKILL.md` — the standalone skill for a team. Copied from `waymark-trail` at version 1.5.1.
- `src/kb/templates/agents-intro.md` — the introduction of `AGENTS.md` (the text is in the requirements).
- `src/kb/agents_file.rs` — `render(audience, email_domains) -> String`, the stripping of the skill text, and the version reader.
- `tests/it_plain_git_access.rs` — end-to-end and text tests.

The two skill files are long. They are copied, not written new. This spec does not create them. The first task of `tasks.md` copies them.

## Technical Approach

1. **Source.** `include_str!("../../plain-git/personal-knowledge-base/SKILL.md")` and the team file. Both are constants in `agents_file.rs`.
2. **Strip.** `strip(skill) -> (version, body)`: read `version:` from the frontmatter (between the first two `---` lines), cut the frontmatter, and cut from the line `### Update of this skill` to the end of the file. A missing `version` or heading is a build-time panic in a unit test, so a bad copy fails the suite.
3. **Render.** `intro` + (for `team` with domains: `\n## Team settings\n\nAllowed email domains for commits: a, b.\n`) + `\n## Plain-Git rules\n\n` + begin marker + body + end marker + the closing part. LF line ends, one final newline.
4. **Marker.** `<!-- plain-git-rules begin: <skill name> <version> -->` and `<!-- plain-git-rules end -->`.
5. **Independence.** A unit test checks that the skill bodies hold neither `waymark.toml` nor `.waymark`, and that the word `waymark` appears only inside `waymark-trail`.
6. **Drift.** A unit test checks that every `###` heading of the personal skill is in the team skill, in the same order. The two skills share a common text, and a hand edit of one could break that.
7. **Pin test.** A test reads the `version` of each file and the marker of the generated `AGENTS.md` and compares them.
8. **No updates.** `init` writes the file once. No other code writes it.

## Cross-Feature Dependencies

- `create-knowledge-base` — the caller.
- `move-documents` — the rule "move with `git mv`" is in the skill. The text must agree with `move-documents` (checked in review).
- `login-and-identity` — the author rules differ in two points (see `decisions.md`).
- `distribution` — the Homebrew formula builds from the repository, so `plain-git/` is in the tarball. `git archive` must not exclude it.

## Risks

- The skills grow, and `AGENTS.md` grows with them. Mitigation: a test that fails above 40 KB.
- The copy in `plain-git/` and the original in `waymark-trail` drift before the retirement. Mitigation: the copy is taken at the end, and the owner retires `waymark-trail` right after.
- The skill text asks for a session-start message and a pull. A web agent without network access may fail the pull. The skill handles "no remote", but a no-network case is not tested (see `todo.md`).

## Open Questions

None.
