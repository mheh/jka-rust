# The comment-behavior ledger

Case law for the DEC-68 comment pass. Each ratified verdict from a review walk appends here as a numbered rule with its class, its verdict, one concrete example, and its origin. Every wave brief carries this ledger verbatim, workers report uncertain calls, the post-wave walk rules on them, and the next wave inherits the new rules. DEC-68 in `docs/decisions.md` is the canonical ruling and names this file its executable appendix. At campaign end the durable rules graduate into `docs/porting-rules.md` and this ledger freezes as the campaign record.

## CB-0 - the constitution (design walk, 2026-08-16)

Three classes govern every comment in scope:

1. **Delete residue**: prose that restates what the code shows, empty scaffold markers, derivation residue, porting-process commentary with no ongoing constraint.
2. **Keep and restyle** the load-bearing classes into STE house style: substantive port notes, behavioral documentation, divergence notes, any constraint the code cannot show. Keep every fact. One sentence per line, 150-column cap, unwrap column wrap, no em dashes, no semicolons in port prose, state the conclusion, not the derivation.
3. **Keep verbatim**, character for character: Raven's own comments and formatting, the QUAKED/spawnflag blocks, `Source:`/`Type definition source:` cites, layout-assert blocks, `//TODO: Port` markers with their `// Source:` lines. `SAFETY:` keeps the prefix and every invariant fact, and its prose may restyle.

On doubt about authorship, check the cited oracle lines. On remaining doubt, keep and restyle conservatively - never delete on doubt.

## CB-1 - PORT-COMPLETE headers are residue (pilot row 1, 2026-08-16)

The `// PORT-COMPLETE: <file>.c` file headers delete everywhere. Their one consumer, the idempotency grep in `.claude/workflows/port-jampgame.js:40-48`, is retired tooling - the jampgame port closed 2026-07-05 and the workflow never re-runs. Example: `g_mover.rs:1`, removed. Greppable pattern: `PORT-COMPLETE`.

## CB-2 - reworded Raven text restores character for character (pilot row 2, 2026-08-16)

Where a prior pass merged, paraphrased, or "corrected" a Raven comment, restore the oracle text exactly - typos, punctuation, and line splits included - and drop `(comment preserved)`-style meta-tags. Example: the `g_utils.rs` WTF block was three Raven lines (`g_utils.c:1068-1070`) merged with an em dash and a meta-tag, now restored, and `TAG_Add`'s gloss restored Raven's `alread` typo. Restoration is comment work, not backfill, because the text already exists at the Rust site.

Batch-3 addendum: casing and terminal punctuation are part of "the text". There is no separate transcription-convention layer, and file-internal uniformity never overrides oracle fidelity (the `g_icarus_set_type.rs` capitalization case).

## CB-3 - cite-shaped doc templates are port prose (pilot row 3, 2026-08-16)

Doc templates that cite the oracle without the literal protected `Source:` prefix are port-authored and restyle. The house separator is the colon: `` Raven `trap_X` (`g_syscalls.c:NNN`): `SYSCALL_NAME`. `` Uniformity across a template family is part of the value - restyle the template once and apply it identically. Example: the ~140 `trap.rs` wrapper docs.

## CB-4 - internal campaign tags are residue (pilot row 4, 2026-08-16)

`SEAM-*`, `STATE-*`, `round-N`, `Stage N`, shard labels, and their kin delete wherever they appear, and the design fact each tag was attached to stays. `DEC-nn` references are the one exception - they point into the live ledger and stay. Example: `world/game_context.rs` dropped `SEAM-Q12`/`STATE-D6`/`round-4` while keeping why fields are `pub` and why the `c_strcpy` write-back stays raw.

## CB-5 - missing Raven content is never backfilled (pilot row 5, 2026-08-16)

Raven comment content that was never transcribed into the Rust file (untranscribed header glosses, absent QUAKED blocks, dropped trailing clauses) stays absent in this pass. Workers flag it in their report, and the finds park in the plan doc as a candidate follow-up transcription wave with its own verification shape. Examples: the `ai_main_consts.rs` glosses, the 13 `g_mover.c` QUAKED blocks, `w_saber.rs:8506`.

## CB-6 - the FLAG: prefix is a durable convention (batch-1 walk, 2026-08-17)

`FLAG:` is a stable repo-wide marker (~557 sites across the crate), not campaign residue. The rule is the w_force pattern: keep the prefix and the restyled one-line fact, and drop the campaign-shorthand parenthetical. Treat the prefix like `SAFETY:`.

Batch-2 addendum: "one-line fact" means one fact, not one line. A second sentence at a `FLAG:` site goes on its own line, per the one-sentence-per-line law.

## CB-7 - wave-recipe tags are residue (batch-1 walk, 2026-08-17)

`recipe 2b`, `recipe 2c`, `trap 2b`, and bare `(2b)` delete uniformly, and the attached facts stay. Only the phrase `recipe rule 2b` is ledger-anchored (DEC-29). A site that leans on the prohibition itself cites `(DEC-29)` inline. A mixed cite keeps its real half: `(recipe §D12 / 2b)` becomes `(§D12)`.

Batch-2 addendum: a frozen campaign gets no carve-out. DEC-31 froze the work, not the naming, so `Stage N` labels from frozen campaigns delete the same way. A citation a worker cannot anchor in `docs/` is residue too, and an invented anchor is worse than none (the fabricated `§B4` case).

Batch-3 addendum: `(task #N)` tracker numbers are campaign shorthand and delete, facts stay. `DEC-nn` remains the only citation family that stays by identity, because the DEC ledger is the live in-repo decision store. Where a task resolved into a DEC, the DEC cite stands in its place if the site needs an anchor.

## CB-8 - DEC-anchored named invariants read as one family (batch-1 walk, 2026-08-17)

A named invariant with a ledger anchor, for example `SEAM-BG-REENTRY (DEC-28, sanctioned)`, is protected and uniform: every member site carries the same named tag in the same shape, so the family greps as one. A member hiding under anonymous prose restyles into the house shape.

Batch-3 addendum: this rule covers DEC-anchored named invariants only. Task-number and plain `FLAG:` families restyle under CB-6, never CB-8.

## CB-9 - deletion beats rewording when CB-0 rule 1 fails (batch-1 walk, 2026-08-17)

A restyle candidate that restates visible code or narrates completed history is residue even when true. Delete it, do not reword it.

Batch-3 addendum: residue must be a full restatement. A banner or marker carrying a clause absent from the adjacent literal is not residue for that clause (the `g_log.rs` "folded onto weapons" banners).

## CB-10 - the 150-column cap binds every line, Raven verbatim included (batch-1 walk, 2026-08-17)

An over-cap Raven line splits at a word boundary with zero word changes. Amends CB-2: "character for character" binds the text, not the line shape, so the CB-2 comparison joins split lines before it diffs against the oracle. A cap-split is not drift, and a reworded line still is.

## CB-11 - false port prose corrects, and every correction is declared (batch-3 walk, 2026-08-18)

A port-authored comment that states a verifiable falsehood about the current code gets corrected during restyle. A restyle must not launder a false claim into house style. Every such correction is named explicitly in the worker's report so the reviewer can verify it. Raven-authored text is exempt: it stays verbatim under CB-0 rule 3 regardless of truth. Origin case: the stale `.world: *mut GameWorld` module-doc claim, corrected in `g_navnew.rs` and carried forward in `g_spawn.rs` by two workers who both left the decision undeclared.
