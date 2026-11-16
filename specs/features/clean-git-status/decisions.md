# Decisions: Clean Git status

## Ignore `.DS_Store` in the `.gitignore` of the memory repository (2026-11-16)

**Decided:** `init` writes `.DS_Store` into `.gitignore`, and a repair adds it to existing memory repositories.

**Alternatives considered:**
- Tell each person to set a global Git ignore file (`core.excludesFile`). Rejected: the person without Git knowledge does not know this, and it must be done on every computer.
- Leave it, because waymark never commits the file. Rejected: Git and `sync` show the file every time, and the person does not know where it comes from.

**Reason:** One line in the repository fixes it for everyone who clones it.

## Write `app.json` in the bytes that Obsidian writes (2026-11-16)

**Decided:** The template has two spaces of indent and no line end after the last `}`, as Obsidian writes it.

**Alternatives considered:**
- Ignore `.obsidian/app.json` too. Rejected: the file carries the link settings of section 11 of the standard. Without it, Obsidian writes wikilinks with short paths.
- Keep the line end, and let the person commit the change of Obsidian. Rejected: the person sees a change that they did not make and does not know if it is safe to commit.

**Reason:** When the file already has the form of Obsidian, a save in Obsidian changes nothing.

## The `app.json` repair changes only the waymark file (2026-11-16)

**Decided:** The repair rewrites the file only when it holds exactly the two waymark settings and nothing else.

**Alternatives considered:** Reformat any `app.json`. Rejected: a file with more keys was saved by Obsidian already, so it has the form of Obsidian. And waymark must not touch the settings of the person.

**Reason:** The repair fixes only what an older waymark wrote.

## No repair of a file with uncommitted changes (2026-11-16)

**Decided:** A repair skips a file that has uncommitted changes, except an `app.json` that Obsidian already saved in the new form. This rule now also holds for the settings file.

**Alternatives considered:** Repair the working-tree text and commit it, as the repair of the settings file did. Rejected: the commit would carry edits of the person under a "repair" message.

**Reason:** waymark commits only what it wrote. The change for the settings file is small: the tools never change that file, so it has uncommitted changes only when a person edits it by hand.

## Separate repairs, one commit per file (2026-11-16)

**Decided:** Each repaired file gets its own commit, `waymark: repair "<path>"`. The settings file keeps `waymark: repair "settings"`.

**Alternatives considered:** One commit for all repairs. Rejected: `undo` of one repair would revert the others too.

**Reason:** Same rule as the repair of the settings file. Each repair can be undone alone.

## A new feature spec instead of a change in two specs (2026-11-16)

**Decided:** The change has its own spec, `clean-git-status`. The specs `create-memory-repository` and `memory-repository` get the new content and a link here.

**Reason:** The change touches two implemented specs. One spec keeps the reason, the tests, and the tasks together, and the two specs stay `implemented`.
