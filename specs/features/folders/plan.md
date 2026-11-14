# Plan: Folders

## Technical Approach

1. **Folder model** (`src/memrepo/folders.rs`). `MemRepo::folders()` walks the root and returns every folder with its documents, other files, and whether it has a `.gitkeep`. A folder exists when it holds a document, a file, a subfolder, or a `.gitkeep`. Hidden names, excluded paths (`.waymark/exclude`), and symbolic links are skipped, like in `documents()`.
2. **`folders`** (`src/tools/folders.rs`). A read tool. It filters the model by the start folder and the depth, counts the documents that the `folders` limit allows, and caps the list at 200 entries.
3. **Write flow.** `FileChange` gets two variants, `WriteFile` and `RemoveFile`, for files that are not documents (`.gitkeep`). `Tx::remove_file` also removes the empty parent folders on disk, because Git keeps none.
4. **`add_empty_folder`** (`src/tools/add_empty_folder.rs`). Checks the path (`writes::check_folder_path`), the limits, and that no document or folder has the path. Writes `<folder>/.gitkeep`. The commit verb is `add-folder`.
5. **`remove_folder`** (`src/tools/remove_folder.rs`). Collects the folders below the path. Checks other files, the limits, the count, and the links from outside (`tools::delete::linked_from`). Removes every document and every `.gitkeep`. The commit verb is `remove-folder`. One commit.
6. **`move`** already moves every file of a folder, so a `.gitkeep` goes along. It accepts a folder that holds no document.
7. **Errors with data.** `Error` has an optional `data` field. `error_json` puts it in the result as `data`. `delete` and `remove_folder` use it for `linked_from`.
8. **Names.** The parameter is `folder`, because no tool may take a parameter named `path` (see `project-binding`).

## Risks

- A bulk delete by one call. The guards are the count, the links rule, the approval question, `undo`, and `tools_denied`.
- A memory repository with very many folders. The cap of 200 and `truncated` keep the result small.
