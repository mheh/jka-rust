#!/bin/sh
# SubagentStop hook: fire the audit decision rule the moment an agent returns.
# The context line tells the session to apply .claude/skills/audit without waiting for a user ask.
cat <<'EOF'
{"hookSpecificOutput":{"hookEventName":"SubagentStop","additionalContext":"An agent returned. Apply the project audit skill (.claude/skills/audit/SKILL.md): broad scope (workspace-wide conversion, multi-crate feature, ~100+ sites, DEC-executing campaign) - write the dated record under docs/audits/ now, without asking. Smaller but useful - ask the user one question. Routine - nothing."}}
EOF
