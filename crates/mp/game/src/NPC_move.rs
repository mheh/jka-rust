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
use crate::g_nav::{NAV_AvoidCollision, NAV_CheckAhead, NAV_MoveToGoal};
use crate::g_navnew::{NAVNEW_AvoidCollision, NAVNEW_MoveToGoal};
use crate::prelude::*;
use crate::q_math::{_VectorCopy, vectoangles, AngleNormalize360, AngleVectors, VectorNormalize};
use crate::trap;
use std::ffi::c_int;

/// Raven `NPC_ClearPathToGoal`.
///
/// Source: `oracle/codemp/game/NPC_move.c:27-70`
pub fn NPC_ClearPathToGoal(ctx: &mut GameContext, dir: vec3_t, goal: Option<EntityId>) -> qboolean {
    unsafe {
        let mut trace: trace_t = core::mem::zeroed();
        let npc = ctx.world.globals.NPC;
        let npc_id = ctx.entity_id_of(npc).unwrap();
        // Raven derefs `goal` unconditionally; callers pass a live goalEntity.
        let goal_id = goal.unwrap();
        // FLAG: NPCInfo (gNPC_t) has no accessor — the deref stays raw.
        let npc_info = &*ctx.world.globals.NPCInfo;

        // Look ahead and see if we're clear to move to our goal position
        let goal_origin = ctx.world.entity(goal_id).r.currentOrigin;
        let clipmask = (ctx.world.entity(npc_id).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP;
        if NAV_CheckAhead(
            ctx,
            npc_id,
            goal_origin,
            &mut trace as *mut trace_t,
            clipmask,
        ) == qtrue
        {
            return qtrue;
        }

        if FlyingCreature(ctx.world.entity(npc_id)) == qfalse {
            // See if we're too far above
            if (ctx.world.entity(npc_id).r.currentOrigin[2]
                - ctx.world.entity(goal_id).r.currentOrigin[2])
                .abs()
                > 48.0
            {
                return qfalse;
            }
        }

        // This is a work around
        let npc_maxs = ctx.world.entity(npc_id).r.maxs;
        let radius = if npc_maxs[0] > npc_maxs[1] {
            npc_maxs[0]
        } else {
            npc_maxs[1]
        };
        let dist = crate::q_math::Distance(
            ctx.world.entity(npc_id).r.currentOrigin,
            ctx.world.entity(goal_id).r.currentOrigin,
        );
        let t_frac = 1.0f32 - (radius / dist);

        if trace.fraction >= t_frac {
            return qtrue;
        }

        // See if we're looking for a navgoal
        if (ctx.world.entity(goal_id).flags & FL_NAVGOAL) != 0 {
            // Okay, didn't get all the way there, let's see if we got close enough
            if NAV_HitNavGoal(
                trace.endpos,
                ctx.world.entity(npc_id).r.mins,
                ctx.world.entity(npc_id).r.maxs,
                ctx.world.entity(goal_id).r.currentOrigin,
                npc_info.goalRadius,
                FlyingCreature(ctx.world.entity(npc_id)),
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
        let npc_id = ctx.entity_id_of(npc).unwrap();
        // FLAG: NPCInfo (gNPC_t) has no accessor — the deref stays raw.
        let npc_info = &*ctx.world.globals.NPCInfo;
        let npc_enemy = ctx.world.entity(npc_id).enemy;

        if (npc_info.goalEntity.is_some()
            && npc_enemy.is_some()
            && npc_info.goalEntity == npc_enemy)
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
        let npc_id = ctx.entity_id_of(npc).unwrap();
        // FLAG: NPC entities carry BG_Alloc'd pool clients (not level.clients);
        // the gclient_t deref stays raw (recipe 2b).
        let client = ctx.world.entity(npc_id).client;

        if (dir[2] > 0.0) || (dir[2] < 0.0 && (*client).ps.groundEntityNum == ENTITYNUM_NONE) {
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
        let npc_id = ctx.entity_id_of(npc).unwrap();
        // FLAG: NPCInfo (gNPC_t) has no accessor — the deref stays raw.
        let npc_info = &mut *ctx.world.globals.NPCInfo;

        // Make sure we have somewhere to go
        if let Some(goal_id) = npc_info.goalEntity {
            let goal_origin = ctx.world.entity(goal_id).r.currentOrigin;
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;

            // Get our move info
            crate::q_math::_VectorSubtract(goal_origin, npc_origin, dir);
            *distance = crate::q_math::VectorNormalize(dir);

            crate::q_math::_VectorCopy(goal_origin, &mut npc_info.blockedDest);

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
        let npc_id = ctx.entity_id_of(npc).unwrap();
        // FLAG: NPCInfo (gNPC_t) has no accessor — the deref stays raw.
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
        if (ctx.world.entity(npc_id).watertype & CONTENTS_LADDER) != 0 {
            NPC_LadderMove(ctx, (*nav).direction);
            return qtrue;
        }

        // Attempt a straight move to goal
        if let Some(goal_id) = npc_info.goalEntity {
            if NPC_ClearPathToGoal(ctx, (*nav).direction, Some(goal_id)) == qfalse {
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
            if NAV_AvoidCollision(ctx, npc_id, Some(goal_id), &mut (*nav)) == qfalse {
                if ((*nav).flags & NIF_MACRO_NAV) == 0 {
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
        let npc_id = ctx.entity_id_of(npc).unwrap();
        // FLAG: NPCInfo (gNPC_t) has no accessor — the deref stays raw.
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
        if (ctx.world.entity(npc_id).watertype & CONTENTS_LADDER) != 0 {
            NPC_LadderMove(ctx, (*nav).direction);
            return qtrue;
        }

        // Attempt a straight move to goal
        if let Some(goal_id) = npc_info.goalEntity {
            if tryStraight == qfalse
                || NPC_ClearPathToGoal(ctx, (*nav).direction, Some(goal_id)) == qfalse
            {
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
                    if NAVNEW_AvoidCollision(ctx, npc_id, Some(goal_id), &mut temp_info, qtrue, 5)
                        == qfalse
                    {
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
                    if NAVNEW_AvoidCollision(ctx, npc_id, Some(goal_id), &mut (*nav), qtrue, 30)
                        == qfalse
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
    unsafe {
        let mut forward = [0.0f32; 3];
        let mut right = [0.0f32; 3];

        // Get forward and right vectors from entity's current angles.
        AngleVectors(
            self_.r.currentAngles,
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
        // FLAG: NPC pool client — the gclient_t deref stays raw (recipe 2b).
        (*self_.client).ps.moveDir = move_dir;

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
        let npc_id = ctx.entity_id_of(npc).unwrap();
        // FLAG: NPCInfo (gNPC_t) has no accessor — the deref stays raw.
        let npc_info = &mut *ctx.world.globals.NPCInfo;
        // FLAG: NPC pool client — the gclient_t deref stays raw (recipe 2b). The
        // raw pointer is stable across the ctx calls below.
        let client = ctx.world.entity(npc_id).client;

        let mut distance = 0.0f32;
        let mut dir = [0.0f32; 3];

        // If taking full body pain, don't move
        let legs_anim = ctx.world.entity(npc_id).s.legsAnim;
        if PM_InKnockDown(&mut (*client).ps) == qtrue
            || (legs_anim >= BOTH_PAIN1 as c_int && legs_anim <= BOTH_PAIN18 as c_int)
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
            (*client).ps.speed = npc_info.stats.walkSpeed as f32;
        } else {
            (*client).ps.speed = npc_info.stats.runSpeed as f32;
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
            if ((*client).ps.eFlags2 & EF2_FLYING) != 0 {
                npc_info.desiredPitch =
                    crate::q_math::AngleNormalize360(npc_info.lastPathAngles[0]);

                if dir[2] != 0.0 {
                    let mut scale = dir[2] * distance;
                    if scale > 64.0 {
                        scale = 64.0;
                    } else if scale < -64.0 {
                        scale = -64.0;
                    }
                    (*client).ps.velocity[2] = scale;
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
        let npc_id = ctx.entity_id_of(npc).unwrap();
        // FLAG: NPCInfo (gNPC_t) has no accessor — the deref stays raw.
        let npc_info = &mut *ctx.world.globals.NPCInfo;

        // FLAG: NPC pool client — the gclient_t deref stays raw (recipe 2b).
        let save_yaw = (*ctx.world.entity(npc_id).client).ps.viewangles[1];

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
        let npc_id = ctx.entity_id_of(npc).unwrap();
        // FLAG: NPC pool client — the gclient_t deref stays raw (recipe 2b).
        let client = ctx.world.entity(npc_id).client;

        BG_PlayerStateToEntityState(
            &mut (*client).ps,
            &mut ctx.world.entity_mut(npc_id).s,
            qfalse,
        );

        // use the precise origin for linking
        crate::trap::LinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(npc.cast()),
        );
    }
}
