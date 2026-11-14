# Plan: Knowledge base repository

## Overview

A `kb` module knows the layout of a content repository: it finds the root, reads and validates `.waymark/settings.toml`, and writes the layout for a new knowledge base (used by the `init` tool). It does not read documents (see `document-format`) or run Git (see `git-sync`).

## Components Affected

- `trust-model` — takes the `[limits]` and `[approval]` sections from this module.
- `create-memory-repository` — calls `kb::init`.
- `project-binding` — compares the `id` with `--memory-repository-id`.

## New Files

- `src/kb/mod.rs` — `KnowledgeBase`: root path, settings, and checks.
- `src/kb/settings.rs` — the structs for `.waymark/settings.toml` with `serde` and `toml`, with defaults and validation.
- `src/kb/layout.rs` — find the root (the folder with `.waymark`), the files that `init` writes, and the templates.
- `src/kb/templates/AGENTS.md` — template of the rules file for agents.
- `src/kb/templates/gitignore` — template for `.gitignore`.
- `src/kb/templates/obsidian-app.json` — Obsidian defaults.
- `tests/it_memory_repository.rs` — end-to-end tests.

## Technical Approach

1. The root is `<repo>/agent-memory/` when the repository has that folder with `.waymark/`. If `--path` points into a folder tree, the first folder that contains `.waymark/` going down from the repository root is the root. If none exists, the call fails with `setup_problem` ("not a knowledge base, run `init`").
2. `settings.rs` parses `settings.toml`. Unknown keys are kept and reported as a notice, never rejected, so newer files work with older binaries. A missing `id` is `setup_problem`. `id` must be a UUID.
3. `init` writes the layout of the requirements: `AGENTS.md`, `.obsidian/app.json`, `.gitignore`, `agent-memory/.waymark/version` (`1.0`), `agent-memory/.waymark/settings.toml` with a new UUID, and `agent-memory/.waymark/exclude` (empty). It refuses to overwrite an existing file.
4. The lock and the sync state live in `.git/` and are owned by `git-sync`.
5. The shared limits are returned as a `Limits` value for the `config` module.

## Dependencies on Other Foundation Items

- `document-format` — for the document rules, the marker folder, and the standard version file.

## Risks

- A settings key that `trust-model` and this file name in two ways. Mitigation: the key names in this spec are the single source, and the `config` module reuses the structs.

## Open Questions

None.
