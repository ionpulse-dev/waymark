# Decisions: Plain-Git access

> **Note 2026-11-11:** All decisions stay valid. Mentions of the CLI now mean the MCP tools.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## The rules ship with every repository — 2026-11-07

**Decision:** `AGENTS.md` in every content repository always carries the plain-Git rules, on by default.

**Reason:** An agent that finds the repository by chance has clear rules and does not damage the data. Loading the repository in Claude Code on the web or iPhone gives the agent the whole knowledge base and the rules, with no CLI or MCP.

## A standalone skill, in its own repository — 2026-11-08

**Decision:** The rules are the standalone skill of the `waymark-trail` project. waymark copies it into the template and pins a version. The skill never depends on waymark.

**Alternatives considered:**
- A separate skill file inside waymark — the skill would be tied to waymark and could not spread on its own.
- Two modes inside one `AGENTS.md` without a standalone text — hard to version and to share.

**Reason:** The skill is useful without the project and can spread on its own.

## Fallback, not main path — 2026-11-07

**Decision:** This is a second access channel and a safe default. It is not a replacement for the CLI.

**Reason:** It opposes the "agents do not run raw Git" rule, so only the skill rule holds.

## MCP tools replace the CLI in `AGENTS.md` — 2026-11-11

**Decision:** The `AGENTS.md` of a content repository says: use the waymark tools if present, else the plain-Git rules.

**Reason:** MCP is the only interface for agents. The plain-Git rules remain the fallback.

## The standalone skills live in `plain-git/` of this repository — 2026-11-14

**Decision:** After `waymark-trail` is retired, the folder `plain-git/` in this repository holds the two standalone skills as `plain-git/personal-knowledge-base/SKILL.md` and `plain-git/team-knowledge-base/SKILL.md`. waymark reads them at build time. They stay installable as skills, because each is a folder with a `SKILL.md`. Only the finished files move. The parts and the build script of `waymark-trail` do not move (see `todo.md`). A change to the text starts in this spec, as the workflow requires.

**Alternatives considered:**
- Another repository for the skills (rejected: a second repository to keep in step, and the standard is already here).
- Keep them in `waymark-trail` (rejected: that repository is to be retired).
- A folder under `standard/` (rejected: the standard defines the format, not the behavior of an agent).

**Reason:** One repository, one spec workflow, and the skills ship with the binary that uses them. Decided autonomously. Answers the open question of this spec and of `docs/next-steps/open-questions.md`.

## The pin is the version in the file — 2026-11-14

**Decision:** The pin is the `version` field in the frontmatter of the file in `plain-git/`. `init` repeats it in the marker line of `AGENTS.md`. A test checks that both are equal. The binary carries the text with `include_str!`.

**Alternatives considered:** A Git tag or commit hash of another repository (rejected: no other repository now). A hash of the text (rejected: a person cannot read it).

**Reason:** A version a person can read, and a copy in the binary that cannot drift from the file. Decided autonomously.

## What the copy removes from the skill — 2026-11-14

**Decision:** The copy in `AGENTS.md` drops the frontmatter and the section "Update of this skill". All other text stays as it is.

**Alternatives considered:** Keep the update section (rejected: it points to a repository that is to be retired, and the agent would ask the person to update a copy that waymark owns). Rewrite parts of the skill for waymark (rejected: the skill must work without waymark).

**Reason:** The version of the copy changes only with a new release of waymark. Decided autonomously.

## One introduction, two skills — 2026-11-14

**Decision:** `AGENTS.md` is built from a fixed introduction (`src/kb/templates/agents-intro.md`), an optional team part, the skill text, and a part for the team's rules. There are no two full templates.

**Supersedes:** the single template file `src/kb/templates/AGENTS.md` in the plan of `knowledge-base-repo`. That file is replaced by `agents-intro.md`, and `kb::agents_file` builds the rest.

**Alternatives considered:** Two full template files (rejected: the skill text would be copied twice into the repository, and a change would need two edits).

**Reason:** One place for the introduction, and the skills stay the only source of the rules. Decided autonomously.

## The skills may differ from the waymark checks — 2026-11-14

**Decision:** The plain-Git rules keep their own text. Two differences stay: the commit prefix (`kb:` against `waymark:`) and the blocked example domains in the author email. They are listed in `todo.md`.

**Reason:** The skill is independent of waymark. A fix of the skill is a later change. Decided autonomously.

## An existing `AGENTS.md` is never changed — 2026-11-14

**Decision:** No tool updates the rules in an existing `AGENTS.md`. A person who wants a newer version copies the text from the release.

**Alternatives considered:** An update tool (rejected: the tool set stays small, and the file may hold the team's own text).

**Reason:** The file belongs to the team. A check of the version in `status` is in `todo.md` of `maintenance`.

## Implementation notes — 2026-11-14

- The two skill files were copied from `waymark-trail` (version 1.5.1) to `plain-git/`. The personal copy is byte-equal to `~/.claude/skills/personal-knowledge-base/SKILL.md`. Decided autonomously.
- `kb::agents_file` has `Audience`, `render`, `strip`, `skill_version`, and `frontmatter_value`. `kb::init_with(dir, name, slug, audience, email_domains)` writes `AGENTS.md` through `render`. `kb::init` keeps its signature and uses `Audience::Personal`. The tool `init` (create-knowledge-base) calls `init_with`.
- A1, A4, and A5 call `kb::init_with` directly, because the `init` tool does not exist yet. `create-knowledge-base` adds the tool test that goes through the tool.
- The text of both skills says "as the section \"Update of this skill\" says". The copy drops that section, so the sentence points to nothing. The test A6 checks the heading only. See `todo.md`.
- The old template `src/kb/templates/AGENTS.md` is removed. `agents-intro.md` replaces it.
- A10 skips itself in a folder that is not a Git checkout, and when `plain-git/` is not yet tracked.
