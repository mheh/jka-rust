---
name: park
description: Park the session - write the live tasks and a handoff file so a fresh session resumes without the full context. Use when the user wants to stop, park, or hand off to a future session.
---

# park

Capture the live session state into `.claude/HANDOFF.md` at the repo root. A fresh session then resumes from the file instead of from a compacted context. The SessionStart hook finds the file by path, so nothing else needs a pointer.

The auto-memory ledger is the durable store, and the DEC ledger plus the GitHub tracker hold the settled work. Park duplicates none of them. The handoff carries only what those homes do not: the live task list, the threads in flight, the questions waiting on the user, and the first action to take next.

## Steps

1. Gather the state. Pull the harness task list and keep the `in_progress` and `pending` entries. From the conversation, collect: running background agents and the instruction each one holds, open branches and who acts next, questions waiting on the user, and rulings from this session that no durable home holds yet.

2. Write `.claude/HANDOFF.md` in STE (read `~/.claude/skills/asd-ste100/SKILL.md` first), with these sections:
   - **Parked** - the date, and one line on what this session was doing.
   - **Active tasks** - the live task list, one line each, worded well enough that a fresh session can recreate the list.
   - **In flight** - each running thread: what runs, where it stopped, and the exact next action. Name issues and branches with their numbers and links. Background agents die with the session, so record what a fresh agent would need, not the old agent id.
   - **Waiting on the user** - each open question or merge, one line each.
   - **Resume here** - the first concrete action the next session takes.
   - **References** - the durable homes: issue links, DEC numbers, packet step folders, repo paths. Never link a path that dies with the session. Move anything that lives only in scratch into a durable home first, then link that.

3. Leave the handoff uncommitted. The file stays untracked, so the resume delete brings the tree back to parity with the last commit, and the park leaves no marks in the git history. Do not commit for the park itself. If the session made other repo changes that deserve a commit, commit those separately before you write the handoff.

## On resume

The SessionStart hook `.claude/hooks/handoff-absorb.sh` injects the parked handoff into the new session with the absorb protocol. The session recreates the Active tasks, acts on Resume here, then deletes the file with a plain `rm`. The file is untracked, so the delete needs no commit. A handoff lives only between two sessions, and git never records it, so a stale handoff must never outlive its resume.

## Notes

- Park does not reconcile the tracker. If an issue is behind, say so in the handoff instead of rewriting the issue at park time. Parking must stay cheap.
- Anything a fresh session cannot reach is lost. This repo holds pushes deliberately, so unpushed commits get a warning line under **Resume here** with the branch and the commit count. Running background agents get their instructions restated.
