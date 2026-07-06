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
| ai_main.rs | 8401 | pending |
| NPC_AI_Jedi.rs | 6988 | pending |
| anim_table.rs | 6202 | pending |
| w_force.rs | 6096 | audited (wave 1) — 2 confirmed + 2 FP suspects fixed + 1 dead-code |
| g_combat.rs | 6072 | audited (wave 1) — 2 confirmed + 1 §19 note |
| g_weapon.rs | 6035 | audited (wave 1) — 1 confirmed + 3 FP suspects fixed + 1 §19 note |
| g_ICARUScb.rs | 5281 | pending |
| g_cmds.rs | 5088 | pending |
| g_client.rs | 4679 | pending |
| g_items.rs | 4197 | pending |
| g_active.rs | 3968 | audited (wave 1) — 1 confirmed (turndelta f64) |
| g_main.rs | 3939 | pending |
| g_mover.rs | 3668 | pending |
| ai_wpnav.rs | 3658 | pending |
| g_misc.rs | 3624 | pending |
| NPC_AI_Stormtrooper.rs | 3372 | pending |
| NPC_spawn.rs | 3193 | pending |
| NPC_combat.rs | 3104 | pending |
| bg_saber.rs | 2962 | pending |
| g_vehicles.rs | 2802 | pending |
| bg_saberLoad.rs | 2743 | pending |
| g_saga.rs | 2690 | pending |
| bg_panimate.rs | 2688 | pending |
| g_utils.rs | 2441 | pending |
| g_trigger.rs | 2394 | pending |
| g_nav.rs | 2360 | pending |
| NPC_stats.rs | 2201 | pending |
| trap.rs | 1963 | pending |
| bg_saga.rs | 1911 | pending |
| bg_misc.rs | 1907 | pending |
| npc_c.rs | 1885 | pending |
| NPC_utils.rs | 1772 | pending |
| NPC_behavior.rs | 1759 | pending |
| NPC_AI_GalakMech.rs | 1686 | pending |
| g_spawn.rs | 1605 | pending |
| g_team.rs | 1595 | pending |
| q_shared.rs | 1575 | pending |
| game_cvars.rs | 1490 | pending |
| NPC_AI_Rancor.rs | 1471 | pending |
| g_turret_G2.rs | 1462 | pending |
| g_bot.rs | 1409 | pending |
| FighterNPC.rs | 1405 | pending |
| g_icarus_set_type.rs | 1330 | pending |
| g_target.rs | 1315 | pending |
| q_math.rs | 1308 | pending |
| g_missile.rs | 1300 | pending |
| NPC_senses.rs | 1251 | pending |
| NPC_AI_Utils.rs | 1232 | pending |
| g_navnew.rs | 1225 | pending |
| ent_fn_enums.rs | 1209 | pending |
| game_globals.rs | 1187 | pending |
| NPC_AI_Mark1.rs | 1175 | pending |
| NPC_reactions.rs | 1146 | pending |
| NPC_AI_Sniper.rs | 1138 | pending |
| bg_slidemove.rs | 1130 | pending |
| bg_vehicleLoad_tables.rs | 1111 | pending |
| ai_util.rs | 1045 | pending |
| g_turret.rs | 1028 | pending |
| NPC_AI_Wampa.rs | 1025 | pending |
| bg_vehicleLoad.rs | 1019 | pending |
| NPC_AI_Grenadier.rs | 869 | pending |
| NPC_AI_Seeker.rs | 812 | pending |
| NPC_AI_ImperialProbe.rs | 744 | pending |
| g_log.rs | 729 | pending |
| NPC_AI_Sentry.rs | 723 | pending |
| g_vehicleTurret.rs | 682 | pending |
| NPC_AI_Default.rs | 680 | pending |
| NPC_AI_Droid.rs | 657 | pending |
| g_svcmds.rs | 648 | pending |
| NPC_AI_Interrogator.rs | 623 | pending |
| AnimalNPC.rs | 560 | pending |
| NPC_AI_Mark2.rs | 560 | pending |
| c_format.rs | 557 | pending |
| tri_coll_test.rs | 556 | pending |
| NPC_move.rs | 546 | pending |
| NPC_AI_Remote.rs | 508 | pending |
| bg_lib.rs | 507 | pending |
| SpeederNPC.rs | 492 | pending |
| prelude.rs | 458 | pending |
| WalkerNPC.rs | 436 | pending |
| NPC_AI_MineMonster.rs | 392 | pending |
| g_init_game.rs | 375 | pending |
| g_timer.rs | 374 | pending |
| g_session.rs | 366 | pending |
| NPC_AI_Atst.rs | 357 | pending |
| g_object.rs | 345 | pending |
| NPC_AI_Howler.rs | 325 | pending |
| g_strap.rs | 316 | pending |
| g_exphysics.rs | 284 | pending |
| veh_dispatch.rs | 215 | pending |
| NPC_goal.rs | 206 | pending |
| bg_g2_utils.rs | 195 | pending |
| ai_main_consts.rs | 194 | pending |
| g_arenas.rs | 191 | pending |
| lib.rs | 177 | pending |
| g_shutdown_game.rs | 146 | pending |
| NPC_sounds.rs | 115 | pending |
| NPC_misc.rs | 104 | pending |
| g_nav_consts.rs | 96 | pending |
| g_mem.rs | 77 | pending |
| q_shared_cvar_flags.rs | 67 | pending |
| cstr_util.rs | 55 | pending |
| g_local_consts.rs | 51 | pending |
| g_public_consts.rs | 40 | pending |
| ent_id.rs | 29 | pending |

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
