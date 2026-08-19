//! Port of `oracle/codemp/game/g_nav.c`.
//!
//! Entity reaches go through the checked `ctx.entity(id)` / `ctx.entity_mut(id)` accessors (§B5), not raw `gentity_t*` re-derives.
//! The raw derefs that remain are, by design, the ABI seam and the pool-allocated structs.
//! The ABI seam hands entity pointers to engine `trap::Nav_*`/`LinkEntity` syscalls.
//! It also threads `*mut trace_t` / `*mut navInfo_t` / `*mut c_int` out-param pointers verbatim (not `gentity_t`).
//! The pool-allocated `gclient_t` (`ent.client`) and `gNPC_t` (`ent.NPC`, `globals.NPCInfo`) structs have no arena accessor.
//! Each such deref is a tight `unsafe` block through a copied pointer value, and it is FLAGged at the site.
#![allow(non_snake_case, unused, clippy::all)]

use crate::client::gclient::gclient_t;
use crate::g_main::Com_Error;
use crate::g_misc::TAG_Add;
use crate::g_mover::{G_EntIsBreakable, G_EntIsDoor, G_EntIsRemovableUsable, G_EntIsUnlockedDoor};
use crate::g_utils::vtos;
use crate::g_utils::G_CheckInSolid;
use crate::npc::g_npc_t::gNPC_t;
use crate::npc::nav_info_s::navInfo_t;
use crate::prelude::*;
use crate::q_shared::Com_sprintf;
use crate::trap;
use crate::NPC_goal::G_BoundsOverlap;
use crate::NPC_utils::{G_ActivateBehavior, NPC_FaceEntity};
use native_string::Q_stricmp;
use native_string::strncpyz_string;

use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_NAV_ADDFAILEDEDGE::GNavAddfailededgeArgs;
use mp_abi::game::syscalls::G_NAV_ADDRAWPOINT::GNavAddrawpointArgs;
use mp_abi::game::syscalls::G_NAV_CHECKBLOCKEDEDGES::GNavCheckblockededgesArgs;
use mp_abi::game::syscalls::G_NAV_FREE::GNavFreeArgs;
use mp_abi::game::syscalls::G_NAV_GETBESTNODE::GNavGetbestnodeArgs;
use mp_abi::game::syscalls::G_NAV_GETNEARESTNODE::GNavGetnearestnodeArgs;
use mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs;
use mp_abi::game::syscalls::G_NAV_GETNUMNODES::GNavGetnumnodesArgs;
use mp_abi::game::syscalls::G_NAV_HARDCONNECT::GNavHardconnectArgs;
use mp_abi::game::syscalls::G_NAV_SETPATHSCALCULATED::GNavSetpathscalculatedArgs;
use mp_abi::game::syscalls::G_NAV_SHOWEDGES::GNavShowedgesArgs;
use mp_abi::game::syscalls::G_NAV_SHOWNODES::GNavShownodesArgs;
use mp_abi::game::syscalls::G_NAV_SHOWPATH::GNavShowpathArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;

// Raven `qboolean` is `c_int`. This port keeps the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

/// Raven `G_Line`.
///
/// Raven: For debug graphics
/// Raven: rwwFIXMEFIXME: Write these at some point for the sake of being able to debug visually
/// Source: `oracle/codemp/game/g_nav.c:11-14`
pub fn G_Line(start: vec3_t, end: vec3_t, color: vec3_t, alpha: f32) {}

/// Raven `G_Cube`.
///
/// This is a debug-graphics stub, part of the group led by `G_Line`. Its oracle body is empty and unimplemented.
/// Source: `oracle/codemp/game/g_nav.c:16-19`
pub fn G_Cube(mins: vec3_t, maxs: vec3_t, color: vec3_t, alpha: f32) {}

/// Raven `G_CubeOutline`.
///
/// This is a debug-graphics stub, part of the group led by `G_Line`. Its oracle body is empty and unimplemented.
/// Source: `oracle/codemp/game/g_nav.c:21-24`
pub fn G_CubeOutline(mins: vec3_t, maxs: vec3_t, time: c_int, color: c_uint, alpha: f32) {}

/// Raven `G_DrawEdge`.
///
/// This is a debug-graphics stub, part of the group led by `G_Line`. Its oracle body is empty and unimplemented.
/// Source: `oracle/codemp/game/g_nav.c:26-29`
pub fn G_DrawEdge(start: vec3_t, end: vec3_t, r#type: c_int) {}

/// Raven `G_DrawNode`.
///
/// This is a debug-graphics stub, part of the group led by `G_Line`. Its oracle body is empty and unimplemented.
/// Source: `oracle/codemp/game/g_nav.c:31-34`
pub fn G_DrawNode(origin: vec3_t, r#type: c_int) {}

/// Raven `G_DrawCombatPoint`.
///
/// This is a debug-graphics stub, part of the group led by `G_Line`. Its oracle body is empty and unimplemented.
/// Source: `oracle/codemp/game/g_nav.c:36-39`
pub fn G_DrawCombatPoint(origin: vec3_t, r#type: c_int) {}

/// Raven `TAG_ShowTags`.
///
/// This is a debug-graphics stub, part of the group led by `G_Line`. Its oracle body is empty and unimplemented.
/// Source: `oracle/codemp/game/g_nav.c:41-44`
pub fn TAG_ShowTags(flags: c_int) {}

/// Raven `FlyingCreature`.
///
/// Source: `oracle/codemp/game/g_nav.c:46-53`
pub fn FlyingCreature(ent: &gentity_t) -> qboolean {
    if !ent.client.is_null() {
        let client = ent.client;
        // FLAG: this gclient_t deref uses a ctx-free leaf helper with no arena accessor.
        // It stays raw.
        if unsafe { (*client).ps.gravity } <= 0 {
            return qtrue;
        }
    }
    qfalse
}

/// Raven `NPC_Blocked`.
///
/// Source: `oracle/codemp/game/g_nav.c:68-104`
pub fn NPC_Blocked(ctx: &mut GameContext, self_: EntityId, blocker: Option<EntityId>) {
    // FLAG: gNPC_t (self->NPC) has no arena accessor.
    // Its derefs stay raw.
    let npc = ctx.entity(self_).NPC;
    if npc.is_null() {
        return;
    }

    // Don't do this too often
    if unsafe { (*npc).blockedSpeechDebounceTime } > ctx.world.level.time {
        return;
    }

    // Attempt to run any blocked scripts
    if G_ActivateBehavior(ctx, Some(self_), BSET_BLOCKED as c_int) != 0 {
        return;
    }

    // Raven derefs `blocker` unconditionally from here (never NULL at call sites).
    let blocker = blocker.unwrap();

    // If this is one of our enemies, then just attack him
    let blocker_client = ctx.entity(blocker).client;
    if !blocker_client.is_null() {
        let self_client = ctx.entity(self_).client;
        // FLAG: gclient_t derefs stay raw.
        if unsafe { (*blocker_client).playerTeam == (*self_client).enemyTeam } {
            G_SetEnemy(ctx, self_, Some(blocker));
            return;
        }
    }

    // Raven's player-blocked voice-event branch body is entirely commented out (`//G_AddVoiceEvent(...)`).
    // This port carries no live behavior beyond the guard.
    let new_time = ctx.world.level.time
        + MIN_BLOCKED_SPEECH_TIME
        + (ctx.world.bg_state.rng.random() * 4000.0) as c_int;
    let blocker_num = ctx.entity(blocker).s.number;
    // FLAG: gNPC_t writes stay raw.
    unsafe {
        (*npc).blockedSpeechDebounceTime = new_time;
        (*npc).blockingEntNum = blocker_num;
    }
}

/// Raven `NPC_SetMoveGoal`.
///
/// Source: `oracle/codemp/game/g_nav.c:112-159`
pub fn NPC_SetMoveGoal(
    ctx: &mut GameContext,
    ent: EntityId,
    point: vec3_t,
    radius: c_int,
    isNavGoal: qboolean,
    combatPoint: c_int,
    targetEnt: Option<EntityId>,
) {
    // FLAG: gNPC_t (ent->NPC) has no arena accessor.
    // Its derefs stay raw.
    let npc = ctx.entity(ent).NPC;
    if npc.is_null() {
        return;
    }

    let temp_goal_id = match unsafe { (*npc).tempGoal } {
        Some(id) => id, //must still have a goal
        None => return,
    };

    // Copy the origin
    ctx.entity_mut(temp_goal_id).r.currentOrigin = point;

    // Copy the mins and maxs to the tempGoal
    let ent_mins = ctx.entity(ent).r.mins;
    ctx.entity_mut(temp_goal_id).r.mins = ent_mins;
    ctx.entity_mut(temp_goal_id).r.maxs = ent_mins;

    ctx.entity_mut(temp_goal_id).target = None;
    let ent_clipmask = ctx.entity(ent).clipmask;
    ctx.entity_mut(temp_goal_id).clipmask = ent_clipmask;
    ctx.entity_mut(temp_goal_id).flags &= !FL_NAVGOAL;
    if let Some(target) = targetEnt {
        if ctx.entity(target).waypoint >= 0 {
            let target_waypoint = ctx.entity(target).waypoint;
            ctx.entity_mut(temp_goal_id).waypoint = target_waypoint;
        } else {
            ctx.entity_mut(temp_goal_id).waypoint = WAYPOINT_NONE;
        }
    } else {
        ctx.entity_mut(temp_goal_id).waypoint = WAYPOINT_NONE;
    }
    ctx.entity_mut(temp_goal_id).noWaypointTime = 0;

    if isNavGoal != 0 {
        debug_assert!(ctx.entity(temp_goal_id).parent.is_some());
        ctx.entity_mut(temp_goal_id).flags |= FL_NAVGOAL;
    }

    ctx.entity_mut(temp_goal_id).combatPoint = combatPoint;
    ctx.entity_mut(temp_goal_id).enemy = targetEnt;

    // FLAG: gNPC_t writes stay raw.
    unsafe {
        (*npc).goalEntity = Some(temp_goal_id);
        (*npc).goalRadius = radius;
    }

    let temp_goal_ptr = ctx.entity_mut(temp_goal_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(temp_goal_ptr.cast()));
}

/// Raven `NAV_HitNavGoal`.
///
/// Source: `oracle/codemp/game/g_nav.c:167-214`
pub fn NAV_HitNavGoal(
    point: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    dest: vec3_t,
    radius: c_int,
    flying: qboolean,
) -> qboolean {
    let mut radius = radius;
    if radius & NAVGOAL_USE_RADIUS != 0 {
        radius &= !NAVGOAL_USE_RADIUS;
        if flying == 0 {
            // Allow for a little z difference
            let mut diff = [0.0f32; 3];
            crate::q_math::_VectorSubtract(point, dest, &mut diff);
            if diff[2].abs() <= 24.0 {
                diff[2] = 0.0;
            }
            return (VectorLengthSquared(diff) <= (radius * radius) as f32) as qboolean;
        } else {
            //must hit exactly
            return (DistanceSquared(dest, point) <= (radius * radius) as f32) as qboolean;
        }
    } else {
        // Construct a dummy bounding box from our radius value
        let mut dmins = [0.0f32; 3];
        let mut dmaxs = [0.0f32; 3];
        let r = radius as f32;
        dmins = [-r, -r, -r];
        dmaxs = [r, r, r];

        // Translate it
        let mut dmins2 = [0.0f32; 3];
        let mut dmaxs2 = [0.0f32; 3];
        crate::q_math::_VectorAdd(dmins, dest, &mut dmins2);
        crate::q_math::_VectorAdd(dmaxs, dest, &mut dmaxs2);

        // Translate the starting box
        let mut pmins = [0.0f32; 3];
        let mut pmaxs = [0.0f32; 3];
        crate::q_math::_VectorAdd(point, mins, &mut pmins);
        crate::q_math::_VectorAdd(point, maxs, &mut pmaxs);

        // See if they overlap
        G_BoundsOverlap(pmins, pmaxs, dmins2, dmaxs2)
    }
}

/// Raven `NAV_ClearPathToPoint`.
///
/// Source: `oracle/codemp/game/g_nav.c:222-344`
pub fn NAV_ClearPathToPoint(
    ctx: &mut GameContext,
    self_: EntityId,
    pmins: vec3_t,
    pmaxs: vec3_t,
    point: vec3_t,
    clipmask: c_int,
    okToHitEntNum: c_int,
) -> qboolean {
    let mut mins = [0.0f32; 3];
    let mut maxs = [0.0f32; 3];
    let mut clipmask = clipmask;
    let mut trace: trace_t = unsafe { core::mem::zeroed() };

    // `self_` is never mutated here. This snapshots the fields it reads.
    let self_flags = ctx.entity(self_).flags;
    let self_client = ctx.entity(self_).client;
    let self_number = ctx.entity(self_).s.number;
    let self_origin = ctx.entity(self_).r.currentOrigin;
    let self_parent = ctx.entity(self_).parent;

    // Test if they're even conceivably close to one another
    if trap::InPVS(
        ctx.engine,
        GInPvsArgs::new(&self_origin as *const vec3_t, &point as *const vec3_t),
    ) == 0
    {
        return qfalse;
    }

    if self_flags & FL_NAVGOAL != 0 {
        let parent_id = match self_parent {
            Some(id) => id,
            None => {
                // SHOULD NEVER HAPPEN!!!
                debug_assert!(self_parent.is_some());
                return qfalse;
            }
        };
        mins = ctx.entity(parent_id).r.mins;
        maxs = ctx.entity(parent_id).r.maxs;
    } else {
        crate::q_math::_VectorCopy(pmins, &mut mins);
        crate::q_math::_VectorCopy(pmaxs, &mut maxs);
    }

    if !self_client.is_null() || (self_flags & FL_NAVGOAL != 0) {
        // Clients can step up things, or if this is a navgoal check, a client will be using this info
        mins[2] += STEPSIZE;

        //don't let box get inverted
        if mins[2] > maxs[2] {
            mins[2] = maxs[2];
        }
    }

    if self_flags & FL_NAVGOAL != 0 {
        let parent_id = match self_parent {
            Some(id) => id,
            None => return qfalse,
        };
        let parent_number = ctx.entity(parent_id).s.number;
        // Trace from point to navgoal
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &point as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &self_origin as *const vec3_t,
                parent_number,
                (clipmask | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP) & !CONTENTS_BODY,
            ),
        );
        if trace.startsolid != 0 && (trace.contents & CONTENTS_BOTCLIP) != 0 {
            //started inside do not enter, so ignore them
            clipmask &= !CONTENTS_BOTCLIP;
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut trace as *mut trace_t,
                    &point as *const vec3_t,
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    &self_origin as *const vec3_t,
                    parent_number,
                    (clipmask | CONTENTS_MONSTERCLIP) & !CONTENTS_BODY,
                ),
            );
        }

        if trace.startsolid != 0 || trace.allsolid != 0 {
            return qfalse;
        }

        // Made it
        if trace.fraction == 1.0 {
            return qtrue;
        }

        if okToHitEntNum != ENTITYNUM_NONE && trace.entityNum as c_int == okToHitEntNum {
            return qtrue;
        }

        // Okay, didn't get all the way there, let's see if we got close enough:
        let parent_mins = ctx.entity(parent_id).r.mins;
        let parent_maxs = ctx.entity(parent_id).r.maxs;
        // FLAG: gNPC_t (NPCInfo) deref stays raw.
        let goal_radius = unsafe { (*ctx.world.globals.NPCInfo).goalRadius };
        let flying = FlyingCreature(ctx.entity(parent_id));
        if NAV_HitNavGoal(
            self_origin,
            parent_mins,
            parent_maxs,
            trace.endpos,
            goal_radius,
            flying,
        ) != 0
        {
            return qtrue;
        } else if ctx.world.globals.NAVDEBUG_showCollision != 0 {
            if (trace.entityNum as c_int) < ENTITYNUM_WORLD
                && ctx.world.g_entities[trace.entityNum as usize].s.eType != ET_MOVER as c_int
            {
                let blocker_mins = ctx.world.g_entities[trace.entityNum as usize].r.mins;
                let blocker_maxs = ctx.world.g_entities[trace.entityNum as usize].r.maxs;
                let blocker_origin = ctx.world.g_entities[trace.entityNum as usize]
                    .r
                    .currentOrigin;
                let mut p1 = [0.0f32; 3];
                let mut p2 = [0.0f32; 3];
                G_DrawEdge(point, trace.endpos, EDGE_PATH);
                crate::q_math::_VectorAdd(blocker_mins, blocker_origin, &mut p1);
                crate::q_math::_VectorAdd(blocker_maxs, blocker_origin, &mut p2);
                G_CubeOutline(p1, p2, FRAMETIME, 0x0000ff, 0.5);
            }
        }
    } else {
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &self_origin as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &point as *const vec3_t,
                self_number,
                clipmask | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
            ),
        );
        if trace.startsolid != 0 && (trace.contents & CONTENTS_BOTCLIP) != 0 {
            clipmask &= !CONTENTS_BOTCLIP;
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut trace as *mut trace_t,
                    &self_origin as *const vec3_t,
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    &point as *const vec3_t,
                    self_number,
                    clipmask | CONTENTS_MONSTERCLIP,
                ),
            );
        }

        if trace.startsolid == 0 && trace.allsolid == 0 && trace.fraction == 1.0 {
            return qtrue;
        }

        if okToHitEntNum != ENTITYNUM_NONE && trace.entityNum as c_int == okToHitEntNum {
            return qtrue;
        }

        if ctx.world.globals.NAVDEBUG_showCollision != 0
            && (trace.entityNum as c_int) < ENTITYNUM_WORLD
            && ctx.world.g_entities[trace.entityNum as usize].s.eType != ET_MOVER as c_int
        {
            let blocker_mins = ctx.world.g_entities[trace.entityNum as usize].r.mins;
            let blocker_maxs = ctx.world.g_entities[trace.entityNum as usize].r.maxs;
            let blocker_origin = ctx.world.g_entities[trace.entityNum as usize]
                .r
                .currentOrigin;
            let mut p1 = [0.0f32; 3];
            let mut p2 = [0.0f32; 3];
            G_DrawEdge(self_origin, trace.endpos, EDGE_PATH);
            crate::q_math::_VectorAdd(blocker_mins, blocker_origin, &mut p1);
            crate::q_math::_VectorAdd(blocker_maxs, blocker_origin, &mut p2);
            G_CubeOutline(p1, p2, FRAMETIME, 0x0000ff, 0.5);
        }
    }

    qfalse
}

/// Raven `NAV_FindClosestWaypointForEnt`.
///
/// Source: `oracle/codemp/game/g_nav.c:352-356`
pub fn NAV_FindClosestWaypointForEnt(ctx: &mut GameContext, ent: EntityId, targWp: c_int) -> c_int {
    // FIXME: Take the target into account
    let waypoint = ctx.entity(ent).waypoint;
    let ent_ptr = ctx.entity_mut(ent) as *mut gentity_t;
    trap::Nav_GetNearestNode(
        ctx.engine,
        GNavGetnearestnodeArgs::new(ent_ptr.cast(), waypoint, NF_CLEAR_PATH, targWp),
    )
}

/// Raven `NAV_FindClosestWaypointForPoint`.
///
/// Source: `oracle/codemp/game/g_nav.c:358-382`
pub fn NAV_FindClosestWaypointForPoint(
    ctx: &mut GameContext,
    ent: EntityId,
    point: vec3_t,
) -> c_int {
    // FIXME: can we make this a static ent?
    let marker_id = G_Spawn(ctx);

    G_SetOrigin(ctx.entity_mut(marker_id), point);

    let ent_mins = ctx.entity(ent).r.mins;
    ctx.entity_mut(marker_id).r.mins = ent_mins; //stepsize?
    ctx.entity_mut(marker_id).r.maxs = ent_mins; //crouching?

    let ent_clipmask = ctx.entity(ent).clipmask;
    ctx.entity_mut(marker_id).clipmask = ent_clipmask;
    ctx.entity_mut(marker_id).waypoint = WAYPOINT_NONE;

    let waypoint = ctx.entity(marker_id).waypoint;
    let marker_ptr = ctx.entity_mut(marker_id) as *mut gentity_t;
    let bestWP = trap::Nav_GetNearestNode(
        ctx.engine,
        GNavGetnearestnodeArgs::new(marker_ptr.cast(), waypoint, NF_CLEAR_PATH, WAYPOINT_NONE),
    );

    G_FreeEntity(ctx, Some(marker_id));

    bestWP
}

/// Raven `NAV_FindClosestWaypointForPoint2`.
///
/// Source: `oracle/codemp/game/g_nav.c:384-408`
pub fn NAV_FindClosestWaypointForPoint2(ctx: &mut GameContext, point: vec3_t) -> c_int {
    // FIXME: can we make this a static ent?
    let marker_id = G_Spawn(ctx);

    G_SetOrigin(ctx.entity_mut(marker_id), point);

    ctx.entity_mut(marker_id).r.mins = [-16.0, -16.0, -6.0]; // includes stepsize
    ctx.entity_mut(marker_id).r.maxs = [16.0, 16.0, 32.0];

    ctx.entity_mut(marker_id).clipmask = MASK_NPCSOLID;
    ctx.entity_mut(marker_id).waypoint = WAYPOINT_NONE;

    let waypoint = ctx.entity(marker_id).waypoint;
    let marker_ptr = ctx.entity_mut(marker_id) as *mut gentity_t;
    let bestWP = trap::Nav_GetNearestNode(
        ctx.engine,
        GNavGetnearestnodeArgs::new(marker_ptr.cast(), waypoint, NF_CLEAR_PATH, WAYPOINT_NONE),
    );

    G_FreeEntity(ctx, Some(marker_id));

    bestWP
}

/// Raven `NAV_ClearBlockedInfo`.
///
/// Source: `oracle/codemp/game/g_nav.c:416-420`
pub fn NAV_ClearBlockedInfo(self_: &mut gentity_t) {
    let npc = self_.NPC;
    // FLAG: gNPC_t deref stays raw.
    unsafe {
        (*npc).aiFlags &= !NPCAI_BLOCKED;
        (*npc).blockingEntNum = ENTITYNUM_WORLD;
    }
}

/// Raven `NAV_SetBlockedInfo`.
///
/// Source: `oracle/codemp/game/g_nav.c:428-432`
pub fn NAV_SetBlockedInfo(self_: &mut gentity_t, entId: c_int) {
    let npc = self_.NPC;
    // FLAG: gNPC_t deref stays raw.
    unsafe {
        (*npc).aiFlags |= NPCAI_BLOCKED;
        (*npc).blockingEntNum = entId;
    }
}

/// Raven `NAV_Steer`.
///
/// Source: `oracle/codemp/game/g_nav.c:440-486`
pub fn NAV_Steer(ctx: &mut GameContext, self_: EntityId, dir: vec3_t, distance: f32) -> c_int {
    let mut right_test = [0.0f32; 3];
    let mut left_test = [0.0f32; 3];
    let mut deviation = dir;
    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    let right_ang = dir[YAW] + 45.0;
    let left_ang = dir[YAW] - 45.0;

    // Get the steering angles
    deviation[YAW] = right_ang;
    AngleVectors(deviation, Some(&mut right_test), None, None);

    deviation[YAW] = left_ang;
    AngleVectors(deviation, Some(&mut left_test), None, None);

    // Find the end positions
    let (self_origin, self_clipmask) = (
        ctx.entity(self_).r.currentOrigin,
        ctx.entity(self_).clipmask,
    );
    let mut right_end = [0.0f32; 3];
    let mut left_end = [0.0f32; 3];
    crate::q_math::_VectorMA(self_origin, distance, right_test, &mut right_end);
    crate::q_math::_VectorMA(self_origin, distance, left_test, &mut left_end);

    // Draw for debug purposes
    if ctx.world.globals.NAVDEBUG_showCollision != 0 {
        G_DrawEdge(self_origin, right_end, EDGE_PATH);
        G_DrawEdge(self_origin, left_end, EDGE_PATH);
    }

    // Find the right influence
    NAV_CheckAhead(
        ctx,
        self_,
        right_end,
        &mut tr as *mut trace_t,
        self_clipmask | CONTENTS_BOTCLIP,
    );
    let right_push = -45.0 * (1.0 - tr.fraction);

    // Find the left influence
    NAV_CheckAhead(
        ctx,
        self_,
        left_end,
        &mut tr as *mut trace_t,
        self_clipmask | CONTENTS_BOTCLIP,
    );
    let left_push = 45.0 * (1.0 - tr.fraction);

    // Influence the mover to respond to the steering
    deviation = dir;
    deviation[YAW] += left_push + right_push;

    deviation[YAW] as c_int
}

/// Raven `NAV_CheckAhead`.
///
/// Source: `oracle/codemp/game/g_nav.c:494-547`
pub fn NAV_CheckAhead(
    ctx: &mut GameContext,
    self_: EntityId,
    end: vec3_t,
    trace: *mut trace_t,
    clipmask: c_int,
) -> qboolean {
    let mut clipmask = clipmask;
    // `self_` is never mutated here. This snapshots the fields it reads.
    let self_mins = ctx.entity(self_).r.mins;
    let self_maxs = ctx.entity(self_).r.maxs;
    let self_origin = ctx.entity(self_).r.currentOrigin;
    let self_number = ctx.entity(self_).s.number;

    // Offset the step height
    let mins = [self_mins[0], self_mins[1], self_mins[2] + STEPSIZE];

    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            trace,
            &self_origin as *const vec3_t,
            &mins as *const vec3_t,
            &self_maxs as *const vec3_t,
            &end as *const vec3_t,
            self_number,
            clipmask,
        ),
    );

    // `trace` is a caller-owned out-param pointer, not an entity, so its derefs stay raw.
    if unsafe { (*trace).startsolid } != 0 && (unsafe { (*trace).contents } & CONTENTS_BOTCLIP) != 0
    {
        //started inside do not enter, so ignore them
        clipmask &= !CONTENTS_BOTCLIP;
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                trace,
                &self_origin as *const vec3_t,
                &mins as *const vec3_t,
                &self_maxs as *const vec3_t,
                &end as *const vec3_t,
                self_number,
                clipmask,
            ),
        );
    }
    // Do a simple check
    if unsafe { (*trace).allsolid } == 0
        && unsafe { (*trace).startsolid } == 0
        && unsafe { (*trace).fraction } == 1.0
    {
        return qtrue;
    }

    // See if we're too far above
    if (self_origin[2] - end[2]).abs() > 48.0 {
        return qfalse;
    }

    // This is a work around
    let radius = if self_maxs[0] > self_maxs[1] {
        self_maxs[0]
    } else {
        self_maxs[1]
    };
    let dist = Distance(self_origin, end);
    let tFrac = 1.0 - (radius / dist);

    if unsafe { (*trace).fraction } >= tFrac {
        return qtrue;
    }

    // Do a special check for doors
    if (unsafe { (*trace).entityNum } as c_int) < ENTITYNUM_WORLD {
        let blocker_idx = unsafe { (*trace).entityNum } as usize;
        let classname = ctx.world.g_entities[blocker_idx].classname_str();
        let blocker_number = ctx.world.g_entities[blocker_idx].s.number;

        if !classname.is_empty() {
            if G_EntIsUnlockedDoor(ctx, blocker_number) != 0 {
                // We're too close, try and avoid the door (most likely stuck on a lip)
                if DistanceSquared(self_origin, unsafe { (*trace).endpos })
                    < MIN_DOOR_BLOCK_DIST_SQR as f32
                {
                    return qfalse;
                }
                return qtrue;
            }
        }
    }

    qfalse
}

/// Raven `NAV_TestBypass`.
///
/// Source: `oracle/codemp/game/g_nav.c:555-581`
pub fn NAV_TestBypass(
    ctx: &mut GameContext,
    self_: EntityId,
    yaw: f32,
    blocked_dist: f32,
    movedir: &mut vec3_t,
) -> qboolean {
    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    let mut avoidAngles = [0.0f32; 3];
    avoidAngles[YAW] = yaw;

    let mut block_test = [0.0f32; 3];
    AngleVectors(avoidAngles, Some(&mut block_test), None, None);
    let mut block_pos = [0.0f32; 3];
    let self_origin = ctx.entity(self_).r.currentOrigin;
    crate::q_math::_VectorMA(self_origin, blocked_dist, block_test, &mut block_pos);

    if ctx.world.globals.NAVDEBUG_showCollision != 0 {
        G_DrawEdge(self_origin, block_pos, EDGE_BLOCKED);
    }

    // See if we're clear to move in that direction
    let self_clipmask = ctx.entity(self_).clipmask;
    if NAV_CheckAhead(
        ctx,
        self_,
        block_pos,
        &mut tr as *mut trace_t,
        (self_clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
    ) != 0
    {
        crate::q_math::_VectorCopy(block_test, movedir);
        return qtrue;
    }

    qfalse
}

/// Raven `NAV_Bypass`.
///
/// Source: `oracle/codemp/game/g_nav.c:589-661`
pub fn NAV_Bypass(
    ctx: &mut GameContext,
    self_: EntityId,
    blocker: Option<EntityId>,
    blocked_dir: vec3_t,
    blocked_dist: f32,
    movedir: &mut vec3_t,
) -> qboolean {
    // Raven derefs `blocker` unconditionally (never NULL at call sites).
    let blocker = blocker.unwrap();
    let mut right = [0.0f32; 3];

    // Draw debug info if requested
    if ctx.world.globals.NAVDEBUG_showCollision != 0 {
        let self_origin = ctx.entity(self_).r.currentOrigin;
        let blocker_origin = ctx.entity(blocker).r.currentOrigin;
        G_DrawEdge(self_origin, blocker_origin, EDGE_NORMAL);
    }

    let self_angles = ctx.entity(self_).r.currentAngles;
    AngleVectors(self_angles, None, Some(&mut right), None);

    // Get the blocked direction
    let yaw = vectoyaw(blocked_dir);

    // Get the avoid radius
    // Raven computes `sqrt(a) + sqrt(b)`. The f32 products promote to f64 for the libm sqrt.
    // The sum stays in f64 and narrows to f32 once, at the store.
    // Source: `oracle/codemp/game/g_nav.c:606-607`
    let blocker_maxs = ctx.entity(blocker).r.maxs;
    let self_maxs = ctx.entity(self_).r.maxs;
    let avoidRadius =
        (((blocker_maxs[0] * blocker_maxs[0] + blocker_maxs[1] * blocker_maxs[1]) as f64).sqrt()
            + ((self_maxs[0] * self_maxs[0] + self_maxs[1] * self_maxs[1]) as f64).sqrt())
            as f32;

    // See if we're inside our avoidance radius
    let mut arcAngle = if blocked_dist <= avoidRadius {
        135.0
    } else {
        (avoidRadius / blocked_dist) * 90.0
    };

    // Check to see what dir the other guy is moving in (if any) and pick the opposite dir
    let blocker_client = ctx.entity(blocker).client;
    if !blocker_client.is_null() {
        // FLAG: gclient_t deref stays raw.
        let blocker_velocity = unsafe { (*blocker_client).ps.velocity };
        if !VectorCompare(blocker_velocity, [0.0, 0.0, 0.0]) {
            let mut blocker_movedir = [0.0f32; 3];
            VectorNormalize2(blocker_velocity, &mut blocker_movedir);
            let dot = crate::q_math::_DotProduct(blocker_movedir, blocked_dir);
            if dot < 0.35 && dot > -0.35 {
                // he's moving to the side of me
                let mut block_pos = [0.0f32; 3];
                let mut tr: trace_t = unsafe { core::mem::zeroed() };
                crate::q_math::_VectorScale(blocker_movedir, -1.0, &mut blocker_movedir);
                let self_origin = ctx.entity(self_).r.currentOrigin;
                crate::q_math::_VectorMA(
                    self_origin,
                    blocked_dist,
                    blocker_movedir,
                    &mut block_pos,
                );
                let self_clipmask = ctx.entity(self_).clipmask;
                if NAV_CheckAhead(
                    ctx,
                    self_,
                    block_pos,
                    &mut tr as *mut trace_t,
                    (self_clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
                ) != 0
                {
                    crate::q_math::_VectorCopy(blocker_movedir, movedir);
                    return qtrue;
                }
            }
        }
    }

    let dot = crate::q_math::_DotProduct(blocked_dir, right);

    // Go right on the first try if that works better
    if dot < 0.0 {
        arcAngle *= -1.0;
    }

    // Test full, best position first
    if NAV_TestBypass(
        ctx,
        self_,
        AngleNormalize360(yaw + arcAngle),
        blocked_dist,
        movedir,
    ) != 0
    {
        return qtrue;
    }

    // Try a smaller arc
    if NAV_TestBypass(
        ctx,
        self_,
        AngleNormalize360(yaw + (arcAngle * 0.5)),
        blocked_dist,
        movedir,
    ) != 0
    {
        return qtrue;
    }

    // Try the other direction
    if NAV_TestBypass(
        ctx,
        self_,
        AngleNormalize360(yaw + (arcAngle * -1.0)),
        blocked_dist,
        movedir,
    ) != 0
    {
        return qtrue;
    }

    // Try the other direction more precisely
    if NAV_TestBypass(
        ctx,
        self_,
        AngleNormalize360(yaw + ((arcAngle * -1.0) * 0.5)),
        blocked_dist,
        movedir,
    ) != 0
    {
        return qtrue;
    }

    // Unable to go around
    qfalse
}

/// Raven `NAV_MoveBlocker`.
///
/// Source: `oracle/codemp/game/g_nav.c:669-688`
pub fn NAV_MoveBlocker(self_: &mut gentity_t, shove_dir: vec3_t) -> qboolean {
    let mut temp_dir = [0.0f32; 3];
    let mut forward = [0.0f32; 3];

    vectoangles(shove_dir, &mut temp_dir);

    temp_dir[YAW] += 45.0;
    AngleVectors(temp_dir, Some(&mut forward), None, None);

    let client = self_.client;
    // FLAG: gclient_t deref stays raw.
    unsafe {
        crate::q_math::_VectorScale(forward, SHOVE_SPEED as f32, &mut (*client).ps.velocity);
        (*client).ps.velocity[2] += SHOVE_LIFT as f32;
    }

    qtrue
}

/// Raven `NAV_ResolveBlock`.
///
/// Source: `oracle/codemp/game/g_nav.c:696-707`
pub fn NAV_ResolveBlock(
    ctx: &mut GameContext,
    self_: EntityId,
    blocker: Option<EntityId>,
    blocked_dir: vec3_t,
) -> qboolean {
    // Raven derefs `blocker` unconditionally (never NULL at call sites).
    let blocker = blocker.unwrap();
    // Stop double waiting
    // FLAG: gNPC_t (blocker->NPC) deref stays raw.
    let blocker_npc = ctx.entity(blocker).NPC;
    if !blocker_npc.is_null() {
        let self_number = ctx.entity(self_).s.number;
        if unsafe { (*blocker_npc).blockingEntNum } == self_number {
            return qtrue;
        }
    }

    // For now, just complain about it
    NPC_Blocked(ctx, self_, Some(blocker));
    NPC_FaceEntity(ctx, Some(blocker), qtrue);

    qfalse
}

/// Raven `NAV_TrueCollision`.
///
/// Source: `oracle/codemp/game/g_nav.c:715-750`
pub fn NAV_TrueCollision(
    self_: &gentity_t,
    blocker: &gentity_t,
    movedir: vec3_t,
    blocked_dir: &mut vec3_t,
) -> qboolean {
    // TODO: Handle all ents
    if blocker.client.is_null() {
        return qfalse;
    }

    let self_client = self_.client;
    let mut velocityDir = [0.0f32; 3];
    // Get the player's move direction and speed
    // FLAG: this gclient_t deref uses a ctx-free leaf helper.
    // It stays raw.
    let speed = VectorNormalize2(unsafe { (*self_client).ps.velocity }, &mut velocityDir);

    // See if it's even feasible
    let dot = crate::q_math::_DotProduct(movedir, velocityDir);

    if dot < 0.85 {
        return qfalse;
    }

    let mut testPos = [0.0f32; 3];
    crate::q_math::_VectorMA(
        self_.r.currentOrigin,
        speed * FRAMETIME as f32,
        velocityDir,
        &mut testPos,
    );

    let mut tmins = [0.0f32; 3];
    let mut tmaxs = [0.0f32; 3];
    crate::q_math::_VectorAdd(blocker.r.currentOrigin, blocker.r.mins, &mut tmins);
    crate::q_math::_VectorAdd(blocker.r.currentOrigin, blocker.r.maxs, &mut tmaxs);

    let mut ptmins = [0.0f32; 3];
    let mut ptmaxs = [0.0f32; 3];
    crate::q_math::_VectorAdd(testPos, self_.r.mins, &mut ptmins);
    crate::q_math::_VectorAdd(testPos, self_.r.maxs, &mut ptmaxs);

    if G_BoundsOverlap(ptmins, ptmaxs, tmins, tmaxs) != 0 {
        crate::q_math::_VectorCopy(velocityDir, blocked_dir);
        return qtrue;
    }

    qfalse
}

/// Raven `NAV_StackedCanyon`.
///
/// Source: `oracle/codemp/game/g_nav.c:758-816`
pub fn NAV_StackedCanyon(
    ctx: &mut GameContext,
    self_: EntityId,
    blocker: Option<EntityId>,
    pathDir: vec3_t,
) -> qboolean {
    // Raven derefs `blocker` unconditionally (never NULL at call sites).
    let blocker = blocker.unwrap();
    let mut perp = [0.0f32; 3];
    let mut cross = [0.0f32; 3];
    let mut test = [0.0f32; 3];
    let mut extraClip = CONTENTS_BOTCLIP;
    let mut tr: trace_t = unsafe { core::mem::zeroed() };

    PerpendicularVector(&mut perp, pathDir);
    CrossProduct(pathDir, perp, &mut cross);

    // Raven computes `sqrt(a) + sqrt(b)`. The f32 products promote to f64 for the libm sqrt.
    // The sum stays in f64 and narrows to f32 once, at the store.
    // Source: `oracle/codemp/game/g_nav.c:768-769`
    let blocker_maxs = ctx.entity(blocker).r.maxs;
    let self_maxs = ctx.entity(self_).r.maxs;
    let avoidRadius =
        (((blocker_maxs[0] * blocker_maxs[0] + blocker_maxs[1] * blocker_maxs[1]) as f64).sqrt()
            + ((self_maxs[0] * self_maxs[0] + self_maxs[1] * self_maxs[1]) as f64).sqrt())
            as f32;

    // `self_` is never mutated below. This snapshots the trace inputs.
    let self_mins = ctx.entity(self_).r.mins;
    let self_number = ctx.entity(self_).s.number;
    let self_clipmask = ctx.entity(self_).clipmask;
    let blocker_origin = ctx.entity(blocker).r.currentOrigin;

    crate::q_math::_VectorMA(blocker_origin, avoidRadius, cross, &mut test);

    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &test as *const vec3_t,
            &self_mins as *const vec3_t,
            &self_maxs as *const vec3_t,
            &test as *const vec3_t,
            self_number,
            self_clipmask | extraClip,
        ),
    );
    if tr.startsolid != 0 && (tr.contents & CONTENTS_BOTCLIP) != 0 {
        extraClip &= !CONTENTS_BOTCLIP;
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &test as *const vec3_t,
                &self_mins as *const vec3_t,
                &self_maxs as *const vec3_t,
                &test as *const vec3_t,
                self_number,
                self_clipmask | extraClip,
            ),
        );
    }

    if ctx.world.globals.NAVDEBUG_showCollision != 0 {
        let mut mins = [0.0f32; 3];
        let mut maxs = [0.0f32; 3];
        let RED = [1.0f32, 0.0, 0.0];
        crate::q_math::_VectorAdd(test, self_mins, &mut mins);
        crate::q_math::_VectorAdd(test, self_maxs, &mut maxs);
        G_Cube(mins, maxs, RED, 0.25);
    }

    if tr.startsolid == 0 && tr.allsolid == 0 {
        return qfalse;
    }

    crate::q_math::_VectorMA(blocker_origin, -avoidRadius, cross, &mut test);

    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &test as *const vec3_t,
            &self_mins as *const vec3_t,
            &self_maxs as *const vec3_t,
            &test as *const vec3_t,
            self_number,
            self_clipmask | extraClip,
        ),
    );
    if tr.startsolid != 0 && (tr.contents & CONTENTS_BOTCLIP) != 0 {
        extraClip &= !CONTENTS_BOTCLIP;
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &test as *const vec3_t,
                &self_mins as *const vec3_t,
                &self_maxs as *const vec3_t,
                &test as *const vec3_t,
                self_number,
                self_clipmask | extraClip,
            ),
        );
    }

    if tr.startsolid == 0 && tr.allsolid == 0 {
        return qfalse;
    }

    if ctx.world.globals.NAVDEBUG_showCollision != 0 {
        let mut mins = [0.0f32; 3];
        let mut maxs = [0.0f32; 3];
        let RED = [1.0f32, 0.0, 0.0];
        crate::q_math::_VectorAdd(test, self_mins, &mut mins);
        crate::q_math::_VectorAdd(test, self_maxs, &mut maxs);
        G_Cube(mins, maxs, RED, 0.25);
    }

    qtrue
}

/// Raven `NAV_ResolveEntityCollision`.
///
/// Source: `oracle/codemp/game/g_nav.c:824-865`
pub fn NAV_ResolveEntityCollision(
    ctx: &mut GameContext,
    self_: EntityId,
    blocker: Option<EntityId>,
    movedir: &mut vec3_t,
    pathDir: vec3_t,
) -> qboolean {
    // `movedir` is threaded as `&mut` (a Raven out-param).
    // `NAV_Bypass` writes the avoid-direction back for `NAV_AvoidCollision` to copy into `info->direction`.
    // Raven derefs `blocker` unconditionally (never NULL at call sites).
    let blocker = blocker.unwrap();
    let mut blocked_dir = [0.0f32; 3];

    let blocker_number = ctx.entity(blocker).s.number;

    // Doors are ignored
    if G_EntIsUnlockedDoor(ctx, blocker_number) != 0 {
        let self_origin = ctx.entity(self_).r.currentOrigin;
        let blocker_origin = ctx.entity(blocker).r.currentOrigin;
        if DistanceSquared(self_origin, blocker_origin) > MIN_DOOR_BLOCK_DIST_SQR as f32 {
            return qtrue;
        }
    }

    let blocker_origin = ctx.entity(blocker).r.currentOrigin;
    let self_origin = ctx.entity(self_).r.currentOrigin;
    crate::q_math::_VectorSubtract(blocker_origin, self_origin, &mut blocked_dir);
    let blocked_dist = VectorNormalize(&mut blocked_dir);

    // See if we can get around the blocker at all (only for player!)
    if blocker_number == 0 && NAV_StackedCanyon(ctx, self_, Some(blocker), pathDir) != 0 {
        NPC_Blocked(ctx, self_, Some(blocker));
        NPC_FaceEntity(ctx, Some(blocker), qtrue);
        return qfalse;
    }

    // First, attempt to walk around the blocker
    if NAV_Bypass(
        ctx,
        self_,
        Some(blocker),
        blocked_dir,
        blocked_dist,
        movedir,
    ) != 0
    {
        return qtrue;
    }

    // Second, attempt to calculate a good move position for the blocker
    if NAV_ResolveBlock(ctx, self_, Some(blocker), blocked_dir) != 0 {
        return qtrue;
    }

    qfalse
}

/// Raven `NAV_TestForBlocked`.
///
/// Source: `oracle/codemp/game/g_nav.c:873-894`
pub fn NAV_TestForBlocked(
    ctx: &mut GameContext,
    self_: EntityId,
    goal: Option<EntityId>,
    blocker: Option<EntityId>,
    distance: f32,
    flags: *mut c_int,
) -> qboolean {
    let goal = match goal {
        Some(g) => g,
        None => return qfalse,
    };
    // Raven derefs `blocker` unconditionally (never NULL at call sites).
    let blocker = blocker.unwrap();

    if ctx.entity(blocker).s.eType == ET_ITEM as c_int {
        return qfalse;
    }

    let blocker_origin = ctx.entity(blocker).r.currentOrigin;
    let blocker_mins = ctx.entity(blocker).r.mins;
    let blocker_maxs = ctx.entity(blocker).r.maxs;
    let goal_origin = ctx.entity(goal).r.currentOrigin;
    if NAV_HitNavGoal(
        blocker_origin,
        blocker_mins,
        blocker_maxs,
        goal_origin,
        12,
        qfalse,
    ) != 0
    {
        // `flags` is a caller-owned out-param pointer, not an entity, so its deref stays raw.
        unsafe {
            *flags |= NIF_BLOCKED;
        }

        if distance <= MIN_STOP_DIST as f32 {
            NPC_Blocked(ctx, self_, Some(blocker));
            NPC_FaceEntity(ctx, Some(blocker), qtrue);
            return qtrue;
        }
    }

    qfalse
}

/// Raven `NAV_AvoidCollision`.
///
/// Source: `oracle/codemp/game/g_nav.c:902-963`
pub fn NAV_AvoidCollision(
    ctx: &mut GameContext,
    self_: EntityId,
    goal: Option<EntityId>,
    info: *mut navInfo_t,
) -> qboolean {
    // `self_` is a plain `EntityId`, so it is never NULL.
    // Raven's `!self_` guard below is vacuous because the body already reads `self_`'s origin, so this port keeps it as an always-true test.
    // `info` is a caller-owned `*mut navInfo_t` out-param, not an entity, so its derefs stay raw.
    let mut movedir = [0.0f32; 3];
    let mut movepos = [0.0f32; 3];

    // Clear our block info for this frame
    let npc_id = ctx.entity_id_of(ctx.world.globals.NPC).unwrap();
    NAV_ClearBlockedInfo(ctx.entity_mut(npc_id));

    // Cap our distance
    if unsafe { (*info).distance } > MAX_COLL_AVOID_DIST as f32 {
        unsafe {
            (*info).distance = MAX_COLL_AVOID_DIST as f32;
        }
    }

    // Get an end position
    let self_origin = ctx.entity(self_).r.currentOrigin;
    unsafe {
        crate::q_math::_VectorMA(
            self_origin,
            (*info).distance,
            (*info).direction,
            &mut movepos,
        );
        crate::q_math::_VectorCopy((*info).direction, &mut movedir);
    }

    // FLAG: gNPC_t (self_->NPC) deref stays raw.
    let npc = ctx.entity(self_).NPC;
    if !npc.is_null() {
        if unsafe { (*npc).aiFlags } & NPCAI_NO_COLL_AVOID != 0 {
            // pretend there's no-one in the way
            return qtrue;
        }
    }
    // Now test against entities
    if NAV_CheckAhead(
        ctx,
        self_,
        movepos,
        unsafe { &mut (*info).trace as *mut trace_t },
        CONTENTS_BODY,
    ) == 0
    {
        // Get the blocker
        let blocker_idx = unsafe { (*info).trace.entityNum } as usize;
        let blocker_ptr = &mut ctx.world.g_entities[blocker_idx] as *mut gentity_t;
        unsafe {
            (*info).blocker = blocker_ptr;
            (*info).flags |= NIF_COLLISION;
        }
        let blocker_id = ctx.entity_id_of(blocker_ptr);

        // Ok to hit our goal entity
        if goal == blocker_id {
            return qtrue;
        }

        // Test for blocking by standing on goal
        let distance = unsafe { (*info).distance };
        if NAV_TestForBlocked(ctx, self_, goal, blocker_id, distance, unsafe {
            &mut (*info).flags as *mut c_int
        }) != 0
        {
            return qfalse;
        }

        // If the above function said we're blocked, don't do the extra checks
        if unsafe { (*info).flags } & NIF_BLOCKED != 0 {
            return qtrue;
        }

        // See if we can get that entity to move out of our way
        let path_direction = unsafe { (*info).pathDirection };
        if NAV_ResolveEntityCollision(ctx, self_, blocker_id, &mut movedir, path_direction) == 0 {
            return qfalse;
        }

        unsafe {
            crate::q_math::_VectorCopy(movedir, &mut (*info).direction);
        }

        return qtrue;
    }

    // Our path is clear, just move there
    if ctx.world.globals.NAVDEBUG_showCollision != 0 {
        let self_origin = ctx.entity(self_).r.currentOrigin;
        G_DrawEdge(self_origin, movepos, EDGE_PATH);
    }

    qtrue
}

/// Raven `NAV_TestBestNode`.
///
/// Source: `oracle/codemp/game/g_nav.c:971-1075`
pub fn NAV_TestBestNode(
    ctx: &mut GameContext,
    self_: EntityId,
    startID: c_int,
    endID: c_int,
    failEdge: qboolean,
) -> c_int {
    let mut end = [0.0f32; 3];
    let mut trace: trace_t = unsafe { core::mem::zeroed() };
    let npc_id = ctx.entity_id_of(ctx.world.globals.NPC).unwrap();
    let mut clipmask = (ctx.entity(npc_id).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP;

    // get the position for the test choice
    trap::Nav_GetNodePosition(
        ctx.engine,
        GNavGetnodepositionArgs::new(endID, &mut end as *mut vec3_t),
    );

    // `self_` is never mutated here. This snapshots the fields it reads.
    let self_mins = ctx.entity(self_).r.mins;
    let self_maxs = ctx.entity(self_).r.maxs;
    let self_origin = ctx.entity(self_).r.currentOrigin;
    let self_number = ctx.entity(self_).s.number;
    let self_weapon = ctx.entity(self_).s.weapon;

    // Offset the step height
    let mins = [self_mins[0], self_mins[1], self_mins[2] + STEPSIZE];

    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut trace as *mut trace_t,
            &self_origin as *const vec3_t,
            &mins as *const vec3_t,
            &self_maxs as *const vec3_t,
            &end as *const vec3_t,
            self_number,
            clipmask,
        ),
    );

    if trace.startsolid != 0 && (trace.contents & CONTENTS_BOTCLIP) != 0 {
        clipmask &= !CONTENTS_BOTCLIP;
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &self_origin as *const vec3_t,
                &mins as *const vec3_t,
                &self_maxs as *const vec3_t,
                &end as *const vec3_t,
                self_number,
                clipmask,
            ),
        );
    }
    // Do a simple check
    if trace.allsolid == 0 && trace.startsolid == 0 && trace.fraction == 1.0 {
        return endID;
    }

    // See if we're too far above
    if self_weapon != WP_SABER as c_int && (self_origin[2] - end[2]).abs() > 48.0 {
        // too far above
    } else {
        // This is a work around
        let radius = if self_maxs[0] > self_maxs[1] {
            self_maxs[0]
        } else {
            self_maxs[1]
        };
        let dist = Distance(self_origin, end);
        let tFrac = 1.0 - (radius / dist);

        if trace.fraction >= tFrac {
            // it's clear
            return endID;
        }
    }

    // Do a special check for doors
    if (trace.entityNum as c_int) < ENTITYNUM_WORLD {
        let blocker_idx = trace.entityNum as usize;
        let classname = ctx.world.g_entities[blocker_idx].classname_str();

        if !classname.is_empty() {
            let blocker_number = ctx.world.g_entities[blocker_idx].s.number;
            // special case: doors are architecture, but are dynamic, like entitites
            if G_EntIsUnlockedDoor(ctx, blocker_number) != 0 {
                // it's unlocked, go for it
                if DistanceSquared(self_origin, trace.endpos) < MIN_DOOR_BLOCK_DIST_SQR as f32 {
                    return startID;
                }
                if self_weapon != WP_SABER as c_int && (self_origin[2] - end[2]).abs() > 48.0 {
                    // too far above
                } else {
                    return endID;
                }
            } else if G_EntIsDoor(ctx, blocker_number) != 0 {
                // a locked door!
                if failEdge != 0 {
                    trap::Nav_AddFailedEdge(
                        ctx.engine,
                        GNavAddfailededgeArgs::new(self_number, startID, endID),
                    );
                }
            } else if G_EntIsBreakable(ctx, blocker_number) != 0 {
                if failEdge != 0 {
                    trap::Nav_AddFailedEdge(
                        ctx.engine,
                        GNavAddfailededgeArgs::new(self_number, startID, endID),
                    );
                }
            } else if G_EntIsRemovableUsable(ctx, blocker_number) != 0 {
                if failEdge != 0 {
                    trap::Nav_AddFailedEdge(
                        ctx.engine,
                        GNavAddfailededgeArgs::new(self_number, startID, endID),
                    );
                }
            } else {
                let targetname = ctx.world.g_entities[blocker_idx].targetname_str();
                let solid = ctx.world.g_entities[blocker_idx].s.solid;
                let contents = ctx.world.g_entities[blocker_idx].r.contents;
                if targetname.is_some()
                    && solid == SOLID_BMODEL as c_int
                    && ((contents & CONTENTS_MONSTERCLIP) != 0
                        || (contents & CONTENTS_BOTCLIP) != 0)
                {
                    if failEdge != 0 {
                        trap::Nav_AddFailedEdge(
                            ctx.engine,
                            GNavAddfailededgeArgs::new(self_number, startID, endID),
                        );
                    }
                }
            }
        }
    }
    // path is blocked
    // use the fallback choice
    startID
}

/// Raven `NAV_GetNearestNode`.
///
/// Source: `oracle/codemp/game/g_nav.c:1083-1086`
pub fn NAV_GetNearestNode(ctx: &mut GameContext, self_: EntityId, lastNode: c_int) -> c_int {
    let self_ptr = ctx.entity_mut(self_) as *mut gentity_t;
    trap::Nav_GetNearestNode(
        ctx.engine,
        GNavGetnearestnodeArgs::new(self_ptr.cast(), lastNode, NF_CLEAR_PATH, WAYPOINT_NONE),
    )
}

/// Raven `NAV_MicroError`.
///
/// Source: `oracle/codemp/game/g_nav.c:1094-1105`
pub fn NAV_MicroError(ctx: &mut GameContext, start: vec3_t, end: vec3_t) -> qboolean {
    if VectorCompare(start, end) {
        let npc_id = ctx.entity_id_of(ctx.world.globals.NPC).unwrap();
        let npc_origin = ctx.entity(npc_id).r.currentOrigin;
        if DistanceSquared(npc_origin, start) < (8.0 * 8.0) {
            return qtrue;
        }
    }

    qfalse
}

/// Raven `NAV_MoveToGoal`.
///
/// Source: `oracle/codemp/game/g_nav.c:1113-1212`
pub fn NAV_MoveToGoal(ctx: &mut GameContext, self_: EntityId, info: *mut navInfo_t) -> c_int {
    // `info` is a caller-owned `*mut navInfo_t` out-param, not an entity, so its derefs stay raw.
    let mut bestNode: c_int;
    let mut origin = [0.0f32; 3];
    let mut end = [0.0f32; 3];

    // FLAG: gNPC_t (self_->NPC) deref stays raw.
    let npc = ctx.entity(self_).NPC;
    // Must have a goal entity to move there
    let goal_id = match unsafe { (*npc).goalEntity } {
        Some(id) => id,
        None => return WAYPOINT_NONE,
    };

    // Check special player optimizations
    if ctx.entity(goal_id).s.number == 0 {
        // If we couldn't find the point, then we won't be able to this turn
        if ctx.entity(goal_id).waypoint == WAYPOINT_NONE {
            return WAYPOINT_NONE;
        }
        // NOTENOTE: Otherwise trust this waypoint for the whole frame (reduce all unnecessary calculations)
    } else {
        // Find the target's waypoint
        let goal_waypoint = ctx.entity(goal_id).waypoint;
        let nn = NAV_GetNearestNode(ctx, goal_id, goal_waypoint);
        ctx.entity_mut(goal_id).waypoint = nn;
        if ctx.entity(goal_id).waypoint == WAYPOINT_NONE {
            return WAYPOINT_NONE;
        }
    }

    // Find our waypoint
    let self_last_waypoint = ctx.entity(self_).lastWaypoint;
    let nn = NAV_GetNearestNode(ctx, self_, self_last_waypoint);
    ctx.entity_mut(self_).waypoint = nn;
    if ctx.entity(self_).waypoint == WAYPOINT_NONE {
        return WAYPOINT_NONE;
    }

    let self_waypoint = ctx.entity(self_).waypoint;
    let goal_waypoint = ctx.entity(goal_id).waypoint;
    bestNode = trap::Nav_GetBestNode(
        ctx.engine,
        GNavGetbestnodeArgs::new(self_waypoint, goal_waypoint, NODE_NONE),
    );

    if bestNode == WAYPOINT_NONE {
        if ctx.world.globals.NAVDEBUG_showEnemyPath != 0 {
            let mut torigin = [0.0f32; 3];
            let goal_waypoint = ctx.entity(goal_id).waypoint;
            trap::Nav_GetNodePosition(
                ctx.engine,
                GNavGetnodepositionArgs::new(goal_waypoint, &mut torigin as *mut vec3_t),
            );
            let self_waypoint = ctx.entity(self_).waypoint;
            trap::Nav_GetNodePosition(
                ctx.engine,
                GNavGetnodepositionArgs::new(self_waypoint, &mut origin as *mut vec3_t),
            );

            G_DrawNode(torigin, NODE_GOAL);
            G_DrawNode(origin, NODE_GOAL);
            let goal_origin = ctx.entity(goal_id).r.currentOrigin;
            G_DrawNode(goal_origin, NODE_START);
        }

        return WAYPOINT_NONE;
    }

    // Check this node
    let goal_waypoint = ctx.entity(goal_id).waypoint;
    bestNode = NAV_TestBestNode(ctx, self_, bestNode, goal_waypoint, qfalse);

    // Get this position
    trap::Nav_GetNodePosition(
        ctx.engine,
        GNavGetnodepositionArgs::new(bestNode, &mut origin as *mut vec3_t),
    );
    let self_waypoint = ctx.entity(self_).waypoint;
    trap::Nav_GetNodePosition(
        ctx.engine,
        GNavGetnodepositionArgs::new(self_waypoint, &mut end as *mut vec3_t),
    );

    // Test the path connection from our current position to the best node
    let self_clipmask = ctx.entity(self_).clipmask;
    if NAV_CheckAhead(
        ctx,
        self_,
        origin,
        unsafe { &mut (*info).trace as *mut trace_t },
        (self_clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
    ) == 0
    {
        // First attempt to move to the closest point on the line between the waypoints
        let mut new_origin = [0.0f32; 3];
        let self_origin = ctx.entity(self_).r.currentOrigin;
        G_FindClosestPointOnLineSegment(origin, end, self_origin, &mut new_origin);
        origin = new_origin;

        // See if we can go there
        let self_clipmask = ctx.entity(self_).clipmask;
        if NAV_CheckAhead(
            ctx,
            self_,
            origin,
            unsafe { &mut (*info).trace as *mut trace_t },
            (self_clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
        ) == 0
        {
            // Just move towards our current waypoint
            bestNode = ctx.entity(self_).waypoint;
            trap::Nav_GetNodePosition(
                ctx.engine,
                GNavGetnodepositionArgs::new(bestNode, &mut origin as *mut vec3_t),
            );
        }
    }

    // Setup our new move information
    let self_origin = ctx.entity(self_).r.currentOrigin;
    unsafe {
        crate::q_math::_VectorSubtract(origin, self_origin, &mut (*info).direction);
        (*info).distance = VectorNormalize(&mut (*info).direction);

        crate::q_math::_VectorSubtract(end, origin, &mut (*info).pathDirection);
        VectorNormalize(&mut (*info).pathDirection);
    }

    // Draw any debug info, if requested
    if ctx.world.globals.NAVDEBUG_showEnemyPath != 0 {
        let mut dest = [0.0f32; 3];
        let mut start = [0.0f32; 3];

        let goal_waypoint = ctx.entity(goal_id).waypoint;
        trap::Nav_GetNodePosition(
            ctx.engine,
            GNavGetnodepositionArgs::new(goal_waypoint, &mut dest as *mut vec3_t),
        );
        trap::Nav_GetNodePosition(
            ctx.engine,
            GNavGetnodepositionArgs::new(bestNode, &mut start as *mut vec3_t),
        );

        G_DrawNode(start, NODE_START);
        G_DrawNode(dest, NODE_GOAL);
        let self_waypoint = ctx.entity(self_).waypoint;
        let goal_waypoint = ctx.entity(goal_id).waypoint;
        trap::Nav_ShowPath(
            ctx.engine,
            GNavShowpathArgs::new(self_waypoint, goal_waypoint),
        );
    }

    bestNode
}

// `DEFAULT_MINS_2` and `DEFAULT_MAXS_2` are canonical in `mp_bg::public::viewheight`.
// This casts them here (from `c_int`) to match the `vec3_t` components they seed.
// Source: `oracle/codemp/game/bg_public.h:41-42`
const DEFAULT_MINS_2: f32 = mp_bg::public::viewheight::DEFAULT_MINS_2 as f32;
const DEFAULT_MAXS_2: f32 = mp_bg::public::viewheight::DEFAULT_MAXS_2 as f32;
// Raven's `CROUCH_MAXS_2` (`bg_public.h`) stays file-local, per the established precedent. It has no canonical crate-wide home yet.
const CROUCH_MAXS_2: f32 = 16.0;

/// Raven `waypoint_testDirection`.
///
/// Source: `oracle/codemp/game/g_nav.c:1220-1243`
pub fn waypoint_testDirection(
    ctx: &mut GameContext,
    origin: vec3_t,
    yaw: f32,
    minDist: c_uint,
) -> c_uint {
    let mut trace_dir = [0.0f32; 3];
    let mut test_pos = [0.0f32; 3];
    let maxs = [15.0f32, 15.0, DEFAULT_MAXS_2];
    let mins = [-15.0f32, -15.0, DEFAULT_MINS_2 + STEPSIZE];
    let angles = [0.0f32, yaw, 0.0];
    let mut tr: trace_t = unsafe { core::mem::zeroed() };

    AngleVectors(angles, Some(&mut trace_dir), None, None);

    crate::q_math::_VectorMA(origin, minDist as f32, trace_dir, &mut test_pos);

    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &origin as *const vec3_t,
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            &test_pos as *const vec3_t,
            ENTITYNUM_NONE,
            CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
        ),
    );

    (minDist as f32 * tr.fraction) as c_uint
}

/// Raven `waypoint_getRadius`.
///
/// Source: `oracle/codemp/game/g_nav.c:1251-1266`
pub fn waypoint_getRadius(ctx: &mut GameContext, ent: EntityId) -> c_uint {
    let mut minDist: c_uint = (MAX_RADIUS_CHECK + 1) as c_uint;

    for i in 0..YAW_ITERATIONS {
        let ent_origin = ctx.entity(ent).r.currentOrigin;
        let dist = waypoint_testDirection(
            ctx,
            ent_origin,
            (360.0 / YAW_ITERATIONS as f32) * i as f32,
            minDist,
        );
        if dist < minDist {
            minDist = dist;
        }
    }

    minDist
}

/// Raven `SP_waypoint`.
///
/// Source: `oracle/codemp/game/g_nav.c:1275-1313`
pub fn SP_waypoint(ctx: &mut GameContext, ent: EntityId) {
    if ctx.world.globals.navCalculatePaths != 0 {
        ctx.entity_mut(ent).r.mins = [-15.0, -15.0, DEFAULT_MINS_2];
        ctx.entity_mut(ent).r.maxs = [15.0, 15.0, DEFAULT_MAXS_2];

        ctx.entity_mut(ent).r.contents = CONTENTS_TRIGGER;
        ctx.entity_mut(ent).clipmask = MASK_DEADSOLID;

        let ent_ptr = ctx.entity_mut(ent) as *mut gentity_t;
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));

        ctx.entity_mut(ent).count = -1;
        ctx.ent_set(ent, PrefixSet::ClassnameStatic(c"waypoint"));

        if (ctx.entity(ent).spawnflags & 1) == 0 && G_CheckInSolid(ctx, ent, qtrue) != 0 {
            // if not SOLID_OK, and in solid
            ctx.entity_mut(ent).r.maxs[2] = CROUCH_MAXS_2;
            if G_CheckInSolid(ctx, ent, qtrue) != 0 {
                let targetname = ctx.entity(ent).targetname_str();
                let ent_origin = ctx.entity(ent).r.currentOrigin;
                let s = format!(
                    "ERROR: Waypoint {} at {} in solid!\n",
                    targetname.as_deref().unwrap_or_default(),
                    vtos(ctx, ent_origin)
                );
                Com_Printf(&s);
                debug_assert!(false, "Waypoint in solid!");
                G_FreeEntity(ctx, Some(ent));
                return;
            }
        }

        let radius = waypoint_getRadius(ctx, ent);

        let ent_origin = ctx.entity(ent).r.currentOrigin;
        let ent_spawnflags = ctx.entity(ent).spawnflags;
        let health = trap::Nav_AddRawPoint(
            ctx.engine,
            GNavAddrawpointArgs::new(
                &ent_origin as *const vec3_t,
                ent_spawnflags,
                radius as c_int,
            ),
        );
        ctx.entity_mut(ent).health = health;
        NAV_StoreWaypoint(ctx, ent);
        G_FreeEntity(ctx, Some(ent));
        return;
    }

    G_FreeEntity(ctx, Some(ent));
}

/// Raven `SP_waypoint_small`.
///
/// Source: `oracle/codemp/game/g_nav.c:1318-1352`
pub fn SP_waypoint_small(ctx: &mut GameContext, ent: EntityId) {
    if ctx.world.globals.navCalculatePaths != 0 {
        ctx.entity_mut(ent).r.mins = [-2.0, -2.0, DEFAULT_MINS_2];
        ctx.entity_mut(ent).r.maxs = [2.0, 2.0, DEFAULT_MAXS_2];

        ctx.entity_mut(ent).r.contents = CONTENTS_TRIGGER;
        ctx.entity_mut(ent).clipmask = MASK_DEADSOLID;

        let ent_ptr = ctx.entity_mut(ent) as *mut gentity_t;
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));

        ctx.entity_mut(ent).count = -1;
        ctx.ent_set(ent, PrefixSet::ClassnameStatic(c"waypoint"));

        if (ctx.entity(ent).spawnflags & 1) == 0 && G_CheckInSolid(ctx, ent, qtrue) != 0 {
            ctx.entity_mut(ent).r.maxs[2] = CROUCH_MAXS_2;
            if G_CheckInSolid(ctx, ent, qtrue) != 0 {
                let targetname = ctx.entity(ent).targetname_str();
                let ent_origin = ctx.entity(ent).r.currentOrigin;
                let s = format!(
                    "ERROR: Waypoint_small {} at {} in solid!\n",
                    targetname.as_deref().unwrap_or_default(),
                    vtos(ctx, ent_origin)
                );
                Com_Printf(&s);
                debug_assert!(false);
                G_FreeEntity(ctx, Some(ent));
                return;
            }
        }

        let ent_origin = ctx.entity(ent).r.currentOrigin;
        let ent_spawnflags = ctx.entity(ent).spawnflags;
        let health = trap::Nav_AddRawPoint(
            ctx.engine,
            GNavAddrawpointArgs::new(&ent_origin as *const vec3_t, ent_spawnflags, 2),
        );
        ctx.entity_mut(ent).health = health;
        NAV_StoreWaypoint(ctx, ent);
        G_FreeEntity(ctx, Some(ent));
        return;
    }

    G_FreeEntity(ctx, Some(ent));
}

/// Raven `SP_waypoint_navgoal`.
///
/// Source: `oracle/codemp/game/g_nav.c:1370-1386`
pub fn SP_waypoint_navgoal(ctx: &mut GameContext, ent: EntityId) {
    let radius: c_int = if ctx.entity(ent).radius != 0.0 {
        (ctx.entity(ent).radius as c_int) | NAVGOAL_USE_RADIUS
    } else {
        12
    };

    ctx.entity_mut(ent).r.mins = [-16.0, -16.0, -24.0];
    ctx.entity_mut(ent).r.maxs = [16.0, 16.0, 32.0];
    ctx.entity_mut(ent).s.origin[2] += 0.125;
    if (ctx.entity(ent).spawnflags & 1) == 0 && G_CheckInSolid(ctx, ent, qfalse) != 0 {
        let targetname = ctx.entity(ent).targetname_str();
        let ent_origin = ctx.entity(ent).r.currentOrigin;
        let s = format!(
            "ERROR: Waypoint_navgoal {} at {} in solid!\n",
            targetname.as_deref().unwrap_or_default(),
            vtos(ctx, ent_origin)
        );
        Com_Printf(&s);
        debug_assert!(false);
    }
    let targetname = ctx.entity(ent).targetname_str();
    let targetname_c = targetname.as_deref().map(cstr);
    let ent_origin = ctx.entity(ent).s.origin;
    let ent_angles = ctx.entity(ent).s.angles;
    TAG_Add(
        ctx,
        targetname_c.as_ref().map_or(core::ptr::null(), |c| c.as_ptr()),
        core::ptr::null(),
        ent_origin,
        ent_angles,
        radius,
        RTF_NAVGOAL,
    );

    ctx.ent_set(ent, PrefixSet::ClassnameStatic(c"navgoal"));
    G_FreeEntity(ctx, Some(ent)); // can't do this, they need to be found later by some functions, though those could be fixed, maybe?
}

/// Raven `SP_waypoint_navgoal_8`.
///
/// Source: `oracle/codemp/game/g_nav.c:1402-1417`
pub fn SP_waypoint_navgoal_8(ctx: &mut GameContext, ent: EntityId) {
    ctx.entity_mut(ent).r.mins = [-8.0, -8.0, -24.0];
    ctx.entity_mut(ent).r.maxs = [8.0, 8.0, 32.0];
    ctx.entity_mut(ent).s.origin[2] += 0.125;
    if (ctx.entity(ent).spawnflags & 1) == 0 && G_CheckInSolid(ctx, ent, qfalse) != 0 {
        let targetname = ctx.entity(ent).targetname_str();
        let ent_origin = ctx.entity(ent).r.currentOrigin;
        let s = format!(
            "ERROR: Waypoint_navgoal_8 {} at {} in solid!\n",
            targetname.as_deref().unwrap_or_default(),
            vtos(ctx, ent_origin)
        );
        Com_Printf(&s);
        debug_assert!(false);
    }

    let targetname = ctx.entity(ent).targetname_str();
    let targetname_c = targetname.as_deref().map(cstr);
    let ent_origin = ctx.entity(ent).s.origin;
    let ent_angles = ctx.entity(ent).s.angles;
    TAG_Add(
        ctx,
        targetname_c.as_ref().map_or(core::ptr::null(), |c| c.as_ptr()),
        core::ptr::null(),
        ent_origin,
        ent_angles,
        8,
        RTF_NAVGOAL,
    );

    ctx.ent_set(ent, PrefixSet::ClassnameStatic(c"navgoal"));
    G_FreeEntity(ctx, Some(ent));
}

/// Raven `SP_waypoint_navgoal_4`.
///
/// Source: `oracle/codemp/game/g_nav.c:1433-1448`
pub fn SP_waypoint_navgoal_4(ctx: &mut GameContext, ent: EntityId) {
    ctx.entity_mut(ent).r.mins = [-4.0, -4.0, -24.0];
    ctx.entity_mut(ent).r.maxs = [4.0, 4.0, 32.0];
    ctx.entity_mut(ent).s.origin[2] += 0.125;
    if (ctx.entity(ent).spawnflags & 1) == 0 && G_CheckInSolid(ctx, ent, qfalse) != 0 {
        let targetname = ctx.entity(ent).targetname_str();
        let ent_origin = ctx.entity(ent).r.currentOrigin;
        let s = format!(
            "ERROR: Waypoint_navgoal_4 {} at {} in solid!\n",
            targetname.as_deref().unwrap_or_default(),
            vtos(ctx, ent_origin)
        );
        Com_Printf(&s);
        debug_assert!(false);
    }

    let targetname = ctx.entity(ent).targetname_str();
    let targetname_c = targetname.as_deref().map(cstr);
    let ent_origin = ctx.entity(ent).s.origin;
    let ent_angles = ctx.entity(ent).s.angles;
    TAG_Add(
        ctx,
        targetname_c.as_ref().map_or(core::ptr::null(), |c| c.as_ptr()),
        core::ptr::null(),
        ent_origin,
        ent_angles,
        4,
        RTF_NAVGOAL,
    );

    ctx.ent_set(ent, PrefixSet::ClassnameStatic(c"navgoal"));
    G_FreeEntity(ctx, Some(ent));
}

/// Raven `SP_waypoint_navgoal_2`.
///
/// Source: `oracle/codemp/game/g_nav.c:1464-1479`
pub fn SP_waypoint_navgoal_2(ctx: &mut GameContext, ent: EntityId) {
    ctx.entity_mut(ent).r.mins = [-2.0, -2.0, -24.0];
    ctx.entity_mut(ent).r.maxs = [2.0, 2.0, 32.0];
    ctx.entity_mut(ent).s.origin[2] += 0.125;
    if (ctx.entity(ent).spawnflags & 1) == 0 && G_CheckInSolid(ctx, ent, qfalse) != 0 {
        let targetname = ctx.entity(ent).targetname_str();
        let ent_origin = ctx.entity(ent).r.currentOrigin;
        let s = format!(
            "ERROR: Waypoint_navgoal_2 {} at {} in solid!\n",
            targetname.as_deref().unwrap_or_default(),
            vtos(ctx, ent_origin)
        );
        Com_Printf(&s);
        debug_assert!(false);
    }

    let targetname = ctx.entity(ent).targetname_str();
    let targetname_c = targetname.as_deref().map(cstr);
    let ent_origin = ctx.entity(ent).s.origin;
    let ent_angles = ctx.entity(ent).s.angles;
    TAG_Add(
        ctx,
        targetname_c.as_ref().map_or(core::ptr::null(), |c| c.as_ptr()),
        core::ptr::null(),
        ent_origin,
        ent_angles,
        2,
        RTF_NAVGOAL,
    );

    ctx.ent_set(ent, PrefixSet::ClassnameStatic(c"navgoal"));
    G_FreeEntity(ctx, Some(ent));
}

/// Raven `SP_waypoint_navgoal_1`.
///
/// Source: `oracle/codemp/game/g_nav.c:1495-1510`
pub fn SP_waypoint_navgoal_1(ctx: &mut GameContext, ent: EntityId) {
    ctx.entity_mut(ent).r.mins = [-1.0, -1.0, -24.0];
    ctx.entity_mut(ent).r.maxs = [1.0, 1.0, 32.0];
    ctx.entity_mut(ent).s.origin[2] += 0.125;
    if (ctx.entity(ent).spawnflags & 1) == 0 && G_CheckInSolid(ctx, ent, qfalse) != 0 {
        let targetname = ctx.entity(ent).targetname_str();
        let ent_origin = ctx.entity(ent).r.currentOrigin;
        let s = format!(
            "ERROR: Waypoint_navgoal_1 {} at {} in solid!\n",
            targetname.as_deref().unwrap_or_default(),
            vtos(ctx, ent_origin)
        );
        Com_Printf(&s);
        debug_assert!(false);
    }

    let targetname = ctx.entity(ent).targetname_str();
    let targetname_c = targetname.as_deref().map(cstr);
    let ent_origin = ctx.entity(ent).s.origin;
    let ent_angles = ctx.entity(ent).s.angles;
    TAG_Add(
        ctx,
        targetname_c.as_ref().map_or(core::ptr::null(), |c| c.as_ptr()),
        core::ptr::null(),
        ent_origin,
        ent_angles,
        1,
        RTF_NAVGOAL,
    );

    ctx.ent_set(ent, PrefixSet::ClassnameStatic(c"navgoal"));
    G_FreeEntity(ctx, Some(ent));
}

/// Raven `Svcmd_Nav_f`.
///
/// Source: `oracle/codemp/game/g_nav.c:1518-1592`
pub fn Svcmd_Nav_f(ctx: &mut GameContext) {
    let cmd = trap::Argv(ctx.engine, 1, 1024);

    if Q_stricmp(&cmd, "show") == 0 {
        let cmd = trap::Argv(ctx.engine, 2, 1024);

        if Q_stricmp(&cmd, "all") == 0 {
            ctx.world.globals.NAVDEBUG_showNodes =
                (ctx.world.globals.NAVDEBUG_showNodes == 0) as qboolean;

            // NOTENOTE: This causes the two states to sync up if they aren't already
            let v = ctx.world.globals.NAVDEBUG_showNodes;
            ctx.world.globals.NAVDEBUG_showCollision = v;
            ctx.world.globals.NAVDEBUG_showNavGoals = v;
            ctx.world.globals.NAVDEBUG_showCombatPoints = v;
            ctx.world.globals.NAVDEBUG_showEnemyPath = v;
            ctx.world.globals.NAVDEBUG_showEdges = v;
            ctx.world.globals.NAVDEBUG_showRadius = v;
        } else if Q_stricmp(&cmd, "nodes") == 0 {
            ctx.world.globals.NAVDEBUG_showNodes =
                (ctx.world.globals.NAVDEBUG_showNodes == 0) as qboolean;
        } else if Q_stricmp(&cmd, "radius") == 0 {
            ctx.world.globals.NAVDEBUG_showRadius =
                (ctx.world.globals.NAVDEBUG_showRadius == 0) as qboolean;
        } else if Q_stricmp(&cmd, "edges") == 0 {
            ctx.world.globals.NAVDEBUG_showEdges =
                (ctx.world.globals.NAVDEBUG_showEdges == 0) as qboolean;
        } else if Q_stricmp(&cmd, "testpath") == 0 {
            ctx.world.globals.NAVDEBUG_showTestPath =
                (ctx.world.globals.NAVDEBUG_showTestPath == 0) as qboolean;
        } else if Q_stricmp(&cmd, "enemypath") == 0 {
            ctx.world.globals.NAVDEBUG_showEnemyPath =
                (ctx.world.globals.NAVDEBUG_showEnemyPath == 0) as qboolean;
        } else if Q_stricmp(&cmd, "combatpoints") == 0 {
            ctx.world.globals.NAVDEBUG_showCombatPoints =
                (ctx.world.globals.NAVDEBUG_showCombatPoints == 0) as qboolean;
        } else if Q_stricmp(&cmd, "navgoals") == 0 {
            ctx.world.globals.NAVDEBUG_showNavGoals =
                (ctx.world.globals.NAVDEBUG_showNavGoals == 0) as qboolean;
        } else if Q_stricmp(&cmd, "collision") == 0 {
            ctx.world.globals.NAVDEBUG_showCollision =
                (ctx.world.globals.NAVDEBUG_showCollision == 0) as qboolean;
        }
    } else if Q_stricmp(&cmd, "set") == 0 {
        let cmd = trap::Argv(ctx.engine, 2, 1024);

        if Q_stricmp(&cmd, "testgoal") == 0 {
            let waypoint = ctx.world.g_entities[0].waypoint;
            let ent0 = &mut ctx.world.g_entities[0] as *mut gentity_t;
            ctx.world.globals.NAVDEBUG_curGoal = trap::Nav_GetNearestNode(
                ctx.engine,
                GNavGetnearestnodeArgs::new(ent0.cast(), waypoint, NF_CLEAR_PATH, WAYPOINT_NONE),
            );
        }
    } else if Q_stricmp(&cmd, "totals") == 0 {
        Com_Printf("Navigation Totals:\n");
        Com_Printf("------------------\n");
        let n = trap::Nav_GetNumNodes(ctx.engine, GNavGetnumnodesArgs::new());
        let s = format!("Total Nodes:         {}\n", n);
        Com_Printf(&s);
        let s2 = format!("Total Combat Points: {}\n", ctx.world.level.numCombatPoints);
        Com_Printf(&s2);
    } else {
        // Print the available commands
        Com_Printf("nav - valid commands\n---\n");
        Com_Printf("show\n - nodes\n - edges\n - testpath\n - enemypath\n - combatpoints\n - navgoals\n---\n");
        Com_Printf("set\n - testgoal\n---\n");
    }
}

/// Raven `NAV_WaypointsTooFar`.
///
/// Source: `oracle/codemp/game/g_nav.c:1618-1655`
pub fn NAV_WaypointsTooFar(ctx: &mut GameContext, wp1: EntityId, wp2: EntityId) -> qboolean {
    let wp1_origin = ctx.entity(wp1).r.currentOrigin;
    let wp2_origin = ctx.entity(wp2).r.currentOrigin;
    if Distance(wp1_origin, wp2_origin) > 1024.0 {
        ctx.world.globals.fatalErrors += 1;

        let wp1_targetname = ctx.entity(wp1).targetname_str();
        let wp2_targetname = ctx.entity(wp2).targetname_str();
        let temp = if wp1_targetname.is_none() && wp2_targetname.is_none() {
            format!(
                "Waypoint conn {}->{} > 1024\n",
                vtos(ctx, wp1_origin),
                vtos(ctx, wp2_origin)
            )
        } else if wp1_targetname.is_none() {
            format!(
                "Waypoint conn {}->{} > 1024\n",
                vtos(ctx, wp1_origin),
                wp2_targetname.as_deref().unwrap()
            )
        } else if wp2_targetname.is_none() {
            format!(
                "Waypoint conn {}->{} > 1024\n",
                wp1_targetname.as_deref().unwrap(),
                vtos(ctx, wp2_origin)
            )
        } else {
            // they both have valid targetnames
            format!(
                "Waypoint conn {}->{} > 1024\n",
                wp1_targetname.as_deref().unwrap(),
                wp2_targetname.as_deref().unwrap()
            )
        };

        let len = temp.len();
        // The oracle guards on the running byte offset (`fatalErrorPointer - fatalErrorString`), not the error count.
        // `fatalErrorPointer` is that write offset.
        if ctx.world.globals.fatalErrorPointer + len >= 4096 {
            // Raven: Com_Error("%s%s%dTOO MANY...", fatalErrorString, temp, fatalErrors).
            // Source: `oracle/codemp/game/g_nav.c:1644`
            let s = format!(
                "{}{}{}TOO MANY FATAL NAV ERRORS!!!\n",
                ctx.world.globals.fatalErrorString,
                temp,
                ctx.world.globals.fatalErrors,
            );
            Com_Error(3 /* ERR_DROP */, cstr(&s).as_ptr());
            return qtrue;
        }
        // The buffer is append-only, so `fatalErrorPointer` always equals the accumulated length.
        // Appending `temp` matches Raven's write at `start`.
        ctx.world.globals.fatalErrorString.push_str(&temp);
        ctx.world.globals.fatalErrorPointer += len;

        qtrue
    } else {
        qfalse
    }
}

/// Raven `NAV_ClearStoredWaypoints`.
///
/// Source: `oracle/codemp/game/g_nav.c:1663-1666`
pub fn NAV_ClearStoredWaypoints(ctx: &mut GameContext) {
    ctx.world.globals.numStoredWaypoints = 0;
}

/// Raven `NAV_StoreWaypoint`.
///
/// Source: `oracle/codemp/game/g_nav.c:1669-1711`
pub fn NAV_StoreWaypoint(ctx: &mut GameContext, ent: EntityId) {
    if ctx.world.globals.numStoredWaypoints >= MAX_STORED_WAYPOINTS as c_int {
        return;
    }
    let i = ctx.world.globals.numStoredWaypoints as usize;
    // `ent`'s string fields are `*mut c_char` into the arena, disjoint from `globals`.
    // This snapshots the pointer values so the `globals` write borrow does not conflict.
    // `Q_strncpyz` reads through them at that point.
    let targetname = ctx.entity(ent).targetname_str();
    let target = ctx.entity(ent).target.clone();
    let target2 = ctx.entity(ent).target2.clone();
    let target3 = ctx.entity(ent).target3.clone();
    let target4 = ctx.entity(ent).target4.clone();
    let health = ctx.entity(ent).health;

    // `targetname`/`target`/`target2` decode through the accessors (`None` is Raven NULL).
    // `target3`/`target4` are owned `String`s (`""` is absent).
    // Each is `Q_strncpyz`-bound (`MAX_QPATH-1` bytes) into the `String` waypoint field.
    // An absent source leaves the field at its cleared (empty) value, matching Raven's skipped copy over zeroed storage.
    if let Some(targetname) = targetname {
        ctx.world.globals.tempWaypointList[i].targetname =
            strncpyz_string(targetname.as_bytes(), MAX_QPATH as usize);
    }
    if let Some(target) = target {
        ctx.world.globals.tempWaypointList[i].target =
            strncpyz_string(target.as_bytes(), MAX_QPATH as usize);
    }
    if let Some(target2) = target2 {
        ctx.world.globals.tempWaypointList[i].target2 =
            strncpyz_string(target2.as_bytes(), MAX_QPATH as usize);
    }
    if !target3.is_empty() {
        ctx.world.globals.tempWaypointList[i].target3 =
            strncpyz_string(target3.as_bytes(), MAX_QPATH as usize);
    }
    if !target4.is_empty() {
        ctx.world.globals.tempWaypointList[i].target4 =
            strncpyz_string(target4.as_bytes(), MAX_QPATH as usize);
    }
    ctx.world.globals.tempWaypointList[i].nodeID = health;

    ctx.world.globals.numStoredWaypoints += 1;
}

/// Raven `NAV_GetStoredWaypoint`.
///
/// Source: `oracle/codemp/game/g_nav.c:1713-1732`
pub fn NAV_GetStoredWaypoint(ctx: &mut GameContext, targetname: &str) -> c_int {
    // Raven's `!targetname || !targetname[0]` guard: NULL and empty are treated
    // identically, so an empty string returns -1.
    if targetname.is_empty() {
        return -1;
    }
    for i in 0..ctx.world.globals.numStoredWaypoints as usize {
        if !ctx.world.globals.tempWaypointList[i].targetname.is_empty()
            && targetname.eq_ignore_ascii_case(&ctx.world.globals.tempWaypointList[i].targetname)
        {
            return i as c_int;
        }
    }
    -1
}

// Oracle's `#if _HARD_CONNECT` is compiled in on the reference build.
// This port includes it unconditionally, per §C10 (behavior, not preprocessor shape).
/// Raven `NAV_CalculatePaths`.
///
/// Source: `oracle/codemp/game/g_nav.c:1734-1838`
pub fn NAV_CalculatePaths(ctx: &mut GameContext, filename: *const c_char, checksum: c_int) {
    // The oracle's `!tempWaypointList` guard is vacuous.
    // Raven's `tempWaypointList` is a fixed array whose address is never null, so this port has no runtime check to reproduce.
    // The storage is now the real `TempWaypointList` field.

    ctx.world.globals.fatalErrors = 0;
    // Raven clears the accumulated log with `memset(fatalErrorString, 0, 4096)`.
    // Source: `oracle/codemp/game/g_nav.c:1745`
    ctx.world.globals.fatalErrorString.clear();
    ctx.world.globals.fatalErrorPointer = 0;

    for i in 0..ctx.world.globals.numStoredWaypoints as usize {
        // This clones each name out first.
        // The field is now an owned `String`, so a borrow of it cannot coexist with the `&mut ctx` the lookup needs.
        let target_name = ctx.world.globals.tempWaypointList[i].target.clone();
        let target = NAV_GetStoredWaypoint(ctx, &target_name);
        let target2_name = ctx.world.globals.tempWaypointList[i].target2.clone();
        if target != -1 {
            trap::Nav_HardConnect(
                ctx.engine,
                GNavHardconnectArgs::new(
                    ctx.world.globals.tempWaypointList[i].nodeID,
                    ctx.world.globals.tempWaypointList[target as usize].nodeID,
                ),
            );
        }

        let target2 = NAV_GetStoredWaypoint(ctx, &target2_name);
        let target3_name = ctx.world.globals.tempWaypointList[i].target3.clone();
        if target2 != -1 {
            trap::Nav_HardConnect(
                ctx.engine,
                GNavHardconnectArgs::new(
                    ctx.world.globals.tempWaypointList[i].nodeID,
                    ctx.world.globals.tempWaypointList[target2 as usize].nodeID,
                ),
            );
        }

        let target3 = NAV_GetStoredWaypoint(ctx, &target3_name);
        let target4_name = ctx.world.globals.tempWaypointList[i].target4.clone();
        if target3 != -1 {
            trap::Nav_HardConnect(
                ctx.engine,
                GNavHardconnectArgs::new(
                    ctx.world.globals.tempWaypointList[i].nodeID,
                    ctx.world.globals.tempWaypointList[target3 as usize].nodeID,
                ),
            );
        }

        let target4 = NAV_GetStoredWaypoint(ctx, &target4_name);
        if target4 != -1 {
            trap::Nav_HardConnect(
                ctx.engine,
                GNavHardconnectArgs::new(
                    ctx.world.globals.tempWaypointList[i].nodeID,
                    ctx.world.globals.tempWaypointList[target4 as usize].nodeID,
                ),
            );
        }
    }

    // Now check all blocked edges, mark failed ones
    trap::Nav_CheckBlockedEdges(ctx.engine, GNavCheckblockededgesArgs::new());

    trap::Nav_SetPathsCalculated(ctx.engine, GNavSetpathscalculatedArgs::new(qfalse));

    if ctx.world.globals.fatalErrors != 0 {
        // Raven: Com_Printf("%s%d FATAL NAV ERRORS\n", fatalErrorString, fatalErrors).
        // Source: `oracle/codemp/game/g_nav.c:1835`
        let s = format!(
            "{}{} FATAL NAV ERRORS\n",
            ctx.world.globals.fatalErrorString,
            ctx.world.globals.fatalErrors,
        );
        Com_Printf(&s);
    }
}

/// Raven `NAV_Shutdown`.
///
/// Source: `oracle/codemp/game/g_nav.c:1846-1849`
pub fn NAV_Shutdown(ctx: &mut GameContext) {
    trap::Nav_Free(ctx.engine, GNavFreeArgs::new());
}

/// Raven `NAV_ShowDebugInfo`.
///
/// Source: `oracle/codemp/game/g_nav.c:1857-1903`
pub fn NAV_ShowDebugInfo(ctx: &mut GameContext) {
    if ctx.world.globals.NAVDEBUG_showNodes != 0 {
        trap::Nav_ShowNodes(ctx.engine, GNavShownodesArgs::new());
    }

    if ctx.world.globals.NAVDEBUG_showEdges != 0 {
        trap::Nav_ShowEdges(ctx.engine, GNavShowedgesArgs::new());
    }

    if ctx.world.globals.NAVDEBUG_showTestPath != 0 {
        let waypoint = ctx.world.g_entities[0].waypoint;
        let ent0 = &mut ctx.world.g_entities[0] as *mut gentity_t;
        // Get the nearest node to the player
        let mut nearestNode = trap::Nav_GetNearestNode(
            ctx.engine,
            GNavGetnearestnodeArgs::new(ent0.cast(), waypoint, NF_ANY, WAYPOINT_NONE),
        );
        let testNode = trap::Nav_GetBestNode(
            ctx.engine,
            GNavGetbestnodeArgs::new(nearestNode, ctx.world.globals.NAVDEBUG_curGoal, NODE_NONE),
        );

        let ent0_id = ctx.entity_id_of(ent0).unwrap();
        nearestNode = NAV_TestBestNode(ctx, ent0_id, nearestNode, testNode, qfalse);

        // Show the connection
        let mut dest = [0.0f32; 3];
        let mut start = [0.0f32; 3];
        trap::Nav_GetNodePosition(
            ctx.engine,
            GNavGetnodepositionArgs::new(
                ctx.world.globals.NAVDEBUG_curGoal,
                &mut dest as *mut vec3_t,
            ),
        );
        trap::Nav_GetNodePosition(
            ctx.engine,
            GNavGetnodepositionArgs::new(nearestNode, &mut start as *mut vec3_t),
        );

        G_DrawNode(start, NODE_START);
        G_DrawNode(dest, NODE_GOAL);
        trap::Nav_ShowPath(
            ctx.engine,
            GNavShowpathArgs::new(nearestNode, ctx.world.globals.NAVDEBUG_curGoal),
        );
    }

    if ctx.world.globals.NAVDEBUG_showCombatPoints != 0 {
        for i in 0..ctx.world.level.numCombatPoints as usize {
            G_DrawCombatPoint(ctx.world.level.combatPoints[i].origin, 0);
        }
    }

    if ctx.world.globals.NAVDEBUG_showNavGoals != 0 {
        TAG_ShowTags(RTF_NAVGOAL);
    }
}

/// Raven `NAV_FindPlayerWaypoint`.
///
/// Source: `oracle/codemp/game/g_nav.c:1911-1914`
pub fn NAV_FindPlayerWaypoint(ctx: &mut GameContext, clNum: c_int) {
    let idx = clNum as usize;
    let last_waypoint = ctx.world.g_entities[idx].lastWaypoint;
    let ent_ptr = &mut ctx.world.g_entities[idx] as *mut gentity_t;
    let wp = trap::Nav_GetNearestNode(
        ctx.engine,
        GNavGetnearestnodeArgs::new(ent_ptr.cast(), last_waypoint, NF_CLEAR_PATH, WAYPOINT_NONE),
    );
    ctx.world.g_entities[idx].waypoint = wp;
}
