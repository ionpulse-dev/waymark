# Decisions: Document names

## A name chosen by the caller, the title stays — 2026-11-16

**Decision (product owner):** `add_document` takes an optional `name`. Without it, the name comes from the title as before. The title stays a required, free sentence.

**Alternatives considered:**
- Remove the title, and use the file name or the first heading (rejected: search results lose the one-sentence claim, a better title would need a rename that changes the ID, and it breaks the standard).
- Only a better rule for names made from titles (rejected: the agent still could not choose `readme`).

**Reason:** The file name is a short, stable handle for links. The title is a sentence that says what the document claims, and it can change. The agent had no allowed way to make `people/readme`, so it went around the tools.

## A taken name fails, with no suffix — 2026-11-16

**Decision:** A name from the caller that is taken fails with `validation_failed`. A name made from the title still gets a hex suffix.

**Reason:** The caller chose the name on purpose. A silent `readme-3f2a` would surprise them.

## `sync` stays and uses the rules of the write tools — 2026-11-16

**Decision (product owner):** `sync` stays. It sorts each manual edit into the matching write tool (`add_document`, `update_document`, `delete_document`, `move`) and applies its rules on top of `lint`.

**Alternatives considered:**
- Remove `sync` and publish manual edits automatically (rejected: an agent can still write files and run `git`. Without `sync` the edits are published by the next write anyway. The gap was the weaker checks, not the tool).
- Only a better description (rejected: the rules would still differ).

**Reason:** One set of rules for every change, wherever it comes from. Then a hand edit is no way around a limit.

## One commit for each edit, with the verb `sync` — 2026-11-16

**Decision (decided autonomously):** Each manual edit gets its own commit. The subject stays `waymark: sync "<title>"`. A line `Action: add|update|delete|move` in the commit body names the rules that applied.

**Alternatives considered:**
- The verb of the matching tool in the subject (rejected: `activity` and `history` would no longer show that a person made the edit by hand).
- One commit for all edits, as before (rejected: `undo` could only take back all edits together).

**Reason:** `activity`, `history`, and `undo` work per edit, and the subject still says that the edit came by hand.

## Only a rename with the same text is a move — 2026-11-16

**Decision (decided autonomously):** A deleted file and a new file with exactly the same text are one rename, published with the rules of `move`. Anything else is a delete and a new file.

**Alternatives considered:** A similarity measure like Git's 50 percent (rejected: harder to explain, and a wrong pair would rewrite links to the wrong document).

**Reason:** Obsidian keeps the text of a renamed file. Exact pairs are safe to rewrite links for.

## Short wikilinks are rewritten when a file name stops being unique — 2026-11-16

**Decision (product owner):** When a write gives a second document a file name that one document already has, every short wikilink that found the first document gets its full path. The rule is in section 8 of the standard. It applies to `add_document`, `move`, and `sync`.

**Alternatives considered:**
- Refuse a name that makes a short link ambiguous (rejected: then `readme` could exist in one folder only).
- Do nothing (rejected: the link silently finds no document, the backlink is lost, and `delete_document` no longer protects the first document).

**Reason:** A link keeps its meaning. Names like `readme` and `overview` will repeat across folders.

## The fixes go into the commit of the write — 2026-11-16

**Decision (decided autonomously):** For `add_document`, the short-link fixes are in the same commit as the new document. For `move`, they are in the link commit. For `sync`, they are in the commit of the edit (the link commit for a rename).

**Reason:** `undo` of the write takes the fixes back too. Without the new document, the short link finds the first document again.

## Documents with pending edits are not rewritten — 2026-11-16

**Decision (decided autonomously):** The short-link fixes skip documents with pending manual edits.

**Reason:** A write never carries an unpublished hand edit into its commit (see `manual-edits`). The skipped document is checked when it is published.

## Edits that need approval wait for `sync` — 2026-11-16

**Decision (decided autonomously):** Before an agent write, waymark publishes pending edits with the same rules. An edit whose matching tool needs approval is held back, with a notice to run `sync`.

**Alternatives considered:** Ask for approval inside the agent's write (rejected: the question would be about a change that the agent did not make).

**Reason:** Without this, an `update_document` that needs no approval would publish a new hand file although `add_document` needs approval.

## The standard is changed in place — 2026-11-16

**Decision (decided autonomously):** The short-link rule goes into `standard/waymark-1.0.md`, section 8, in place. The version stays 1.0.

**Reason:** The freeze of 1.0 at the first release (see `document-format/decisions.md`) was not done: the file has no "Versioning" section and no changelog, and later releases changed it in place. The rule is a duty of a producer and makes no valid document invalid. The product owner decides when to freeze and version the standard (see `todo.md`).

## The `sync` rule is in `AGENTS.md`, not in the server — 2026-11-16

**Decision (decided autonomously):** The rule "Do not call `sync` unless the person asks for it after a manual edit" goes into both `AGENTS.md` suggestions. The description of `sync` only says what it is for. The server instructions do not change.

**Alternatives considered:** The rule in the description of `sync` and in the server instructions, as discussed with the product owner (rejected for now: `mcp-server` and `session-start` say that descriptions and instructions carry no behavior advice, and a test guards the instructions. The product owner can reverse this with one line in each place).

**Reason:** "The server enforces, the team advises." With the rules of the write tools in `sync`, an agent that calls it gains nothing, so advice in the server is not needed for safety.

## In `sync`, only `tools_denied` counts as a switched-off tool — 2026-11-16

**Decision (decided autonomously):** The rules of `sync` check the matching write tool with the check of a read tool: only `tools_denied` makes it `tool-switched-off`. Read-only access and a pause are checked by `sync` itself before (it only pulls), and by the write tool that publishes edits first.

**Reason:** Found in review. With the check of a write tool, a `sync` preview with read-only access reported every edit as switched off, which is not true.

## The link fixes of a move belong to the commit before them — 2026-11-16

**Decision (decided autonomously):** When the `Move:` line of a link-fix commit names no known commit, `activity` joins it to the commit just before it, if that is a move or a `sync` commit of the same person.

**Alternatives considered:** A stable ID line on both commits (rejected for now: a change of the commit format that older entries do not have). A merge instead of a rebase in `sync` (rejected: it changes the history shape for every person).

**Reason:** Found in review. `sync` pulls with rebase after it commits, so the hash in `Move:` is often old. Without this, `undo` took back only the rename and left a broken link. The `move` tool had the same gap after a retried push.

