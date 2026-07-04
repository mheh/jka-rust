// PORT-COMPLETE: NPC_utils.c 11/24
//! Port of `oracle/oracle/codemp/game/NPC_utils.c` (jampgame mega-pass).
//!
//! Generated from `tools/closure-prototype/fnskel.py`; bodies filled per the
//! jampgame mega-pass (settled fork rulings,
//! `docs/handoffs/jampgame-fork-discovery.md`).
//!
//! SPINE (fork rulings 1/4 + `docs/architecture/engine-seam.md`, precedent
//! `w_force.rs`/`g_client.rs`): logic fns that reach `level`/cvars/`g_entities`/
//! traps thread the `GameContext<'_>` receiver (`.world: *mut GameWorld`,
//! `.engine`) as an ADDITIVE first parameter (the faithful C signature carries
//! none). Globals are `GameWorld` fields (fork 1): `level` →
//! `(*ctx.world).level`, `g_entities[i]` → `(*ctx.world).entities[i]`; this
//! file's own `teamNumbers`/`teamStrength`/`teamCounter` file-scope globals
//! were added to `GameWorld` (additive, Raven names kept — see
//! `world/game_world.rs`). Traps go through `trap::X(ctx.engine, …)`.
//! Cross-file callees are invoked with the packet's resolved raw-pointer
//! signatures verbatim (their own porters thread the spine). Raw
//! `gentity_t*`/`gclient_t*` chains are transcribed as `unsafe` raw-pointer
//! field access mirroring the C exactly.
//!
//! PARKED (see PORT-ESCALATION markers): the bulk of this file's functions
//! read the ambient bot-AI "current actor" globals (`NPC`, `NPCInfo`,
//! `client`, `ucmd`) that Raven's `ai_main.c` think-loop sets per NPC frame —
//! there is no `GameWorld`/`GameContext` field for them and no entity
//! parameter to substitute (topic `ai-context`, matching the `NPC_combat.rs`/
//! `NPC_AI_Jedi.rs` precedent in this same mega-pass). Two more are byval
//! `vec3_t` out-params (`CalcEntitySpot`'s `point`, `G_GetBoltPosition`'s
//! `pos`) — the fnskel generator keeps the C by-value shape for these, which
//! cannot carry a write back to the caller in Rust; this is the same
//! unresolved `vec3-outparam-seam` fork flagged in `g_utils.rs`. One more
//! (`G_ActivateBehavior`) calls `va(fmt, args…)` with real variadic arguments
//! (topic `va-varargs`; the resolved `va` signature drops the C varargs, same
//! as the `g_client.rs`/`w_force.rs` precedent) — this differs from the
//! zero-variadic-arg `Com_Printf`/`G_DebugPrint` call sites elsewhere in this
//! file, which ARE portable (called exactly like `bg_saberLoad.rs` calls the
//! still-parked `Com_Printf`).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::NPC_senses::InFOV;
use crate::g_utils::G_BoneIndex;
use crate::q_shared::Q_stricmp;
use crate::trap;
use crate::world::GameContext;

/// Raven `BONE_ANGLES_POSTMULT` (ghoul2 bone-angle apply mode).
/// Source: `oracle/oracle/code/game/ghoul2_shared.h:54`
const BONE_ANGLES_POSTMULT: c_int = 0x0002;

/// Raven `BG_NUM_TOGGLEABLE_SURFACES`.
/// Source: `oracle/oracle/codemp/game/bg_public.h:138`
const BG_NUM_TOGGLEABLE_SURFACES: c_int = 31;

/// Raven `PMF_FOLLOW` — spectate following another player.
/// Source: `oracle/oracle/codemp/game/bg_public.h:415`
const PMF_FOLLOW: c_int = 4096;

use mp_bg::public::team::TEAM_SPECTATOR;
use mp_qshared::shared::MAX_CLIENTS;

use mp_abi::game::syscalls::G_G2_ANGLEOVERRIDE::GG2AngleoverrideArgs;
use mp_abi::game::syscalls::G_G2_SETSURFACEONOFF::GG2SetsurfaceonoffArgs;
use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;

/// Raven `CalcEntitySpot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:20-168`
// PORT-ESCALATION(vec3-outparam-seam): `point` is the C out-param this
// faithful signature keeps by value (`vec3_t`, itself `[f32; 3]`, `Copy`) —
// writes inside the body would not propagate back to the caller. The
// SPOT_GROUND branch also calls `trap_Trace` (needs a `GameContext`/`Engine`
// handle this signature carries none of). Same unresolved fork as
// `g_utils.rs`'s `vec3-outparam-seam` sites.
pub fn CalcEntitySpot(
    ctx: GameContext<'_>,
    ent: *const gentity_t,
    spot: spot_t,
    point: vec3_t,
) {
    todo!("Port CalcEntitySpot — parked: vec3-outparam-seam")
}

// PORT-ESCALATION(ai-context): reads/writes the ambient `NPC`/`NPCInfo`/
// `client`/`ucmd`/`level` bot-AI actor globals; the faithful zero-param
// signature carries no channel to reach them (no `GameWorld` field holds the
// "current NPC" — that's per-think ai_main.c state, out of `NPC_utils.c`'s
// closure). Also calls `trap_Cvar_VariableStringBuffer`/
// `trap_ICARUS_TaskIDPending`/`trap_ICARUS_TaskIDComplete` (need an `Engine`
// handle this signature carries none of).
/// Raven `NPC_UpdateAngles`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:182-517`
pub fn NPC_UpdateAngles(
    ctx: GameContext<'_>,
    doPitch: qboolean,
    doYaw: qboolean,
) -> qboolean {
    todo!("Port NPC_UpdateAngles — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads/writes ambient `NPCInfo`/`NPC`/`level`;
// no channel to reach them from this context-free faithful signature.
/// Raven `NPC_AimWiggle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:519-533`
pub fn NPC_AimWiggle(
    ctx: GameContext<'_>,
    enemy_org: vec3_t,
) {
    todo!("Port NPC_AimWiggle — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads/writes ambient `NPC`/`NPCInfo`/`client`/
// `ucmd`/`level`; no channel to reach them from this context-free faithful
// signature.
/// Raven `NPC_UpdateFiringAngles`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:540-731`
pub fn NPC_UpdateFiringAngles(
    ctx: GameContext<'_>,
    doPitch: qboolean,
    doYaw: qboolean,
) -> qboolean {
    todo!("Port NPC_UpdateFiringAngles — parked: ai-context")
}

// PORT-ESCALATION(ai-context): writes the ambient `NPCInfo->shootAngles`; no
// channel to reach it from this context-free faithful signature.
/// Raven `NPC_UpdateShootAngles`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:740-808`
pub fn NPC_UpdateShootAngles(
    ctx: GameContext<'_>,
    angles: vec3_t,
    doPitch: qboolean,
    doYaw: qboolean,
) {
    todo!("Port NPC_UpdateShootAngles — parked: ai-context")
}

/// Raven `SetTeamNumbers`.
///
/// Raven: Sets the number of living clients on each team. FIXME: Does not
/// account for non-respawned players! FIXME: Don't include medics?
///
/// Raven's outer loop is `for ( i = 0; i < 1 ; i++ )` — a known-dead loop
/// bound (only ever checks `g_entities[0]`), preserved faithfully (§C10/§19).
/// The average-health division is UB in Raven when `teamNumbers[i] == 0`
/// (int/int, then implicit float->int on the `floor()` result); Rust's
/// `f32` division yields `NaN`/`inf`, and `as c_int` on those saturates to 0
/// — the one defined behavior, picked per §19.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:818-847`
pub fn SetTeamNumbers(ctx: GameContext<'_>) {
    unsafe {
        for i in 0..4usize {
            (*ctx.world).teamNumbers[i] = 0;
            (*ctx.world).teamStrength[i] = 0;
        }

        for i in 0..1usize {
            let found = &mut (*ctx.world).entities[i] as *mut gentity_t;
            if !(*found).client.is_null() {
                if (*found).health > 0 {
                    let client = (*found).client as *mut gclient_t;
                    let team = (*client).playerTeam as usize;
                    (*ctx.world).teamNumbers[team] += 1;
                    (*ctx.world).teamStrength[team] += (*found).health;
                }
            }
        }

        for i in 0..4usize {
            // Raven: `floor( ((float)(teamStrength[i])) / ((float)(teamNumbers[i])) )`.
            let strength = (*ctx.world).teamStrength[i] as f32;
            let count = (*ctx.world).teamNumbers[i] as f32;
            (*ctx.world).teamStrength[i] = (strength / count).floor() as c_int;
        }
    }
}

// PORT-ESCALATION(va-varargs): calls `va( "%s/%s", Q3_SCRIPT_DIR, bs_name )`
// with real variadic arguments to build the ICARUS script path — the
// resolved `va` signature drops the C varargs (same fork as `g_client.rs`/
// `w_force.rs`'s parked `va(fmt, …)` call sites), so this call cannot be
// transcribed faithfully yet. (The `if (0) G_DebugPrint(...)` branch is dead
// code in the oracle — not itself a blocker — but the trailing `va()`/
// `trap_ICARUS_RunScript` path is live.)
/// Raven `G_ActivateBehavior`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:851-894`
pub fn G_ActivateBehavior(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    bset: c_int,
) -> qboolean {
    todo!("Port G_ActivateBehavior — parked: va-varargs")
}

/// Raven `NPC_SetBoneAngles`.
///
/// Raven: rww - special system for sync'ing bone angles between client and
/// server. The `#ifdef _XBOX` byte-index branch is dead on this build; the
/// plain `int *` branch below is the compiled one (per house ruling on
/// `_XBOX` branches).
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:906-995`
pub fn NPC_SetBoneAngles(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    bone: *mut c_char,
    angles: vec3_t,
) {
    unsafe {
        let boneIndex = G_BoneIndex(ctx, bone as *const c_char);

        // Walk the 4 fixed bone-index/bone-angle slot pairs looking for
        // `boneIndex` (or the first free slot if not already present).
        let mut thebone: *mut c_int = &mut (*ent).s.boneIndex1;
        let mut boneVector: *mut vec3_t = &mut (*ent).s.boneAngles1;
        let mut firstFree: *mut c_int = core::ptr::null_mut();
        let mut freeBoneVec: *mut vec3_t = core::ptr::null_mut();
        let mut i = 0;
        let mut found = false;

        loop {
            if thebone.is_null() {
                break;
            }
            if *thebone == 0 && firstFree.is_null() {
                firstFree = thebone;
                freeBoneVec = boneVector;
            } else if *thebone != 0 {
                if *thebone == boneIndex {
                    found = true;
                    break;
                }
            }

            match i {
                0 => {
                    thebone = &mut (*ent).s.boneIndex2;
                    boneVector = &mut (*ent).s.boneAngles2;
                }
                1 => {
                    thebone = &mut (*ent).s.boneIndex3;
                    boneVector = &mut (*ent).s.boneAngles3;
                }
                2 => {
                    thebone = &mut (*ent).s.boneIndex4;
                    boneVector = &mut (*ent).s.boneAngles4;
                }
                _ => {
                    thebone = core::ptr::null_mut();
                    boneVector = core::ptr::null_mut();
                }
            }
            i += 1;
        }

        if thebone.is_null() {
            // didn't find it, create it
            if firstFree.is_null() {
                let msg = std::ffi::CString::new("WARNING: NPC has no free bone indexes\n").unwrap();
                crate::g_main::Com_Printf(msg.as_ptr());
                return;
            }
            thebone = firstFree;
            *thebone = boneIndex;
            boneVector = freeBoneVec;
        }

        // Copy the angles over the vector in the entitystate, so we can use
        // the corresponding index to set the bone angles on the client.
        *boneVector = angles;

        // Now set the angles on our server instance if we have one.
        if (*ent).ghoul2.is_null() {
            return;
        }

        let flags = BONE_ANGLES_POSTMULT;
        let up = POSITIVE_X as c_int;
        let right = NEGATIVE_Y as c_int;
        let forward = NEGATIVE_Z as c_int;

        //first 3 bits is forward, second 3 bits is right, third 3 bits is up
        (*ent).s.boneOrient = forward | (right << 3) | (up << 6);

        let bone_name = std::ffi::CStr::from_ptr(bone as *const c_char).to_owned();
        trap::G2API_SetBoneAngles(
            ctx.engine,
            GG2AngleoverrideArgs::new(
                (*ent).ghoul2,
                0,
                bone_name,
                &angles as *const vec3_t,
                flags,
                up,
                right,
                forward,
                core::ptr::null_mut(),
                100,
                (*ctx.world).level.time,
            ),
        );
    }
}

// PORT-ESCALATION(unported-global): reads `bgToggleableSurfaces` (bg-shared
// lookup table, `NPC_utils.c:1006`) — a genuinely unported file-scope global
// (fork-discovery ruling 1), not just a missing `use`.
/// Raven `NPC_SetSurfaceOnOff`.
///
/// Raven: rww - and another method of automatically managing surface status
/// for the client and server at once.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1001-1039`
pub fn NPC_SetSurfaceOnOff(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    surfaceName: *const c_char,
    surfaceFlags: c_int,
) {
    todo!("Port NPC_SetSurfaceOnOff — parked: unported-global (bgToggleableSurfaces)")
}

/// Raven `NPC_SomeoneLookingAtMe`.
///
/// Raven: rww - cheap check to see if an armed client is looking in our
/// general direction.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1042-1067`
pub fn NPC_SomeoneLookingAtMe(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) -> qboolean {
    unsafe {
        let mut i: usize = 0;
        while i < MAX_CLIENTS {
            let pEnt = &mut (*ctx.world).entities[i] as *mut gentity_t;

            let eligible = !pEnt.is_null()
                && (*pEnt).inuse != QFALSE
                && !(*pEnt).client.is_null()
                && {
                    let cl = (*pEnt).client as *mut gclient_t;
                    (*cl).sess.sessionTeam != TEAM_SPECTATOR
                        && ((*cl).ps.pm_flags & PMF_FOLLOW) == 0
                        && (*pEnt).s.weapon != WP_NONE
                };

            if eligible
                && trap::InPVS(
                    ctx.engine,
                    GInPvsArgs::new(
                        &(*ent).r.currentOrigin as *const vec3_t,
                        &(*pEnt).r.currentOrigin as *const vec3_t,
                    ),
                ) != 0
                //I'm in a 30 fov or so cone from this player.. that's enough I guess.
                && InFOV(ctx, pEnt, ent, 30, 30) != 0
            {
                return QTRUE;
            }

            i += 1;
        }

        QFALSE
    }
}

// PORT-ESCALATION(ai-context): calls `G_ClearLOS( NPC, start, end )` — reads
// the ambient `NPC` bot-AI actor global; no channel to reach it (no entity
// param on this faithful signature, no `GameWorld` field holds "current NPC").
/// Raven `NPC_ClearLOS`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1069-1072`
pub fn NPC_ClearLOS(
    ctx: GameContext<'_>,
    start: vec3_t,
    end: vec3_t,
) -> qboolean {
    todo!("Port NPC_ClearLOS — parked: ai-context")
}

// PORT-ESCALATION(ai-context): calls `G_ClearLOS5( NPC, end )` — ambient `NPC`.
/// Raven `NPC_ClearLOS5`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1073-1076`
pub fn NPC_ClearLOS5(
    ctx: GameContext<'_>,
    end: vec3_t,
) -> qboolean {
    todo!("Port NPC_ClearLOS5 — parked: ai-context")
}

// PORT-ESCALATION(ai-context): calls `G_ClearLOS4( NPC, ent )` — ambient `NPC`.
/// Raven `NPC_ClearLOS4`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1077-1080`
pub fn NPC_ClearLOS4(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) -> qboolean {
    todo!("Port NPC_ClearLOS4 — parked: ai-context")
}

// PORT-ESCALATION(ai-context): calls `G_ClearLOS3( NPC, start, ent )` —
// ambient `NPC`.
/// Raven `NPC_ClearLOS3`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1081-1084`
pub fn NPC_ClearLOS3(
    ctx: GameContext<'_>,
    start: vec3_t,
    ent: *mut gentity_t,
) -> qboolean {
    todo!("Port NPC_ClearLOS3 — parked: ai-context")
}

// PORT-ESCALATION(ai-context): calls `G_ClearLOS2( NPC, ent, end )` —
// ambient `NPC`.
/// Raven `NPC_ClearLOS2`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1085-1088`
pub fn NPC_ClearLOS2(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    end: vec3_t,
) -> qboolean {
    todo!("Port NPC_ClearLOS2 — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads the ambient `NPC` actor
// (`NPC->client->playerTeam`/`enemyTeam`, etc.) throughout — no channel to
// reach it from this signature (only `ent`, the candidate enemy, is a param).
/// Raven `NPC_ValidEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1096-1187`
pub fn NPC_ValidEnemy(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) -> qboolean {
    todo!("Port NPC_ValidEnemy — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads the ambient `NPC`/`NPCInfo` actor
// (`NPCInfo->stats.{visrange,hfov,vfov}`, `NPC->r.currentOrigin`) — no
// channel to reach them from this signature.
/// Raven `NPC_TargetVisible`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1195-1210`
pub fn NPC_TargetVisible(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) -> qboolean {
    todo!("Port NPC_TargetVisible — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads ambient `NPCInfo->stats.visrange`; also
// calls `trap_EntitiesInBox` (needs an `Engine` handle this signature
// carries none of).
/// Raven `NPC_FindNearestEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1246-1294`
pub fn NPC_FindNearestEnemy(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) -> c_int {
    todo!("Port NPC_FindNearestEnemy — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads the ambient `NPC`/`level` (`g_entities`,
// `level.alertEvents`) — no channel to reach them from this signature.
/// Raven `NPC_PickEnemyExt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1302-1348`
pub fn NPC_PickEnemyExt(
    ctx: GameContext<'_>,
    checkAlerts: qboolean,
) -> *mut gentity_t {
    todo!("Port NPC_PickEnemyExt — parked: ai-context")
}

/// Raven `NPC_FindPlayer`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1356-1359`
pub fn NPC_FindPlayer(ctx: GameContext<'_>) -> qboolean {
    unsafe { NPC_TargetVisible(ctx, &mut (*ctx.world).entities[0] as *mut gentity_t) }
}

/// Raven `NPC_CheckPlayerDistance`.
///
/// Raven: the live body is a hardcoded `return qfalse; //MOOT in MP` — the
/// entire real implementation is `#if 0`-style commented out (dead in this
/// build); faithfully preserved as an always-false stub.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1367-1399`
fn NPC_CheckPlayerDistance() -> qboolean {
    QFALSE
}

// PORT-ESCALATION(ai-context): reads the ambient `NPC`/`NPCInfo`/`level`
// actor (`NPCInfo->confusionTime`, `NPC->client->NPC_class`, `NPC->enemy`) —
// no channel to reach them from this signature.
/// Raven `NPC_FindEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1407-1461`
pub fn NPC_FindEnemy(
    ctx: GameContext<'_>,
    checkAlerts: qboolean,
) -> qboolean {
    todo!("Port NPC_FindEnemy — parked: ai-context")
}

/// Raven `NPC_CheckEnemyExt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1469-1483`
pub fn NPC_CheckEnemyExt(
    ctx: GameContext<'_>,
    checkAlerts: qboolean,
) -> qboolean {
    NPC_FindEnemy(ctx, checkAlerts)
}

// PORT-ESCALATION(ai-context): reads/writes the ambient `NPC`/`client`/
// `level`/`ucmd`/`NPCInfo` actor throughout (`CalcEntitySpot`/
// `NPC_UpdateAngles` calls, `ucmd.angles`, `client->ps.delta_angles`) — no
// channel to reach them from this signature.
/// Raven `NPC_FacePosition`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1491-1547`
pub fn NPC_FacePosition(
    ctx: GameContext<'_>,
    position: vec3_t,
    doPitch: qboolean,
) -> qboolean {
    todo!("Port NPC_FacePosition — parked: ai-context")
}

/// Raven `NPC_FaceEntity`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1555-1563`
pub fn NPC_FaceEntity(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    doPitch: qboolean,
) -> qboolean {
    let mut entPos: vec3_t = [0.0; 3];
    CalcEntitySpot(ctx, ent as *const gentity_t, spot_t::SPOT_HEAD_LEAN, entPos);
    NPC_FacePosition(ctx, entPos, doPitch)
}

// PORT-ESCALATION(ai-context): reads the ambient `NPC` actor (`NPC->enemy`)
// — no channel to reach it from this signature.
/// Raven `NPC_FaceEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1571-1580`
pub fn NPC_FaceEnemy(
    ctx: GameContext<'_>,
    doPitch: qboolean,
) -> qboolean {
    todo!("Port NPC_FaceEnemy — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads the ambient `NPCInfo`/`NPC` actor
// (`NPCInfo->scriptFlags`, `NPC->enemy`) — no channel to reach them from this
// signature.
/// Raven `NPC_CheckCanAttackExt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1588-1603`
pub fn NPC_CheckCanAttackExt(ctx: GameContext<'_>) -> qboolean {
    todo!("Port NPC_CheckCanAttackExt — parked: ai-context")
}

/// Raven `NPC_ClearLookTarget`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1611-1625`
pub fn NPC_ClearLookTarget(
    self_: *mut gentity_t,
) {
    unsafe {
        if (*self_).client.is_null() {
            return;
        }
        let client = (*self_).client as *mut gclient_t;

        if (*client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
            //lookTarget is set by and to the monster that's holding you, no
            //other operations can change that
            return;
        }

        (*client).renderInfo.lookTarget = ENTITYNUM_NONE; //ENTITYNUM_WORLD;
        (*client).renderInfo.lookTargetClearTime = 0;
    }
}

/// Raven `NPC_SetLookTarget`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1632-1646`
pub fn NPC_SetLookTarget(
    self_: *mut gentity_t,
    entNum: c_int,
    clearTime: c_int,
) {
    unsafe {
        if (*self_).client.is_null() {
            return;
        }
        let client = (*self_).client as *mut gclient_t;

        if (*client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
            //lookTarget is set by and to the monster that's holding you, no
            //other operations can change that
            return;
        }

        (*client).renderInfo.lookTarget = entNum;
        (*client).renderInfo.lookTargetClearTime = clearTime;
    }
}

/// Raven `NPC_CheckLookTarget`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1653-1679`
pub fn NPC_CheckLookTarget(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) -> qboolean {
    unsafe {
        if !(*self_).client.is_null() {
            let client = (*self_).client as *mut gclient_t;
            let lookTarget = (*client).renderInfo.lookTarget;

            if lookTarget >= 0 && lookTarget < ENTITYNUM_WORLD {
                //within valid range
                let target = &mut (*ctx.world).entities[lookTarget as usize] as *mut gentity_t;
                if (target.is_null()) || (*target).inuse == QFALSE {
                    //lookTarget not inuse or not valid anymore
                    NPC_ClearLookTarget(self_);
                } else if (*client).renderInfo.lookTargetClearTime != 0
                    && (*client).renderInfo.lookTargetClearTime < (*ctx.world).level.time
                {
                    //Time to clear lookTarget
                    NPC_ClearLookTarget(self_);
                } else if !(*target).client.is_null()
                    && !(*self_).enemy.is_null()
                    && target != (*self_).enemy
                {
                    //should always look at current enemy if engaged in
                    //battle... FIXME: this could override certain scripted
                    //lookTargets...???
                    NPC_ClearLookTarget(self_);
                } else {
                    return QTRUE;
                }
            }
        }

        QFALSE
    }
}

// PORT-ESCALATION(ai-context): reads/writes the ambient `NPC`/`NPCInfo`/
// `level` actor (`NPCInfo->charmedTime`, `NPC->client`, `NPC->genericValue*`)
// — no channel to reach them from this zero-param faithful signature.
/// Raven `NPC_CheckCharmed`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1687-1705`
pub fn NPC_CheckCharmed(ctx: GameContext<'_>) {
    todo!("Port NPC_CheckCharmed — parked: ai-context")
}

/// Raven `G_GetBoltPosition`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1707-1740`
// PORT-ESCALATION(vec3-outparam-seam): `pos` is the C out-param this
// faithful signature keeps by value (`vec3_t`, `Copy`) — writes inside the
// body would not propagate back to the caller. Same unresolved fork as
// `CalcEntitySpot` above / `g_utils.rs`'s `vec3-outparam-seam` sites.
pub fn G_GetBoltPosition(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    boltIndex: c_int,
    pos: vec3_t,
    modelIndex: c_int,
) {
    todo!("Port G_GetBoltPosition — parked: vec3-outparam-seam")
}

// PORT-ESCALATION(ai-context): calls `G_GetBoltPosition( NPC, boltIndex, org, 0 )`
// — reads the ambient `NPC` actor; no channel to reach it from this signature.
/// Raven `NPC_EntRangeFromBolt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1742-1754`
pub fn NPC_EntRangeFromBolt(
    ctx: GameContext<'_>,
    targEnt: *mut gentity_t,
    boltIndex: c_int,
) -> f32 {
    todo!("Port NPC_EntRangeFromBolt — parked: ai-context")
}

// PORT-ESCALATION(ai-context): reads the ambient `NPC` actor (`NPC->enemy`)
// — no channel to reach it from this signature.
/// Raven `NPC_EnemyRangeFromBolt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1756-1759`
pub fn NPC_EnemyRangeFromBolt(
    ctx: GameContext<'_>,
    boltIndex: c_int,
) -> f32 {
    todo!("Port NPC_EnemyRangeFromBolt — parked: ai-context")
}

// PORT-ESCALATION(ai-context): calls `G_GetBoltPosition( NPC, boltIndex, org, 0 )`
// — reads the ambient `NPC` actor; no channel to reach it from this
// signature (also calls `trap_EntitiesInBox`, needing an `Engine` handle).
/// Raven `NPC_GetEntsNearBolt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_utils.c:1761-1782`
pub fn NPC_GetEntsNearBolt(
    ctx: GameContext<'_>,
    radiusEnts: *mut c_int,
    radius: f32,
    boltIndex: c_int,
    boltOrg: vec3_t,
) -> c_int {
    todo!("Port NPC_GetEntsNearBolt — parked: ai-context")
}
