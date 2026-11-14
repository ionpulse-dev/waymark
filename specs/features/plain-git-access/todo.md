# Todo: Plain-Git access

## Bugs

None.

## Deferred

- Copy the finished skill files into `plain-git/` when the task runs. Retire `waymark-trail` right after. Until then, both hold the same text and changes are made here (see `docs/decisions.md`).
- Decide whether to move the parts (`src/`) and `build.sh` of `waymark-trail`. Today only the finished files move.
- The commit prefix differs: the plain-Git rules write `kb: add "<title>"`, the tools write `waymark: add "<title>"`. `activity` and `undo` read only `waymark:` commits. Align the skill with the tool format, or teach `activity` both.
- The team skill blocks the example domains. The waymark check does not (see `login-and-identity`). Align them.
- Change the link to `waymark-trail` in the step "README" of the skills when that repository is retired.
- A tool or a check in `status` that tells a person when their `AGENTS.md` carries an older version of the rules.
- Test the plain-Git rules on a host other than GitHub and in the plain Claude chat.
- A no-network case for the plain-Git agent (a web agent without access to the remote).
- A rule file for server mode (out of scope for now).

## Open Questions

None.
