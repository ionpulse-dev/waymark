# Decisions: Join a knowledge base

> **Note 2026-11-11:** "Install Git for the person" and "Two steps to join" are superseded by "One MCP entry" below. "Shared and personal settings are split" stays valid.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Shared and personal settings are split — 2026-11-06

**Decision:** The shared file holds everything the same for all. Personal values go to `~/.config/waymark/config.toml`, keyed by UUID.

**Reason:** People who join never type shared values again, and each person controls their own levels.

## Two steps to join — 2026-11-06

**Decision:** Target: under 5 minutes. Download and open the app, answer defaults. For terminals: `install.sh`, then `waymark setup <url>`.

**Reason:** Least friction for non-developers.

## Install Git for the person — 2026-11-06

**Decision:** If `git` is missing, setup starts Apple's installer (one dialog, a download of a few hundred MB, once).

**Reason:** People need no Git knowledge, and developers already have it.

## One MCP entry — 2026-11-11

**Decision:** Joining is: install the binary and add one MCP entry. The server clones on first start. The personal config is optional.

**Reason:** no app and no setup command. IT handles the Git installation and login, which was the assumption of the design. Alternative considered: a setup tool in the server. It would clone without a person's entry.
