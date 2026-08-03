#!/bin/bash
# SessionStart hook.
# If a parked handoff exists, inject its content and carry the absorb
# protocol. The hook only reads. The session deletes the handoff after it
# acts on the Resume here line.
set -u

cd "$(dirname "$0")/../.." || exit 0

f=".claude/HANDOFF.md"
[ -f "$f" ] || exit 0

ctx="Active handoff found at session start (${f}):

$(cat "$f")

Absorb protocol:
1. Recreate the handoff's Active tasks in the harness task list, matching the recorded statuses.
2. Greet with the Parked line and the Resume here line, then act on Resume here.
3. Delete ${f} with a plain rm. The file is untracked, so the delete brings the tree back to parity with the last commit, and no commit follows. If git tracks the file, commit the delete with the subject 'chore: handoff absorbed'."

jq -n --arg ctx "$ctx" '{hookSpecificOutput: {hookEventName: "SessionStart", additionalContext: $ctx}}'
