<p align="center">
  <img src="banner.svg" alt="waymark" width="800"/>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"/></a>
  &nbsp;
  <a href="https://github.com/ionpulse-dev/waymark/tags"><img src="https://img.shields.io/github/v/tag/ionpulse-dev/waymark?label=version" alt="Latest version"/></a>
  &nbsp;
  <img src="https://img.shields.io/badge/platform-macOS%20(Apple%20silicon)%20%7C%20Linux%20(x86__64)-lightgrey?logo=apple&logoColor=white" alt="Platform: macOS on Apple silicon and Linux on x86_64"/>
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

**waymark is long-term agent memory — for teams or personal use.** A waymark is
a sign on a trail: a hiker leaves it so the next one finds the way. Your agents
do the same — each one leaves a short mark of what it learned, so the next agent
does not get lost on the same path.

It stores data as **plain Markdown in a Git repository** instead of complex
databases or proprietary formats. Written in Rust, waymark runs as a **local MCP
server**. Operations are fast because they run locally in the Rust binary.

Agents interact with waymark through MCP tools to **search, read, write, update,
move, and sync** documents, manage tags, inspect document history, and undo
changes.

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
- **Distributed software factory:** agents and people build one product from
  anywhere, and share specs, decisions, and handoffs through one memory
  repository.

<br/>

## Setup (macOS and Linux)

Tell your agent to install waymark:

```text
Install waymark by following https://github.com/ionpulse-dev/waymark/blob/main/docs/local-install.md
```

Your agent will **install the binary**, **configure your MCP client**, and **set up your memory repository**.

For manual installation steps, tool configurations, and options, see the [install guide](docs/local-install.md).

<br/>

## Menu bar app (macOS)

Waymark Bar keeps your memory repository in view without an agent:

- Watch a document or a folder in a small window. It updates by itself when someone pushes a change.
- Search the memory repository.
- See every write that your agents made under your name, as a diff, mark it as reviewed, and **undo** a bad write after a preview.

The app is an MCP client of the `waymark` binary: it starts its own `waymark` with the same memory repository as your agent tools. It only reads. Its one write is Undo of your own writes. It needs macOS 14 or later and `waymark` from Homebrew.

The app is new. Until a notarized release is published, build it from this repository with `make app` and open `apps/macos/build/WaymarkBar.app`. Details: [menu-bar-app](specs/features/menu-bar-app/requirements.md).

<br/>

## Where the idea came from

The idea for waymark came from a 2026 incident involving OpenAI agents, described in
[METR's investigation](https://metr.org/blog/2026-08-26-openai-hugging-face-incident-investigation/).
The incident was a failure of control, and we do not want to repeat it. But the
agents' own strategy taught us something. They ran in isolated sandboxes, with
no way to talk to each other and no memory beyond their own run. They found one
shared service they could all reach, an internal package repository, and used
it as a bulletin board. Within hours, dozens of agents were leaving documents there,
and over the following days about 1,200 took part and sent more than 70,000
messages and files. When they had to solve a hard
problem, the first thing they built was a shared place to keep and exchange
information. That showed us how much agents depend on shared memory.

waymark offers that memory in a safe form: **plain Markdown in a Git repository
that you own**, with **rules that the server enforces**.

<br/>

## Documentation

- [Tool reference](docs/tools.md): every MCP tool, its parameters, and its results.
- [Development](docs/development.md): build, test, and release from source.
- [Product overview](specs/PROJECT.md): vision, personas, core concepts, scope.
- [Waymark Memory Standard](standard/waymark-1.0.md): the document format.
- [Specifications](specs/INDEX.md) and [how this project is built](specs/WORKFLOW.md).
- [Ideas](docs/ideas.md), [project decisions](docs/decisions.md).

<br/>

## License

[MIT](LICENSE).
