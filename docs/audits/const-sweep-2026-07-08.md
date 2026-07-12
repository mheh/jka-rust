# Placeholder-const sweep — 2026-07-08

Integration record for the branch-`skeleton` placeholder-const sweep. Eight
parallel phase-4 agents adjudicated every flagged local `const` in `crates/mp`
against the oracle; this doc records the outcome, the wrong-value fixes, the two
bugs the sweep itself missed, the recorded consolidation candidates, and the
verification results. Agent reports live (gitignored) under
`tools/closure-prototype/out/constsweep/phase4/`.

## Bucket counts

| Bucket | Rows | Disposition |
|---|---|---|
| WRONG-VALUE (sweep-flagged) | 66 | corrected to the oracle value |
| WRONG-VALUE (sweep-missed, found on manual re-review) | 2 | MASK_SOLID, NF_CLEAR_PATH |
| SHADOWING | 415 | deduped to canonical import/prelude glob, or kept local where no legal canonical exists |
| HOUSE-NAMED | 222 | all verified faithful to the owning oracle `.c`/`.h`; cites added/fixed |
| **Total flagged** | **703** | |

Per-agent: a1 45 (38W/7S), a2 61 (17W/38S/6H), a3 48 (11W/36S/1H),
a4 102 (93H/9S + MASK_SOLID), a5 183 (149S/34H + NF_CLEAR_PATH),
a6 86 (74S/12H), a7 107 (77S/30H), a8 71 (25S/46H).

## Wrong-value fixes

All values verified against the oracle; every enclosing function was
re-reviewed line-by-line and found otherwise faithful — only the const
definitions were defective. `old -> new`; blast radius in one line.

### NPC subsystem (commit a)

| File | Const | old -> new | Blast radius |
|---|---|---|---|
| NPC_reactions | WP_SABER | 1 -> 3 | attacker/jedi saber checks used the wrong weapon id |
| NPC_reactions | WP_THERMAL | 19 -> 12 | thermal-detonator anim branch |
| NPC_reactions | MOD_SABER | 16 -> 3 | dead-enemy means-of-death swap |
| NPC_reactions | MOD_MELEE / MOD_CRUSH | 18/12 -> 2/36 | pain means-of-death gates |
| NPC_reactions | BOTH_PAIN1/2/3/18 | 335/336/337/352 -> 95/96/97/112 | wrong pain animation numbers |
| NPC_reactions | HL_GENERIC1 | 19 -> 17 | hit-location -> pain-anim map |
| NPC_reactions | CLASS_GALAKMECH/PROTOCOL/DESANN | 13/5/8 -> 25/33/6 | NPC-class pain compares |
| NPC_reactions | CLASS_JAN/LANDO/LUKE/JEDI/PRISONER/REBEL/BESPIN_COP/R2D2/R5D2/MOUSE/GONK | (11 wrong) -> canonical | droid/humanoid respond dispatch |
| NPC_reactions | NPCTEAM_PLAYER | 0 -> 2 | playerTeam compare |
| NPC_reactions | BSET_FLEE/PAIN/FFIRE | 4/2/5 -> 8/7/14 | G_ActivateBehavior script slots |
| NPC_reactions | BSET_USE | 3 -> 1 | **behaviorSet index bug**: player-use read slot 3 (ANGER) not 1 (USE) |
| NPC_reactions | PM_DEAD | 3 -> 5 | pm_type dead check |
| NPC_reactions | MAX_CLIENTS | 64 -> 32 | NPC_Touch client-index bound |
| NPC_reactions | NPCAI_TOUCHED_GOAL | 8192 -> 8 | touched-goal ai flag |
| NPC_reactions | CLASS_VEHICLE/GONK | 19/23 -> 53/11 | NPC_Use class compares |
| SpeederNPC | VEH_SLIDEBREAKING | 0x4 -> 0x80 | slide-brake mask tested/set the wrong bit |
| SpeederNPC | ENTITYNUM_NONE | -1 -> 1023 | groundEntityNum compare never matched |
| SpeederNPC | SETANIM_FLAG_OVERRIDE/HOLD/RESTART/HOLDLESS | 0x100/0x200/0x400/0x800 -> 1/2/4/8 | wrong flag bits to BG_SetAnim |
| NPC_AI_GalakMech | SCF_CHASE_ENEMIES | 0x80 -> 0x400 | wrong scriptFlags mask |
| NPC_AI_GalakMech | SCF_DONT_FIRE | 0x800 -> 0x4000 | wrong scriptFlags mask |
| NPC_AI_GalakMech | SCF_FIRE_WEAPON | 0x1000 -> 0x40000 | wrong scriptFlags mask |
| NPC_AI_GalakMech | ARMOR_EFFECT_TIME | 3000 -> 500 | dead-code (`if(0)`) block; corrected for record |
| npc_c | SVF_ICARUS_FREEZE | 0x400 -> 0x8000 | ICARUS ai-freeze check masked the wrong bit; never fired |
| NPC_misc | DEBUG_LEVEL_DETAIL/WARNING/ERROR | 1/5/7 -> 4/2/1 | inverted debug-severity ladder |
| NPC_combat | NF_CLEAR_PATH | 0x1 -> 0x2 | **sweep-missed**; nav path-request flag (copy/paste of NF_CLEAR_LOS) |

### g_cmds / g_items / g_team / g_spawn (commit b)

| File | Const | old -> new | Blast radius |
|---|---|---|---|
| g_spawn | ENTITYNUM_WORLD | 0 -> 1022 | SP_worldspawn indexed g_entities[0] (client slot) not the world entity |
| g_team | SIEGETEAM_TEAM1 / TEAM2 | 0/1 -> 1/2 | siege spawn-class pick compared the wrong team |
| g_items | SVF_SINGLECLIENT | 0x40 -> 0x100 | pickup temp-entities flagged SVF_PORTAL, not single-client |
| g_items | DAMAGE_DEATH_KNOCKBACK | 0x08 -> 0x80 | e-web missile got DAMAGE_NO_PROTECTION dflags |
| g_cmds | MAX_SIEGE_CLASSES | 12 -> 128 | class scan capped; siege classes 12-127 unreachable |
| g_cmds | MAX_CUSTOM_SIEGE_SOUNDS | 32 -> 30 | latent OOB (array is len 30) |
| g_cmds | MAX_VOTE_COUNT (x2) | 5 -> 3 | vote-count gate |
| g_cmds | SETANIM_BOTH | 2 -> 3 | body-anim flag |
| g_cmds | MAX_CLIENT_SCORE_SEND | 32 -> 20 | scoreboard batch size |

### g_weapon (commit c)

| File | Const | old -> new | Blast radius |
|---|---|---|---|
| g_weapon | MASK_SOLID | 1 -> 4097 | **sweep-missed**; dropped CONTENTS_TERRAIN, so rifle/flechette-mine/emplaced-ATST traces missed terrain brushes |

## The two sweep-missed bugs

The sweep classifier reported 0 WRONG-VALUE for agents 4 and 5; both bugs were
caught by the agents during the mandated enclosing-function re-review, not by the
classifier:

- **`g_weapon.rs` MASK_SOLID** = `CONTENTS_SOLID` (1) instead of
  `CONTENTS_SOLID|CONTENTS_TERRAIN` (4097). Three trace sites missed terrain.
- **`NPC_combat.rs` NF_CLEAR_PATH** = `0x1` instead of `0x2` — `0x1` is the
  value of the commented-out `NF_CLEAR_LOS` directly above it (a copy/paste
  slip). Passed as the `flags` arg to `trap_Nav_GetBestPathBetweenEnts`.

## Consolidation candidates (recorded, not actioned)

Deferred to a future pass because they need a new shared home the sweep must not
invent:

- **q_shared.h net limits -> `mp_qshared`**: `MAX_NAME_LENGTH`,
  `MAX_STRING_TOKENS`, `MAX_TOKEN_CHARS`, `MAX_MODELS`/`MAX_SOUNDS`/`MAX_FX`/
  `MAX_ICONS`, `MAX_SUB_BSP`, `MAX_WPARRAY_SIZE` — currently mirrored across
  bg/cgame/game/engine/renderer at ineligible tiers.
- **ghoul2 `BONE_*` / `BONE_ANGLES_POSTMULT` flags (`ghoul2/G2.h`)**: scattered
  fn-locals in g_client, g_turret_G2, g_items, NPC_utils; want one shared
  ghoul2-flags const module.
- **`FRAMETIME` = 100**: per-file in GalakMech, npc_c (x3), g_items (the de-facto
  canonical), g_mover, g_ICARUScb, g_trigger, g_turret_G2 — belongs in a shared
  bg `bg_public` const.
- **`CP_*` / `CPF_*` combat-point request flags (`b_local.h`)**: duplicated across
  NPC_combat/behavior/Grenadier/Sniper/Stormtrooper; want `crate::npc::combat_point_flags`.
- **`ALERT_CLEAR_TIME`, `DEBUG_LEVEL_*`, `TURN_OFF`** and other per-`.c` `#define`
  copies with no shared home.
- **Type-width splits** to reconcile in one home: `HL_MAX` (c_int enum vs usize),
  `MAX_WPARRAY_SIZE`/`MAX_NODETABLE_SIZE`/`MAX_SPAWNPOINT_ARRAY`/`MAX_SIEGE_INFO_SIZE`
  (usize vs c_int/i32).
- **`CS_ITEMS` / `CS_CLIENT_JEDIMASTER` / `DEFAULT_MINS_2`/`DEFAULT_MAXS_2`**:
  game-crate copies of bg canonicals — game should import from bg (partially done
  this pass via g_client explicit imports; the `pub` re-export copies remain to
  avoid breaking bare-use sites).

## Verification

- `cargo build --workspace`: green.
- `cargo check --workspace --all-targets`: green (the two pmove parity tests
  regained `DEFAULT_MAXS_2` via the game prelude — see integration repairs).
- `cargo test -p mp_game -- --test-threads=1`: all pass except
  `saberload::saberload_parity`, which is a **pre-existing** failure unrelated to
  this sweep (confirmed by reproducing it on clean HEAD — it panics in
  `g_strap.rs:54`, an engine-seam GAME_INIT ordering issue). `cargo test -p mp_bg`:
  0 tests, green.
- **A/B referee (release dylib vs oracle C dylib, lockstep, map mp/ffa1, seed 42,
  batch 1)**:
  - `corpus-ffa1.rec`: PASS, 4000 frames, no divergence.
  - `corpus-ffa1-combat.rec`: PASS, 5000 frames (full 4800-frame recording
    covered), no divergence.

  The wave fixed real behavior bugs yet the corpora stayed byte-for-byte in sync
  with the oracle — the fixed consts guard code paths the recorded FFA inputs
  do not exercise (siege, e-web, specific NPC classes, terrain-brush traces),
  so parity held while the latent bugs were removed.

## Integration repairs (mechanical)

- `crates/mp/game/src/prelude.rs`: added `DEFAULT_MAXS_2` to the viewheight
  re-export line (agent 7 removed the g_client `pub const` copy; the two pmove
  parity tests reach it through the game prelude).
- `crates/mp/game/tests/gcombat_parity.rs`: removed the shadowing local `YAW`
  (agent 8); the canonical `q_math::YAW` resolves via `mp_game::prelude::*`.

No value conflicts between agents required oracle adjudication; agent 1's flagged
suspicion (npc_c.rs SVF_ICARUS_FREEZE = 0x400) was independently fixed by agent 2
who owned the file.

## Commits (branch skeleton)

1. `fix(game): correct wrong-value placeholder consts in NPC modules`
2. `fix(game): correct wrong-value placeholder consts in g_cmds/g_items/g_team/g_spawn`
3. `fix(game): restore CONTENTS_TERRAIN in g_weapon MASK_SOLID`
4. `refactor(bg): make entity_flags the canonical EF_* home`
5. `refactor(game): dedupe shadowing consts to canonical imports, fix lying comments`
6. `refactor(engine,cgame,renderer,ui): dedupe shadowing consts, fix cites`
7. `chore(game): const-sweep integration repairs + audit (this doc)`
