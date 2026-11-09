---
type: foundation
status: draft
created: 2026-11-08
updated: 2026-11-09
---

# Trust model

## Technical Goal

Define who is responsible for what agents write, how much an agent may do alone, and how readers judge a document. waymark works like a company wiki that AI agents execute: the person is responsible, and the crowd fixes mistakes.

## Dependents

- `cli-core` — every result carries the current settings.
- `approval-dialog`, `pause-and-resume`, `project-binding` — enforce parts of this model.
- `agent-skill` — turns the levels into rules of behavior.
- `verification` — the optional owner check builds on it.

## Deliverables

- The wiki model and the rules of authorship.
- The read level, the write level, and the confirm-in setting, with their limits.
- The repository access modes.
- The rules for disputed documents.
- A list of the honest limits of the model.

## Scope

**In scope:**
- Responsibility, authorship, levels, access modes, disputes, confidentiality.

**Out of scope:**
- A pull request gate where a second person approves a write. This is an idea for after the first release.
- Hard enforcement against a person or agent who bypasses waymark.

## Constraints

- Every person needs an account on the Git host anyway. Every commit and push is made with that account. There is no generic bot user and no agent attribution.
- Users need no Git knowledge. They never see a branch or a pull request.
- There are no cloud agents. Every agent action happens in a session that a person starts.

### Wiki model

- The person whose agent writes a document is responsible for it. History and blame show who changed what.
- Wrong documents are fixed like in a wiki: another agent or person changes them, or marks them `disputed` with a reason.
- Readers treat a document as a wiki page: use it, name the author and date, and check important facts against the code.
- Owner verification is optional. A knowledge base turns it on in `.mkbds/waymark.toml`. It is off by default.

### Levels

| Setting | Values | Default |
|---|---|---|
| **Read level** | `auto`: the agent searches and opens documents on its own. `manual`: only when the person asks. | `manual` |
| **Write level** | `auto`: the agent decides what to write and writes; undo is the safety net. `confirm`: the agent decides and asks first. `manual`: the person asks, the agent writes without confirmation. | `manual` |
| **Confirm in** | `chat`: the agent asks in the conversation (advice only). `app`: the native dialog shows the full document (enforced). | `chat` |

Rules:
- Read level and write level are independent.
- Confirm in applies only at write level `confirm`. At that level it also covers moves and bulk changes.
- Only the person changes the levels, in the app. The CLI can read them but has no command to change them. An agent can never change them.
- The shared settings set a start value and the loosest value allowed. A person can be stricter, never looser. Order from loosest to strictest: `auto`, `confirm`, `manual` (read: `auto`, `manual`).
- The repository access mode wins over the levels.
- The app keeps a hash of the personal settings. If the file differs, the CLI uses the last approved version and the app notifies the person.

### Repository access modes

Each connected knowledge base has one mode: `off`, `read only`, or `read and write`. A new one starts `off`. The mode is set only in the app and applies at once, because the CLI checks it on every call. In `read only`, writes fail at any write level.

### Disputed documents

- A disputed document is not to be relied on. Readers mention the dispute and its reason.
- A document has no stale mark. If `updated` is old, a reader checks the facts that matter before relying on them and says how old the document is.
- The folder shows which team to ask.

### Limits (honest)

- The CLI cannot prove that a human, not an agent, answered "yes" in the chat. Only `confirm in = app` is enforced.
- Nothing stops an agent from running raw Git commands in the content repository. Only the rule in the skill and in the repository `AGENTS.md` forbids it. Without the CLI, the plain-Git rules allow Git.
- The dialog prevents mistakes, not deliberate bypass. A person can click Publish without reading.
- Direct writes to `main` need a branch protection setting that allows them.
- Once a document is pushed, assume every reader has it. Prevention is the only real safeguard.

## Acceptance Criteria

- **Given** an agent write, **When** it is committed, **Then** the commit author is the person's Git account.
- **Given** a shared limit `write.max = "confirm"`, **When** a person sets `auto`, **Then** the CLI uses `confirm`.
- **Given** an agent, **When** it tries to change a level through the CLI, **Then** no such command exists.
- **Given** access mode `read only`, **When** a write runs at write level `auto`, **Then** the write fails.
- **Given** a new setup, **When** the person has not changed anything, **Then** read level and write level are `manual` and confirm in is `chat`.

## Open Questions

- Which per-person defaults suit private use better than the strictest start?
