# Open questions

Questions that need an answer before or during planning. Each spec also lists its own open questions in its `requirements.md`. When a question is answered, record the answer in the `decisions.md` of the affected spec and remove the question here.

## Project level

- **Confluence and Jira.** What is the relation of waymark to Confluence and Jira: link only, import, or replace? Today documents may link to wiki pages as sources, and nothing more.
- **CI on the Git host.** Is CI (for example GitHub Actions) allowed on the Git hosts of the target organizations? If yes, CI can run `lint`, index generation, and secret scans without a server that we operate.
- **Contributions.** Is a contributor license agreement needed before the first outside pull request?

## Build environment

- Minimum macOS version.
- Rust toolchain version.
- Xcode and Swift version.
- Message format of the app link (see `specs/foundation/app-link`).
- Apple Developer account for signing and notarization.
- Obsidian version for the release check of the sample vault.

## Agent tools

- Verified global instruction and skill paths of Claude Code, GitHub Copilot, and OpenAI Codex (see `specs/features/agent-tool-setup`).
- How waymark pins a version of the standalone plain-Git skill (see `specs/features/plain-git-access`).
