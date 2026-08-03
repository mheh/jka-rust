---
name: packet
description: Design and audit the delivery packet for a code-writing agent. Use before ANY code-writing agent spawn - every code spawn gets a packet, with no floor for small work. The packet binds the scope, the surface contract, and the commit bundle.
---

# packet

An agent without a packet widens scope, assumes behavior, and bundles commits. The packet binds the work before the spawn: what the agent may write, which surfaces may exist, and which commits carry the work. The user audits the packet, and the approved packet is the explicit go for the lane.

## Where packets live

Packets live at `.claude/packets/<gh-issue>/step-NNN/packet.md` and commit with the repo. `<gh-issue>` is the issue number on the GitHub tracker `mheh/jka-rust` (DEC-52). Step numbers are zero-padded to three digits and only climb. A new step means new work: the next commit bundle, a follow-up lane, or a rework pass. The agent's reply lands beside the packet as `finished.md`, or `finished-<role>.md` when several agents share a step. The issue carries one link line to the step folder, never a pasted copy.

## Procedure

1. Find the issue number and the next step number. Read the issue with `gh issue view <n>`, list `.claude/packets/<n>/`, and add one to the highest step.
2. Spawn a read-only explorer on sonnet, or on haiku for a small sweep. Never the session model. It surveys the exact files the work touches and grounds every signature in those specific neighbors, never in the house-wide dominant idiom. The wrong neighborhood is how presumptive shapes get in. Its prompt states the repo idiom: `cargo check` is the ground truth because rust-analyzer is stale here, `oracle/` is read-only and gets cited by `file:line`, and imports are canonical short names at the file top.
3. Draft `step-NNN/packet.md` from the template below and commit the draft.
4. Post the audit in chat: the file path, the surface contract inline with its signatures, one line per planned commit, and the disposition.
5. Wait for explicit approval. Fold chat rulings into the file and commit. An edit to the file by the user is also a ruling. Vague assent is not approval.
6. Spawn the agent. The brief carries the packet path, the write scopes, the pause rules, and the standing constraints: `oracle/` is read-only, source files change through the Edit tool only, never touch `~/Developer/jka/`, verify with `cargo build` or `cargo check`, do not push, and a worktree builder runs `git merge master --no-gpg-sign` as its first act. The brief also tells the agent to read `~/.claude/skills/asd-ste100/SKILL.md` and `~/.claude/skills/house-style/SKILL.md` by path, and to pass both to its sub-agents.

## The packet template

- **Scope** - what this step delivers, and what it does not.
- **Surface contract** - the surface the work may create or change: `pub` items with real Rust signatures, `#[repr]` layouts with their `size_of` and `offset_of!` asserts, trap and dispatcher arms, cvars, `FrameEvent` variants, and engine hooks. Close the world with this sentence: "Anything not on this list is out of scope, and the agent must not add it." A new third-party crate is a dependency ruling of the DEC-49 kind, so it comes from the user only. The packet may never grant one.
- **Commit bundle** - one entry per planned commit: the intent, the files, the surface items it may create, and the gate battery it must pass. Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind: no `Co-Authored-By`, no generated-with footer. Name the gates per commit. `cargo build --workspace` and `cargo test --workspace` gate every commit. A commit that touches the renderer also needs the world goldens byte-identical. A commit that touches `mp_game` or the server also needs the lockstep referee.
- **Write scopes** - the branch, the crates and paths the agent may edit, plus this step folder for the finished file.
- **Disposition** - what happens after a clean lane-review: merge to master, hold on the branch, or leave for the next step. This repo merges locally and holds pushes, so no disposition pushes and no disposition opens a pull request.
- **Amendments** - dated mid-lane rulings, appended in place.

## Rules

- The scope binds, the steps do not. The agent may split a planned commit into smaller ones or reorder neighbors. It must never widen scope: no new functionality, no assumed behavior, no commit the bundle does not cover.
- Pause triggers: an unlisted surface the work seems to need, any change to a contract signature, a new dependency, or reality that contradicts the plan. The agent stops without writing it and asks. The ruling lands as an Amendment in the same packet, then the lane resumes.
- The finished file is the last act before the agent returns: assumptions and choices keyed to their commits, deviations or the word "none", the commit list with the gate results, and open gaps. A lane without a finished file fails lane-review.
