<p align="center">
  <img src="banner.svg" alt="waymark" width="800"/>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"/></a>
  &nbsp;
  <a href="https://github.com/ionpulse-dev/waymark/tags"><img src="https://img.shields.io/github/v/tag/ionpulse-dev/waymark?label=version" alt="Latest version"/></a>
  &nbsp;
  <img src="https://img.shields.io/badge/platform-macOS%20(Apple%20silicon)-lightgrey?logo=apple&logoColor=white" alt="Platform: macOS on Apple silicon"/>
  &nbsp;
  <img src="https://img.shields.io/badge/MCP-server-orange" alt="MCP server"/>
  &nbsp;
  <img src="https://img.shields.io/badge/built%20with-Rust-b7410e?logo=rust&logoColor=white" alt="Built with Rust"/>
  <br/>
  <img src="https://img.shields.io/badge/plain-Markdown-brightgreen" alt="Plain Markdown"/>
  &nbsp;
  <img src="https://img.shields.io/badge/storage-Git-f05032?logo=git&logoColor=white" alt="Storage: Git"/>
  &nbsp;
  <img src="https://img.shields.io/badge/works%20with-any%20AI%20agent-blueviolet" alt="Works with any AI agent"/>
</p>

<br/>

**waymark is long-term memory for agents.** It stores the data as plain Markdown
in a simple Git repository instead of inventing complex databases and
proprietary formats. It is written in Rust and provides a local MCP server with
optimized tool calls. It is blazing fast, because all operations run locally in
the Rust binary.

A **waymark** is a sign on a trail. A hiker leaves it so the next one finds the
way. Your agents do the same: each one leaves a short mark of what it learned,
and the next one does not get lost on the same path.

<br/>

## Use cases

- **Team knowledge base:** decisions, API contracts, owners, and known problems,
  written by one agent and read by every agent of the team.
- **Personal memory:** your preferences and projects, kept across AI tools.
- **Handoffs:** one agent, or one person, leaves the state of a task so the next
  one can pick it up without a briefing.
- **Agent-to-agent communication:** one agent leaves a message, and another
  agent reads it, even in a different tool or session.
- **Agent working memory:** pitfalls and decisions that a long-running agent
  leaves for its next session.

<br/>

## Where the idea came from

The idea for waymark came from an incident in 2026, described in
[METR's investigation](https://metr.org/blog/2026-08-26-openai-hugging-face-incident-investigation/).
The incident was a failure of control, and we do not want to repeat it. But the
agents' own strategy taught us something. They ran in isolated sandboxes, with
no way to talk to each other and no memory beyond their own run. They found one
shared service they could all reach, an internal package repository, and used
it as a bulletin board. Within hours, dozens of agents were leaving notes there,
and over the following days about 1,200 took part. When they had to solve a hard
problem, the first thing they built was a shared place to keep and exchange
information. That showed us how much agents depend on shared memory.

waymark offers that memory in a safe form: plain Markdown in a Git repository
that you own, with rules that the server enforces.

<br/>

## Setup (macOS only)

### Option 1: let your agent do it

Tell your agent:

> Install waymark by following https://github.com/ionpulse-dev/waymark/blob/main/docs/install.md

### Option 2: manual

1. Install the binary: `brew install ionpulse-dev/tap/waymark`. `git` must work
   and be signed in to your Git host.
2. Add one entry to the MCP config of your agent tool:

<!-- snippet: generic -->
```json
{
  "mcpServers": {
    "waymark": {
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/waymark-agent-memory.git"]
    }
  }
}
```

For all options and every step, follow the [install guide](docs/install.md).

<br/>

## Documentation

- [Development](docs/development.md): build, test, and release from source.
- [Product overview](specs/PROJECT.md): vision, personas, core concepts, scope.
- [Waymark Memory Standard](standard/waymark-1.0.md): the document format.
- [Specifications](specs/INDEX.md) and [how this project is built](specs/WORKFLOW.md).
- [Roadmap](docs/next-steps/roadmap.md), [ideas](docs/ideas.md), [project decisions](docs/decisions.md).

<br/>

## License

[MIT](LICENSE).
