// PORT-COMPLETE: NPC_move.c 2/11
//! FAITHFUL port of `oracle/codemp/game/NPC_move.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`. Bodies re-derive the raw pointers verbatim at the
//! top (`// STAGE-1:` markers) — Stage-2 debt. Callers bridge at the boundary
//! via `ctx.entity_id_of(ptr)`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::bg_panimate::PM_InKnockDown;
use crate::g_nav::{NAV_AvoidCollision, NAV_MoveToGoal};
use crate::g_navnew::{NAVNEW_AvoidCollision, NAVNEW_MoveToGoal};
use crate::prelude::*;
use crate::q_math::{_VectorCopy, vectoangles, AngleNormalize360, AngleVectors, VectorNormalize};
use crate::trap;
use std::ffi::c_int;

/// Resolve a stored `Option<EntityId>` field back to a `gentity_t*` (the
/// id->pointer half of the entity-id seam; `None` -> Raven's NULL).
#[inline]
unsafe fn ent_ptr(ctx: &mut GameContext, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => &mut ctx.world.g_entities[i.index()] as *mut gentity_t,
        None => core::ptr::null_mut(),
    }
}

/// Raven `NPC_ClearPathToGoal`.
///
/// Source: `oracle/codemp/game/NPC_move.c:27-70`
pub fn NPC_ClearPathToGoal(ctx: &mut GameContext, dir: vec3_t, goal: Option<EntityId>) -> qboolean {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let goal: *mut gentity_t = unsafe { ent_ptr(ctx, goal) };
    unsafe {
        let mut trace: trace_t = core::mem::zeroed();
        let npc = ctx.world.globals.NPC;
        let npc_info = &*ctx.world.globals.NPCInfo;

        // Look ahead and see if we're clear to move to our goal position
        if crate::g_nav::NAV_CheckAhead(
            ctx,
            ctx.entity_id_of(npc).unwrap(),
            (*goal).r.currentOrigin,
            &mut trace as *mut trace_t,
            ((*npc).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
        ) == qtrue
        {
            return qtrue;
        }

        if FlyingCreature(&*npc) == qfalse {
            // See if we're too far above
            if ((*npc).r.currentOrigin[2] - (*goal).r.currentOrigin[2]).abs() > 48.0 {
                return qfalse;
            }
        }

        // This is a work around
        let radius = if (*npc).r.maxs[0] > (*npc).r.maxs[1] {
            (*npc).r.maxs[0]
        } else {
            (*npc).r.maxs[1]
        };
        let dist = crate::q_math::Distance((*npc).r.currentOrigin, (*goal).r.currentOrigin);
        let t_frac = 1.0f32 - (radius / dist);

        if trace.fraction >= t_frac {
            return qtrue;
        }

        // See if we're looking for a navgoal
        if ((*goal).flags & FL_NAVGOAL) != 0 {
            // Okay, didn't get all the way there, let's see if we got close enough
            if NAV_HitNavGoal(
                trace.endpos,
                (*npc).r.mins,
                (*npc).r.maxs,
                (*goal).r.currentOrigin,
                npc_info.goalRadius,
                FlyingCreature(&*npc),
            ) == qtrue
            {
                return qtrue;
            }
        }

        return qfalse;
    }
}

/// Raven `NPC_CheckCombatMove`.
///
/// Source: `oracle/codemp/game/NPC_move.c:78-95`
pub fn NPC_CheckCombatMove(ctx: &mut GameContext) -> qboolean {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = &*ctx.world.globals.NPCInfo;

        if (npc_info.goalEntity.is_some()
            && (*npc).enemy.is_some()
            && npc_info.goalEntity == (*npc).enemy)
            || npc_info.combatMove != 0
        {
            return qtrue;
        }

        if npc_info.goalEntity.is_some() && npc_info.watchTarget.is_some() {
            if npc_info.goalEntity != npc_info.watchTarget {
                return qtrue;
            }
        }

        return qfalse;
    }
}

/// Raven `NPC_LadderMove`.
///
/// Source: `oracle/codemp/game/NPC_move.c:103-118`
pub fn NPC_LadderMove(ctx: &mut GameContext, dir: vec3_t) {
    unsafe {
        let npc = ctx.world.globals.NPC;

        if (dir[2] > 0.0)
            || (dir[2] < 0.0 && (*((*npc).client)).ps.groundEntityNum == ENTITYNUM_NONE)
        {
            // Set our movement direction
            ctx.world.globals.ucmd.upmove = if dir[2] > 0.0 { 127 } else { -127 };

            // Don't move around on XY
            ctx.world.globals.ucmd.forwardmove = 0;
            ctx.world.globals.ucmd.rightmove = 0;
        }
    }
}

/// Raven `NPC_GetMoveInformation`.
///
/// Source: `oracle/codemp/game/NPC_move.c:126-141`
pub fn NPC_GetMoveInformation(
    ctx: &mut GameContext,
    dir: &mut vec3_t,
    distance: *mut f32,
) -> qboolean {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = &mut *ctx.world.globals.NPCInfo;

        // Make sure we have somewhere to go
        if let Some(goal_id) = npc_info.goalEntity {
            let goal_ptr = &mut ctx.world.g_entities[goal_id.index()] as *mut gentity_t;

            // Get our move info
            crate::q_math::_VectorSubtract(
                (*goal_ptr).r.currentOrigin,
                (*npc).r.currentOrigin,
                dir,
            );
            *distance = crate::q_math::VectorNormalize(dir);

            crate::q_math::_VectorCopy((*goal_ptr).r.currentOrigin, &mut npc_info.blockedDest);

            return qtrue;
        }

        return qfalse;
    }
}

/// Raven `NAV_GetLastMove`.
///
/// Copies the file-scope `frameNavInfo` to the caller's buffer. Called after
/// navigation functions have populated `frameNavInfo` in the same frame.
///
/// Source: `oracle/codemp/game/NPC_move.c:149-152`
pub fn NAV_GetLastMove(ctx: &mut GameContext, info: *mut navInfo_t) {
    unsafe {
        *info = ctx.world.globals.frameNavInfo.0;
    }
}

/// Raven `NPC_GetMoveDirection`.
///
/// Source: `oracle/codemp/game/NPC_move.c:160-230`
pub fn NPC_GetMoveDirection(
    ctx: &mut GameContext,
    out: &mut vec3_t,
    distance: *mut f32,
) -> qboolean {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = &mut *ctx.world.globals.NPCInfo;
        let mut angles = [0.0f32; 3];
        // STAGE-2b: irreducible — the raw `&mut` into the ambient `frameNavInfo`
        // global is passed alongside `ctx` to the raw-ABI nav callees.
        let nav = &raw mut ctx.world.globals.frameNavInfo.0;

        // Clear the struct
        (*nav) = std::mem::zeroed();

        // Get our movement, if any
        if NPC_GetMoveInformation(ctx, &mut (*nav).direction, &mut (*nav).distance) == qfalse {
            return qfalse;
        }

        // Setup the return value
        *distance = (*nav).distance;

        // For starters
        _VectorCopy((*nav).direction, &mut (*nav).pathDirection);

        // If on a ladder, move appropriately
        if ((*npc).watertype & CONTENTS_LADDER) != 0 {
            NPC_LadderMove(ctx, (*nav).direction);
            return qtrue;
        }

        // Attempt a straight move to goal
        if let Some(goal_id) = npc_info.goalEntity {
            let goal_ptr = &mut ctx.world.g_entities[goal_id.index()] as *mut gentity_t;
            if NPC_ClearPathToGoal(ctx, (*nav).direction, ctx.entity_id_of(goal_ptr)) == qfalse {
                let npc_id = ctx.entity_id_of(npc).unwrap();
                // See if we're just stuck
                if NAV_MoveToGoal(ctx, npc_id, &mut (*nav) as *mut navInfo_t) == WAYPOINT_NONE {
                    // Can't reach goal, just face
                    vectoangles((*nav).direction, &mut angles);
                    npc_info.desiredYaw = AngleNormalize360(angles[1]);
                    _VectorCopy((*nav).direction, out);
                    *distance = (*nav).distance;
                    return qfalse;
                }

                (*nav).flags |= NIF_MACRO_NAV;
            }
        }

        // Avoid any collisions on the way
        if let Some(goal_id) = npc_info.goalEntity {
            let goal_ptr = &mut ctx.world.g_entities[goal_id.index()] as *mut gentity_t;
            if NAV_AvoidCollision(
                ctx,
                ctx.entity_id_of(npc).unwrap(),
                ctx.entity_id_of(goal_ptr),
                &mut (*nav),
            ) == qfalse
            {
                if ((*nav).flags & NIF_MACRO_NAV) == 0 {
                    let npc_id = ctx.entity_id_of(npc).unwrap();
                    // we had a clear path to goal and didn't try macro nav, but can't avoid collision so try macro nav here
                    // See if we're just stuck
                    if NAV_MoveToGoal(ctx, npc_id, &mut (*nav) as *mut navInfo_t) == WAYPOINT_NONE {
                        // Can't reach goal, just face
                        vectoangles((*nav).direction, &mut angles);
                        npc_info.desiredYaw = AngleNormalize360(angles[1]);
                        _VectorCopy((*nav).direction, out);
                        *distance = (*nav).distance;
                        return qfalse;
                    }

                    (*nav).flags |= NIF_MACRO_NAV;
                }
            }
        }

        // Setup the return values
        _VectorCopy((*nav).direction, out);
        *distance = (*nav).distance;

        return qtrue;
    }
}

/// Raven `NPC_GetMoveDirectionAltRoute`.
///
/// Source: `oracle/codemp/game/NPC_move.c:239-322`
pub fn NPC_GetMoveDirectionAltRoute(
    ctx: &mut GameContext,
    out: &mut vec3_t,
    distance: *mut f32,
    tryStraight: qboolean,
) -> qboolean {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = &mut *ctx.world.globals.NPCInfo;
        let mut angles = [0.0f32; 3];
        // STAGE-2b: irreducible — the raw `&mut` into the ambient `frameNavInfo`
        // global is passed alongside `ctx` to the raw-ABI nav callees.
        let nav = &raw mut ctx.world.globals.frameNavInfo.0;

        npc_info.aiFlags &= !NPCAI_BLOCKED;

        // Clear the struct
        (*nav) = std::mem::zeroed();

        // Get our movement, if any
        if NPC_GetMoveInformation(ctx, &mut (*nav).direction, &mut (*nav).distance) == qfalse {
            return qfalse;
        }

        // Setup the return value
        *distance = (*nav).distance;

        // For starters
        _VectorCopy((*nav).direction, &mut (*nav).pathDirection);

        // If on a ladder, move appropriately
        if ((*npc).watertype & CONTENTS_LADDER) != 0 {
            NPC_LadderMove(ctx, (*nav).direction);
            return qtrue;
        }

        // Attempt a straight move to goal
        if let Some(goal_id) = npc_info.goalEntity {
            let goal_ptr = &mut ctx.world.g_entities[goal_id.index()] as *mut gentity_t;
            if tryStraight == qfalse
                || NPC_ClearPathToGoal(ctx, (*nav).direction, ctx.entity_id_of(goal_ptr)) == qfalse
            {
                let npc_id = ctx.entity_id_of(npc).unwrap();
                // blocked — Can't get straight to goal, use macro nav
                if NAVNEW_MoveToGoal(ctx, npc_id, &mut (*nav)) == WAYPOINT_NONE {
                    // Can't reach goal, just face
                    vectoangles((*nav).direction, &mut angles);
                    npc_info.desiredYaw = AngleNormalize360(angles[1]);
                    _VectorCopy((*nav).direction, out);
                    *distance = (*nav).distance;
                    return qfalse;
                }
                // else we are on our way
                (*nav).flags |= NIF_MACRO_NAV;
            } else {
                // we have no architectural problems, see if there are ents inthe way and try to go around them
                // not blocked
                if ctx.world.cvars.d_altRoutes.integer != 0 {
                    // try macro nav
                    let mut temp_info = (*nav);
                    if NAVNEW_AvoidCollision(
                        ctx,
                        ctx.entity_id_of(npc).unwrap(),
                        ctx.entity_id_of(goal_ptr),
                        &mut temp_info,
                        qtrue,
                        5,
                    ) == qfalse
                    {
                        let npc_id = ctx.entity_id_of(npc).unwrap();
                        // revert to macro nav — Can't get straight to goal, dump tempInfo and use macro nav
                        if NAVNEW_MoveToGoal(ctx, npc_id, &mut (*nav)) == WAYPOINT_NONE {
                            // Can't reach goal, just face
                            vectoangles((*nav).direction, &mut angles);
                            npc_info.desiredYaw = AngleNormalize360(angles[1]);
                            _VectorCopy((*nav).direction, out);
                            *distance = (*nav).distance;
                            return qfalse;
                        }
                        // else we are on our way
                        (*nav).flags |= NIF_MACRO_NAV;
                    } else {
                        // otherwise, either clear or can avoid
                        (*nav) = temp_info;
                    }
                } else {
                    // OR: just give up
                    if NAVNEW_AvoidCollision(
                        ctx,
                        ctx.entity_id_of(npc).unwrap(),
                        ctx.entity_id_of(goal_ptr),
                        &mut (*nav),
                        qtrue,
                        30,
                    ) == qfalse
                    {
                        // give up
                        return qfalse;
                    }
                }
            }
        }

        // Setup the return values
        _VectorCopy((*nav).direction, out);
        *distance = (*nav).distance;

        return qtrue;
    }
}

/// Raven `G_UcmdMoveForDir`.
///
/// Converts a direction vector into forward/right movement commands for a
/// usercmd. Normalizes the direction, computes dot products with the entity's
/// facing angles, and clamps movement values to [-127, 127]. NPCs cheat by
/// storing the precise direction in playerstate to avoid precision loss from
/// the ucmd conversion.
///
/// Source: `oracle/codemp/game/NPC_move.c:324-370`
pub fn G_UcmdMoveForDir(self_: &mut gentity_t, cmd: *mut usercmd_t, dir: vec3_t) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = self_;
    unsafe {
        let mut forward = [0.0f32; 3];
        let mut right = [0.0f32; 3];

        // Get forward and right vectors from entity's current angles.
        AngleVectors(
            (*self_).r.currentAngles,
            Some(&mut forward),
            Some(&mut right),
            None,
        );

        // Zero out vertical component of movement direction.
        let mut move_dir = dir;
        move_dir[2] = 0.0;

        // Normalize the direction.
        VectorNormalize(&mut move_dir);

        // Store the movement direction in playerstate for NPC cheating
        // (preserves precision lost in ucmd conversion).
        (*((*self_).client)).ps.moveDir = move_dir;

        // Compute dot products with forward and right vectors, scaled to [-127, 127].
        let mut fDot =
            (forward[0] * move_dir[0] + forward[1] * move_dir[1] + forward[2] * move_dir[2])
                * 127.0f32;
        let mut rDot =
            (right[0] * move_dir[0] + right[1] * move_dir[1] + right[2] * move_dir[2]) * 127.0f32;

        // Clamp values to [-127, 127] to avoid overflow in signed byte.
        // DotProduct is not guaranteed to return [-1, 1] due to floating-point errors.
        if fDot > 127.0f32 {
            fDot = 127.0f32;
        }
        if fDot < -127.0f32 {
            fDot = -127.0f32;
        }
        if rDot > 127.0f32 {
            rDot = 127.0f32;
        }
        if rDot < -127.0f32 {
            rDot = -127.0f32;
        }

        // Store in usercmd as signed bytes.
        (*cmd).forwardmove = fDot.floor() as c_schar;
        (*cmd).rightmove = rDot.floor() as c_schar;
    }
}

/// Raven `NPC_MoveToGoal`.
///
/// Source: `oracle/codemp/game/NPC_move.c:382-467`
pub fn NPC_MoveToGoal(ctx: &mut GameContext, tryStraight: qboolean) -> qboolean {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = &mut *ctx.world.globals.NPCInfo;

        let mut distance = 0.0f32;
        let mut dir = [0.0f32; 3];

        // If taking full body pain, don't move
        if PM_InKnockDown(&mut (*((*npc).client)).ps) == qtrue
            || ((*npc).s.legsAnim >= BOTH_PAIN1 as c_int
                && (*npc).s.legsAnim <= BOTH_PAIN18 as c_int)
        {
            return qtrue;
        }

        // Get our movement direction
        if NPC_GetMoveDirectionAltRoute(ctx, &mut dir, &mut distance, tryStraight) == qfalse {
            return qfalse;
        }

        npc_info.distToGoal = distance;

        // Convert the move to angles
        crate::q_math::vectoangles(dir, &mut npc_info.lastPathAngles);
        if (ctx.world.globals.ucmd.buttons & BUTTON_WALKING) != 0 {
            (*((*npc).client)).ps.speed = npc_info.stats.walkSpeed as f32;
        } else {
            (*((*npc).client)).ps.speed = npc_info.stats.runSpeed as f32;
        }

        // If in combat move, then move directly towards our goal
        if NPC_CheckCombatMove(ctx) == qtrue {
            // keep current facing
            // STAGE-2b: irreducible — raw entity + ucmd aliases into ctx.world
            // handed to the raw-ABI G_UcmdMoveForDir (&mut gentity_t + *mut
            // usercmd_t, no ctx param).
            let npc_id = ctx.entity_id_of(npc).unwrap();
            let self_ent = &raw mut ctx.world.g_entities[npc_id.index()];
            let ucmd_ptr = &raw mut ctx.world.globals.ucmd;
            G_UcmdMoveForDir(&mut *self_ent, ucmd_ptr, dir);
        } else {
            // face our goal
            npc_info.desiredPitch = 0.0f32;
            npc_info.desiredYaw = crate::q_math::AngleNormalize360(npc_info.lastPathAngles[1]);

            // Pitch towards the goal and also update if flying or swimming
            if ((*((*npc).client)).ps.eFlags2 & EF2_FLYING) != 0 {
                npc_info.desiredPitch =
                    crate::q_math::AngleNormalize360(npc_info.lastPathAngles[0]);

                if dir[2] != 0.0 {
                    let mut scale = dir[2] * distance;
                    if scale > 64.0 {
                        scale = 64.0;
                    } else if scale < -64.0 {
                        scale = -64.0;
                    }
                    (*((*npc).client)).ps.velocity[2] = scale;
                }
            }

            // Set any final info
            ctx.world.globals.ucmd.forwardmove = 127;
        }

        return qtrue;
    }
}

/// Raven `NPC_SlideMoveToGoal`.
///
/// Source: `oracle/codemp/game/NPC_move.c:476-488`
pub fn NPC_SlideMoveToGoal(ctx: &mut GameContext) -> qboolean {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = &mut *ctx.world.globals.NPCInfo;

        let save_yaw = (*((*npc).client)).ps.viewangles[1];

        npc_info.combatMove = 1;

        let ret = NPC_MoveToGoal(ctx, qtrue);

        npc_info.desiredYaw = save_yaw;

        return ret;
    }
}

/// Raven `NPC_ApplyRoff`.
///
/// Source: `oracle/codemp/game/NPC_move.c:497-505`
pub fn NPC_ApplyRoff(ctx: &mut GameContext) {
    unsafe {
        let npc = ctx.world.globals.NPC;

        BG_PlayerStateToEntityState(&mut (*((*npc).client)).ps, &mut (*npc).s, qfalse);

        // use the precise origin for linking
        crate::trap::LinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(npc.cast()),
        );
    }
}
