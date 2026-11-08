---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - agent-tool-setup
---

# Maintenance

## User Story

> As a person, I want to check, update, and remove waymark easily, so that I always know it works and I can leave without leftovers.

## Personas

- **Every person** — checks health, updates, uninstalls.
- **IT** — runs the same commands in scripts.

## Behaviour

| Command | Task |
|---|---|
| `waymark doctor` | Checks the setup and prints a fix for each problem: repository present and clean, remote reachable, login valid, background sync on and the time of the last good sync, instruction blocks and skills present for each tool, binary up to date, `git` works. |
| `waymark status` | Prints running or paused, last good sync, last error, and login state. |
| `waymark update` | Updates the binary to the latest release, writes the embedded skill again into each tool, and pulls the content repository. |
| `waymark uninstall` | Removes everything that setup and the installer put on the machine. |

### Uninstall

`uninstall` reads the manifest first, then removes exactly its items, in this order:

1. The app: quit it, remove its login item, and move `Waymark.app` to the Trash.
2. The instruction block in each agent tool, only the part between the waymark markers.
3. The skill files in each agent tool.
4. The token in the macOS Keychain.
5. The status file and logs.
6. The content repository clone. waymark asks first. Before it deletes the clone, it checks for changes that are not pushed and shows them. `--keep-repo` keeps it.
7. The `PATH` line in the shell profile, if the installer added one.
8. The personal config (`~/.config/waymark/`, including the manifest), then the `waymark` binary itself, last.

The `.waymark/` folders and skill files that `link` wrote into project folders belong to the projects and stay.

It prints what it removed. Running it twice does no harm.

## Interfaces & Contracts

`doctor --json` returns a list of checks, each with `name`, `ok`, and `fix`.

## Acceptance Criteria

- **Given** a broken login, **When** `doctor` runs, **Then** it reports the problem and the fix `waymark login`.
- **Given** a healthy setup, **When** `doctor` runs, **Then** all checks are ok and the exit code is 0.
- **Given** unpushed changes in the clone, **When** `uninstall` runs without `--keep-repo`, **Then** it shows them and asks before deleting.
- **Given** `uninstall` ran, **When** the machine is inspected, **Then** no waymark file or Keychain item remains, except the kept repository.
- **Given** `uninstall` ran twice, **When** the second run ends, **Then** it reports nothing to remove.
- **Given** a new release, **When** `update` runs, **Then** binary and skill have the same version.

## Constraints & Edge Cases

- `/usr/bin/git` can be a stub. `doctor` checks that `git --version` really works.

## Open Questions

- How does `update` replace the app while it runs?
