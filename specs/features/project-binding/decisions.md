# Decisions: Project binding

> **Note 2026-11-11:** "Bind a folder to one knowledge base" stays valid. "The binding holds a UUID", "Name and URL in the binding, setup by confirmation", and "A personal default for private use" are superseded by "The binding is an MCP entry" below.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Bind a folder to one knowledge base — 2026-11-07

**Decision:** A hidden `.waymark/` folder in the project root binds it to exactly one knowledge base.

**Alternatives considered:**
- Only one active knowledge base at a time — too rigid, and a closed one cannot be switched per session.
- A global active knowledge base — mixing needs only one wrong switch.
- Guess from the working directory — implicit and fragile.

**Reason:** Mixing needs two knowledge bases in one session, and a session sees only the one of its folder.

## The binding holds a UUID — 2026-11-07

**Decision:** The binding is a UUID that `init` stores in the shared file. The agent passes it on every call.

**Alternatives considered:**
- The slug — two organizations may use the same slug.

**Reason:** Explicit and robust. The agent knows which knowledge base it works in, and the skill stays static and version-matched.

## Name and URL in the binding, setup by confirmation — 2026-11-07

**Decision:** The binding also holds the name and Git URL, used for messages and a setup offer. A person must confirm any clone.

**Reason:** Easy joining, without the risk of a poisoned knowledge base.

## A personal default for private use — 2026-11-07

**Decision:** A person can set a default knowledge base that works in every folder. A project binding wins. The default is off unless set.

**Reason:** Private use works everywhere. Binding a folder changes which knowledge base is used, so an agent must not do it.

## The binding is an MCP entry — 2026-11-11

**Decision:** A project binds a knowledge base by a project-level MCP entry with `--repo`. A user-level entry is the personal default. There is no `.waymark/` folder and no `link` command.

**Reason:** the config of the agent tool is the natural place, it is committed with the project, and one server serves one knowledge base. Alternative considered: a hidden folder with an ID. It needs a CLI and an app to bind safely.
