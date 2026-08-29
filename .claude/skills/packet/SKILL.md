---
name: packet
description: Design and audit the delivery packet for a code-writing agent. Use before ANY code-writing agent spawn - every code spawn gets a packet, with no floor for small work. A drafter agent writes the packet, the session reads only the synopsis, and the user rules on the open rows.
---

# packet

An agent without a packet widens scope, assumes behavior, and bundles commits. The packet binds the work before the spawn: what the agent may write, which surfaces may exist, and which commits carry the work. A drafter agent writes the packet and returns a short synopsis. The session works from the synopsis, walks the open rows with the user, and spawns the lane. The approved packet is the explicit go for the lane.

## Where packets live

Packets live at `.claude/packets/<gh-issue>/step-NNN/packet.md` and commit with the repo. `<gh-issue>` is the issue number on the GitHub tracker `mheh/jka-rust` (DEC-52). Step numbers are zero-padded to three digits and only climb. A new step means new work: the next commit bundle, a follow-up lane, or a rework pass. The agent's reply lands beside the packet as `finished.md`, or `finished-<role>.md` when several agents share a step. The issue carries one link line to the step folder, never a pasted copy.

## The context rule

The session never opens `packet.md`, the surveyed source files, or the oracle. The session reads three artifacts only: `synopsis.md`, `audit.md`, and the rulings it collects in the walk. A zoom during a ratification walk is the one exception, and it stays a targeted read of the cited lines, never a full file.

## Procedure

1. Find the issue number and the next step number. Read the issue with `gh issue view <n>`, list `.claude/packets/<n>/`, and add one to the highest step.
2. Spawn the drafter on opus. The drafter surveys the exact files the work touches, grounds every signature in those specific neighbors, and cites the oracle by `file:line`. The wrong neighborhood is how presumptive shapes get in. The drafter may spawn read-only scouts on sonnet or haiku. It writes `packet.md` from the template below, writes `synopsis.md` to the contract below, commits both, and returns the synopsis text as its reply. Its brief states the repo idiom: `cargo check` is the ground truth because rust-analyzer is stale here, `oracle/` is read-only and gets cited by `file:line`, and imports are canonical short names at the file top. The brief also tells the drafter to read `~/.claude/skills/asd-ste100/SKILL.md` and `~/.claude/skills/house-style/SKILL.md` by path, and to pass both to its scouts.
3. Read the synopsis. When any dispatch flag is true, spawn the auditor on opus. Fable needs the user's explicit approval, asked per spawn. The auditor walks the packet's oracle cites in the source first, then reads the draft against them, so it judges the draft against the world and not against the draft's own frame. It writes `audit.md`: each mechanical row cleared or challenged, each claim confirmed or disputed, with the evidence quoted. When every flag is false, skip the auditor.
4. Walk the open rows with the user through the ratification-walk skill: the user-ruling rows plus every row the auditor challenged. Post the synopsis in chat first, then walk one row per message.
5. Send the rulings to the drafter over SendMessage. The drafter folds them into `packet.md`, commits, and confirms each fold in its reply. When the drafter is gone, the session appends the ruling text to the packet's Amendments section with a tail read only, and reads nothing above it.
6. Spawn the lane agent. The brief carries the packet path, the write scopes, the pause rules, and the standing constraints: `oracle/` is read-only, source files change through the Edit tool only, never touch `~/Developer/jka/`, verify with `cargo build` or `cargo check`, do not push, and a worktree builder runs `git merge master --no-gpg-sign` as its first act. The brief also tells the agent to read `~/.claude/skills/asd-ste100/SKILL.md` and `~/.claude/skills/house-style/SKILL.md` by path, and to pass both to its sub-agents.

## The synopsis contract

`synopsis.md` is the one packet artifact the session reads, so it is complete on its own and short. Hard cap: 60 lines.

- The step's intent, two sentences.
- The surface contract as a bare list of item names, no signatures.
- One line per planned commit.
- The open rows, each tagged `mechanical` or `user ruling`, with the proposed default in one line each.
- The dispatch flags, each marked true or false: oracle ambiguity, a new state home, ABI or parity-gate surface, a divergence proposal.

## The packet template

- **Scope** - what this step delivers, and what it does not.
- **Surface contract** - the surface the work may create or change: `pub` items with real Rust signatures, `#[repr]` layouts with their `size_of` and `offset_of!` asserts, trap and dispatcher arms, cvars, `FrameEvent` variants, and engine hooks. Close the world with this sentence: "Anything not on this list is out of scope, and the agent must not add it." A new third-party crate is a dependency ruling of the DEC-49 kind, so it comes from the user only. The packet may never grant one.
- **Commit bundle** - one entry per planned commit: the intent, the files, the surface items it may create, and the gate battery it must pass. Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind: no `Co-Authored-By`, no generated-with footer. Name the gates per commit with their exact invocations. `cargo build --workspace` and `cargo test --workspace` gate every commit. A commit that touches the renderer also needs the world goldens byte-identical. A commit that touches `mp_game` or the server also needs the lockstep referee.
- **Write scopes** - the branch, the crates and paths the agent may edit, plus this step folder for the finished file.
- **Disposition** - what happens after a clean lane-review: open the pull request to master and merge it on GitHub with a merge commit (DEC-67), hold on the branch, or leave for the next step. Never squash, and never commit directly on master.
- **Amendments** - dated mid-lane rulings, appended in place.

## Rules

- The scope binds, the steps do not. The agent may split a planned commit into smaller ones or reorder neighbors. It must never widen scope: no new functionality, no assumed behavior, no commit the bundle does not cover.
- Pause triggers: an unlisted surface the work seems to need, any change to a contract signature, a new dependency, or reality that contradicts the plan. The agent stops without writing it and asks. The ruling lands as an Amendment in the same packet, then the lane resumes.
- The finished file is the last act before the agent returns: assumptions and choices keyed to their commits, deviations or the word "none", the commit list with the gate results, and open gaps. A lane without a finished file fails lane-review.
