//! This is a port of `oracle/codemp/game/NPC_goal.c`.
//!
//! Entity-pointer params are `EntityId`/`Option<EntityId>` handles (§B5), not raw `gentity_t*`.
//! `ReachedGoal` re-derives a raw `gentity_t` pointer from the goal handle at the top of its body.
#![allow(non_snake_case, unused, clippy::all)]

use crate::ent_id;
use crate::prelude::*;

// Raven `qboolean` is `c_int`. Keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

/// Raven `SetGoal`.
///
/// Source: `oracle/codemp/game/NPC_goal.c:10-24`
pub fn SetGoal(ctx: &mut GameContext, goal: Option<EntityId>, rating: f32) {
    // SAFETY: `NPCInfo` is Raven's ambient AI global (`gNPC_t *`).
    // Its raw deref has no accessor yet, and it is not an entity deref.
    // `goalEntity` is already `Option<EntityId>`, so the goal handle assigns directly.
    let npc_info: *mut gNPC_t = ctx.world.globals.NPCInfo;
    let goal_time = ctx.world.level.time;
    unsafe {
        (*npc_info).goalEntity = goal;
        (*npc_info).goalTime = goal_time;
    }
}

/// Raven `NPC_SetGoal`.
///
/// Source: `oracle/codemp/game/NPC_goal.c:31-58`
pub fn NPC_SetGoal(ctx: &mut GameContext, goal: Option<EntityId>, rating: f32) {
    // SAFETY: `NPCInfo` is an ambient global with a raw deref and no accessor yet.
    // The goal entity is reached through the safe `ctx.world.entity` accessor.
    let npc_info: *mut gNPC_t = ctx.world.globals.NPCInfo;
    unsafe {
        if goal == (*npc_info).goalEntity {
            return;
        }

        let Some(goal_id) = goal else {
            return;
        };

        if !ctx.world.entity(goal_id).client.is_null() {
            return;
        }

        if (*npc_info).goalEntity.is_some() {
            (*npc_info).lastGoalEntity = (*npc_info).goalEntity;
        }

        SetGoal(ctx, goal, rating);
    }
}

/// Raven `NPC_ClearGoal`.
///
/// Source: `oracle/codemp/game/NPC_goal.c:65-86`
pub fn NPC_ClearGoal(ctx: &mut GameContext) {
    // SAFETY: `NPCInfo` is an ambient global with a raw deref and no accessor yet.
    // The goal entity is reached through the safe `ctx.world.entity` accessor.
    let npc_info: *mut gNPC_t = ctx.world.globals.NPCInfo;
    unsafe {
        if (*npc_info).lastGoalEntity.is_none() {
            SetGoal(ctx, None, 0.0);
            return;
        }

        let last_goal_id = (*npc_info).lastGoalEntity;
        (*npc_info).lastGoalEntity = None;

        if let Some(goal_id) = last_goal_id {
            let goal = ctx.world.entity(goal_id);

            if goal.inuse != 0 && (goal.s.eFlags & EF_NODRAW) == 0 {
                SetGoal(ctx, Some(goal_id), 0.0);
                return;
            }
        }

        SetGoal(ctx, None, 0.0);
    }
}

/// Raven `G_BoundsOverlap`.
///
/// This checks whether two 3D bounds overlap by comparing bounds on each axis.
/// NOTE: flush up against counts as overlapping
///
/// Source: `oracle/codemp/game/NPC_goal.c:94-115`
pub fn G_BoundsOverlap(mins1: vec3_t, maxs1: vec3_t, mins2: vec3_t, maxs2: vec3_t) -> qboolean {
    if mins1[0] > maxs2[0] {
        return qfalse;
    }
    if mins1[1] > maxs2[1] {
        return qfalse;
    }
    if mins1[2] > maxs2[2] {
        return qfalse;
    }

    if maxs1[0] < mins2[0] {
        return qfalse;
    }
    if maxs1[1] < mins2[1] {
        return qfalse;
    }
    if maxs1[2] < mins2[2] {
        return qfalse;
    }

    qtrue
}

/// Raven `NPC_ReachedGoal`.
///
/// Source: `oracle/codemp/game/NPC_goal.c:117-129`
pub fn NPC_ReachedGoal(ctx: &mut GameContext) {
    NPC_ClearGoal(ctx);

    let npc_info: *mut gNPC_t = ctx.world.globals.NPCInfo;
    let goal_time = ctx.world.level.time;
    unsafe {
        (*npc_info).goalTime = goal_time;

        (*npc_info).aiFlags &= !NPCAI_MOVING;
        ctx.world.globals.ucmd.forwardmove = 0;

        let npc = ctx.world.globals.NPC;
        trap::ICARUS_TaskIDComplete(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_TASKIDCOMPLETE::GIcarusTaskidcompleteArgs::new(
                npc.cast(),
                TID_MOVE_NAV as c_int,
            ),
        );
    }
}

/// Raven `ReachedGoal`.
///
/// This checks whether the NPC reached its goal entity by comparing the NPC position
/// and bounds against the goal location.
/// Most of the original waypoint logic is commented out in Raven, and only the final
/// nav-system check is ported.
///
/// Source: `oracle/codemp/game/NPC_goal.c:136-231`
pub fn ReachedGoal(ctx: &mut GameContext, goal: Option<EntityId>) -> qboolean {
    let goal: *mut gentity_t =
        unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), goal) };
    let npc_info: *mut gNPC_t = ctx.world.globals.NPCInfo;
    unsafe {
        if ((*npc_info).aiFlags & NPCAI_TOUCHED_GOAL) != 0 {
            (*npc_info).aiFlags &= !NPCAI_TOUCHED_GOAL;
            return qtrue;
        }

        let npc = ctx.world.globals.NPC;
        let flying = FlyingCreature(&*npc);

        NAV_HitNavGoal(
            (*npc).r.currentOrigin,
            (*npc).r.mins,
            (*npc).r.maxs,
            (*goal).r.currentOrigin,
            (*npc_info).goalRadius,
            flying,
        )
    }
}

/// Raven `UpdateGoal`.
///
/// This updates the NPC goal state.
/// It returns the current goal entity when the entity is valid.
/// It clears the goal and returns null when the goal is no longer in use or already reached.
///
/// Source: `oracle/codemp/game/NPC_goal.c:243-267`
pub fn UpdateGoal(ctx: &mut GameContext) -> *mut gentity_t {
    // SAFETY: `NPCInfo` is an ambient global with a raw deref and no accessor yet.
    // The goal entity is reached through the safe accessor, then re-derived as a raw pointer
    // at the return boundary for the still-raw caller.
    let npc_info: *mut gNPC_t = ctx.world.globals.NPCInfo;
    unsafe {
        if (*npc_info).goalEntity.is_none() {
            return core::ptr::null_mut();
        }

        let goal_id = (*npc_info).goalEntity.unwrap();

        if ctx.world.entity(goal_id).inuse == 0 {
            NPC_ClearGoal(ctx);
            return core::ptr::null_mut();
        }

        if ReachedGoal(ctx, Some(goal_id)) != 0 {
            NPC_ReachedGoal(ctx);
            return core::ptr::null_mut();
        }

        &mut ctx.world.g_entities[goal_id.index()] as *mut gentity_t
    }
}
