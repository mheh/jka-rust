---
name: lane-review
description: The return ceremony for a completed code lane. Use on every agent completion before any merge - a vet agent walks the commits against the packet one commit at a time, the session rules on the findings, then acts on the packet's disposition or stops and flags the deviations.
---

# lane-review

The lane's report is a claim. The commits are the evidence. Lane-review closes every packet lane: a clean pass proceeds on the packet's disposition with no further ruling from the user, and any finding stops the lane before a single merge. The vet below reads the evidence and hunts, and the session rules. The session never reads the diff. It rules from three artifacts: the finished file, the vet report, and the packet clauses the vet quotes.

## Procedure

1. Read `step-NNN/finished.md`. It must carry the assumptions and choices keyed to commits, the deviations or the word "none", the commit list with gate results, and the open gaps. A missing or thin finished file is itself a deviation.
2. Spawn the vet on fable with the fixed brief below. The vet is blind to the finished file: the brief never names it, and the vet must not open it. Blindness makes the vet's findings independent evidence beside the lane's confession. Before the spawn, name the hunks the vet must quote back regardless of findings: the ABI seams, the surfaces, and the spots where the step's intent lives, chosen from the synopsis and the arc's rulings.
3. Read `vet.md` and compare it against the finished file. A finding the lane confessed is a confessed deviation. A finding the lane did not confess is an unconfessed deviation. A confession with no matching finding stands as confessed.
4. On zero findings with the full gate battery green, act on the packet's disposition. A merge happens locally with `--no-gpg-sign`. This repo holds pushes, so the lane never pushes and never opens a pull request. Report the landed commits in chat, and add the step-folder link to the GitHub issue.
5. On findings, stop. No merge. Present each finding to the user plainly: the packet clause the vet quoted, and the commit evidence the vet quoted. The ruling lands as a packet Amendment or as a rework step. A fix round continues the same lane agent over SendMessage, and the same vet then walks the new commits over SendMessage. The agents keep their context, and the session keeps only the verdicts.

## The vet

The vet hunts defects. It walks the packet's oracle cites in the source before it reads a single commit, so it judges the work against the world and not against the work's own frame. Then it walks the commit sequence one commit at a time, in order, every hunk of every commit, never a sample. It quotes evidence for every finding, and it lists what it could not check. It runs on fable and on no other model.

The brief below is fixed text. Paste it whole with the five values filled, and do not rewrite it per lane, so no session drifts the vet's duties.

---

You are the vet of a lane-review. You hunt defects and you quote evidence. You never approve. The words "looks good" must not appear in your report. You do not open `finished.md` or any finished file in the step folder.

The packet: `<packet path>`, including its Amendments.
The repo and diff range: `<repo path>`, `<base>..<branch>`.
The step folder for your report: `<step folder path>`, write `vet.md` there.
The named hunks: `<the files or symbols whose hunks you quote verbatim regardless of findings>`.
The oracle ground: read every `oracle/` cite the packet names, at the cited lines, before you read any commit.

Read the packet whole. Walk the oracle cites. Then list the commits with `git log --oneline <base>..<branch>` and walk them one commit at a time with `git show`, in order, every hunk, never a sample. Report exactly eight things, one section per commit where a thing is per-commit:

1. Letter violations. Every hunk that creates or changes a surface the contract does not list - a `pub` item, a signature, a `#[repr]` layout, a trap or dispatcher arm, a cvar, a `FrameEvent` variant, an engine hook, a dependency, or a file outside the write scopes. Quote each hunk verbatim beside the packet clause it violates.
2. Oracle divergences. Every hunk whose behavior differs from the cited oracle lines: a float width, an operator or evaluation order, a hoisted macro argument, a changed constant, a dropped side effect. Quote the hunk and the oracle lines together.
3. The named hunks. Quote each one verbatim, even where you found nothing wrong with it.
4. The inventories. The files changed against the write scopes, and the commits against the bundle - splits and reorders are legal, a widened commit, an unplanned commit, or a bundled commit is a finding. Check each commit message: a heading subject, an STE body, no trailer of any kind.
5. Repo mechanics on added lines. Flag each with the line quoted: a `use` declaration inside a function body, a `todo!()` or other placeholder that lacks both the `//TODO: Port <subject>` marker and its `// Source:` cite, a newly ported item with no oracle `Source:` cite, a new extern forward-declaration block, and a `format!` call that builds a wire string.
6. House-style violations on added lines. Read `~/.claude/skills/house-style/SKILL.md` and `~/.claude/skills/asd-ste100/SKILL.md` by path, then flag each violation with the line quoted: an em dash, a semicolon in prose, pet vocabulary, banned voice, a comment that narrates mechanics, a doc comment off the content rules.
7. The gate battery, re-run. Take every gate the packet's commit bundle names, run each with the exact invocation the packet gives, and report the real output. A "goldens byte-identical" claim is never trusted, always re-run.
8. The unverified list. Everything you could not check mechanically, named plainly as unverified and never assumed fine.

Write the report to `vet.md` in the step folder. Reply with the `vet.md` path, the finding count, and one line per finding.

---
