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

## CB-3 - cite-shaped doc templates are port prose (pilot row 3, 2026-08-16)

Doc templates that cite the oracle without the literal protected `Source:` prefix are port-authored and restyle. The house separator is the colon: `` Raven `trap_X` (`g_syscalls.c:NNN`): `SYSCALL_NAME`. `` Uniformity across a template family is part of the value - restyle the template once and apply it identically. Example: the ~140 `trap.rs` wrapper docs.

## CB-4 - internal campaign tags are residue (pilot row 4, 2026-08-16)

`SEAM-*`, `STATE-*`, `round-N`, `Stage N`, shard labels, and their kin delete wherever they appear, and the design fact each tag was attached to stays. `DEC-nn` references are the one exception - they point into the live ledger and stay. Example: `world/game_context.rs` dropped `SEAM-Q12`/`STATE-D6`/`round-4` while keeping why fields are `pub` and why the `c_strcpy` write-back stays raw.

## CB-5 - missing Raven content is never backfilled (pilot row 5, 2026-08-16)

Raven comment content that was never transcribed into the Rust file (untranscribed header glosses, absent QUAKED blocks, dropped trailing clauses) stays absent in this pass. Workers flag it in their report, and the finds park in the plan doc as a candidate follow-up transcription wave with its own verification shape. Examples: the `ai_main_consts.rs` glosses, the 13 `g_mover.c` QUAKED blocks, `w_saber.rs:8506`.
