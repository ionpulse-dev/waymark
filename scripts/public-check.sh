#!/usr/bin/env bash
# Scan tracked files for secrets and private details. Exit 1 on a hit.
set -uo pipefail
cd "$(dirname "$0")/.."
git rev-parse --git-dir >/dev/null 2>&1 || { echo "public-check: this is not a working Git repository (in a container: a worktree needs its Git folder mounted, see scripts/linux.sh)" >&2; exit 2; }
fail=0
scan() { # description, pattern
  hits=$(git grep --untracked -nIE "$2" -- . ':!scripts/public-check.sh' ':!docs/security-review.md' ':!tests' ':!src/lint/secrets.rs' ':!standard' 2>/dev/null | grep -v '/Users/jane/' || true)
  if [ -n "$hits" ]; then echo "== $1"; echo "$hits" | head -10; fail=1; fi
}
scan "private key" "BEGIN (RSA|OPENSSH|EC|DSA|PGP)? ?PRIVATE KEY"
scan "token" "(ghp_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}|sk-[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16}|xox[bap]-[0-9A-Za-z-]{10,})"
scan "home path" "/Users/[a-z]+/"
# A host name ends in .internal, .corp, or .local. The name before the dot keeps `~/.local/bin` out.
scan "internal host" "[A-Za-z0-9]\.(internal|corp|local)([:/]|$)"
[ -f LICENSE ] || { echo "== LICENSE is missing"; fail=1; }
[ $fail = 0 ] && echo "public-check: ok"
exit $fail
