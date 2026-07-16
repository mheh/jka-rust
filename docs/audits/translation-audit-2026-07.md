# Translation-bug audit — mp_game file-by-file (task #10 / GOAL item 5)

Working ledger of the file-by-file oracle comparison. Findings accumulate
here per wave; fixes land in referee-gated batches and graduate to
`referee-catches-2026-07.md` with their commit hashes. Lens list and method:
GOAL item 5.

Status: wave 1 of ~27 (live-in-FFA files first).

## Wave 1 (2026-07-15) — g_utils, g_items (audited clean-ish), g_active, g_missile (pending report)

### g_utils.rs + g_items.rs — 6 findings, none live-combat

| # | Site | Oracle | Finding | Severity |
|---|---|---|---|---|
| 1 | `g_utils.rs:440` G_Throw kvel[2] | `g_utils.c:333` | C folds `newDir[2]` into the f32 product chain, then `*1.5` in double; port restructured the association (newDir last, in f64) — different intermediate rounding on the Z knockback. Fix: match C's association exactly. | latent, live branch (~1-ULP) |
| 2 | `g_utils.rs:719` G_UseTargets2 | `g_utils.c:574` | `level.time * 0.001` is double in C; port is all-f32. Shader-remap timeOffset, masked by `%5.2f`. | rare-branch |
| 3 | `g_items.rs:202,205` adjustRespawnTime | `g_items.c:74,78` | `20.0 /` and `8.0 /` are double divides in C; port all-f32. Behind `g_adaptRespawn` (default 0), masked by int truncation. | rare-branch |
| 4 | `g_utils.rs:2578` ShortestLineSegBewteen2LineSegs | `g_public.h:9` | `Q3_INFINITE` is **16777216**, port used `f32::INFINITY`. Masked (map distances ≪ 2^24) but wrong constant. | latent (masked) |
| 5 | `g_items.rs:1004,1089,3358` | `g_items.c:628,690,3139` | C truncates the whole `level.time + K + random()*X` sum as one float; port truncates only the random part. Diverges once `level.time` exceeds f32 precision (~4.6 h uptime). | latent (very rare) |
| 6 | `g_utils.rs:2085` TryUse | `g_utils.c:1829-1836` | `Touch_Button` dispatch arm dropped (marked PORT-NOTE): USE on a touch-activated func_button does nothing. | rare-branch (documented) |

Re-verified clean under scrutiny: both crandom sites (d6ef7674 holds); pas_think
sin (oracle literals are float-suffixed); G_Throw kvel[0]/[1] macro expansion;
LaunchItem FL_BOUNCE_HALF clobber (faithful Raven bug); G_KillBox zero-vector
NULL convention (residual world-origin edge lives in g_combat, astronomically
rare). Full clean-lists in the wave-1 agent reports (session task outputs).

### g_active.rs + g_missile.rs — CLEAN (0 behavioral divergences)

Systematic top-to-bottom walk; every float-flattening candidate numerically
swept and cleared (P_DamageFeedback pitch-byte math, DoImpact `>= 0.2`
inclusive compare, fall-damage `delta*0.16` — all byte-identical to f64;
contrast the strict-`>` bounce compare that WAS a real bug, 435f7d57).
Eval-order, loop-increment, switch-completeness, NULL-sentinel, and
dead-quirk preservation all verified. Two §19 UB-hardening guards diverge
from Raven only where Raven would crash, both **unmarked** — add the ≤2-line
site notes per porting-rules §19:

- `g_missile.rs:353-359` G_ExplodeMissile: null-client guard on
  `accuracy_hits++` (oracle g_missile.c:245-252 derefs unconditionally).
- `g_missile.rs:984-987` G_MissileImpact: `think` null guard (oracle
  g_missile.c:677 calls unconditionally).

Wave-1 fix batch landed: `b0bee6e4` (findings 1-5 fixed + the two §19
markers; #6 TryUse stays documented-parked).

## Wave 2 (2026-07-15) — bg_pmove, w_saber, g_combat+g_client, bg_saber+g_cmds

### bg_pmove.rs — 5 findings (all f32-vs-double threshold class), core path CLEAN

Full-coverage walk (lines 127-10377, no gaps); prior fixes (lastHeadAngles,
neck-angle f64s) re-verified intact. Every finding is a libm-double or
bare-double-literal C expression evaluated in f32 (Q_fabs-for-fabs accounts
for 3 of 5); none in the non-vehicle live-combat hot path:

| # | Site | Oracle | Finding | Severity |
|---|---|---|---|---|
| 1 | `bg_pmove.rs:384-385` PM_SetVehicleAngles | `bg_pmove.c:526` | bare-double `0.5` keeps the pitch adds in f64; sibling branch's `0.5f` asymmetry is real in the oracle | latent (vehicle-in-water) |
| 2 | `bg_pmove.rs:415` PM_SetVehicleAngles | `bg_pmove.c:563` | `speed *= sin(...)` product is double in C; port narrows sin first | latent (vehicle banking) |
| 3 | `bg_pmove.rs:549-550,640-642` PM_HoverTrace | `bg_pmove.c:757,835` | `fabs+fabs > 100` sums in double; f32 sum can flip at the boundary — gated arm calls Q_irand → **RNG-stream desync** | latent, PRIORITY (RNG-coupled) |
| 4 | `bg_pmove.rs:2790` PM_CheckJump | `bg_pmove.c:1988` | `fabs(dotR) > fabs(dotF)*1.5` double product vs f32 — flip-anim axis tie-break | latent (anim, crosses wire) |
| 5 | `bg_pmove.rs:4225` PM_CrashLandEffect | `bg_pmove.c:3691` | `fabs(v)/10` double divide vs f32 at the `>= 30.0` dust-effect gate | latent/cosmetic |
| — | `bg_pmove.rs:9907-9955` PM_VehicleViewAngles | `bg_pmove.c:9646-9709` | oracle reads clampMin/Max uninitialized (UB); zero-init pick needs its §19 site note | doc-only |

### g_combat.rs + g_client.rs — CLEAN (damage/death spine and client lifecycle byte-faithful)

No live-combat or rare-branch bugs. Prior fixes (four point-null guards)
re-verified. One systemic observation promoted to a sweep item (below);
one stale PORT-NOTE (G_DamageFromKiller claims a zero-vector dir, code
correctly passes None) — doc-drift fix rides the next batch. g_client:
zero findings across ~45 fns (field-copy completeness, IPstring
truncation, spawn-sort loops, all NULL-guard polarities verified).

### w_saber.rs — 1 LIVE-COMBAT bug (fixed same-day) + 2 latents; 82 fns otherwise faithful

| # | Site | Oracle | Finding | Severity |
|---|---|---|---|---|
| 1 | `w_saber.rs:6699` WP_SaberStartMissileBlockCheck | `w_saber.c:5561` | **One-token enumerator flip**: oracle gates on `!= ET_NPC` ("player doesn't auto-activate"); port wrote `!= ET_PLAYER` — inverted holstered-saber missile-block behavior between players and NPCs. Behind `BG_SabersOff && !CLASS_BOBAFETT`, which is why duel1 scenarios never hit it. FIXED same-day (referee suite green incl. real-map combat). | live-combat |
| 2 | `w_saber.rs:11661-11675` WP_SaberBlockNonRandom | `w_saber.c:9150-9173` | `rightdot > 0.3/0.1` double compares in f32 — F3 class, measure-zero. | latent (→ F3 sweep) |
| 3 | `w_saber.rs:658-664` SetSaberBoxSize | `w_saber.c:450-461` | The one `g_saberDebugPrint` diagnostic (of 15) dropped marker-less. Diagnostic-only; needs a marker or the print. | latent (doc) |

Prior fixes re-verified (saberEnd write-back, lastHeadAngles, ghoul2 bolt
consumption); CheckSaberDamage's full double-vs-float literal set verified;
RNG-consuming short-circuit chains verified operand-by-operand.

### bg_saber.rs + g_cmds.rs — bg_saber CLEAN in full; g_cmds priority paths clean

One cross-cutting latent: say/tell/vote paths round-trip netnames/chat
through lossy UTF-8 (`cstr_to_str`) — non-ASCII bytes (0x80-0xFF, common in
player names) become U+FFFD, diverging wire bytes/logs and team-vote name
matching vs C's raw-byte copies. Referee-blind (bots are ASCII). Fix shape:
raw-byte handling through the say/tell/vote formatting. Also: g_cmds has an
explicitly-listed NOT-audited tail (~38 lower-priority fns — team/duel/
follow/give handlers) for a wave-3 pass. bg_saber: every fn faithful, incl.
preserved saber1/saber2 copy-paste oracle bug and the vote `msg[1]` typo.

Wave-2 fix batch landed: `70d706ea` (bg_pmove 1-5 + §19 note, w_saber 2-3,
G_GetHitQuad/G_GetHitLocation F3 sites, G_DamageFromKiller doc-drift;
adversarially validated, full referee suite green incl. real-map duel1).

## Wave 3 (2026-07-15) — g_weapon+w_force, bg_slidemove+bg_misc+g_main, ai_main+ai_util+ai_wpnav

### w_force.rs — 1 LIVE-COMBAT bug + 1 rare-branch

| # | Site | Oracle | Finding | Severity |
|---|---|---|---|---|
| 1 | `w_force.rs:6202-6217` WP_ForcePowersUpdate | `w_force.c:5444-5454`, `:7` | **Preprocessor-polarity error**: the two `else if` charge-jump arms live inside `#ifndef METROID_JUMP`, and `#define METROID_JUMP 1` is unconditional — dead in every retail/referee build. The port transcribed them live, re-enabling the cut "charge jump" mechanic for any grounded force-capable player holding jump ≥150ms (built forceJumpCharge, played jumpbuild.wav, suppressed force regen, fired a charged jump retail never fires). Invisible to duel1 tapes only because bots don't produce that input pattern. Arms dropped with site note. | live-combat |
| 2 | `w_force.rs:4321-4328` ForceThrow func_door arm | `w_force.c:3735-3737` | Oracle recomputes into the fn-local `forward` (persists into later same-loop `G_ReflectMissile`); port writes a fresh local. ~1-ULP, needs spawnflags-2 func_door ordered before a pushable missile in one throw. | rare-branch (parked) |

### g_weapon.rs — 3 latents + 2 doc-only (all fixed/aligned in batch)

charge_stick f64 chain + accumulator (c:2671-2673, `vNor[1]`-on-[2] oracle
bug preserved); BlowDetpacks single-truncation (c:2863); WP_GetVehicleCamPos
fabs product (c:3971); CalcMuzzlePointOrigin (dead fn) aligned to
snap_vector; flechette arg-order verified matching g++-16 eval order.

### bg_slidemove.rs — 2 findings (fixed)

PM_StepSlideMove `DotProduct < 0.7` f32-vs-double compare (c:902 — movement
clip spine, same class as 435f7d57); PM_VehicleImpact `fabs+fabs < 100.0f`
f32 sum (c:102, vehicle-only). bg_misc.rs CLEAN (trajectory arms, PS→ES
copies all verified; NOTE: bg_itemlist table lives in
`crates/mp/bg/src/public/bg_itemlist.rs` — table lens still to run there).
g_main.rs: G_RunThink int-vs-float think compare (documented divergence,
>4.6h uptime — parity decision parked); g_listEntity §19 note added.

### ai_main.rs + ai_util.rs + ai_wpnav.rs — combat spine CLEAN

Substantive negative result for the bot-lethality residual: aim, fire
gating, reaction timers, weapon weights, chase/range, saber handling,
enemy acquisition all byte-faithful (incl. RNG-consuming short-circuits,
accVal/reflex float-division checks, CON_CONNECTED==CA_AUTHORIZING quirk).
One fix: BotUtilizePersonality allocated readbuf/group before the
personality-load early returns while freeing only buf — two failed loads
exhausted the 3-slot BG_TempAlloc table (oracle allocates after the
returns; reordered to match). Five stale "no ported home" PORT-NOTEs
removed. Unaudited tails: CTF/Siege team handlers, BotDoChat/chat parsers,
ai_wpnav nav infrastructure (stub-swept clean, not line-diffed).

Wave-3 fix batch landed: `2a9b171e` (adversarially validated; full referee
suite green). Sharp F3 criterion from this wave, for the port-wide sweep:
with SSE f32 operands, a double-vs-f32 threshold compare diverges ONLY at
`x == f32(lit)` and only when the operator direction matches the literal's
f32 rounding direction — `>`/`>=` diverge for round-UP literals (0.1, 0.2,
0.3, 0.4, 0.6, 0.8…), `<`/`<=` for round-DOWN literals (0.7, 0.9…).

## Wave 4 (2026-07-15) — bg_itemlist+g_cmds-tail+g_main-helpers, g_mover+g_trigger+g_target, g_bot+g_session+g_spawn+g_svcmds

### CLEAN in full

bg_itemlist (all 52 rows field-by-field, missing-comma quirk preserved —
closes the wave-3 coverage note); g_main scoring/vote/tournament helpers
(vote int-division thresholds, SortRanks comparator signs); g_trigger (all
42 fns — RNG-coupled strike/asteroid paths operand-faithful, crandom
nextthink chains correctly f64); g_bot (G_AddRandomBot RNG stream-faithful
— relevant negative result for the lethality residual); g_session
round-trip; g_svcmds filters.

### Findings (fix batch below)

| # | Site | Oracle | Finding | Severity |
|---|---|---|---|---|
| 1 | `g_mover.rs:261-271` G_TryPushingEntity | `g_mover.c:185` | Rotating-crusher G_Damage passes dflags 0; oracle passes DAMAGE_NO_KNOCKBACK (port flings crushed entities retail doesn't). Stale "const not ported" comment — it is. | live-branch |
| 2 | `g_mover.rs:794,809` SetMoverState | `g_mover.c:560,575` | `1000.0/trDuration` f64 divide flattened to f32 — trDelta crosses the wire into mover prediction. | latent (wire) |
| 3 | `g_spawn.rs:1122-1124` G_ParseSpawnVars | `g_spawn.c:1061-1064` | **Structural**: HandleEntityAdjustment called per key/value pair; oracle calls once after the loop. Sub-BSP instances re-rotate/re-prefix N times. | latent (sub-BSP) |
| 4 | `g_spawn.rs:623-700` FIELDS behaviorSet | `g_spawn.c:106-121` | Offsets hard-code 8-byte pointer stride — wrong on ILP32 retail target (4-byte slots); referee-blind (LP64 correct). | latent (ILP32) |
| 5 | `g_cmds.rs:854` G_CheckTKAutoKickBan | `g_cmds.c:541` | Added `IPstring[0]!=0` guard; C tests the array address (always true) — AddIP skipped for empty-IP clients. Twin in Cmd_Kill_f ported correctly. | rare-branch |
| 6 | `g_cmds.rs:285` ConcatArgs | `g_cmds.c:137-149` | **Root of the UTF-8 latent class**: from_utf8_lossy inflates 0x80-0xFF and shifts the MAX_STRING_CHARS break point vs C's strlen/memcpy. | latent (class root) |
| 7 | `g_cmds.rs:825` Cmd_TeamTask_f | `g_cmds.c:520-521` | Whole userinfo lossily re-encoded and persisted via SetUserinfo; C mutates one key in place. | latent |
| 8 | `g_spawn.rs:948,1009` HandleEntityAdjustment | `g_spawn.c:905,945-949` | DEG2RAD all-f32 (C: f64 via double M_PI); fresh `direction` buffer vs oracle's reused `angles` on partial sscanf. | latent (sub-BSP) |
| 9 | `g_target.rs:253-259` Use_Target_Print | `g_target.c:149-181` | `#ifndef FINAL_BUILD` Com_Error aborts dropped (live in referee build; genericValue15 write kept — inconsistent). | low |
| 10 | doc-only | — | §19 notes needed: Cmd_DebugSetSaberMove_f negative-index panic pick; G_LoadIPBans defined banIPFile. | doc-only |

Wave-4 fix batch landed: `c4d17d75` (findings 1-10 above; adversarially
validated, full referee suite green).

Parked (recorded, no action): remaining lossy-UTF-8 netname print
instances (class shrinks once ConcatArgs is byte-exact); g_session
sscanf-abort vs parse-continue on corrupt cvars; w_force ForceThrow
`forward` persistence (wave 3 #2); the `(float)level.time + X`
single-truncation parity decision (same as g_main G_RunThink, wave 3).
Mover/trigger recalibration recorded by the auditor: power-of-two scalars
(0.5/2.0) make f32 mul-add bit-exact vs f64 intermediates — not findings.

## Wave 5 (2026-07-15) — g_team+g_saga+g_arenas, g_misc+g_object+g_exphysics+g_timer+g_log, vehicles+turrets

### CLEAN in full

g_arenas (UpdateTournamentInfo byte-faithful, dead podium block correctly
unported); g_exphysics (every literal f32-suffixed in oracle); g_log (the
855b73ef §19 marker stands, sibling guards all marked); g_vehicleTurret
(all 6 fns).

### Findings (fix batch below)

| # | Site | Oracle | Finding | Severity |
|---|---|---|---|---|
| 1 | `g_team.rs:384` Team_SetFlagStatus | `g_team.c:316` | **Literal 6 instead of CS_FLAGSTATUS (23)** — every CTF/CTY flag-status update clobbers CS_SCORES1 and the flag HUD never updates. Referee-blind (duel1 has no flags). | **live CTF** |
| 2 | `g_vehicles.rs:1371-1383` Update | `g_vehicles.c:1770,2245` | Oracle truncates `prevSpeed`/`nextSpeed` to int and compares as ints; port compares f32 — the gear-shift gate flips at fractional speeds and short-circuits `Q_irand(0,1000)` differently → **RNG stream desync**, every frame per active vehicle. | latent PRIORITY (vehicle maps/Siege) |
| 3 | `game_globals.rs:1073` gSiegeBeginTime | `g_saga.c:39` | Oracle file-scope-inits to Q3_INFINITE; port defaults to 0 — can skip the 5s Siege warmup when both teams populate at first check. | latent (siege) |
| 4 | `g_object.rs:239` G_RunObject | `g_object.c:196` | `normal[2] < 0.7` f32 compare; bare-double 0.7 + `<` is divergence-capable (sharp criterion) — slope-vs-bounce spine. | latent (F3) |
| 5 | `g_misc.rs:1968,1287` shield_floor/holocron | `g_misc.c:1618-1631,1020-1033` | Spawn Z-nudge `±0.1` bare-double flattened to f32 (oracle inconsistent: ammo_floor uses 0.1f — port matches that one). | latent (spawn, low) |
| 6 | `g_team.rs:415` Team_ForceGesture | `g_team.c:341` | Loop bound g_maxclients vs oracle MAX_CLIENTS (dead fn — call site rww-commented). | low (dead) |
| 7 | doc-only | — | §19 notes: Team_SetFlagStatus zero-init `st[4]` (oracle sends stack garbage in non-CTF); six g_timer null guards. | doc-only |

**CRITERION CORRECTION (2026-07-15, F3 sweep):** the sharp criterion as
first recorded below lumps `>=` with `>` — wrong. Corrected truth table
(divergence only at `x == f32(L)`, exact-f32 operand): `>` diverges iff
L rounds UP; `<` iff DOWN; **`>=` iff DOWN** (C false / Rust true);
**`<=` iff UP**. Inclusive operators diverge in the OPPOSITE rounding
direction from their strict forms. All wave-2/3/5 strict-operator fixes
stand; the sweep confirmed the inclusive-divergent class is empty in the
whole game corpus.

Wave-5 fix batch landed: `a1649893` (findings 1-7 + validator GAP closure:
gPainHitLoc/gLastPrintedIndex/saberClashEventParm seeded to Raven's
file-scope values for exact load-state parity; adversarially validated,
full referee suite green). Validator lifecycle ruling: GameGlobals
rebuilds per GAME_INIT, matching the oracle's dylib-reload-per-map-change
init; light map_restart retention is a pre-existing architectural
property, not batch-introduced.

Parked additions to the single-truncation class (no action, same parity
decision as G_RunThink): g_turret bounceCount, g_turret_G2
attackDebounce/bounceCount, g_object hitTime, g_misc fx_runner nextthink.
Also recorded, no action: g_saga Com_sprintf team-name-as-format (needs
`%` in a .siege name); g_saga objective-cfg 1024 cap (needs ~511
objectives); G_StartObjectMoving dir shadow (zero callers).

RNG-coupled paths verified stream-faithful: CTF/Siege spawn selection
(`rand() % count`), SiegeItemThink Q_irand velocities, HolocronPopOut /
misc_faller draw orders, G_RunObject flrand order.

## Wave 6 (2026-07-15) — nav/ICARUS, ai tails, NPC spine + full NPC surface

### CLEAN in full

Bot-AI trio now 100% line-audited (wave 3 spine + wave 6 tails):
BotInputToUserCommand wire path double-verified, BotDoChat RNG exact,
ai_wpnav editor/save infra faithful (two %f-format riders fixed).
g_ICARUScb SET_*/GET_* dispatch mechanically diffed COMPLETE (208/208,
109/109, 61/61). NPC spine (combat/move/senses/reactions/goal) —
float+RNG spine clean, §19 notes only. NPC_stats 116-key parse table
complete row-by-row; NPC_utils/misc/sounds clean; WalkerNPC,
g_vehicleTurret-style clean list continues (Atst, Droid, Howler, Mark2,
MineMonster, Sentry, ImperialProbe, Interrogator, GalakMech).

### Round-1 findings (fixed: `89fa85db`)

**movedir out-param threading** (g_nav + g_navnew AvoidCollision→
Resolve→Bypass chains ported by value — NPC collision sidestepping
silently dropped; same class as the 25-fn botlib catch fddd3eca);
NAVNEW_MoveToGoal 7 goto-failed sites dropped the failed-label
trap_Nav_GetNodePosition (syscall-trace parity); PushBlocker *1.2 /
3× avoidRadius sqrt sums / Q3_SetParm += atof flattened to f32; NAV
fatal-print args dropped; NPC pain-anim [-1] §19 + null-guard cluster;
.wnt %f fields; personality-filename lossy round-trip.

### Round-2 findings (fix batch in flight)

| Area | Headline findings |
|---|---|
| NPC_spawn (live `npc` cmd) | **`npc kill <team>` null-TeamTable crash**; ground-snap traces mask `1` vs MASK_SOLID (terrain tunneling); `npc score` G_Find fieldofs 0; default-falls-into-WP_BLASTER switch collapsed (ST_ClearTimers skipped); tempGoal-fail returns null vs oracle's goto-finish newent; FRAMETIME 50 vs 100. |
| RNG hoists (5× PRIORITY) | ST_CheckMoveState (dead draw + **missing return** — reached-goal path runs keep-going tail); Rancor_Attack + Wampa_Pain (draw hoisted above NPC_SetAnim, reads stale legsTimer); Mark1_dying, Grenadier_CheckMoveState (dead/reordered draws). One shape: RNG `let`-hoist above branch-local draws. |
| npc_c | G_DroidSounds `_ => return` skips the fall-through Q_irand+TIMER_Set C runs for unmatched classes (1/21 per-frame stream desync, live); NPC_HandleAIFlags nested-Q_irand arg eval order (empirical g++ check in batch). |
| NPC_behavior | NPC_BSWander tail statements mis-nested inside the numEdges check — no-edge NPCs stall (live-branch). |
| NPC_AI_Jedi | Jedi_CheckDanger missing `alertEvent == -1` guard — oracle reads alertEvents[-1] (silent UB), Rust **panics** (§19 guard added). Saber/evasion/force RNG spine otherwise exceptionally faithful. |
| **NPC_AI_Sniper** | **Knowingly 2/13 ported, 11 marker-less silent stub bodies** (zeroed muzzle vectors, `if true` trace guards, dropped G_SetEnemy/aim math) — the saber-whiff class at file scale. Dedicated transcription agent completing the file. |
| Vehicle NPCs | FighterNPC dropped live G_DamageFromKiller (damaged fighters land without dying); **AnimalNPC #ifndef _JK2MP polarity** (SP-only auto-aim ported live — second METROID_JUMP-class catch); SpeederNPC electrify wobble parked-but-portable; sin(serverTime*0.001) f64 ×6. |
| Misc latents | Remote idealDist double compares; Seeker VectorMA f32-scale truncation; NPC_FacePosition += f64; Mark1 gPainHitLoc read order. |

Wave-6 round-2 fix batch landed: `76d974c1` (all findings above + the
completed NPC_AI_Sniper 15/15 port). **NEW TRAP RATIFIED THIS BATCH —
VectorMA/VectorScale/DotProduct are the live `#if 1` MACROS
(q_shared.h:1352); the `_Vector*` float-scale prototypes are the dead
`#else`. An RNG call (or any side-effecting/double-typed expression) in
the scale substitutes PER COMPONENT — three draws, per-component values.**
Three sites fixed (Sniper miss-loop ×4 calls, Jedi aim-lead); one wrong
"fix" built on the dead signature reverted (Seeker — auditor false
premise, caught in gate-1 diff review). Sweep of _VectorMA/_VectorScale/
VectorAdvance with RNG in scale: no further sites. NPC_HandleAIFlags
arg-eval order verified empirically on g++-16 arm64 (left-to-right) —
re-verify if the referee-oracle compiler/arch ever changes.

Parked-class additions (record only): NPC_SetBlocked/NPC_Blocked/
Q3_Lerp2Angles/NPC_BSPatrol vigilance/species nextthink sites — all
`(float)level.time + X` single-truncation. Validator observation:
g_nav.rs:2240 `Com_Error(3 /* ERR_DROP */)` — the literal is inert (the
port's Com_Error drops `level`, matching Raven's G_Error("%s")), but the
comment mislabels 3 as ERR_DROP (enum value 1).

## F sweeps (2026-07-15) — F1/F2/F3 port-wide: ALL THREE CLASSES CLOSED

- **F3 (threshold compares): CLOSED.** Mechanical scan of all 89 game .c
  files (scratchpad f3_scan.py, corrected truth table above): 126
  FP-literal compares → 25 divergence-capable, every live one already
  fixed in waves 2/3/5; the inclusive-op divergent class is empty in the
  corpus; sole remaining f32 site was dead DebugLine (promoted anyway).
- **F1 (double-arithmetic flattening), unaudited tail: CLOSED.** q_math
  — the float substrate — audited fully CLEAN (all libm/M_PI chains
  promoted; Q_crandom/AngleSubtract/powf-bug individually proven
  bit-exact). bg_lib linkage trap grounded: Raven's own unguarded atof
  (f32 accumulator) is what natively links, and the port matches it,
  leading-dot quirk included. bg_saberLoad/bg_vehicleLoad/tri_coll_test/
  bg_g2_utils/g_mem/g_strap/veh_dispatch/g_init/g_shutdown/
  g_icarus_set_type (212-row table machine-diffed) all clean. Two real
  findings, both fixed this batch: **bg_panimate `numFrames-1`** (C
  promotes the ushort to signed int, 0 → -1 into the dur<=1 fallback;
  the u16 port panicked/wrapped — also the product runs in f64) and
  **bg_saga class_shader missing-key arm** (port forced SPC_INFANTRY and
  dropped the error print; oracle prints and leaves the value).
- **F2 (dropped nullable-param guards): CLOSED, zero GUARD-LOST.**
  Scanner over 384 guard sites / 297 func-params: the 956101f7 restores
  all stand; ~90 params guard-preserved via Option/null/zero-vec
  conventions with fallbacks intact; 5 provably-dead guards documented;
  no reachable NULL-branch behavior lost anywhere. Two stale g_vehicles
  PORT-NOTEs fixed (dir is `None` now, not a zero-vec). Out-of-class
  flag for a future ABI-arg parity pass: WP_SaberAddG2Model passes an
  empty CString where C passes raw NULL to G2API_InitGhoul2Model.
- Doc riders this batch: ColorBytes3 §19 note, g_icarus_set_type cite.

### Wave-7 queue (PAUSED here at user request)

- Port-wide F3 sweep using the sharp criterion (only round-direction-
  matching literals can diverge — much smaller site set than the naive grep).
- bg_* remainder: bg_panimate, bg_saberLoad, bg_saga, bg_vehicleLoad,
  bg_g2_utils, q_math (vs oracle q_math.c), bg_lib divergence check.
- Small g_* stragglers: g_init_game/g_shutdown_game, w_force wave-3 #2
  (ForceThrow forward persistence), veh_dispatch, tri_coll_test,
  g_exphysics follow-ups if any.
- Cross-cutting: hoisted-RNG-draw grep results (round-2 batch item 12),
  remaining lossy-UTF-8 print instances, single-truncation parity
  decision (one ruling closes ~15 recorded sites).
- **Scenario burn-down (user-requested 2026-07-15): convert referee-blind
  surface to gated tapes** — CTF (flags → CS_FLAGSTATUS class), team FFA,
  vehicle map (gear-shift/Fighter/Animal fixes), Siege (gSiegeBeginTime
  warmup), NPC-spawn (`npc` command path), map_restart + full map-change
  tape (GAME_INIT/module-load state). Prereq check: does the input tap
  record server-console/rcon commands as tape events? Plus gcov on the
  oracle build over the tape set to measure exercised coverage, and
  randomized-usercmd fuzz tapes for boundary-value volume.
- **Engine/game-specific verification tracks (user-ratified 2026-07-15
  as required work, not optional):** (a) expression-level differential
  harness for the ~30 audit-batch float-width sites (§F golden pattern:
  compile touched oracle fns standalone, drive boundary+random inputs,
  compare bits — retro-validates every `as f64` decision mechanically);
  (b) **dual-host referee** for the engine side of the ABI seam: same
  oracle module dylib under our engine AND a reference engine built from
  oracle source (throwaway patches per build.sh discipline), seam-recorder
  proxy logs vmMain+syscall traffic both directions, taped clock/packets
  for determinism — any log divergence is an engine bug by construction;
  (c) syscall-granularity goldens for stateful arms (world-snapshot +
  fuzzed SV_Trace/EntitiesInBox/etc. against compiled oracle) and a
  wire-level diff (same taped client packets → both servers, compare
  outbound UDP) as cheaper partial steps.

### NEW SWEEP ITEM — F3: unsuffixed-double THRESHOLD COMPARES in f32

`float dot; if (dot > 0.3)` promotes to double in C (`0.3` is a double
literal); the port compares in f32 wherever libm/multiplication didn't
force f64. Sites: G_GetHitQuad (0.3×4), G_GetHitLocation (0.1/.800/.400/
.333/.666/0.4), and port-wide instances of the same convention.
Measure-zero boundary flips (dismember quadrant, hit-location modifier),
but `435f7d57` already ruled this class a bug when it promoted
G_BounceMissile's `> 0.7`. Method: grep oracle for unsuffixed FP literals
in comparisons against float lvalues, promote per `(x as f64) > lit`.
