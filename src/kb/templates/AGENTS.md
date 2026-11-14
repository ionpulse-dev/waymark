# Rules for agents

This repository is a knowledge base: Markdown documents for people and AI agents. It follows the MKBase Document Standard. The documents are in the folder `knowledge/`.

## With the waymark MCP server

Use the tools of the `waymark` server to search, read, and write documents. The server checks the format, the size, the links, and secrets, and it makes the Git commits for you.

## Without the waymark MCP server

Do not edit documents by hand unless the person asks for it. If you must, follow the format in the standard and these rules:

- One topic per document. A document has frontmatter with `title`, `description`, `status`, `created`, `updated`, `created_by`, and `updated_by`.
- File names use lowercase letters, digits, and `-` only.
- Never write secrets, passwords, or tokens into a document.
- Commit only the files you changed. Never force-push. Never rewrite history.

Add your own rules for your team below this line.
