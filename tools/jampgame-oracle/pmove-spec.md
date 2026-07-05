SCOUT REPORT — pmove single-step differential slice

## 1. Entry + globals

**Oracle** `Pmove` (bg_pmove.c:11167-11215): early-return if `finalTime < ps->commandTime`; clamp `commandTime` to `finalTime-1000`; `fallingToDeath` zeroes cmd; `pmove_framecount = (n+1)&((1<<PS_PMOVEFRAMECOUNTBITS)-1)`; chop loop `msec ≤ 66` (or `≤ pmove->pmove_msec` when `pmove->pmove_fixed` — both are **pmove_t fields** the game caller fills from cvars; bg never reads cvars); after each `PmoveSingle`, `PMF_JUMP_HELD → cmd.upmove=20`. File statics: `pm`, `pml` (memset at :10533 each step), `pm_entSelf`/`pm_entVeh` (`PM_BGEntForNum` = `baseEnt + entSize*num` overlay), `pm_flying`, `gPMDoSlowFall`, `pm_cancelOutZoom`, `c_pmove` (debug only).

**Port**: `bg_pmove.rs:10229` — `pub fn Pmove(pmove: *mut pmove_t, bg: &mut BgState, traps: &dyn BgTraps, callbacks: &mut dyn GameCallbacks)`; builds one `PmoveContext` (`bg_channel/pmove_context.rs`) per Pmove call; `PmoveSingle` is a `PmoveContext` method (`self.pm = pmove` at :1012). `c_pmove` and the RNG live in `BgState` (`bg_channel/bg_state.rs`, `BgState::new()` at :166). **A test constructs**: `pmove_t` + `playerState_t` (repr-C, zeroed + pins), a zeroed `bgEntity_t` arena for `baseEnt`/`entSize`, `BgState::new()` + anim load, a test `BgTraps`, a test `GameCallbacks`. No todo!() stubs remain in bg_pmove/bg_slidemove/bg_panimate/bg_misc bodies on the basic path.

## 2. Trap surface (basic on-foot step)

Per `PmoveSingle`, pipeline order (stand/walk/jump/fall/land):

| Call site | Trap | Count/step |
|---|---|---|
| PM_SetWaterLevel (:4285, runs 2×/step) | `pointcontents` | 2 (dry) |
| PM_CheckDuck (:4410) | `trace` | 0–1 (only un-duck/un-roll) |
| PM_GroundTrace (:4141, runs 2×/step) + PM_CorrectAllSolid/PM_GroundTraceMissed | `trace` | 2–4 (allsolid jitter: up to 28) |
| PM_WalkMove/PM_AirMove → PM_SlideMove/PM_StepSlideMove (bg_slidemove.c:633/861) | `trace` | 1–10 |
| PM_CrashLand → PM_TryRoll (:3596) | `trace` | 0 (needs melee/saber + FP_LEVITATION usable — pin FP=0) |
| end of PmoveSingle (:11128) | `trap_SnapVector(ps->velocity)` → `BgTraps::snap_vector` (bg_pmove.rs:1833) | 1 |

Port: **BgTraps** = `trace`, `pointcontents`, `snap_vector` (+ `fs_*` once at setup for animation.cfg). **GameCallbacks** actually reachable: `entity_legs_anim`/`entity_torso_anim` (BG_Start{Legs,Torso}Anim restart check, bg_panimate.c:2610/2677 — reads `g_entities[n].s.legsAnim`); `get_time`+`client_check_impact_bbrush` fire only when a slide trace hits `entityNum < ENTITYNUM_WORLD` (PM_ClientImpact early-returns on world hits — a world-only trace stub never reaches them). Everything else (damage, effects, vehicle Update/Board/AttachRiders, cheap_weapon_fire) is vehicle/entity-gated: unreachable. Events (EV_JUMP, EV_FALL/EV_ROLL/EV_FOOTSTEP from PM_CrashLand, EV_CHANGE_WEAPON) are **not traps** — `BG_AddPredictableEventToPlayerstate` writes `ps->events[]` directly; MP `PM_Footsteps` emits no per-cycle footstep events (only FOOTSPLASH/SWIM in water).

**Branch avoidance pins**: `weapon = cmd.weapon = WP_MELEE` (PM_WeaponLightsaber only fires on `ps->weapon==WP_SABER`, :6991); `m_iVehicleNum=0`, `clientNum=0`, arena entity zeroed (`NPC_class=CLASS_NONE`, `eType=ET_GENERAL`) kills all vehicle/NPC paths; `fd.forcePowerLevel[*]=0, forcePowersKnown=0` → plain 270-unit jump (:2754) and no force-land/roll traces; `saberMove=0, saberLockTime=0, emplacedIndex=0, zoomMode=0, heldByClient=0, legsAnim/torsoAnim=0`; `pm_type=PM_NORMAL, weaponstate=WEAPON_READY, stats[STAT_HEALTH]=100, gravity=800, basespeed=250` (BG_AdjustClientSpeed :8331 resets `ps->speed` from basespeed each step), `crouchheight/standheight/viewheight` set (CheckDuck derives mins/maxs from them), `tracemask=MASK_PLAYERSOLID`, `pmove_t.trace/pointcontents` fn-ptrs can stay None in the Rust test (ported bodies call `self.traps.*`; the fn-ptr is only passed through on vehicle paths). No G2 calls exist anywhere in the on-foot path.

## 3. World model / trace stub

Floor = huge AABB brush (top at z=0) + axis-aligned box brushes. **Spec: transcribe Q3's `CM_ClipBoxToBrush` restricted to axial brushes** — this reproduces the semantics pmove depends on for free:
- per plane: `ofs[i] = normal[i]<0 ? maxs[i] : mins[i]`; `dist = plane.dist - dot(ofs,normal)`; `d1=dot(start,n)-dist`, `d2=dot(end,n)-dist`; startout/getout bookkeeping; `enterFrac=(d1-0.125f)/(d1-d2)`, `leaveFrac=(d1+0.125f)/(d1-d2)`; keep max enterFrac plane; clamp fraction `<0→0`.
- outputs pmove consumes: `allsolid`, `startsolid` (PM_CorrectAllSolid), `fraction`, `endpos[i]=start[i]+fraction*(end[i]-start[i])`, `plane.normal` (axial, exact), `surfaceFlags` (per-brush const; MATERIAL_MASK feeds EV_FALL/EV_FOOTSTEP eventParm, SURF_NODAMAGE/NOSTEPS honored), `contents=CONTENTS_SOLID`, `entityNum = fraction<1 ? ENTITYNUM_WORLD : ENTITYNUM_NONE`. `pointcontents` = inside-any-brush → CONTENTS_SOLID else 0.
- **Bit-identity**: only f32 `+ - * /` and compares — IEEE-deterministic both sides given the harness's existing `-ffp-contract=off`. Two pinned rules in the spec: (a) every C literal carries the `f` suffix (a bare `0.125` promotes the whole expression to double; Rust f32 would diverge); (b) no libm/fabs/macros — axial normals need no VectorNormalize/sqrt. `snap_vector`: pin as `v[i]=rintf(v[i])` ↔ `f32::round_ties_even` (note: may differ from real jamp engine's SnapVector — record for the live-engine seam later). Recommend a standalone raw-trace golden (a few sweeps) to prove the stub before layering pmove on it.

## 4. State dump

Fields that change in basic movement: `origin, velocity, viewangles` (bit-hex), `delta_angles[3]`, `commandTime`, `groundEntityNum`, `pm_flags`, `pm_time`, `legsAnim/legsTimer/torsoAnim/torsoTimer`, `legsFlip/torsoFlip` (BG_FlipPart), `bobCycle`, `viewheight`, `eFlags`, `eventSequence + events[2] + eventParms[2]` (MAX_PS_EVENTS=2 — dump both slots so wraps are visible), `weaponTime/weaponstate`, `speed`, `inAirAnim`, `fallingToDeath`, `fd.forceJumpZStart`, plus pm outs: `waterlevel/watertype`, `numtouch`, `mins[2]/maxs[2]`, `xyspeed`. Format (one line per step, harness bit-hex convention):
`s=N t=%d org=%08x,%08x,%08x vel=... va=... da=%d,%d,%d gnd=%d pmf=%x pmt=%d la=%d:%d ta=%d:%d fl=%d%d bob=%d vh=%d ef=%x seq=%d ev=%d:%d,%d:%d wt=%d ws=%d spd=%08x wl=%d ntr=%d rng=%08x`
(`ntr`=trace-call count, `rng`=holdrand — tripwires). Cadence: after every **Pmove** call with fixture dt ≤ 66 so 1 Pmove == 1 PmoveSingle; one scenario uses dt=200 to exercise the chop loop.

## 5. Fixture format + scenarios

Text file both dumpers parse: `brush x0 y0 z0 x1 y1 z1 surf=<hex>` lines; `ps key=value` overrides over a spec'd zero+pins baseline (floats as bit-hex where fractional); `cmd dt fwd right up buttons yaw pitch roll xN` rows with **cmd.angles as raw i16** (no ANGLE2SHORT float math in the parser). Scenarios:
1. **idle** — spawn 0.25 above floor, 20×50ms empty cmds (ground snap, stand anim, weaponTime drain)
2. **walk-fwd** — fwd=127 ×40, then BUTTON_WALKING rows (bobCycle, PMF_BACKWARDS_RUN off)
3. **strafe-turn** — fwd=64 right=64, yaw += ~800 shorts/cmd (pml.forward/right projection, PM_Friction)
4. **jump-land** — idle 5, upmove=127 ×2, release, ~25 steps (EV_JUMP, PMF_JUMP_HELD + Pmove's upmove=20 refeed, inAirAnim, EV_FALL/EV_FOOTSTEP param)
5. **fall-onto-box** — spawn z=300 over a box, fall, land (CrashLand quadratic, legsTimer=TIMER_LAND, bobCycle=0, velocity[2]=0)
6. **wall-step** — walk into 16-high box (PM_StepSlideMove step-up) then 128-high wall (clip-plane slide, corner crease), one dt=200 chop row

## 6. Risks → containment

| Risk | Containment |
|---|---|
| PM_SetAnim/Continue*Anim need `pm->animations` + BgState tables; no animation.cfg in repo | Commit a **synthetic animation.cfg fixture**; both sides load via their own BG_ParseAnimationFile through stubbed FS (dumper fopen redirect; Rust test BgTraps::fs_*). Zeroed tables are a deterministic fallback but degenerate — prefer parsed. |
| bg_panimate QAGAME restart-check reads `g_entities[n].s.legsAnim/torsoAnim` | Spec identical mirroring both sides: after each Pmove copy `ps.legsAnim/torsoAnim` into the stub entity (what BG_PlayerStateToEntityState does live); Rust `entity_legs/torso_anim` returns the mirror. |
| C dumper link closure (-DQAGAME to match jampgame flavor): needs bg_pmove, bg_slidemove, bg_panimate, bg_saber, bg_saberLoad, bg_misc, bg_weapons, bg_vehicleLoad, q_shared, q_math + externs `g_entities`, `level`, `Client_CheckImpactBBrush`, `G_CheapWeaponFire`, `G_PlayEffect(ID)`, `G_FlyVehicleSurfaceDestruction`, `G_CanBeEnemy`, `G_DamageFromKiller`, `FighterIsLanded`, trap_* | Zeroed `g_entities`/`level` definitions; every G_/trap stub `abort()`s (a hit = fixture leaked out of the basic path). Rust GameCallbacks likewise panics on all but the two anim reads. Reuse harness flags (`-D__linux__ -ffp-contract=off shim.h`). |
| RNG draws mid-pmove | Only PM_HoverTrace/jetpack (Q_irand :759/:837/:10755) — outside fixtures. Seed both sides identically; dump holdrand per step as tripwire. |
| Double-promotion parity in **ported** code (PM_CrashLand `sqrt(den)`, `delta*delta*0.0001`, xyspeed `sqrt`) | That's the point of the diff — expect first mismatches here; they're port bugs, not harness bugs. Keep the trace stub itself promotion-free so it's never the suspect. |
| Event ring wrap (2 slots) | Dump seq + both slots every step. |
| SnapVector spec vs real engine | Pinned rintf both sides; note in README for the engine-seam revisit. |

## Build split — recommend 2 agents after the spec is frozen

The single coupling point is the **spec doc** (trace-stub pseudocode with exact f32 literals, fixture grammar, baseline ps pins, dump line format, mirror rule). Parent writes/freezes that from this report first; then:
- **Agent A (C dumper)**: `tools/jampgame-oracle/main_pmove.c` + run.sh wiring + fixtures + raw-trace mini-golden + `golden/pmove.txt`. Hand it: spec doc, link/stub list above, harness conventions (README pattern).
- **Agent B (Rust test)**: `crates/mp/game/tests/pmove_parity.rs` — fixture parser, TestTraps (same pseudocode), TestCallbacks (panic + mirror), BgState + anim load, drive `mp_game::bg_pmove::Pmove`, compare goldens. Hand it: spec doc, `Pmove` signature (bg_pmove.rs:10229), `PmoveContext`/`BgState` paths, jampgame_parity.rs as the pattern.
Run A to completion (goldens committed) before B's compare runs — or land B `#[ignore]`d until goldens exist. If the spec can't be frozen up front, use 1 agent sequentially instead.