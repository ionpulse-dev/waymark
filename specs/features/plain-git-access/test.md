# Tests: Plain-Git access

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** the `init` tool, **When** it ends, **Then** `AGENTS.md` contains the introduction, the plain-Git rules, and the skill version in the marker line. | A1 |
| **Given** the repository opened in Claude Code on the web, **When** the agent follows `AGENTS.md`, **Then** it can search and write a valid document with no install. | M1 |
| **Given** an agent with the waymark MCP tools, **When** it reads `AGENTS.md`, **Then** it uses the tools. | M2 |
| **Given** a new release, **When** the template is built, **Then** the copy of the skill has the pinned version. | A2 |
| **Given** a repository without waymark, **When** the standalone skill is used, **Then** it never mentions waymark IDs or settings files. | A3 |
| **Given** `audience: personal`, **When** `AGENTS.md` is written, **Then** it holds the personal skill. **Given** `team`, **Then** it holds the team skill and the allowed email domains. | A4, A5 |
| **Given** the skill copy in `AGENTS.md`, **When** it is read, **Then** it has no frontmatter and no section about updating the skill. | A6 |

## Test Cases

- [x] [auto] A1: `init` on `World::empty()`: the pushed `AGENTS.md` equals `render(audience, domains)`, starts with the introduction, has the begin and end markers, and ends with the section "Rules of this team" (`tests/it_plain_git_access.rs`)
- [x] [auto] A2: the version in the marker equals the `version` of the file in `plain-git/` for both skills (`tests/it_plain_git_access.rs`)
- [x] [auto] A3: the skill bodies have no `settings.toml`, no `.waymark`, and the word `waymark` only in `waymark-trail` (`src/memrepo/agents_file.rs`)
- [x] [auto] A4: `personal` gives the personal skill name in the marker and no "Team settings" part (`tests/it_plain_git_access.rs`)
- [x] [auto] A5: `team` with `email_domains = ["example.org", "example.net"]` gives the team skill name and the line `Allowed email domains for commits: example.org, example.net.`. `team` without domains has no such part (`tests/it_plain_git_access.rs`)
- [x] [auto] A6: the text between the markers has no `---` frontmatter at its start, no `name:` line, and no heading "Update of this skill" (`src/memrepo/agents_file.rs`)
- [x] [auto] A7: every `###` heading of the personal skill is in the team skill, in the same order (`src/memrepo/agents_file.rs`)
- [x] [auto] A8: `AGENTS.md` is under 40 KB and uses LF line ends and one final newline (`src/memrepo/agents_file.rs`)
- [x] [auto] A9: both skill files in `plain-git/` start with a frontmatter that has `name`, `description`, and `version` (`src/memrepo/agents_file.rs`)
- [x] [auto] A10: `git archive HEAD` of a clean checkout holds `plain-git/personal-agent-memory/SKILL.md` (`tests/it_plain_git_access.rs`)
- [ ] [manual] M1: Claude Code on the web. Steps: create a private repository with the `init` tool of waymark. Open it in Claude Code on the web (no MCP, no install). Ask the agent: "Save a note about X as a document." Expected: the agent reads `AGENTS.md`, asks before it writes (the skill starts in manual mode), writes a document with the required properties, commits `memory: add "<title>"`, and pushes. Then start a waymark server on a clone and call `lint`. Expected: no finding for the new document.
- [ ] [manual] M2: Claude Code on a Mac with the waymark entry. Steps: open the memory repository clone as the working folder. Ask the agent to find a document. Expected: it uses the `search` tool and does not run `git` or `grep` on the repository.
- [ ] [manual] M3: Claude Code on iPhone or the web with a team repository (`audience: team`). Steps: ask the agent to write a document with a no-reply email in the Git config. Expected: the agent stops and asks for a real address.

## Notes

A1 to A5 and A10 use the harness (`World::empty()`). M1 and M3 need a real Git host and a real agent tool.
