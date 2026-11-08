# Project decisions

Decisions that apply to the whole project. Decisions that belong to one spec are in the `decisions.md` of that spec. Never delete an entry. Add new ones, and mark a replaced one as superseded.

---

## Storage is Markdown in Git, and nothing else — 2026-11-06

**Decision:** Documents are Markdown files in a Git repository. There is no backend interface and no wiki backend such as Confluence.

**Alternatives considered:**
- A second backend for a wiki. A live wiki read has no local copy, and a cache of a wiki gets out of sync fast.

**Reason:** One storage keeps the tool small and lets every feature use Git fully: local search, blame per line, history, and undo. Reasons against a wiki as agent knowledge: pages are long and mix current and old content, so agents load too much; every read is an API call that is slow and rate-limited; there are no format or secret checks before a write; history is per page version, not per line; the page format is not plain Markdown; and it needs a license, while admins often restrict write access through the API.

## Everything runs locally, no servers that we operate — 2026-11-06

**Decision:** The Git remote belongs to the owner. The project operates no servers.

**Reason:** A hard user constraint. The only shared part is the Git host that the owner already has.

## No cloud agents — 2026-11-06

**Decision:** Every agent action happens in a local session that a person starts, with the person's own license. There is no background curator.

**Reason:** The person stays in control. Capture must be part of normal work, so the skill tells the agent when to write.

## Users need no Git knowledge — 2026-11-06

**Decision:** waymark does all Git work. Users never see a branch, a commit, or a conflict.

**Reason:** Managers and other non-developers are users too.

## A generic, open source product — 2026-11-06

**Decision:** waymark is a generic knowledge base and AI memory for companies, teams, and private people. One knowledge base is one Git repository, whoever owns it. The project is open source under the MIT license. Nothing company-specific is in this repository.

**Reason:** Private people want their own AI memory under their control, and other companies can use the same tool.

## The name is `waymark` — 2026-11-06

**Decision:** The product and CLI are named `waymark`.

**Reason:** A waymark is a marker on a trail: each traveler leaves a mark for the next one, and agents do the same with knowledge. The name is easy to say and type, and a search found no developer tool or AI project with the same name.

## The specification is public — 2026-11-08

**Decision:** The specification, the design decisions, the roadmap, and the ideas are public and live in this repository. The project follows spec-driven development. Business, company-specific, and internal planning material stays outside.

**Reason:** In spec-driven development the spec and the code share one checkout, and contributors must see the spec to improve it.

## Sources of the design problem — 2026-11-06

**Decision:** The hard problems are write quality, concurrent writes, and staleness, not storage. The design starts with the file layout and the write rules for these three.

**Reason:** Git plus Markdown is a good base: every agent can read files and use Git, history, blame, and review come free, and there is no lock-in. Up to some thousand files, plain search is enough.

## The standalone skill is a separate project — 2026-11-08

**Decision:** The plain-Git rules for agents without the CLI are the `waymark-trail` skills, in their own repository. waymark depends on them, never the other way around.

**Reason:** They are useful without waymark and can spread on their own.

## License and copyright holder — 2026-11-08

**Decision:** MIT license. The copyright holder is IonPulse, the company behind the project's GitHub organization, not a person. The Unlicense and CC0 were rejected: the rights of an author cannot be fully waived in some countries (for example Austria and Germany), and the Unlicense has no fallback license.

**Reason:** MIT is simple and permissive, and the holder should be the organization that owns the repositories.
