# Todo: Folders

## Bugs

None.

## Deferred

- Copy of a document or a folder (`show` and `add` do it for one document). Not a tool yet.
- Several writes in one commit. With the rule that a write never creates a broken link, two documents that link to each other cannot both be created in one go (see `write-documents`).
- Check that `lint`, `list`, `stats`, and `search` ignore a `.gitkeep` in a deep folder (a test in this spec).
