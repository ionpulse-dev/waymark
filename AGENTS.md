# AGENTS.md — waymark

This file is the entry point for all AI agents (Claude Code, GitHub Copilot,
Cursor, Codex, and others). Read it at the start of every session.

## This is a pure spec-driven project

Every change to waymark starts with a change to the specification. The spec
drives the code, never the other way around. There is no work without a spec.

**At the start of EVERY session, read these files again, in this order:**

1. [specs/WORKFLOW.md](specs/WORKFLOW.md): the rules of the process.
2. [specs/INDEX.md](specs/INDEX.md): the current status of all specs.
3. [specs/PROJECT.md](specs/PROJECT.md): the vision, the personas, and the
   core concepts.

Do not rely on memory of an earlier session, a summary, or what you think the
workflow says. The files change. Read them anew, every session, before you
plan, write, or change anything.

The short version of the rules (the full rules are in `WORKFLOW.md`):

- A feature or foundation item gets a folder in `specs/features/` or
  `specs/foundation/` with `requirements.md`, `plan.md`, `tasks.md`,
  `test.md`, `decisions.md`, and `todo.md`.
- No implementation without an approved spec and plan.
- Ask for approval at each phase gate.
- Keep the status in `requirements.md` and in `specs/INDEX.md` in sync.
- Record non-obvious choices in the `decisions.md` of the spec.
- If code changes without a spec, update the spec at once.
- Never delete spec files. Git keeps the history.
- Mark work as done at once: tick the task in `tasks.md`, the `[auto]` items in
  `test.md`, and the plan you follow, as soon as the work is finished.

## What waymark is

A generic, open source knowledge base and AI memory for AI agents and the
people who use them: companies, teams, and private people. Plain Markdown in a
Git repository that the owner controls. One program, an MCP server: it runs
locally with no server, or as a shared server that anyone can host.
Read [specs/PROJECT.md](specs/PROJECT.md) for the details.

## Where things are

- `specs/`: the specification (the source of truth). Decisions that belong to one spec are in its `decisions.md`.
- `standard/`: the MKBase Document Standard and its conformance fixtures. Part of this project. A change starts here, in the standard, and then in `specs/foundation/document-format`.
- `docs/`: roadmap, ideas, similar projects, project-wide decisions, and open questions.
- Versions and automatic updates (settings schema, repairs, migrations): [specs/foundation/knowledge-base-repo](specs/foundation/knowledge-base-repo/requirements.md), section "Versions and updates".
- `CHANGELOG.md` and `scripts/release.sh`: what each release is about, and the one command that makes a release (see the skill `.claude/skills/release/SKILL.md`).
- `README.md`: the public project page. Keep it in sync with the specs. Only list advantages that a spec supports.

## What does not belong in this repository

This repository is public. Never commit secrets, company or customer names,
internal host names, or business details such as pricing or revenue plans.
Maintainers keep non-public notes outside this repository.

## Writing conventions

- Use Simple English (ASD-STE100 style): short sentences, active voice.
- Use Markdown. Use relative links between files.
- Say "document", not "note", for the unit of a knowledge base.
- Ask the user one question at a time. Never list several questions in one
  message.
- Write all code, comments, documentation, and commit messages in English.

## License

MIT. See [LICENSE](LICENSE).
