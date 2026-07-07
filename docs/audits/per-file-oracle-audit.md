# Per-file oracle audit ledger

Line-level transcription audit of every `crates/mp/game/src/*.rs` file against
its oracle TU (`oracle/oracle/codemp/game/`), per the GOAL.md audit gate.
Divergence classes hunted: inverted/mistranscribed conditions, f32-vs-f64
promotion through double libm, empty-`Vec`-vs-fixed-array state init, silent
C UB needing §19 decisions. Status: `pending` / `in-review` / `audited`
(+ findings count). Updated per wave; regenerate rows, never hand-edit counts.

| File | Lines | Status |
|---|---|---|
| w_saber.rs | 11149 | audited (wave 1) — 3 confirmed + 1 type-fidelity + 2 §19 notes |
| bg_pmove.rs | 10342 | audited (wave 1) — 4 confirmed (f64-sqrt), 2 shared-convention parked |
| ai_main.rs | 8401 | audited (wave 2) — batch ai_main: 1 confirmed / 2 findings, 2 fixes |
| NPC_AI_Jedi.rs | 6988 | audited (wave 2) — batch npc_jedi: 1 confirmed / 3 findings, 5 fixes |
| anim_table.rs | 6202 | audited (wave 2) — batch data_tables: 0 confirmed / 1 findings, 1 fixes |
| w_force.rs | 6096 | audited (wave 1) — 2 confirmed + 2 FP suspects fixed + 1 dead-code |
| g_combat.rs | 6072 | audited (wave 1) — 2 confirmed + 1 §19 note |
| g_weapon.rs | 6035 | audited (wave 1) — 1 confirmed + 3 FP suspects fixed + 1 §19 note |
| g_ICARUScb.rs | 5281 | audited (wave 2) — batch icarus: 2 confirmed / 4 findings, 4 fixes |
| g_cmds.rs | 5088 | audited (wave 2) — batch g_cmds: 2 confirmed / 3 findings, 3 fixes |
| g_client.rs | 4679 | no-oracle (port-native, sanity-skimmed wave 2) |
| g_items.rs | 4197 | audited (wave 2) — batch items_utils: 7 confirmed / 8 findings, 8 fixes |
| g_active.rs | 3968 | audited (wave 1) — 1 confirmed (turndelta f64) |
| g_main.rs | 3939 | audited (wave 2) — batch g_main: 4 confirmed / 10 findings, 10 fixes |
| g_mover.rs | 3668 | audited (wave 2) — batch mover_trigger: 5 confirmed / 10 findings, 11 fixes |
| ai_wpnav.rs | 3658 | audited (wave 2) — batch nav: 1 confirmed / 2 findings, 2 fixes |
| g_misc.rs | 3624 | audited (wave 2) — batch misc_target: 4 confirmed / 7 findings, 7 fixes |
| NPC_AI_Stormtrooper.rs | 3372 | audited (wave 2) — batch npc_troopers: 13 confirmed / 13 findings, 11 fixes |
| NPC_spawn.rs | 3193 | audited (wave 2 re-run) — 5 confirmed fixed |
| NPC_combat.rs | 3104 | audited (wave 2) — batch npc_combat: 4 confirmed / 10 findings, 9 fixes |
| bg_saber.rs | 2962 | audited (wave 2) — batch bg_saber_anim: 4 confirmed / 4 findings, 4 fixes |
| g_vehicles.rs | 2802 | audited (wave 2) — batch vehicles: 6 confirmed / 9 findings, 8 fixes |
| bg_saberLoad.rs | 2743 | audited (wave 2) — batch loaders: 2 confirmed / 3 findings, 3 fixes |
| g_saga.rs | 2690 | audited (wave 2) — batch siege: 2 confirmed / 3 findings, 3 fixes |
| bg_panimate.rs | 2688 | audited (wave 2) — batch bg_saber_anim: 4 confirmed / 4 findings, 4 fixes |
| g_utils.rs | 2441 | audited (wave 2) — batch items_utils: 7 confirmed / 8 findings, 8 fixes |
| g_trigger.rs | 2394 | audited (wave 2) — batch mover_trigger: 5 confirmed / 10 findings, 11 fixes |
| g_nav.rs | 2360 | audited (wave 2) — batch nav: 1 confirmed / 2 findings, 2 fixes |
| NPC_stats.rs | 2201 | audited (wave 2 re-run) — 1 deferred (vehicle-info-shape park) |
| trap.rs | 1963 | audited (wave 2) — batch trap_infra: 0 confirmed / 0 findings, 0 fixes |
| bg_saga.rs | 1911 | audited (wave 2) — batch siege: 2 confirmed / 3 findings, 3 fixes |
| bg_misc.rs | 1907 | audited (wave 2) — batch qshared_bg: 5 confirmed / 8 findings, 8 fixes |
| npc_c.rs | 1885 | audited (wave 2 re-run) — 2 confirmed fixed |
| NPC_utils.rs | 1772 | audited (wave 2) — batch npc_combat: 4 confirmed / 10 findings, 9 fixes |
| NPC_behavior.rs | 1759 | audited (wave 2) — batch npc_behavior: 5 confirmed / 6 findings, 7 fixes |
| NPC_AI_GalakMech.rs | 1686 | audited (wave 2) — batch npc_creatures: 6 confirmed / 7 findings, 8 fixes |
| g_spawn.rs | 1605 | audited (wave 2) — batch g_main: 4 confirmed / 10 findings, 10 fixes |
| g_team.rs | 1595 | audited (wave 2) — batch team_bot: 2 confirmed / 4 findings, 17 fixes |
| q_shared.rs | 1575 | audited (wave 2) — batch qshared_bg: 5 confirmed / 8 findings, 8 fixes |
| game_cvars.rs | 1490 | audited (wave 2) — batch infra_consts: 0 confirmed / 0 findings, 0 fixes |
| NPC_AI_Rancor.rs | 1471 | audited (wave 2) — batch npc_creatures: 6 confirmed / 7 findings, 8 fixes |
| g_turret_G2.rs | 1462 | audited (wave 2) — batch turrets: 5 confirmed / 7 findings, 7 fixes |
| g_bot.rs | 1409 | audited (wave 2) — batch team_bot: 2 confirmed / 4 findings, 17 fixes |
| FighterNPC.rs | 1405 | audited (wave 2) — batch vehicles: 6 confirmed / 9 findings, 8 fixes |
| g_icarus_set_type.rs | 1330 | audited (wave 2) — batch icarus: 2 confirmed / 4 findings, 4 fixes |
| g_target.rs | 1315 | audited (wave 2) — batch misc_target: 4 confirmed / 7 findings, 7 fixes |
| q_math.rs | 1308 | audited (wave 2) — batch qshared_bg: 5 confirmed / 8 findings, 8 fixes |
| g_missile.rs | 1300 | audited (wave 2) — batch missiles: 2 confirmed / 4 findings, 7 fixes |
| NPC_senses.rs | 1251 | audited (wave 2) — batch npc_combat: 4 confirmed / 10 findings, 9 fixes |
| NPC_AI_Utils.rs | 1232 | audited (wave 2) — batch npc_droids: 4 confirmed / 8 findings, 8 fixes |
| g_navnew.rs | 1225 | audited (wave 2) — batch nav: 1 confirmed / 2 findings, 2 fixes |
| ent_fn_enums.rs | 1209 | audited (wave 2) — batch data_tables: 0 confirmed / 1 findings, 1 fixes |
| game_globals.rs | 1187 | audited (wave 2) — batch infra_consts: 0 confirmed / 0 findings, 0 fixes |
| NPC_AI_Mark1.rs | 1175 | audited (wave 2) — batch npc_droids: 4 confirmed / 8 findings, 8 fixes |
| NPC_reactions.rs | 1146 | audited (wave 2) — batch npc_behavior: 5 confirmed / 6 findings, 7 fixes |
| NPC_AI_Sniper.rs | 1138 | audited (wave 2) — batch npc_troopers: 13 confirmed / 13 findings, 11 fixes |
| bg_slidemove.rs | 1130 | audited (wave 2) — batch slide_coll: 2 confirmed / 5 findings, 6 fixes |
| bg_vehicleLoad_tables.rs | 1111 | audited (wave 2) — batch loaders: 2 confirmed / 3 findings, 3 fixes |
| ai_util.rs | 1045 | audited (wave 2) — batch ai_util: 0 confirmed / 3 findings, 4 fixes |
| g_turret.rs | 1028 | audited (wave 2) — batch turrets: 5 confirmed / 7 findings, 7 fixes |
| NPC_AI_Wampa.rs | 1025 | audited (wave 2) — batch npc_creatures: 6 confirmed / 7 findings, 8 fixes |
| bg_vehicleLoad.rs | 1019 | audited (wave 2) — batch loaders: 2 confirmed / 3 findings, 3 fixes |
| NPC_AI_Grenadier.rs | 869 | audited (wave 2) — batch npc_troopers: 13 confirmed / 13 findings, 11 fixes |
| NPC_AI_Seeker.rs | 812 | audited (wave 2) — batch npc_droids: 4 confirmed / 8 findings, 8 fixes |
| NPC_AI_ImperialProbe.rs | 744 | audited (wave 2) — batch npc_droids: 4 confirmed / 8 findings, 8 fixes |
| g_log.rs | 729 | audited (wave 2) — batch g_cmds: 2 confirmed / 3 findings, 3 fixes |
| NPC_AI_Sentry.rs | 723 | audited (wave 2) — batch npc_droids: 4 confirmed / 8 findings, 8 fixes |
| g_vehicleTurret.rs | 682 | audited (wave 2) — batch turrets: 5 confirmed / 7 findings, 7 fixes |
| NPC_AI_Default.rs | 680 | audited (wave 2) — batch npc_droids: 4 confirmed / 8 findings, 8 fixes |
| NPC_AI_Droid.rs | 657 | audited (wave 2) — batch npc_droids: 4 confirmed / 8 findings, 8 fixes |
| g_svcmds.rs | 648 | audited (wave 2) — batch g_main: 4 confirmed / 10 findings, 10 fixes |
| NPC_AI_Interrogator.rs | 623 | audited (wave 2) — batch npc_droids: 4 confirmed / 8 findings, 8 fixes |
| AnimalNPC.rs | 560 | audited (wave 2) — batch vehicles: 6 confirmed / 9 findings, 8 fixes |
| NPC_AI_Mark2.rs | 560 | audited (wave 2) — batch npc_droids: 4 confirmed / 8 findings, 8 fixes |
| c_format.rs | 557 | no-oracle (port-native, sanity-skimmed wave 2) |
| tri_coll_test.rs | 556 | audited (wave 2) — batch slide_coll: 2 confirmed / 5 findings, 6 fixes |
| NPC_move.rs | 546 | audited (wave 2) — batch npc_behavior: 5 confirmed / 6 findings, 7 fixes |
| NPC_AI_Remote.rs | 508 | audited (wave 2) — batch npc_droids: 4 confirmed / 8 findings, 8 fixes |
| bg_lib.rs | 507 | audited (wave 2) — batch qshared_bg: 5 confirmed / 8 findings, 8 fixes |
| SpeederNPC.rs | 492 | audited (wave 2) — batch vehicles: 6 confirmed / 9 findings, 8 fixes |
| prelude.rs | 458 | no-oracle (port-native, sanity-skimmed wave 2) |
| WalkerNPC.rs | 436 | audited (wave 2) — batch vehicles: 6 confirmed / 9 findings, 8 fixes |
| NPC_AI_MineMonster.rs | 392 | audited (wave 2) — batch npc_creatures: 6 confirmed / 7 findings, 8 fixes |
| g_init_game.rs | 375 | audited (wave 2) — batch trap_infra: 0 confirmed / 0 findings, 0 fixes |
| g_timer.rs | 374 | audited (wave 2) — batch missiles: 2 confirmed / 4 findings, 7 fixes |
| g_session.rs | 366 | audited (wave 2) — batch g_client: 6 confirmed / 9 findings, 11 fixes |
| NPC_AI_Atst.rs | 357 | audited (wave 2) — batch npc_droids: 4 confirmed / 8 findings, 8 fixes |
| g_object.rs | 345 | audited (wave 2) — batch misc_target: 4 confirmed / 7 findings, 7 fixes |
| NPC_AI_Howler.rs | 325 | audited (wave 2) — batch npc_creatures: 6 confirmed / 7 findings, 8 fixes |
| g_strap.rs | 316 | audited (wave 2) — batch missiles: 2 confirmed / 4 findings, 7 fixes |
| g_exphysics.rs | 284 | audited (wave 2) — batch slide_coll: 2 confirmed / 5 findings, 6 fixes |
| veh_dispatch.rs | 215 | no-oracle (port-native, sanity-skimmed wave 2) |
| NPC_goal.rs | 206 | audited (wave 2) — batch npc_behavior: 5 confirmed / 6 findings, 7 fixes |
| bg_g2_utils.rs | 195 | pending |
| ai_main_consts.rs | 194 | audited (wave 2) — batch ai_util: 0 confirmed / 3 findings, 4 fixes |
| g_arenas.rs | 191 | audited (wave 2) — batch team_bot: 2 confirmed / 4 findings, 17 fixes |
| lib.rs | 177 | no-oracle (port-native, sanity-skimmed wave 2) |
| g_shutdown_game.rs | 146 | audited (wave 2) — batch trap_infra: 0 confirmed / 0 findings, 0 fixes |
| NPC_sounds.rs | 115 | audited (wave 2) — batch npc_behavior: 5 confirmed / 6 findings, 7 fixes |
| NPC_misc.rs | 104 | audited (wave 2) — batch npc_behavior: 5 confirmed / 6 findings, 7 fixes |
| g_nav_consts.rs | 96 | audited (wave 2) — batch nav: 1 confirmed / 2 findings, 2 fixes |
| g_mem.rs | 77 | audited (wave 2) — batch trap_infra: 0 confirmed / 0 findings, 0 fixes |
| q_shared_cvar_flags.rs | 67 | audited (wave 2) — batch infra_consts: 0 confirmed / 0 findings, 0 fixes |
| cstr_util.rs | 55 | no-oracle (port-native, sanity-skimmed wave 2) |
| g_local_consts.rs | 51 | audited (wave 2) — batch infra_consts: 0 confirmed / 0 findings, 0 fixes |
| g_public_consts.rs | 40 | audited (wave 2) — batch infra_consts: 0 confirmed / 0 findings, 0 fixes |
| ent_id.rs | 29 | no-oracle (port-native, sanity-skimmed wave 2) |

## Wave 1 findings log (2026-07-06)

Six files, ~380 oracle functions compared line-level, **zero oracle functions
missing from the port**. Fix commits land together at the wave boundary.
Refuted candidates are recorded so later waves don't re-litigate them.

### bg_pmove.rs (~100/100 functions; none skimmed)
- **Fixed, class 2 (f32 where C promotes through double)** — inlined
  `VectorLength`/`sqrt` transcribed as f32 `.sqrt()`:
  `PM_Friction` speed (rs:~850 / c:1012); `PM_CmdScale` total **and** the
  `127.0`-literal scale division (rs:~1948 / c:1217-1219); `PM_Footsteps`
  xyspeed (rs:~5739 / c:5200); `BG_IK_MoveArm` distToDest (rs:~8587 / c:8662).
- **Parked (shared with faithful port — fixing here alone would diverge from
  the differential baseline):** `PM_CheckJump` flip-anim `fabs*1.5` compare
  (rs:~2793 / c:1988); `PM_SetVehicleAngles` water-pitch outer add (rs:~398 /
  c:526). Fix only together with `oracle/` faithful port, if ever.
- Verified-clean quirks preserved: `mins[0]`-twice duck bug, always-false NPC
  `VectorCompare`, uninitialized-clamp vehicle view path.
- Lead: `vectoyaw`/`vectoangles` atan2 width (checked in the wave-1 fix round).

### w_saber.rs (72/72 functions; none skimmed)
- **Fixed, class 4 (CRITICAL):** saberlock trail-copy indexed
  `saber[saberNum]` (saber *entity* number) — silent OOB write in C
  (c:8762-8764), guaranteed panic on **every saberlock** in Rust. §19 →
  `saber[rSaberNum]` per loop bound (rs:10245-10259).
- **Fixed, class 2:** `CheckSaberDamage` broken-parry threshold
  `Q_irand(1,10) >= otherSaberLevel*1.5` compared in double in C; port
  truncated RHS to int, firing broken parries too often (rs:~5511 / c:4984).
- **Fixed, type fidelity:** `totalDmg` was `[c_int; MAX_SABER_VICTIMS]`, C is
  `static float` (c:3506) — now f32 with call-site truncation
  (game_globals.rs:1180).
- **§19 notes added (behavior already correct):** `SetSaberBoxSize`
  leftover-`j` OOB read in C → defined 0,0 read (rs:~577 / c:465);
  `WP_SaberPositionUpdate` uninitialized `clientUnlinked` in C → zero-init
  (rs:~10499 / c:8906).
- Follow-up: a second independent pass over `CheckSaberDamage` (~1,400 C
  lines) and `WP_SaberPositionUpdate` (~1,000) is the highest-value re-audit.

### w_force.rs (52/52) / g_active.rs (36/36; none skimmed)
- **Fixed, class 2/7 (HIGH IMPACT):** `ForceThrow` `pushPowerMod` is `int` in
  C with per-step double-compute-then-truncate; port kept it f32, so every
  force push/pull velocity diverged (rs:~3767+ / c:3070, 3508-3702).
- **Fixed, class 3:** `bgSiegeClasses` was `Vec::new()` — per-frame panic
  under GT_SIEGE; pre-sized zeroed to MAX_SIEGE_CLASSES (bg_state.rs), reads
  zeroed data exactly like C's unpopulated static.
- **Fixed, class 2:** `ClientThink_real` NPC `turndelta` — C's `fabs` promotes
  the expression to f64 (g_active rs:~2469 / c:2369); `SeekerDroneUpdate`
  angle — C narrows to float then re-widens for trig (rs:~5068+ / c:4715+);
  force-regen debounce — double literals with an inner f32 division
  (rs:~5949+ / c:5630+).
- **Fixed, dead-code fidelity:** `WP_InitForcePowers` NPC-temp branch now
  mirrors C's userinfo write + `Info_ValueForKey` extract (unreachable in
  both trees).
- Raven bugs verified preserved (do NOT fix): `WP_GetVelocityForForceJump`
  second `VectorMA` overwrites the forward push; `WP_InitForcePowers` final
  `forcePowerBaseLevel` loop never executes.

### g_combat.rs (47/47) / g_weapon.rs (85/85; none skimmed)
- **Fixed, class 7:** `WP_FireConcussionAlt` dropped C's
  `if (tr.fraction >= 1.0f) break;` — a miss processed
  `g_entities[ENTITYNUM_NONE]` and kept tracing (g_weapon rs:~3700 /
  c:3068-3072).
- **Fixed, classes 3+4:** `modNames` sized 42 vs C's `[MOD_MAX]`=45 — MODs
  42-44 (incl. common MOD_TRIGGER_HURT) panicked instead of C's in-bounds
  NULL → `%s` "(null)" (§19) (g_combat rs:~64, ~2281 / c:755, 2527). The
  39-41 name misalignment (MOD_COLLISION logs as "MOD_SUICIDE") is Raven's
  own bug, preserved.
- **Fixed, class 2:** `G_RadiusDamage` falloff `damage*(1.0 - dist/radius)`
  double-literal promotion (g_combat rs:~5990 / c:5892); DEMP2 alt
  `count*0.6` / `count*0.8` factors; `WP_LobFire` `travelTime*0.5*gravity`
  (g_weapon rs:~1423, ~1617, ~2514).
- **§19 notes added:** `DetPackBlow` uninitialized dir → zeroed (g_weapon
  rs:~3299 / c:2742-2751); `G_GetHitLocation` uninitialized `tangles` →
  zeroed (g_combat rs:~192 / c:50).
- Cleanup: stale WP_MuzzlePoint PORT-NOTE (g_weapon rs:~4211).
- Latent, parked (from slice round): `G_GetHitLocation` threshold compares
  f32-vs-f32 where C promotes to f64 — ~1e-8 window, no fixture hits it.

## Wave 2 findings log (2026-07-06, workflow run wf_c53e0d39-bb0)

All 101 remaining files audited in one 57-agent workflow (28 of 29 batches
complete; the npc_spawn batch — NPC_spawn.rs, NPC_stats.rs, npc_c.rs — died on
an API error and is re-run separately). **153 findings (95 CONFIRMED), 174
fixes applied, 3 rejected on verification, 3 cross-file fixes applied by the
serial finisher.** Full per-finding detail (quotes, cites, impact):
`docs/audits/audit-findings-wave2-2026-07-06.json`. Workspace green after:
54/54 tests, verified in-workflow and independently.

Headline confirmed findings (see JSON for the rest):
- `g_items.rs` JETPACK_TOGGLE_TIME was 500 vs Raven's 1000 (double-rate
  jetpack toggling).
- `g_cmds.rs` Cmd_CallTeamVote_f numeric-vs-name client-id parse logic
  mistranscribed.
- `NPC_AI_Jedi.rs` health thresholds `< maxHealth*0.75f` truncated to int
  before compare — flips rage/heal triggers at exact boundaries (3 sites).
- `vehicles`: FighterNPC.c's static per-class `Update` (c:188) is MISSING —
  G_SetFighterVehicleFunctions never wires it, so BG_FighterUpdate is
  unreachable in the live update path. **Port follow-up required.**
- `bg_panimate.c` BG_ParseAnimationEvtFile + 2 static helpers are not ported
  anywhere in crates/ (animevents parser). **Port follow-up required.**
- `NPC_ShotEntity` out-param was `[f32;3]` by-value where C writes through a
  nullable pointer — signature now `Option<&mut vec3_t>`, 6 call sites across
  5 files (finisher).
- `bg_lib.rs` `_atoi` and `q_math.rs` `VectorNormalizeFast` absent but dead in
  game-module scope (documented, not ported).
- `bg_misc.rs` `BG_GiveMeVectorFromMatrix` lives in NPC_AI_Mark2.rs — file
  placement inconsistency; transcription unverified there.

Skimmed/parked items worth a follow-up pass (auditors flagged honestly; full
list in the JSON): ai_wpnav.rs bulk (16 functions read Rust-side only),
g_vehicles.rs (~42 functions lightly compared), g_main.rs G_RunFrame middle
section, g_client.rs SetupGameGhoul2Model/G_UpdateClientAnims, bg_misc.rs
table-heavy functions, NPC_AI_Default.rs NPC_BSDefault.

### npc_spawn batch re-run (2026-07-06, after workflow agent API failure)
7 confirmed fixed + 1 consistency note: npc_c.rs groundEntityNum read from
entityState instead of playerState (NPC.c:59); npc_c.rs f64 fabs chain
(NPC.c:455); NPC_WeaponsForTeam strncmp was case-insensitive vs Q_strncmp
case-sensitive (q_shared.c:881); NPC_Kill_f wrote stats[4]=STAT_WEAPONS
instead of STAT_HEALTH=0 (NPC_spawn.c:4154) — `npc kill` zeroed the weapons
bitfield; NPC_PrintScore read persistant[12] instead of PERS_SCORE=0
(NPC_spawn.c:4174); two loop bounds 2048 vs ENTITYNUM_MAX_NORMAL/WORLD.
Deferred: NPC_ParseParms standheight int-truncation chain — parked behind the
existing PORT-NOTE(vehicle-info-shape) missing-symbol gate (fix belongs with
m_pVehicle resolution). Reported gaps: PORT-NOTE(fn-ptr) player->die calls in
NPC_Kill_f still commented out (NPCs not killed via that path). Coverage:
npc_c 43/43, NPC_stats 8/8 live (NPC_ParseParms end-to-end), NPC_spawn all
substantive fns (the ~55 trivial SP_NPC_* setters spot-verified).

### Audit follow-up round (2026-07-06, post-merge): all five findings resolved
- **FighterNPC per-class Update — PORTED** (FighterNPC.rs:194-227 + veh_dispatch.rs
  VH_FIGHTER arm; oracle FighterNPC.c:188-209/1965). BG_FighterUpdate now
  reachable; its traceFunc retyped to the port's ctx-carrying G_VehicleTrace
  shape, contentmask placeholder fixed to MASK_NPCSOLID & !CONTENTS_BODY, and
  the silent Ghost no-op loop wired to veh_dispatch::ghost. Remaining known
  gap: AnimateVehicle's BG_SetAnim thread (pre-existing documented PORT-NOTE).
- **BG_ParseAnimationEvtFile — FALSE POSITIVE, §20 drop documented.** All three
  functions live inside `#ifndef QAGAME` (bg_panimate.c:1756-2328); callers and
  bgAllEvents reads are CGAME-only. Module-doc note added to bg_panimate.rs;
  BgState::bgAllEvents stays as a faithful-but-inert mirror.
- **NPC_Kill_f player->die — WIRED** (stale PORT-NOTE(fn-ptr); the
  ent_fn_enums::dispatch_die mechanism already existed). Three call sites match
  oracle NPC_spawn.c:4122-4159; `npc kill` now actually kills NPCs.
- **NPC_ParseParms vehicle height — PARK STALE, RESOLVED** (NPC_stats.rs:1460-92).
  Full VH_FIGHTER gate implemented (was missing the vehicleInfo checks — a real
  mis-branch for non-fighter vehicles) + the deferred standheight chained-assign
  int truncation (maxs[2] == floor(n/2), oracle NPC_stats.c:1972-1980).
- **BG_GiveMeVectorFromMatrix — transcription CLEAN** (all matrix indices verified
  vs bg_misc.c:736-776); relocated to bg_misc.rs:1917 (its oracle home), prelude
  re-export repointed, 3 explicit imports + 10 inline-path call sites cleaned.

### atof/sscanf parity round (2026-07-06): resolved + atoi class parked
- **atof — RESOLVED.** Oracle dylib links Raven's own `atof` (nm: `_atof` T);
  Rust `bg_lib::atof`/`_atof` verified faithful (f32 accumulator, f64 fraction
  round-back, `_atof` leading-dot bug). All 53 live oracle atof sites route
  through it — the one bypass, `G_SpawnFloat`'s exponent-accepting
  `parse::<f64>` helper ("1e2" → 100.0 vs oracle 1.0), fixed (commit 13910be).
- **sscanf %f — RESOLVED.** Oracle links libc sscanf (nm: `_sscanf` U). All 12
  Rust `%f` sites had a shift-on-skip `filter_map` idiom; replaced by the shared
  libc-faithful `cstr_util::sscanf_f32s` (longest-prefix, stop-at-first-failure,
  count-returning, unmatched slots untouched per §19) + 5 unit tests.
- **PARKED: atoi class.** Oracle links libc `atoi` (155 sites in codemp/game).
  Many Rust sites use `.trim().parse().unwrap_or(0)` — diverges on trailing
  garbage (libc atoi("12abc")=12, parse→0). Confirmed reachable-from-client
  examples: g_cmds.rs:447,500,791,2612,2621; w_force.rs:351-368
  (forceRank/forceSide); g_client.rs:2854-2866 (customRGBA userinfo).
  Fix caveat: parity target is LIBC atoi, not `bg_lib::atoi` (Raven's is
  Q3_VM-only there) — they differ on whitespace class (isspace vs `<= ' '`
  signed-char) and overflow (strtol clamp vs wrap); `g_spawn.rs`'s local
  `c_str_to_i32` is close but misses `\x0b`. Needs a census (type-inferred
  `.parse()` undercounts) + one shared libc-atoi helper before a swap round.

### atoi parity round (2026-07-06): RESOLVED (was parked above)
- 3-shard census of all 154 live oracle atoi sites → commit 2b3f8263.
  `cstr_util::atoi`/`atoi_str` (libc `(int)strtol` semantics: isspace class,
  digit-prefix, clamp-in-i64-then-truncate overflow) is now the one game-logic
  atoi; the prelude `atoi` re-export points at it (bg_lib::atoi stays as the
  faithful Q3_VM-only port, doc-noted). 28 `.parse().unwrap_or` sites swapped
  — four porter-invented defaults removed (-1 in Cmd_Tell_f/clientkick/
  ClientForString, 999 in give-ammo's arg path; oracle is plain atoi → 0);
  5 local reimplementations deleted (g_spawn c_str_to_i32, q_shared +
  bg_saberLoad c_atoi twins, ai_util c_atoi/c_atoi_ptr); ~20 Unicode-trimming
  `.trim()` pre-passes removed (libc atoi does not skip U+00A0 etc.).
- Provably-OK sites annotated in place: g_svcmds.rs StringToFilter
  (digit-only extraction), w_force.rs + bg_misc.rs single-char to_digit.
- NOT in scope, still open: the 6 MISSING `#ifndef FINAL_BUILD` debug commands
  (debugSaberSwitch/debugIK*/debugShipDamage, g_cmds.c:3801-4066) — compiled
  into the oracle dylib (build.sh defines neither _DEBUG nor FINAL_BUILD), so
  they are live ground truth; tracked by PORT-NOTE(debug-build-gated-cmds) at
  g_cmds.rs:~4972. A referee scenario issuing those commands would diverge.

### Linux referee lane bring-up (2026-07-07): NDEBUG ruling + arm64 portability note
- Oracle build regime change (user-ruled): `-DNDEBUG` added to
  tools/referee-oracle/build.sh — retail jampgame was an MSVC Release build
  (asserts compiled away), so asserts-active was a non-retail oracle behavior;
  also unblocks glibc's C++ assert() under gnu++98. macOS baseline verified
  unchanged (66/0/6 + 3 scenarios byte-identical after rebuild).
- PARKED (arch portability, not reachable from current targets): on
  aarch64-Linux `c_char = u8`, exposing 29 E0308s in mp_game from hardcoded
  `*mut i8` buffers (e.g. g_spawn.rs:1101 com_token). Harmless on
  macOS/x86_64-Linux/Windows (c_char = i8). Fix class: declare seam buffers
  as c_char, not i8. Revisit only if arm64-Linux becomes a target.

### FINDING (2026-07-07, Linux CI lane first run): x86_64 ULP divergence in pmove velocity
- Lane infrastructure fully green: recursive submodule checkout, oracle .so
  builds (89 TUs, g++-13/14), SELFTEST passes (Linux oracle self-deterministic),
  idle scenario 100 frames byte-identical.
- OPEN: solo diverges frame 11 (velocity[0], 2 ULP: 0xc2a9d88d vs 0xc2a9d88f);
  melee-brawl frame 45 (velocity[2], 1 ULP: 0x424b7230 vs 0x424b722f). Same
  1-ULP FP class as the frametime fix (3ad83173) but Linux-x86_64-only — both
  scenarios pass byte-identical on macOS arm64. Since IEEE add/mul/div/sqrt are
  arch-deterministic and both sides share the host libm, prime suspects are a
  libm call-pattern mismatch (e.g. sinf vs (float)sin — Apple libm may mask it,
  glibc not) or a gcc-vs-LLVM excess-precision/codegen edge in an air-path
  pmove expression (clients are AIRBORNE in the mock). Run: gh run 28835337105.

### Referee CI lane removed (2026-07-07, user ruling)
- The Linux x86_64 lane's 1-2 ULP pmove-velocity divergences (entry above) are
  the OpenJK-#589 class: compiler FP-evaluation differences (gcc vs LLVM) in
  the 2003 C, not a same-host parity failure — the FP-regime ruling's parity
  target is same-host strict-IEEE, satisfied and byte-identical on the macOS
  dev platform. Lane removed from Actions rather than chased; the referee
  remains the local gate (cargo test -p jampgame --test referee -- --ignored).
- Kept from the bring-up (all still useful): build.sh Darwin/Linux portability,
  -fsigned-char + -DNDEBUG regime pins, referee.rs cfg-branched oracle path,
  the mheh/jediacademy fork submodule chain, skeleton-push CI trigger.
