# The jampgame comment pass

User ask 2026-08-16: apply the `asd-ste100` and `house-style` skills to all jampgame code comments, so they keep their information but lose the odd formatting (column word wrap) and the confusing text.

## Relationship to DEC-39

DEC-39 (2026-07-26) ratified a post-parity two-pass sweep: pass 1 strips port-added prose down to Raven-verbatim comments plus cites, and pass 2 is an optional organic rewrite with its design deferred. This pass differs on one axis: it retains the information instead of deleting it. It is an information-preserving restyle, not a strip. The open rows below settle how the two rulings compose. DEC-39's protected classes carry over unchanged either way.

## Scope

Default: the `mp_game` crate only (`crates/mp/game`, 171 files, 166,805 lines, 25,314 comment lines). The sibling crates (`bg` 5,715 comment lines, `qshared` 2,833, `abi` 10,388) are shared with other modules and wait for their own passes.

## What changes

Only comment text. For every port-authored comment in scope:

- Column word wrap unwraps. One sentence per line, 150-column cap, and never a wrap width inferred from the surrounding file.
- Confusing or derivation-heavy prose rewrites into STE house style: state the conclusion, keep every fact, cut re-derivations of C mechanics the cited oracle lines already show.
- The house comment shape stays: doc-comment plus `Source:` cite on every item, in the format `docs/porting-rules.md` fixes.

## What never changes

- Code. Not one token. The gate below enforces this mechanically.
- Raven-verbatim comment text, including the `QUAKED`/spawnflag blocks, character for character (DEC-39 rule 1).
- `Source:` cites (badge and assert tooling greps them).
- The layout-assert blocks.
- `SAFETY:` comments keep their prefix and their invariant content. Their prose may restyle.

## The gate: `tools/comment-gate`

A small Rust binary, built first, before any worker runs. For each changed file it lexes the git-HEAD version and the worktree version with `proc-macro2` (comments and whitespace drop out of the token stream by construction) and requires the two streams identical. One command over the whole diff, exit nonzero on any drift. A worker whose diff fails the gate is rejected without review. This is the DEC-39 rule 3 gate, built now because this pass needs it first.

## Execution shape

The established fleet pattern: packets, blind workers, mechanical referee.

1. **Anchor.** A plain GitHub issue holds the campaign, and its packets live at `.claude/packets/<issue>/`.
2. **Waves.** The 171 files shard into waves of roughly 10-15 files per worker lane, grouped by subsystem prefix (`g_*`, `ai_*`, `bg_*`, `NPC_*`, ...). About 12-15 lanes total.
3. **Workers.** Fleet tier per the model policy. Every brief carries the two skill-invoke lines, the What-changes and What-never-changes lists above verbatim, and the standing constraints: Edit tool only, `oracle/` read-only, no pushes.
4. **Per-lane gates.** `tools/comment-gate` over the lane's diff, then `cargo build --workspace`. Tests and the referee add nothing per lane, because an identical token stream compiles to an identical binary.
5. **Belt-and-suspenders.** One lockstep-referee run and one `cargo test --workspace -- --test-threads=1` at campaign end (DEC-39 rule 3 wording).
6. **Review.** Lane-review per returning worker: the vet samples each lane's diff for information loss, Raven-verbatim drift, and format violations.

## Landing

Everything lands on this branch (`chore/dec-67-gate-mechanics`), so the PR becomes the whole docs pass: the DEC-67 amendment, this plan, the gate tool, and the comment waves. The deployment gate re-arms on every push, so the one approval and the one CI run come at the end, when the branch is complete.

## Rulings (walked and closed 2026-08-16)

1. **Scope** - ratified: the `mp_game` crate only.
2. **DEC-39 composition** - ratified as amended: DEC-68, one combined triage-and-restyle pass (delete residue, restyle load-bearing notes into STE, Raven-documented blocks verbatim). DEC-39 stands for the other modules.
3. **Worker tier** - ratified: sonnet medium for all lanes.
4. **Campaign anchor** - ratified: a new plain GitHub issue, packets under it.
5. **Landing** - dictated: all on the one docs-pass PR branch.

The behavior rules live in `docs/comment-behavior.md` (CB-0 through CB-5 seeded from the design walk and the pilot review). Every wave brief carries that ledger verbatim, and post-wave walks append to it.

## The pilot (10 files, reviewed 2026-08-16)

Ten sonnet workers ran one file each: `g_misc`, `g_mover`, `g_utils`, `trap`, `game_globals`, `npc_c`, `w_saber`, `entity/gentity`, `world/game_context`, `ai_main_consts`. Net -292 lines, workspace build green, and the code-content heuristic matched exactly on every changed line. The review walk produced CB-1 through CB-5.

Lessons for the waves:

- `replace_all` on a comment substring corrupted code whitespace five times in `w_saber` (deeper-indented copies). The worker caught it in its own diff audit. Every wave brief mandates the diff self-audit, and the `tools/comment-gate` build is a precondition for wave 1.
- Prior passes silently reworded Raven comments in at least three files. CB-2 makes restoration part of every lane, so briefs tell workers to compare suspect Raven text against the oracle, not judge by tone.
- Three workers ignored the no-cargo instruction without harm. The waves keep the instruction and add why: parallel cargo runs contend on the target lock.
- `w_saber` cost 4-5x a normal file. Files over ~5,000 lines get a lane of their own, and the wave sharding balances by comment-line count, not file count.

## Parked follow-up

Untranscribed Raven comment content found by the pilot (CB-5): the `ai_main_consts.rs` header glosses, the 13 `g_mover.c` QUAKED blocks, the `w_saber.rs:8506` trailing clause, the commented-out `VectorCopy` under the `g_utils` WTF block. A candidate transcription wave after this campaign, verified against the oracle side, not by token identity. The wave-1 through wave-4 reviews grew this inventory heavily: QUAKED blocks are missing across `g_team` (4), `g_target` (15), `NPC_spawn` (~50), `g_trigger` (12), `g_saga` (5), `g_nav` waypoints, and Raven FIXME/dead-code comment clusters are missing across the NPC AI files (`NPC_AI_GalakMech`, `NPC_AI_Sniper`, `NPC_reactions`, `NPC_senses`, `NPC_AI_Utils`, `g_turret_G2`, `g_log`, `g_navnew`, `NPC_stats`, `g_spawn`) - each wave's worker reports hold the per-file oracle line lists.

Retroactive CB-12/CB-13/CB-14 sweep - EXECUTED 2026-08-19 (five lanes over the 40 pilot/wave-1-3 files, ~300 sites, plus the CB-15 seam rule and the leftover process cites). The `NPC_AI_Sniper.rs` `WTF?` double space rode the sweep as planned. The sweep walk added the referee-boilerplate example to CB-13 and ran a token-gap sweep over restored Raven comments.

Flagged for a future look (facts about test coverage, not assurance - plausibly fine): the "referee"/"byte-identical" scoping statements in `NPC_AI_Rancor.rs:7`, `NPC_AI_Wampa.rs:15`, `g_session.rs:168`, `g_icarus_set_type.rs:8`, `g_target.rs:243`, `trap.rs:1942`.

The sweep grew the CB-5 inventory: `g_client.rs`'s `SET_ATTACKER` block drops two oracle lines about suicide-respawn rules (`g_client.c:292-293`), and `G_TempEntity`'s `//WTF?` block drops the trailing `//VectorCopy(...)` line (`g_utils.c:1071`).

Wave 6 grew the CB-5 inventory further: untranscribed Raven inline comments and dead-code comment blocks across `NPC_AI_Mark1` (the largest set, ~20 sites), `NPC_AI_Droid` (~15 sites), `NPC_AI_Sentry` (7), `NPC_AI_ImperialProbe` (4), `g_object` (4), `g_timer` (3), and `g_vehicleTurret` (1) - the per-file oracle line lists live in the wave-6 worker reports.

Wave 7 grew the CB-5 inventory again: `NPC_AI_MineMonster` (~20 inline sites across five functions), `tri_coll_test` (the oracle header parameter block at `tri_coll_test.c:6-13` and the `POINT_IN_TRI` notes at `:127-128`), `g_init_game` (3 dead-code blocks in `g_main.c:1009-1101`), `NPC_AI_Howler` (3), `NPC_AI_Atst` (the `#if 0` `ATST_PlayEffect` function at `NPC_AI_Atst.c:37-56` and the commented-out damage-location logic at `:80-112`), `FighterNPC` (2, including the `G_AllocateVehicleObject` rationale at `FighterNPC.c:1999-2001`), and the `-slc` pointer-field warning above `gNPC_t` at `b_public.h:109-111` - the per-file oracle line lists live in the wave-7 worker reports.

Wave 8 (30 files) grew the CB-5 inventory heavily: `NPC_AI_Mark2` (~20 inline sites), `NPC_AI_Default` (the largest set - function-header blocks and FIXME notes across nearly every function, plus the dead-code block at `NPC_AI_Default.c:776-877`), `NPC_goal` (~15 sites including the MCG-signed wrappers and the commented-out waypoint block at `NPC_goal.c:139-227`), `NPC_sounds` (~13 sites plus the commented-out `NPC_AngerSound` at `:5-20`), `g_session` (~14 sites including the `bk`-signed format markers), `gclient` (~10 pre-existing trailing-clause truncations against `g_local.h:536-748`), `AnimalNPC` (~7), `SpeederNPC` (5), `WalkerNPC` (3), `tag_owner` (2), `q_shared_cvar_flags` (2), and one each in `g_nav_consts` (`g_nav.h:9` `//???`), `g_local_consts` (`g_local.h:44`), `g_exphysics` (`g_exphysics.c:18-20`), `w_saber_consts` (`w_saber.h:30` trailing `//3000.0f`), and `NPC_misc` (`NPC_misc.c:52-55`) - the per-file oracle line lists live in the wave-8 worker and reviewer reports.

The wave-8 reviewer also found two `STAGE-2b` tags outside this campaign's scope, at `crates/mp/cgame/src/bg_channel/cg_bg_traps.rs:24,84` - they ride the cgame comment pass, not this one.

Wave 9 (the final census wave, 60 type files) added: the two `-slc` continuations on `NPCTEAM_FREE`/`NPCTEAM_NEUTRAL` (`teams.h:6,9`), `render_info`'s `customRGB` header and `//RF?` (`g_local.h:479,483`), the `//JK2 flags` section label (`g_local.h:1176`), `script_flags`' two truncated FIXME clauses (`b_public.h:37,39`), and the `//# lookMode_e` / `//# jumpState_e` enum tag comments (`b_public.h:70,77`). `FL_VEH_BOARDING`'s oracle trailing `// special shrapnel flag` (`g_local.h:66`) is a known Raven copy-paste artifact and stays untranscribed by choice.
