---
name: lane-review
description: The return ceremony for a completed code lane. Use on every agent completion before any merge - diff the work against the packet's surface contract and commit bundle, cross-check the finished file, then act on the packet's disposition or stop and flag the deviations.
---

# lane-review

The agent's report is a claim. The diff is the evidence. Lane-review closes every packet lane: a clean pass proceeds on the packet's disposition with no further ruling from the user, and any deviation stops the lane before a single merge. The judgment never delegates. The session that holds the arc's rulings is the one thing that catches a change that is legal inside the packet's letter and wrong against its intent. The conformance clerk below reads evidence and quotes it, and the session rules.

## Procedure

1. Read `step-NNN/finished.md`. It must carry the assumptions and choices keyed to commits, the deviations or the word "none", the commit list with gate results, and the open gaps. A missing or thin finished file is itself a deviation.
2. Pick the review mode. **Full-read** where the leg is parity-bearing or architecturally novel - an ABI seam, a wire format, a new subsystem, or the first leg of an arc: the session reads the whole diff itself, and no clerk spawns. **Hybrid** for a routine mechanical leg: spawn the conformance clerk below, on opus.
3. Gather the evidence. Full-read: the actual git diff of the lane branch, whole. Hybrid: the clerk's report, whose quotes are the evidence floor beside the finished file. Never rule from the lane's own report alone. Any doubt in the finished file or the clerk's report pulls the full diff into the session, which supersedes the hybrid for that lane.
4. Check the diff against the surface contract. Any surface the contract does not list is a deviation: a `pub` item, a signature, a `#[repr]` layout, a trap or dispatcher arm, a cvar, a `FrameEvent` variant, an engine hook, or a dependency.
5. Check the commits against the bundle. Splits and reorders are legal. A widened commit, an unplanned commit, or a bundled commit is a deviation. Check each message: a heading subject, an STE body, no trailer.
6. Cross-check the finished file against the diff. Behavior in the diff that the finished file does not mention is a deviation.
7. On a clean pass, act on the packet's disposition. A merge happens locally with `--no-gpg-sign`. This repo holds pushes, so the lane never pushes and never opens a pull request. Report the landed commits in chat, and add the step-folder link to the GitHub issue.
8. On deviations, stop. No merge. Present each deviation to the user plainly: what the packet said, what the diff shows. The ruling lands as an Amendment or as a rework step, and the lane reopens from there.

## The conformance clerk

The clerk is a conformance clerk and never a judge. It reads the whole diff - every file, every hunk, never a sample - and its output is quotes, inventories, and flags. It renders no opinion on whether the code is right. Before the spawn, the session names the hunks the clerk must quote back regardless of findings: the ABI seams, the surfaces, and the spots where the leg's intent lives, chosen from the arc's own knowledge. The named hunks are a reporting obligation and never a reading scope.

The brief below is fixed text. Paste it whole with the five values filled, and do not rewrite it per lane, so no session drifts the clerk's duties.

---

You are the conformance clerk of a lane-review. You read evidence and quote it. You never judge whether code is right, you never recommend, and you never approve. The words "looks good" must not appear in your report.

The packet: `<packet path>`, including its Amendments.
The finished file: `<finished.md path>`.
The repo and diff range: `<repo path>`, `git diff <base>..<branch>`.
The named hunks: `<the files or symbols whose hunks you quote verbatim regardless of findings>`.

Read the packet whole, then walk the ENTIRE diff hunk by hunk - every file, never a sample. Then report exactly eight things:

1. Letter violations. Every hunk that creates or changes a surface the contract does not list - a `pub` item, a signature, a `#[repr]` layout, a trap or dispatcher arm, a cvar, a `FrameEvent` variant, an engine hook, a dependency, or a file outside the write scopes. Quote each hunk verbatim.
2. The named hunks. Quote each one verbatim, even where you found nothing wrong with it.
3. Ledger mismatches. List every behavior visible in the diff that the finished file does not mention, with the hunk quoted. A confessed choice is not a mismatch.
4. The inventories. The files changed against the write scopes, the commits against the bundle - splits and reorders are legal - and each commit message against the rules: a heading subject, an STE body, no trailer of any kind.
5. Repo mechanics on added lines. Flag each with the line quoted: a `use` declaration inside a function body, a `todo!()` or other placeholder that lacks both the `//TODO: Port <subject>` marker and its `// Source:` cite, a newly ported item with no oracle `Source:` cite, a new extern forward-declaration block, and a `format!` call that builds a wire string.
6. House-style violations on added lines. Read `~/.claude/skills/house-style/SKILL.md` and `~/.claude/skills/asd-ste100/SKILL.md` by path, then flag each violation with the line quoted: an em dash, a semicolon in prose, pet vocabulary, banned voice, a comment that narrates mechanics, a doc comment off the content rules.
7. The gate claims, re-run. Take every gate the finished file claims and run it yourself, then report the real output. A "goldens byte-identical" claim is never trusted, always re-run.
8. The unverified list. Everything you could not check mechanically, named plainly as unverified and never assumed fine.

---
