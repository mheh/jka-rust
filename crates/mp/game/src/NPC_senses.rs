// PORT-COMPLETE: NPC_senses.c
//! FAITHFUL signature skeleton for `oracle/codemp/game/NPC_senses.c`.
//!
//! Filled by the jampgame mega-pass. Most of this file reaches file-scope game state
//! (`level`, the current-NPC `NPC`/`NPCInfo` globals, `g_entities`,
//! `vec3_origin`) or calls `trap_Trace`/`trap_InPVS` (whose resolved
//! wrappers take `&Engine`, which this faithful raw-pointer signature set
//! carries none of) or `CalcEntitySpot`/`vectoangles`/`AngleVectors`/
//! `VectorNormalize` (whose resolved signatures take `vec3_t` out-params
//! by value, so they cannot write results back — see `g_combat.rs`'s
//! established `vec3-outparam-seam` park reason).
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`. Bodies re-derive the raw pointers verbatim at the
//! top (`// STAGE-1:` markers) — Stage-2 debt. Callers bridge at the boundary
//! via `ctx.entity_id_of(ptr)`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_public_consts::SVF_GLASS_BRUSH;
use crate::level::alert_event::{
    alertEventLevel_e, alertEventLevel_e::AEL_DANGER, alertEventType_e, alertEvent_t,
    MAX_ALERT_EVENTS,
};
use crate::level::interest_point::MAX_INTEREST_POINTS;
use crate::npc::check_flags::{CHECK_360, CHECK_FOV, CHECK_PVS, CHECK_SHOOT, CHECK_VISRANGE};
use crate::ent_id;
use crate::npc::script_flags::SCF_DONT_FLEE;
use crate::prelude::*;
use crate::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin,
    vectoangles, AngleDelta, AngleVectors, VectorLength, VectorLengthSquared, VectorNormalize,
    VectorNormalize2,
};
use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_qshared::shared::{
    CONTENTS_OPAQUE, ENTITYNUM_NONE, ENTITYNUM_WORLD, MASK_OPAQUE, MAX_GENTITIES,
};

/// Raven `G_ClearLineOfSight`.
///
/// Raven: "returns true if can see from point 1 to 2, even through glass (1
/// pane)- doesn't work with portals".
/// Source: `oracle/codemp/game/NPC_senses.c:11-36`
pub fn G_ClearLineOfSight(
    ctx: &mut GameContext,
    point1: vec3_t,
    point2: vec3_t,
    ignore: c_int,
    clipmask: c_int,
) -> qboolean {
    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &point1 as *const vec3_t,
            core::ptr::null(),
            core::ptr::null(),
            &point2 as *const vec3_t,
            ignore,
            clipmask,
        ),
    );

    if tr.fraction == 1.0 {
        return 1;
    }

    let hit_id = EntityId(tr.entityNum as u32);

    if EntIsGlass(ctx.entity(hit_id)) != 0 {
        let mut newpoint1 = tr.endpos;
        let hit_num = ctx.entity(hit_id).s.number;
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &newpoint1 as *const vec3_t,
                core::ptr::null(),
                core::ptr::null(),
                &point2 as *const vec3_t,
                hit_num,
                clipmask,
            ),
        );

        if tr.fraction == 1.0 {
            return 1;
        }
    }

    0
}

/// Raven `CanSee`.
///
/// Raven: determine if NPC can see an entity. This is a straight line trace
/// check. This function does not look at PVS or FOV, or take any AI related
/// factors (for example, the NPC's reaction time) into account.
/// Source: `oracle/codemp/game/NPC_senses.c:47-80`
pub fn CanSee(ctx: &mut GameContext, ent: Option<EntityId>) -> qboolean {
    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    let mut eyes = [0.0; 3];
    let mut spot = [0.0; 3];

    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_num = ctx.entity(npc_id).s.number;
    CalcEntitySpot(ctx, Some(npc_id), spot_t::SPOT_HEAD_LEAN, &mut eyes);

    CalcEntitySpot(ctx, ent, spot_t::SPOT_ORIGIN, &mut spot);
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &eyes as *const vec3_t,
            core::ptr::null(),
            core::ptr::null(),
            &spot as *const vec3_t,
            npc_num,
            MASK_OPAQUE,
        ),
    );
    ShotThroughGlass(ctx, &mut tr as *mut trace_t, ent, spot, MASK_OPAQUE);
    if tr.fraction == 1.0 {
        return 1;
    }

    CalcEntitySpot(ctx, ent, spot_t::SPOT_HEAD, &mut spot);
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &eyes as *const vec3_t,
            core::ptr::null(),
            core::ptr::null(),
            &spot as *const vec3_t,
            npc_num,
            MASK_OPAQUE,
        ),
    );
    ShotThroughGlass(ctx, &mut tr as *mut trace_t, ent, spot, MASK_OPAQUE);
    if tr.fraction == 1.0 {
        return 1;
    }

    CalcEntitySpot(ctx, ent, spot_t::SPOT_LEGS, &mut spot);
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &eyes as *const vec3_t,
            core::ptr::null(),
            core::ptr::null(),
            &spot as *const vec3_t,
            npc_num,
            MASK_OPAQUE,
        ),
    );
    ShotThroughGlass(ctx, &mut tr as *mut trace_t, ent, spot, MASK_OPAQUE);
    if tr.fraction == 1.0 {
        return 1;
    }

    0
}

/// Raven `InFront`.
///
/// Source: `oracle/codemp/game/NPC_senses.c:82-98`
pub fn InFront(spot: vec3_t, from: vec3_t, fromAngles: vec3_t, threshHold: f32) -> qboolean {
    let mut dir = [0.0; 3];
    let mut forward = [0.0; 3];
    let mut angles = [0.0; 3];
    let dot: f32;

    _VectorSubtract(spot, from, &mut dir);
    dir[2] = 0.0;
    VectorNormalize(&mut dir);

    _VectorCopy(fromAngles, &mut angles);
    angles[0] = 0.0;
    AngleVectors(angles, Some(&mut forward), None, None);

    dot = _DotProduct(dir, forward);

    if dot > threshHold {
        1
    } else {
        0
    }
}

/// Raven `InFOV3`.
///
/// Raven: IDEA: further off to side of FOV range, higher chance of failing
/// even if technically in FOV, keep core of 50% to sides as always
/// succeeding.
/// Source: `oracle/codemp/game/NPC_senses.c:109-125`
pub fn InFOV3(
    spot: vec3_t,
    from: vec3_t,
    fromAngles: vec3_t,
    hFOV: c_int,
    vFOV: c_int,
) -> qboolean {
    let mut deltaVector = [0.0; 3];
    let mut angles = [0.0; 3];
    let mut deltaAngles = [0.0; 3];

    _VectorSubtract(spot, from, &mut deltaVector);
    vectoangles(deltaVector, &mut angles);

    deltaAngles[0] = AngleDelta(fromAngles[0], angles[0]);
    deltaAngles[1] = AngleDelta(fromAngles[1], angles[1]);

    if deltaAngles[0].abs() <= vFOV as f32 && deltaAngles[1].abs() <= hFOV as f32 {
        1
    } else {
        0
    }
}

/// Raven `InFOV2`.
///
/// Raven: NPC to position.
/// Source: `oracle/codemp/game/NPC_senses.c:129-145`
pub fn InFOV2(
    ctx: &mut GameContext,
    origin: vec3_t,
    from: EntityId,
    hFOV: c_int,
    vFOV: c_int,
) -> qboolean {
    let mut fromAngles = [0.0; 3];
    let mut eyes = [0.0; 3];

    let client = ctx.entity(from).client;
    if !client.is_null() {
        // §2b: NPC/vehicle entities carry BG_Alloc'd pool clients, not
        // level.clients; deref the entity's own client pointer raw, as Raven does.
        _VectorCopy(unsafe { (*client).ps.viewangles }, &mut fromAngles);
    } else {
        _VectorCopy(ctx.entity(from).s.angles, &mut fromAngles);
    }

    CalcEntitySpot(ctx, Some(from), spot_t::SPOT_HEAD, &mut eyes);

    InFOV3(origin, eyes, fromAngles, hFOV, vFOV)
}

/// Raven `InFOV`.
///
/// Raven: Entity to entity.
/// Source: `oracle/codemp/game/NPC_senses.c:149-208`
pub fn InFOV(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    from: EntityId,
    hFOV: c_int,
    vFOV: c_int,
) -> qboolean {
    let mut eyes = [0.0; 3];
    let mut spot = [0.0; 3];
    let mut deltaVector = [0.0; 3];
    let mut angles = [0.0; 3];
    let mut fromAngles = [0.0; 3];
    let mut deltaAngles = [0.0; 3];

    let client = ctx.entity(from).client;
    if !client.is_null() {
        // §2b: pool client (may be an NPC), deref raw as Raven does.
        // Check if renderInfo.eyeAngles is not zero
        if !VectorCompare(unsafe { (*client).renderInfo.eyeAngles }, vec3_origin) {
            // Actual facing of tag_head!
            _VectorCopy(unsafe { (*client).renderInfo.eyeAngles }, &mut fromAngles);
        } else {
            _VectorCopy(unsafe { (*client).ps.viewangles }, &mut fromAngles);
        }
    } else {
        _VectorCopy(ctx.entity(from).s.angles, &mut fromAngles);
    }

    CalcEntitySpot(ctx, Some(from), spot_t::SPOT_HEAD_LEAN, &mut eyes);

    CalcEntitySpot(ctx, ent, spot_t::SPOT_ORIGIN, &mut spot);
    _VectorSubtract(spot, eyes, &mut deltaVector);

    vectoangles(deltaVector, &mut angles);
    deltaAngles[0] = AngleDelta(fromAngles[0], angles[0]);
    deltaAngles[1] = AngleDelta(fromAngles[1], angles[1]);
    if deltaAngles[0].abs() <= vFOV as f32 && deltaAngles[1].abs() <= hFOV as f32 {
        return 1;
    }

    CalcEntitySpot(ctx, ent, spot_t::SPOT_HEAD, &mut spot);
    _VectorSubtract(spot, eyes, &mut deltaVector);
    vectoangles(deltaVector, &mut angles);
    deltaAngles[0] = AngleDelta(fromAngles[0], angles[0]);
    deltaAngles[1] = AngleDelta(fromAngles[1], angles[1]);
    if deltaAngles[0].abs() <= vFOV as f32 && deltaAngles[1].abs() <= hFOV as f32 {
        return 1;
    }

    CalcEntitySpot(ctx, ent, spot_t::SPOT_LEGS, &mut spot);
    _VectorSubtract(spot, eyes, &mut deltaVector);
    vectoangles(deltaVector, &mut angles);
    deltaAngles[0] = AngleDelta(fromAngles[0], angles[0]);
    deltaAngles[1] = AngleDelta(fromAngles[1], angles[1]);
    if deltaAngles[0].abs() <= vFOV as f32 && deltaAngles[1].abs() <= hFOV as f32 {
        return 1;
    }

    0
}

/// Raven `InVisrange`.
///
/// Raven: FIXME: make a calculate visibility for ents that takes into
/// account lighting, movement, turning, crouch/stand up, other anims, hide
/// brushes, etc.
/// Source: `oracle/codemp/game/NPC_senses.c:210-251`
pub fn InVisrange(ctx: &mut GameContext, ent: Option<EntityId>) -> qboolean {
    let mut eyes = [0.0; 3];
    let mut spot = [0.0; 3];
    let mut deltaVector = [0.0; 3];

    let npc = ctx.world.globals.NPC;
    let npcinfo = ctx.world.globals.NPCInfo;

    CalcEntitySpot(
        ctx,
        ctx.entity_id_of(npc),
        spot_t::SPOT_HEAD_LEAN,
        &mut eyes,
    );
    CalcEntitySpot(ctx, ent, spot_t::SPOT_ORIGIN, &mut spot);
    _VectorSubtract(spot, eyes, &mut deltaVector);

    // §2c: NPCInfo (gNPC_t) has no accessor; deref stays raw.
    let visrange = unsafe { (*npcinfo).stats.visrange * (*npcinfo).stats.visrange };

    if VectorLengthSquared(deltaVector) > visrange {
        0
    } else {
        1
    }
}

/// Raven `NPC_CheckVisibility`.
///
/// Source: `oracle/codemp/game/NPC_senses.c:257-325`
pub fn NPC_CheckVisibility(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    flags: c_int,
) -> visibility_t {
    // Visibility check flags: `crate::npc::check_flags` (`b_local.h:165-169`).
    // flags should never be 0
    if flags == 0 {
        return visibility_t::VIS_NOT;
    }

    let npc = ctx.world.globals.NPC;
    let npcinfo = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    // check PVS
    if (flags & CHECK_PVS) != 0 {
        let ent_origin = ctx.entity(ent.unwrap()).r.currentOrigin;
        let npc_origin = ctx.entity(npc_id).r.currentOrigin;
        if trap::InPVS(
            ctx.engine,
            GInPvsArgs::new(&ent_origin as *const vec3_t, &npc_origin as *const vec3_t),
        ) == 0
        {
            return visibility_t::VIS_NOT;
        }
    }
    if (flags & (CHECK_360 | CHECK_FOV | CHECK_SHOOT)) == 0 {
        return visibility_t::VIS_PVS;
    }

    // check within visrange
    if (flags & CHECK_VISRANGE) != 0 {
        if InVisrange(ctx, ent) == 0 {
            return visibility_t::VIS_PVS;
        }
    }

    // check 360 degree visibility
    if (flags & CHECK_360) != 0 {
        if CanSee(ctx, ent) == 0 {
            return visibility_t::VIS_PVS;
        }
    }
    if (flags & (CHECK_FOV | CHECK_SHOOT)) == 0 {
        return visibility_t::VIS_360;
    }

    // check FOV
    if (flags & CHECK_FOV) != 0 {
        // §2c: NPCInfo (gNPC_t) has no accessor; deref stays raw.
        if InFOV(ctx, ent, npc_id, unsafe { (*npcinfo).stats.hfov }, unsafe {
            (*npcinfo).stats.vfov
        }) == 0
        {
            return visibility_t::VIS_360;
        }
    }

    if (flags & CHECK_SHOOT) == 0 {
        return visibility_t::VIS_FOV;
    }

    // check shootability
    if (flags & CHECK_SHOOT) != 0 {
        if CanShoot(ctx, ent.unwrap(), npc_id) == 0 {
            return visibility_t::VIS_FOV;
        }
    }

    visibility_t::VIS_SHOOT
}

/// Raven `G_CheckSoundEvents`.
///
/// Raven: NPC_CheckSoundEvents.
/// Source: `oracle/codemp/game/NPC_senses.c:332-386`
pub fn G_CheckSoundEvents(
    ctx: &mut GameContext,
    self_: EntityId,
    maxHearDist: f32,
    ignoreAlert: c_int,
    mustHaveOwner: qboolean,
    minAlertLevel: c_int,
) -> c_int {
    let mut bestEvent = -1;
    let mut bestAlert = -1;
    let mut bestTime = -1;
    let max_hear_dist_squared = maxHearDist * maxHearDist;
    let self_origin = ctx.entity(self_).r.currentOrigin;

    for i in 0..ctx.world.level.numAlertEvents as usize {
        // are we purposely ignoring this alert?
        if i as c_int == ignoreAlert {
            continue;
        }
        // We're only concerned about sounds
        if ctx.world.level.alertEvents[i].r#type != alertEventType_e::AET_SOUND {
            continue;
        }
        // must be at least this noticeable
        if (ctx.world.level.alertEvents[i].level as i32) < minAlertLevel {
            continue;
        }
        // must have an owner?
        if mustHaveOwner != 0 && ctx.world.level.alertEvents[i].owner.is_null() {
            continue;
        }
        // Must be within range
        let dist = DistanceSquared(ctx.world.level.alertEvents[i].position, self_origin);

        // can't hear it
        if dist > max_hear_dist_squared {
            continue;
        }

        let radius = ctx.world.level.alertEvents[i].radius * ctx.world.level.alertEvents[i].radius;
        if dist > radius {
            continue;
        }

        if ctx.world.level.alertEvents[i].addLight != 0.0 {
            // a quiet sound, must have LOS to hear it
            if G_ClearLOS5(ctx, self_, ctx.world.level.alertEvents[i].position) == 0 {
                // no LOS, didn't hear it
                continue;
            }
        }

        // See if this one takes precedence over the previous one
        if ctx.world.level.alertEvents[i].level as i32 >= bestAlert
            || (ctx.world.level.alertEvents[i].level as i32 == bestAlert
                && ctx.world.level.alertEvents[i].timestamp >= bestTime)
        {
            bestEvent = i as c_int;
            bestAlert = ctx.world.level.alertEvents[i].level as i32;
            bestTime = ctx.world.level.alertEvents[i].timestamp;
        }
    }

    bestEvent
}

/// Raven `G_GetLightLevel`.
///
/// Raven: rwwFIXMEFIXME: ...this is evil. We can possibly read from the
/// server BSP data, or load the lightmap along with collision data and
/// whatnot, but is it worth it? Presently a stub returning full brightness.
/// Source: `oracle/codemp/game/NPC_senses.c:388-402`
pub fn G_GetLightLevel(pos: vec3_t, fromDir: vec3_t) -> f32 {
    // rwwFIXMEFIXME: ...this is evil. We can possibly read from the server BSP
    // data, or load the lightmap along with collision data and whatnot, but is
    // it worth it?
    255.0
}

/// Raven `G_CheckSightEvents`.
///
/// Raven: NPC_CheckSightEvents.
/// Source: `oracle/codemp/game/NPC_senses.c:408-468`
pub fn G_CheckSightEvents(
    ctx: &mut GameContext,
    self_: EntityId,
    hFOV: c_int,
    vFOV: c_int,
    maxSeeDist: f32,
    ignoreAlert: c_int,
    mustHaveOwner: qboolean,
    minAlertLevel: c_int,
) -> c_int {
    let mut bestEvent = -1;
    let mut bestAlert = -1;
    let mut bestTime = -1;
    let max_see_dist_squared = maxSeeDist * maxSeeDist;
    let self_origin = ctx.entity(self_).r.currentOrigin;

    for i in 0..ctx.world.level.numAlertEvents as usize {
        // are we purposely ignoring this alert?
        if i as c_int == ignoreAlert {
            continue;
        }
        // We're only concerned with sight events
        if ctx.world.level.alertEvents[i].r#type != alertEventType_e::AET_SIGHT {
            continue;
        }
        // must be at least this noticeable
        if (ctx.world.level.alertEvents[i].level as i32) < minAlertLevel {
            continue;
        }
        // must have an owner?
        if mustHaveOwner != 0 && ctx.world.level.alertEvents[i].owner.is_null() {
            continue;
        }

        // Must be within range
        let dist = DistanceSquared(ctx.world.level.alertEvents[i].position, self_origin);

        // can't see it
        if dist > max_see_dist_squared {
            continue;
        }

        let radius = ctx.world.level.alertEvents[i].radius * ctx.world.level.alertEvents[i].radius;
        if dist > radius {
            continue;
        }

        // Must be visible
        if InFOV2(
            ctx,
            ctx.world.level.alertEvents[i].position,
            self_,
            hFOV,
            vFOV,
        ) == 0
        {
            continue;
        }

        if G_ClearLOS5(ctx, self_, ctx.world.level.alertEvents[i].position) == 0 {
            continue;
        }

        // See if this one takes precedence over the previous one
        if ctx.world.level.alertEvents[i].level as i32 >= bestAlert
            || (ctx.world.level.alertEvents[i].level as i32 == bestAlert
                && ctx.world.level.alertEvents[i].timestamp >= bestTime)
        {
            bestEvent = i as c_int;
            bestAlert = ctx.world.level.alertEvents[i].level as i32;
            bestTime = ctx.world.level.alertEvents[i].timestamp;
        }
    }

    bestEvent
}

/// Raven `G_CheckAlertEvents`.
///
/// Raven: NPC_CheckAlertEvents. NOTE: Should all NPCs create alertEvents too
/// so they can detect each other?
/// Source: `oracle/codemp/game/NPC_senses.c:478-530`
pub fn G_CheckAlertEvents(
    ctx: &mut GameContext,
    self_: EntityId,
    checkSight: qboolean,
    checkSound: qboolean,
    maxSeeDist: f32,
    maxHearDist: f32,
    ignoreAlert: c_int,
    mustHaveOwner: qboolean,
    minAlertLevel: c_int,
) -> c_int {
    let mut bestSoundEvent = -1;
    let mut bestSightEvent = -1;
    let mut bestSoundAlert = -1;
    let mut bestSightAlert = -1;

    if ctx.world.g_entities[0].health <= 0 {
        // player is dead
        return -1;
    }

    // get sound event
    bestSoundEvent = G_CheckSoundEvents(
        ctx,
        self_,
        maxHearDist,
        ignoreAlert,
        mustHaveOwner,
        minAlertLevel,
    );
    // get sound event alert level
    if bestSoundEvent >= 0 {
        bestSoundAlert = ctx.world.level.alertEvents[bestSoundEvent as usize].level as i32;
    }

    // get sight event
    let self_npc = ctx.entity(self_).NPC;
    if !self_npc.is_null() {
        // §2c: gNPC_t has no accessor; deref stays raw.
        bestSightEvent = G_CheckSightEvents(
            ctx,
            self_,
            unsafe { (*self_npc).stats.hfov },
            unsafe { (*self_npc).stats.vfov },
            maxSeeDist,
            ignoreAlert,
            mustHaveOwner,
            minAlertLevel,
        );
    } else {
        bestSightEvent = G_CheckSightEvents(
            ctx,
            self_,
            80,
            80,
            maxSeeDist,
            ignoreAlert,
            mustHaveOwner,
            minAlertLevel,
        );
    }
    // get sight event alert level
    if bestSightEvent >= 0 {
        bestSightAlert = ctx.world.level.alertEvents[bestSightEvent as usize].level as i32;
    }

    // return the one that has a higher alert (or sound if equal)
    if bestSightEvent >= 0 && bestSightAlert > bestSoundAlert {
        // valid best sight event, more important than the sound event
        // get the light level of the alert event for this checker
        let mut eyePoint = [0.0; 3];
        let mut sightDir = [0.0; 3];
        // get eye point
        CalcEntitySpot(ctx, Some(self_), spot_t::SPOT_HEAD_LEAN, &mut eyePoint);
        _VectorSubtract(
            ctx.world.level.alertEvents[bestSightEvent as usize].position,
            eyePoint,
            &mut sightDir,
        );
        ctx.world.level.alertEvents[bestSightEvent as usize].light =
            ctx.world.level.alertEvents[bestSightEvent as usize].addLight
                + G_GetLightLevel(
                    ctx.world.level.alertEvents[bestSightEvent as usize].position,
                    sightDir,
                );
        // return the sight event
        return bestSightEvent;
    }
    // return the sound event
    bestSoundEvent
}

/// Raven `NPC_CheckAlertEvents`.
///
/// Source: `oracle/codemp/game/NPC_senses.c:532-535`
pub fn NPC_CheckAlertEvents(
    ctx: &mut GameContext,
    checkSight: qboolean,
    checkSound: qboolean,
    ignoreAlert: c_int,
    mustHaveOwner: qboolean,
    minAlertLevel: c_int,
) -> c_int {
    let npc = ctx.world.globals.NPC;
    let npcinfo = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    // §2c: NPCInfo (gNPC_t) has no accessor; deref stays raw.
    G_CheckAlertEvents(
        ctx,
        npc_id,
        checkSight,
        checkSound,
        unsafe { (*npcinfo).stats.visrange },
        unsafe { (*npcinfo).stats.earshot },
        ignoreAlert,
        mustHaveOwner,
        minAlertLevel,
    )
}

/// Raven `G_CheckForDanger`.
///
/// Raven: FIXME: more bStates need to call this?
/// Source: `oracle/codemp/game/NPC_senses.c:537-567`
pub fn G_CheckForDanger(ctx: &mut GameContext, self_: EntityId, alertEvent: c_int) -> qboolean {
    if alertEvent == -1 {
        return 0;
    }

    if (ctx.world.level.alertEvents[alertEvent as usize].level as i32) >= AEL_DANGER as i32 {
        // run away!
        let owner = ctx.world.level.alertEvents[alertEvent as usize].owner;
        let owner_id = ctx.entity_id_of(owner);
        // §2b: owner may be an NPC (pool client); read its client pointer via the
        // entity borrow, then deref raw as Raven does.
        let owner_team = if let Some(oid) = owner_id {
            let oc = ctx.entity(oid).client;
            if !oc.is_null() {
                Some(unsafe { (*oc).playerTeam })
            } else {
                None
            }
        } else {
            None
        };

        let self_client = ctx.entity(self_).client;
        let should_flee = if let Some(team) = owner_team {
            // §19: Raven derefs `self->client->playerTeam` unconditionally here; the
            // `self->client` null guard is defensive. Source: NPC_senses.c:546.
            owner_id.is_some()
                && owner_id != Some(self_)
                && !self_client.is_null()
                && team != unsafe { (*self_client).playerTeam }
        } else {
            // Reaching here means `!owner || !owner->client`, either of which makes
            // the C `if` condition true.
            true
        };

        if should_flee {
            let self_npc = ctx.entity(self_).NPC;
            if !self_npc.is_null() {
                // §2c: gNPC_t has no accessor; deref stays raw.
                if (unsafe { (*self_npc).scriptFlags } & SCF_DONT_FLEE) != 0 {
                    // can't flee
                    return 0;
                } else {
                    NPC_StartFlee(
                        ctx,
                        owner_id,
                        ctx.world.level.alertEvents[alertEvent as usize].position,
                        ctx.world.level.alertEvents[alertEvent as usize].level as c_int,
                        3000,
                        6000,
                    );
                    return 1;
                }
            } else {
                return 1;
            }
        }
    }
    0
}

/// Raven `NPC_CheckForDanger`.
///
/// Raven: FIXME: more bStates need to call this?
/// Source: `oracle/codemp/game/NPC_senses.c:568-571`
pub fn NPC_CheckForDanger(ctx: &mut GameContext, alertEvent: c_int) -> qboolean {
    let npc = ctx.world.globals.NPC;
    G_CheckForDanger(ctx, ctx.entity_id_of(npc).unwrap(), alertEvent)
}

/// Raven `AddSoundEvent`.
///
/// Source: `oracle/codemp/game/NPC_senses.c:579-615`
pub fn AddSoundEvent(
    ctx: &mut GameContext,
    owner: Option<EntityId>,
    position: vec3_t,
    radius: f32,
    alertLevel: alertEventLevel_e,
    needLOS: qboolean,
) {
    // `alertEvent_t.owner` is still a raw `*mut gentity_t` field, so the handle is
    // materialized back to a pointer for storage (id→pointer seam bridge).
    let owner: *mut gentity_t =
        unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), owner) };

    // FIXME: Handle this in another manner?
    if ctx.world.level.numAlertEvents >= MAX_ALERT_EVENTS as c_int {
        if RemoveOldestAlert(ctx) == 0 {
            // how could that fail?
            return;
        }
    }

    if owner.is_null() && (alertLevel as i32) < AEL_DANGER as i32 {
        // allows un-owned danger alerts
        return;
    }

    _VectorCopy(
        position,
        &mut ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].position,
    );

    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].radius = radius;
    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].level = alertLevel;
    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].r#type =
        alertEventType_e::AET_SOUND;
    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].owner = owner;
    if needLOS != 0 {
        // a very low-level sound, when check this sound event, check for LOS
        ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].addLight = 1.0;
    } else {
        ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].addLight = 0.0;
    }
    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].ID =
        ctx.world.level.curAlertID;
    ctx.world.level.curAlertID += 1;
    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].timestamp =
        ctx.world.level.time;

    ctx.world.level.numAlertEvents += 1;
}

/// Raven `AddSightEvent`.
///
/// Source: `oracle/codemp/game/NPC_senses.c:623-652`
pub fn AddSightEvent(
    ctx: &mut GameContext,
    owner: Option<EntityId>,
    position: vec3_t,
    radius: f32,
    alertLevel: alertEventLevel_e,
    addLight: f32,
) {
    // `alertEvent_t.owner` is still a raw `*mut gentity_t` field, so the handle is
    // materialized back to a pointer for storage (id→pointer seam bridge).
    let owner: *mut gentity_t =
        unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), owner) };

    // FIXME: Handle this in another manner?
    if ctx.world.level.numAlertEvents >= MAX_ALERT_EVENTS as c_int {
        if RemoveOldestAlert(ctx) == 0 {
            // how could that fail?
            return;
        }
    }

    if owner.is_null() && (alertLevel as i32) < AEL_DANGER as i32 {
        // allows un-owned danger alerts
        return;
    }

    _VectorCopy(
        position,
        &mut ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].position,
    );

    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].radius = radius;
    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].level = alertLevel;
    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].r#type =
        alertEventType_e::AET_SIGHT;
    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].owner = owner;
    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].addLight = addLight;
    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].ID =
        ctx.world.level.curAlertID;
    ctx.world.level.curAlertID += 1;
    ctx.world.level.alertEvents[ctx.world.level.numAlertEvents as usize].timestamp =
        ctx.world.level.time;

    ctx.world.level.numAlertEvents += 1;
}

/// Raven `ClearPlayerAlertEvents`.
///
/// Source: `oracle/codemp/game/NPC_senses.c:660-693`
pub fn ClearPlayerAlertEvents(ctx: &mut GameContext) {
    // Raven `ALERT_CLEAR_TIME` — single-owner header, deliberately kept local
    // (not consolidated; fn-local, not importable from here).
    // Source: `oracle/codemp/game/b_local.h:164`
    pub const ALERT_CLEAR_TIME: c_int = 200;

    let cur_num_alerts = ctx.world.level.numAlertEvents;
    let mut i = 0;
    // loop through them all (max 32)
    while i < cur_num_alerts {
        // see if the event is old enough to delete
        if ctx.world.level.alertEvents[i as usize].timestamp != 0
            && ctx.world.level.alertEvents[i as usize].timestamp + ALERT_CLEAR_TIME
                < ctx.world.level.time
        {
            // this event has timed out
            // drop the count
            ctx.world.level.numAlertEvents -= 1;
            // shift the rest down
            if ctx.world.level.numAlertEvents > 0 {
                // still have more in the array
                if (i + 1) < MAX_ALERT_EVENTS as c_int {
                    // memmove shifts [i+1..MAX) down into [i..MAX-1); the final
                    // slot MAX-1 is left untouched (stale), not zeroed.
                    for j in i as usize..(MAX_ALERT_EVENTS - 1) {
                        ctx.world.level.alertEvents[j] = ctx.world.level.alertEvents[j + 1];
                    }
                }
            } else {
                // just clear this one... or should we clear the whole array?
                ctx.world.level.alertEvents[i as usize] = alertEvent_t::default();
            }
        }
        i += 1;
    }
    // make sure this never drops below zero... if it does, something very very bad happened
    assert!(ctx.world.level.numAlertEvents >= 0);

    if ctx.world.globals.eventClearTime < ctx.world.level.time {
        // this is just a 200ms debouncer so things that generate constant alerts (like corpses and missiles) add an alert every 200 ms
        ctx.world.globals.eventClearTime = ctx.world.level.time + ALERT_CLEAR_TIME;
    }
}

/// Raven `RemoveOldestAlert`.
///
/// Source: `oracle/codemp/game/NPC_senses.c:695-730`
pub fn RemoveOldestAlert(ctx: &mut GameContext) -> qboolean {
    let mut oldest_event = -1;
    let mut oldest_time = 16777216; // Q3_INFINITE
    let mut i;

    // loop through them all (max 32)
    i = 0;
    while i < ctx.world.level.numAlertEvents {
        // see if the event is old enough to delete
        if ctx.world.level.alertEvents[i as usize].timestamp < oldest_time {
            oldest_event = i;
            oldest_time = ctx.world.level.alertEvents[i as usize].timestamp;
        }
        i += 1;
    }
    if oldest_event != -1 {
        // drop the count
        ctx.world.level.numAlertEvents -= 1;
        // shift the rest down
        if ctx.world.level.numAlertEvents > 0 {
            // still have more in the array
            if (oldest_event + 1) < MAX_ALERT_EVENTS as c_int {
                // memmove shifts [oldest+1..MAX) down into [oldest..MAX-1); the
                // final slot MAX-1 is left untouched (stale), not zeroed.
                for j in (oldest_event as usize)..(MAX_ALERT_EVENTS - 1) {
                    ctx.world.level.alertEvents[j] = ctx.world.level.alertEvents[j + 1];
                }
            }
        } else {
            // just clear this one... or should we clear the whole array?
            ctx.world.level.alertEvents[oldest_event as usize] = alertEvent_t::default();
        }
    }
    // make sure this never drops below zero... if it does, something very very bad happened
    assert!(ctx.world.level.numAlertEvents >= 0);
    // return true if have room for one now
    if ctx.world.level.numAlertEvents < MAX_ALERT_EVENTS as c_int {
        1
    } else {
        0
    }
}

/// Raven `G_ClearLOS`.
///
/// Raven: Position to position.
/// Source: `oracle/codemp/game/NPC_senses.c:739-764`
pub fn G_ClearLOS(ctx: &mut GameContext, self_: EntityId, start: vec3_t, end: vec3_t) -> qboolean {
    // `self_` is unused in Raven's body (traces ignore `ENTITYNUM_NONE`).
    let _ = self_;
    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    let mut trace_count = 0;

    // FIXME: ENTITYNUM_NONE ok?
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &start as *const vec3_t,
            core::ptr::null(),
            core::ptr::null(),
            &end as *const vec3_t,
            ENTITYNUM_NONE,
            CONTENTS_OPAQUE,
        ),
    );
    while tr.fraction < 1.0 && trace_count < 3 {
        // can see through 3 panes of glass
        if (tr.entityNum as c_int) < ENTITYNUM_WORLD {
            if tr.entityNum < (MAX_GENTITIES as u32) as i16 {
                if ctx.world.g_entities[tr.entityNum as usize].r.svFlags
                    & (SVF_GLASS_BRUSH as c_int)
                    != 0
                {
                    // can see through glass, trace again, ignoring me
                    trap::Trace(
                        ctx.engine,
                        GTraceArgs::new(
                            &mut tr as *mut trace_t,
                            &tr.endpos as *const vec3_t,
                            core::ptr::null(),
                            core::ptr::null(),
                            &end as *const vec3_t,
                            tr.entityNum as c_int,
                            MASK_OPAQUE,
                        ),
                    );
                    trace_count += 1;
                    continue;
                }
            }
        }
        return 0;
    }

    if tr.fraction == 1.0 {
        1
    } else {
        0
    }
}

/// Raven `G_ClearLOS2`.
///
/// Raven: Entity to position.
/// Source: `oracle/codemp/game/NPC_senses.c:767-774`
pub fn G_ClearLOS2(
    ctx: &mut GameContext,
    self_: EntityId,
    ent: Option<EntityId>,
    end: vec3_t,
) -> qboolean {
    let mut eyes = [0.0; 3];

    CalcEntitySpot(ctx, ent, spot_t::SPOT_HEAD_LEAN, &mut eyes);

    G_ClearLOS(ctx, self_, eyes, end)
}

/// Raven `G_ClearLOS3`.
///
/// Raven: Position to entity. Look for the chest first, then the head.
/// Source: `oracle/codemp/game/NPC_senses.c:777-794`
pub fn G_ClearLOS3(
    ctx: &mut GameContext,
    self_: EntityId,
    start: vec3_t,
    ent: Option<EntityId>,
) -> qboolean {
    let mut spot = [0.0; 3];

    // Look for the chest first
    CalcEntitySpot(ctx, ent, spot_t::SPOT_ORIGIN, &mut spot);

    if G_ClearLOS(ctx, self_, start, spot) != 0 {
        return 1;
    }

    // Look for the head next
    CalcEntitySpot(ctx, ent, spot_t::SPOT_HEAD_LEAN, &mut spot);

    if G_ClearLOS(ctx, self_, start, spot) != 0 {
        return 1;
    }

    0
}

/// Raven `G_ClearLOS4`.
///
/// Raven: NPC's eyes to entity.
/// Source: `oracle/codemp/game/NPC_senses.c:797-805`
pub fn G_ClearLOS4(ctx: &mut GameContext, self_: EntityId, ent: Option<EntityId>) -> qboolean {
    let mut eyes = [0.0; 3];

    // Calculate my position
    CalcEntitySpot(ctx, Some(self_), spot_t::SPOT_HEAD_LEAN, &mut eyes);

    G_ClearLOS3(ctx, self_, eyes, ent)
}

/// Raven `G_ClearLOS5`.
///
/// Raven: NPC's eyes to position.
/// Source: `oracle/codemp/game/NPC_senses.c:808-816`
pub fn G_ClearLOS5(ctx: &mut GameContext, self_: EntityId, end: vec3_t) -> qboolean {
    let mut eyes = [0.0; 3];

    // Calculate the my position
    CalcEntitySpot(ctx, Some(self_), spot_t::SPOT_HEAD_LEAN, &mut eyes);

    G_ClearLOS(ctx, self_, eyes, end)
}

/// Raven `NPC_GetHFOVPercentage`.
///
/// Source: `oracle/codemp/game/NPC_senses.c:824-839`
pub fn NPC_GetHFOVPercentage(spot: vec3_t, from: vec3_t, facing: vec3_t, hFOV: f32) -> f32 {
    let mut deltaVector = [0.0; 3];
    let mut angles = [0.0; 3];

    _VectorSubtract(spot, from, &mut deltaVector);

    vectoangles(deltaVector, &mut angles);

    let delta = (AngleDelta(facing[1], angles[1])).abs();

    if delta > hFOV {
        0.0
    } else {
        (hFOV - delta) / hFOV
    }
}

/// Raven `NPC_GetVFOVPercentage`.
///
/// Source: `oracle/codemp/game/NPC_senses.c:847-862`
pub fn NPC_GetVFOVPercentage(spot: vec3_t, from: vec3_t, facing: vec3_t, vFOV: f32) -> f32 {
    let mut deltaVector = [0.0; 3];
    let mut angles = [0.0; 3];

    _VectorSubtract(spot, from, &mut deltaVector);

    vectoangles(deltaVector, &mut angles);

    let delta = (AngleDelta(facing[0], angles[0])).abs();

    if delta > vFOV {
        0.0
    } else {
        (vFOV - delta) / vFOV
    }
}

/// Raven `G_FindLocalInterestPoint`.
///
/// Source: `oracle/codemp/game/NPC_senses.c:871-907`
pub fn G_FindLocalInterestPoint(ctx: &mut GameContext, self_: EntityId) -> c_int {
    pub const MAX_INTEREST_DIST: f32 = 256.0 * 256.0; // 65536.0

    let mut best_point = ENTITYNUM_NONE;
    let mut best_dist = 16777216.0; // Q3_INFINITE
    let mut eyes = [0.0; 3];
    let mut diff_vec = [0.0; 3];

    let self_num = ctx.entity(self_).s.number;
    CalcEntitySpot(ctx, Some(self_), spot_t::SPOT_HEAD_LEAN, &mut eyes);
    for i in 0..ctx.world.level.numInterestPoints as usize {
        // Don't ignore portals?  If through a portal, need to look at portal!
        if trap::InPVS(
            ctx.engine,
            GInPvsArgs::new(
                &ctx.world.level.interestPoints[i].origin as *const vec3_t,
                &eyes as *const vec3_t,
            ),
        ) != 0
        {
            _VectorSubtract(
                ctx.world.level.interestPoints[i].origin,
                eyes,
                &mut diff_vec,
            );
            // C's `fabs` is the double libm function: the magnitude sum and the
            // `/2` divide evaluate in f64, so the two boundary comparisons are
            // f64. f32-throughout would diverge at the `< 48` / up-down cutoff.
            if ((diff_vec[0].abs() as f64 + diff_vec[1].abs() as f64) / 2.0) < 48.0
                && (diff_vec[2].abs() as f64)
                    > ((diff_vec[0].abs() as f64 + diff_vec[1].abs() as f64) / 2.0)
            {
                // Too close to look so far up or down
                continue;
            }
            let dist = VectorLengthSquared(diff_vec);
            // Some priority to more interesting points
            // dist -= ((int)level.interestPoints[i].lookMode * 5) * ((int)level.interestPoints[i].lookMode * 5);
            if dist < MAX_INTEREST_DIST && dist < best_dist {
                if G_ClearLineOfSight(
                    ctx,
                    eyes,
                    ctx.world.level.interestPoints[i].origin,
                    self_num,
                    MASK_OPAQUE,
                ) != 0
                {
                    best_dist = dist;
                    best_point = i as c_int;
                }
            }
        }
    }
    if best_point != ENTITYNUM_NONE
        && !ctx.world.level.interestPoints[best_point as usize]
            .target
            .is_null()
    {
        let target = unsafe {
            cstr_to_str(ctx.world.level.interestPoints[best_point as usize].target)
        };
        G_UseTargets2(ctx, Some(self_), Some(self_), Some(&target));
    }
    best_point
}

/// Raven `SP_target_interest`.
///
/// Raven: `//QUAKED target_interest (1 0.8 0.5) (-4 -4 -4) (4 4 4)` — a
/// point that a squadmate will look at if standing still. `target` fires
/// when someone looks at this thing. FIXME: rename point_interest.
/// Source: `oracle/codemp/game/NPC_senses.c:915-934`
pub fn SP_target_interest(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.level.numInterestPoints >= MAX_INTEREST_POINTS as c_int {
        // ERROR: Too many interest points, limit is MAX_INTEREST_POINTS
        Com_Printf(&format!(
            "ERROR:  Too many interest points, limit is {}\n",
            MAX_INTEREST_POINTS as c_int
        ));
        G_FreeEntity(ctx, Some(self_));
        return;
    }

    let origin = ctx.entity(self_).r.currentOrigin;
    _VectorCopy(
        origin,
        &mut ctx.world.level.interestPoints[ctx.world.level.numInterestPoints as usize].origin,
    );

    // `self->target` is now an owned `Option<String>`; the interest point's
    // `target` slot stays a `*mut c_char`, now filled from the level-lifetime
    // prefix arena via `prefix_string` (which reproduced `G_NewString`'s copy)
    // under Raven's `if (self->target && self->target[0])` non-empty guard.
    let target = ctx.entity(self_).target.clone();
    if target.as_deref().is_some_and(|s| !s.is_empty()) {
        let idx = ctx.world.level.numInterestPoints as usize;
        ctx.world.level.interestPoints[idx].target = ctx.prefix_string(target.as_deref().unwrap());
    }

    ctx.world.level.numInterestPoints += 1;

    G_FreeEntity(ctx, Some(self_));
}
