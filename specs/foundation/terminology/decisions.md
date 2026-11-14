# Decisions: Terminology

_Records why choices were made. Never delete entries — add new ones._

---

## Name the concept "agent memory" — 2026-11-14

**Decision:** The concept is "agent memory", in running text "memory". The tagline
is "Long-term agent memory."

**Alternatives considered:**
- "Knowledge base" — it names one use case, and it makes waymark sound like a
  wiki.
- "Context" — it is what agents consume, but it is vague and overused.
- "Vault" — neutral, but it suggests secrecy and storage only.
- "Memory for AI agents" — "agent memory" is shorter and names the category.

**Reason:** waymark is memory that outlives the short context window of an agent,
for companies, teams, and private people. The name "waymark" (a mark left for the
next traveler) already says this.

## Name the container "memory repository" — 2026-11-14

**Decision:** The Git repository that holds the memory is a "memory repository".
It is written in full in prose. "Repository" alone means the code repository.

**Alternatives considered:**
- "Memory storage" — shorter, but it hides that the thing is Git.
- "Memory git-storage", "memory git-repository" — clumsy hyphenated terms, and
  "git-repository" is redundant.
- "Memory repo" — rejected for prose. It is fine in short identifiers.
- "Memory" for both concept and container — it makes sentences like "a team's
  memory needs a newer waymark" and "several memories per tenant" unclear.

**Reason:** It is the most natural and the most precise. The clash with the code
repository already existed (the specs said "content repository"), and the
"repository alone" rule solves it.

## Keep "knowledge base" as a use case — 2026-11-14

**Decision:** "Knowledge base" stays as the most prominent example use case in the
README, and in text about other tools. It is never the name of what waymark
creates.

**Alternatives considered:**
- Remove the phrase completely — it loses the clearest story for companies.

**Reason:** A team knowledge base is an accurate and common use of waymark. The
problem is only the use of the phrase as the name of the whole product.

## Do not rewrite dated records — 2026-11-14

**Decision:** Entries in `decisions.md`, the project decisions, and the superseded
specs keep their words.

**Reason:** They record what was said at the time. Git keeps the history, and a
rewrite would make old entries disagree with their own context.

## Rename the identifiers now — 2026-11-14

**Decision:** The identifiers that carry the old word are renamed in this spec:
the clone folder suffix (`-agent-memory`), the two plain-Git skills
(`personal-agent-memory`, `team-agent-memory`), the slug fallback, the spec
folders, and the code names. There is no fallback for the old names.

**Alternatives considered:**
- Defer the identifiers to a later spec — only needed to protect existing users
  and versioned formats.

**Reason:** The project is not used anywhere, so no migration is needed. Renaming
now is cheaper than later, and it keeps words and names in agreement.

## No check script — 2026-11-14

**Decision:** No check script and no allowlist. The vocabulary lives in the
writing conventions of `AGENTS.md`.

**Reason:** After the rename there is nothing to check. The conventions are the
guard. A final search closes the sweep. Whether one automated test on tool
descriptions is still needed is an open question.

## Example repository name — 2026-11-14

**Decision:** A new memory repository has no default display name. The example
name in docs is `waymark-agent-memory`.

**Reason:** A fixed default would give every user the same name.

## Rename the standard to Waymark Memory Standard — 2026-11-14

**Decision:** The standard is the **Waymark Memory Standard**. The marker folder is
`.waymark/` and the file is `standard/waymark-1.0.md`.

**Alternatives considered:**
- MAM (Markdown Agent Memory), `.mam/` — neutral and independent of the product.
- AMS (Agent Memory Standard), `.ams/` — neutral, but broader than the real scope.

**Reason:** One name for the product, the standard, and the marker folder means
less to explain, and a person who opens a repository sees `.waymark/` and knows
what made it. The cost is that the standard is no longer independent of the
product. We accept that.

## Settings file is `.waymark/settings.toml` — 2026-11-14

**Decision:** The settings file moves from `.mkbase/waymark.toml` to
`.waymark/settings.toml`.

**Alternatives considered:**
- Keep `waymark.toml` — `.waymark/waymark.toml` repeats the name.

**Reason:** The folder already says "waymark". The file says what it holds.

## Code name `memrepo`, spec folders `*-memory-repository` — 2026-11-14

**Decision:** The code name `kb` becomes `memrepo` (module `src/memrepo`, type
`MemRepo`, variable `memrepo`). The spec folders become `memory-repository`,
`create-memory-repository`, and `join-memory-repository`.

**Alternatives considered:**
- `memory` — too generic for code, and it suggests RAM.
- `memory_repository` — precise but long. It would appear in hundreds of places.
- `agent_memory` — names the idea, not the repository on disk.

**Reason:** Spec folders are named by what the user accomplishes, so the full words
fit. In code a short, unique name is easy to read and to grep. Rule 1 allows short
identifiers.
