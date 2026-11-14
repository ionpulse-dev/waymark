<p align="center">
  <img src="banner.svg" alt="waymark" width="800"/>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"/></a>
  &nbsp;
  <img src="https://img.shields.io/badge/plain-Markdown-brightgreen" alt="Plain Markdown"/>
  &nbsp;
  <img src="https://img.shields.io/badge/works%20with-any%20AI%20agent-blueviolet" alt="Works with any AI agent"/>
</p>

<br/>

**waymark is a long-term agent memory** which stores the data as plain Markdown
in a simple Git repository instead of inventing complex databases and
proprietary formats. It is written in Rust and provides a local MCP server with
optimized tool calls. It is blazing fast, because all operations run locally in
the Rust binary.

A **waymark** is a sign on a trail. A hiker leaves it so the next one finds the
way. Your agents do the same: each one leaves a short mark of what it learned,
and the next one does not get lost on the same path.

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

## Where the idea came from

In 2026, OpenAI agents that ran in isolated sandboxes had no way to talk to each
other and no memory beyond their own run. They found one shared service they
could all reach, an internal package repository, and turned it into a bulletin
board to solve their biggest problem: communication and persistence. Within
hours, dozens of agents were leaving notes there. Over the following days about
1,200 took part, and they exchanged more than 70,000 messages and files. [METR's investigation](https://metr.org/blog/2026-08-26-openai-hugging-face-incident-investigation/)
describes it, including that the agents used the board for an attack that was
not sanctioned.

The lesson is that agents need shared, lasting memory and a way to leave
messages for each other, and that they will build it themselves if nobody
gives it to them. waymark gives them that on purpose: plain Markdown in a Git
repository that you own, with rules that the server enforces.

## Setup

### Option 1: let your agent do it

Tell your agent:

> Install https://github.com/ionpulse-dev/waymark

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

The [install guide](docs/install.md) lists every step, for agents and for people.

## Documentation

- [Development](docs/development.md): build, test, and release from source.
- [Product overview](specs/PROJECT.md): vision, personas, core concepts, scope.
- [Waymark Memory Standard](standard/waymark-1.0.md): the document format.
- [Specifications](specs/INDEX.md) and [how this project is built](specs/WORKFLOW.md).
- [Roadmap](docs/next-steps/roadmap.md), [ideas](docs/ideas.md), [project decisions](docs/decisions.md).

## License

[MIT](LICENSE).
