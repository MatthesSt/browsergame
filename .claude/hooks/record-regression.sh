#!/usr/bin/env bash
#
# Stop hook: nudge when game code changed but TESTING.md did not.
#
# TESTING.md is only worth having if it grows every time a bug is found, so this
# is the backstop for the convention in CLAUDE.md. It cannot tell a bug fix from a
# feature - nothing deterministic can - so it never blocks. It only points out the
# one situation where a missing regression entry is possible.
#
# Exits 0 and silent whenever there is nothing to say.

set -uo pipefail

root="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$root" 2>/dev/null || exit 0
git rev-parse --git-dir >/dev/null 2>&1 || exit 0

# Everything a player-visible bug can live in.
code_changed=$(git status --porcelain -- \
    index.html leaderboard-client.js server/src 2>/dev/null | head -c 2000)
[ -n "$code_changed" ] || exit 0

# Already recorded something - nothing to say.
tests_changed=$(git status --porcelain -- TESTING.md 2>/dev/null | head -c 200)
[ -n "$tests_changed" ] && exit 0

files=$(git status --porcelain -- index.html leaderboard-client.js server/src 2>/dev/null \
    | awk '{print $NF}' | paste -sd', ' - | head -c 300)

printf '%s' "$(cat <<JSON
{"systemMessage": "TESTING.md untouched while $files changed. If this turn fixed a bug, add a regression item to the matching section - symptom, root cause, and why a naive single check would miss it. Feature-only work can ignore this."}
JSON
)"
