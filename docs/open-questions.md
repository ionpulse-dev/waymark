# Open questions

Questions that need an answer before or during planning. Each spec also lists its own open questions in its `requirements.md`. When a question is answered, record the answer in the `decisions.md` of the affected spec and remove the question here.

## Project level

- **Confluence and Jira.** What is the relation of waymark to Confluence and Jira: link only, import, or replace? Today documents may link to wiki pages as sources, and nothing more.
- **CI on the Git host.** Is CI (for example GitHub Actions) allowed on the Git hosts of the target organizations? If yes, CI can run `lint`, index generation, and secret scans without a server that we operate.
- **Contributions.** Is a contributor license agreement needed before the first outside pull request?

## Architecture direction

See [architecture-direction.md](architecture-direction.md) for the full list. The main ones:

- Do Claude Code, GitHub Copilot, and OpenAI Codex support remote MCP servers, managed MCP config, per-tool permission prompts, elicitation, and server instructions?
- Does a project MCP entry override a user entry of the same name in all three tools, and do they ask before they start a project server?
- Which login flows do Claude and ChatGPT accept for a self-hosted MCP server?
- How does a read budget per session fit a stateless server, and how does the server find the knowledge base?
- Should the built-in default of the limit `access` be `read-only`?

- **Home of the plain-Git skills.** The standard now lives in `standard/`. Where do the standalone plain-Git skills (`personal-knowledge-base`, `team-knowledge-base`) live once `waymark-trail` is retired? Options: a folder in this repository, or another repository.

## Build environment

- Rust toolchain version and the MCP library.
- The signing identity for the binary.
- The container registry for the server image.
- Obsidian version for the release check of the sample vault.

## Agent tools

- Verified MCP config formats and locations of Claude Code, GitHub Copilot, and OpenAI Codex (see `specs/features/agent-tool-setup`).
- How waymark pins a version of the standalone plain-Git skill, and where the skill lives after `waymark-trail` is retired (see `specs/features/plain-git-access`).
