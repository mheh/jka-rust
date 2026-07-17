// PORT-COMPLETE: NPC_goal.c
//! FAITHFUL port of `oracle/codemp/game/NPC_goal.c`.
//!
//! Filled by the jampgame mega-pass. Functions that reach file-scope AI globals (`NPC`,
//! `NPCInfo`, `level`) or engine traps (`trap_ICARUS_TaskIDComplete`)
//! without a `GameWorld`/engine handle on the staged raw-pointer skeleton
//! are parked; only the pure-logic bounds-overlap checker is ported.
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`. Bodies re-derive the raw pointers verbatim at the
//! top (`// STAGE-1:` markers) — Stage-2 debt. Callers bridge at the boundary
//! via `ctx.entity_id_of(ptr)`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

/// Resolve a stored `Option<EntityId>` field back to a `gentity_t*` (the
/// id->pointer half of the entity-id seam; `None` -> Raven's NULL).
#[inline]
unsafe fn ent_ptr(ctx: &mut GameContext, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => &mut ctx.world.g_entities[i.index()] as *mut gentity_t,
        None => core::ptr::null_mut(),
    }
}

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

/// Raven `SetGoal`.
///
/// Source: `oracle/codemp/game/NPC_goal.c:10-24`
pub fn SetGoal(ctx: &mut GameContext, goal: Option<EntityId>, rating: f32) {
    // SAFETY: `NPCInfo` is Raven's ambient AI global (`gNPC_t *`); its raw deref is
    // the still-open ambient-globals seam (2c task #7), not an entity deref. The
    // Stage-1 `ent_id_opt(base, ent_ptr(goal))` round-trip is the identity on
    // `Option<EntityId>`, so the goal handle assigns directly.
    unsafe {
        let npc_info = &mut *ctx.world.globals.NPCInfo;
        npc_info.goalEntity = goal;
        npc_info.goalTime = ctx.world.level.time;
    }
}

/// Raven `NPC_SetGoal`.
///
/// Source: `oracle/codemp/game/NPC_goal.c:31-58`
pub fn NPC_SetGoal(ctx: &mut GameContext, goal: Option<EntityId>, rating: f32) {
    // SAFETY: `NPCInfo` ambient-global raw deref (2c task #7); the goal entity is
    // reached through the safe `ctx.world.entity` accessor.
    unsafe {
        let npc_info = &mut *ctx.world.globals.NPCInfo;

        if goal == npc_info.goalEntity {
            return;
        }

        let Some(goal_id) = goal else {
            return;
        };

        if !ctx.world.entity(goal_id).client.is_null() {
            return;
        }

        if npc_info.goalEntity.is_some() {
            npc_info.lastGoalEntity = npc_info.goalEntity;
        }

        SetGoal(ctx, goal, rating);
    }
}

/// Raven `NPC_ClearGoal`.
///
/// Source: `oracle/codemp/game/NPC_goal.c:65-86`
pub fn NPC_ClearGoal(ctx: &mut GameContext) {
    // SAFETY: `NPCInfo` ambient-global raw deref (2c task #7); the goal entity is
    // reached through the safe `ctx.world.entity` accessor.
    unsafe {
        let npc_info = &mut *ctx.world.globals.NPCInfo;

        if npc_info.lastGoalEntity.is_none() {
            SetGoal(ctx, None, 0.0);
            return;
        }

        let last_goal_id = npc_info.lastGoalEntity;
        npc_info.lastGoalEntity = None;

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
/// Pure logic: checks if two 3D bounds overlap by comparing component-wise
/// min/max values. NOTE: flush up against counts as overlapping (equality
/// compares use `<=`/`>=`).
///
/// Source: `oracle/codemp/game/NPC_goal.c:94-115`
pub fn G_BoundsOverlap(mins1: vec3_t, maxs1: vec3_t, mins2: vec3_t, maxs2: vec3_t) -> qboolean {
    // Check if mins1 is beyond maxs2 on any axis
    if mins1[0] > maxs2[0] {
        return qfalse;
    }
    if mins1[1] > maxs2[1] {
        return qfalse;
    }
    if mins1[2] > maxs2[2] {
        return qfalse;
    }

    // Check if maxs1 is before mins2 on any axis
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

    unsafe {
        let npc_info = &mut *ctx.world.globals.NPCInfo;
        npc_info.goalTime = ctx.world.level.time;

        npc_info.aiFlags &= !NPCAI_MOVING;
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
/// Checks if an NPC has reached its goal entity by comparing the NPC's position
/// and bounds against the goal's location. Most of the original complex waypoint
/// logic is commented out; only the final nav-system check is active.
///
/// Source: `oracle/codemp/game/NPC_goal.c:136-231`
pub fn ReachedGoal(ctx: &mut GameContext, goal: Option<EntityId>) -> qboolean {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let goal: *mut gentity_t = unsafe { ent_ptr(ctx, goal) };
    unsafe {
        let npc_info = &mut *ctx.world.globals.NPCInfo;

        if (npc_info.aiFlags & NPCAI_TOUCHED_GOAL) != 0 {
            npc_info.aiFlags &= !NPCAI_TOUCHED_GOAL;
            return qtrue;
        }

        let npc = ctx.world.globals.NPC;
        let flying = FlyingCreature(&*npc);

        NAV_HitNavGoal(
            (*npc).r.currentOrigin,
            (*npc).r.mins,
            (*npc).r.maxs,
            (*goal).r.currentOrigin,
            npc_info.goalRadius,
            flying,
        )
    }
}

/// Raven `UpdateGoal`.
///
/// Updates the NPC's goal state: returns the current goal entity if it's valid,
/// or clears the goal and returns NULL if it's no longer in use or has been
/// reached.
///
/// Source: `oracle/codemp/game/NPC_goal.c:243-267`
pub fn UpdateGoal(ctx: &mut GameContext) -> *mut gentity_t {
    // SAFETY: `NPCInfo` ambient-global raw deref (2c task #7); the goal entity is
    // reached through the safe accessor, then re-derived as a raw pointer at the
    // return boundary for the still-raw caller.
    unsafe {
        let npc_info = &*ctx.world.globals.NPCInfo;

        if npc_info.goalEntity.is_none() {
            return core::ptr::null_mut();
        }

        let goal_id = npc_info.goalEntity.unwrap();

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
