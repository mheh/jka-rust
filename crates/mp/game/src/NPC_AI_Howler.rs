// PORT-COMPLETE: NPC_AI_Howler.c 3/6
//!
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_Howler.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//!
//! PORT STATUS: 3 functions ported (NPC_Howler_Precache, Howler_Idle,
//! NPC_Howler_Pain), 6 parked under `ambient-ai-state` escalation —
//! Howler_Patrol, Howler_Move, Howler_TryDamage, Howler_Attack,
//! Howler_Combat, NPC_BSHowler_Default all read/write the ambient `NPC`
//! and `NPCInfo` globals (b_local.h) that are set per-frame by ai_main.c
//! before calling through the bState fn-pointer table. The fnskel-generated
//! signatures take the C signature faithfully (zero params for most), and
//! neither the packet's rulings nor GameWorld establish how a behavior-state
//! fn reaches "the NPC currently being thought for" — this is a cross-file
//! architecture decision above single-file packet scope.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::public::set_anim::{SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE};

// EntityId seam helper: resolve `Option<EntityId>` back to the raw pointer the
// verbatim body still expects (`None` -> null), per the `NPC_AI_Stormtrooper.rs`
// precedent.
#[inline]
unsafe fn ent_resolve_opt(ctx: GameContext<'_>, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => unsafe { &mut (*ctx.world).g_entities[i.index()] as *mut gentity_t },
        None => core::ptr::null_mut(),
    }
}

// Raven `#define LSTATE_*` — file-scope local state for Howler NPC
// (stored in `gNPC_t::localState`).
// Source: `oracle/codemp/game/NPC_AI_Howler.c:10-11`
pub const LSTATE_CLEAR: i32 = 0;
pub const LSTATE_WAITING: i32 = 1;

// Combat distance constants for Howler melee attacks.
// Source: `oracle/codemp/game/NPC_AI_Howler.c:4,7`
const MIN_DISTANCE: c_int = 54;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;
const MAX_DISTANCE: c_int = 128;

// SETANIM_BOTH (= SETANIM_TORSO|SETANIM_LEGS), BOTH_PAIN1, and BOTH_ATTACK1 come
// from the prelude (set_anim / anim_number); no local copies here so the enum
// values stay authoritative. SETANIM_FLAG_* imported from `mp_bg::public::set_anim`.
// Source: `oracle/codemp/game/bg_public.h:500`, `anims.h`

/// Raven `NPC_Howler_Precache`.
///
/// Precache sounds/effects for Howler NPC (currently a no-op in Raven).
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:18-20`
pub fn NPC_Howler_Precache() {
    // Empty in oracle (faithfully ported as no-op).
}

/// Raven `Howler_Idle`.
///
/// Idle behavior for Howler NPC (currently a no-op in Raven).
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:28-30`
pub fn Howler_Idle() {
    // Empty in oracle (faithfully ported as no-op).
}

/// Raven `Howler_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:38-71`
pub fn Howler_Patrol(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        (*npc_info).localState = LSTATE_CLEAR;

        // If we have somewhere to go, then do that
        if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
            (*ctx.world).globals.ucmd.buttons &= !BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        } else {
            if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"patrolTime".as_ptr()) != 0 {
                crate::g_timer::TIMER_Set(
                    ctx,
                    ctx.entity_id_of(npc),
                    c"patrolTime".as_ptr(),
                    ((*ctx.world).bg_state.rng.crandom() * 5000.0 + 5000.0) as c_int,
                );
            }
        }

        // rwwFIXMEFIXME: Care about all clients, not just client 0
        let mut dif: vec3_t = [0.0; 3];
        crate::q_math::_VectorSubtract(
            (*ctx.world).g_entities[0].r.currentOrigin,
            (*npc).r.currentOrigin,
            &mut dif,
        );

        if crate::q_math::VectorLengthSquared(dif) < 256.0 * 256.0 {
            crate::NPC_combat::G_SetEnemy(
                ctx,
                ctx.entity_id_of(npc).unwrap(),
                EntityId::from_num(0),
            );
        }

        if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue) == qfalse {
            Howler_Idle();
            return;
        }
    }
}

/// Raven `Howler_Move`.
///
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:78-86`
pub fn Howler_Move(ctx: GameContext<'_>, visible: qboolean) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        if (*npc_info).localState != LSTATE_WAITING {
            (*npc_info).goalEntity = (*npc).enemy;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
            (*npc_info).goalRadius = MAX_DISTANCE;
        }
    }
}

/// Raven `Howler_TryDamage`.
///
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:89-109`
pub fn Howler_TryDamage(ctx: GameContext<'_>, enemy: Option<EntityId>, damage: c_int) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let enemy: *mut gentity_t = unsafe { ent_resolve_opt(ctx, enemy) };
    unsafe {
        let npc = (*ctx.world).globals.NPC;

        if enemy.is_null() {
            return;
        }

        let mut end: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let mut tr: trace_t = std::mem::zeroed();

        crate::q_math::AngleVectors(
            (*((*npc).client as *mut gclient_t)).ps.viewangles,
            Some(&mut dir),
            None,
            None,
        );
        crate::q_math::_VectorMA((*npc).r.currentOrigin, MIN_DISTANCE as f32, dir, &mut end);

        // Should probably trace from the mouth, but, ah well.
        crate::trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &(*npc).r.currentOrigin as *const vec3_t,
                &vec3_origin as *const vec3_t,
                &vec3_origin as *const vec3_t,
                &end as *const vec3_t,
                (*npc).s.number,
                MASK_SHOT,
            ),
        );

        if tr.entityNum != ENTITYNUM_WORLD as c_short {
            crate::g_combat::G_Damage(
                ctx,
                EntityId::from_num(tr.entityNum as c_int),
                ctx.entity_id_of(npc),
                ctx.entity_id_of(npc),
                Some(&mut dir),
                tr.endpos,
                damage,
                DAMAGE_NO_KNOCKBACK,
                MOD_MELEE as c_int,
            );
        }
    }
}

/// Raven `Howler_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:112-131`
pub fn Howler_Attack(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;

        if crate::g_timer::TIMER_Exists(ctx, ctx.entity_id_of(npc), c"attacking".as_ptr()) == qfalse
        {
            // Going to do ATTACK1
            crate::g_timer::TIMER_Set(
                ctx,
                ctx.entity_id_of(npc),
                c"attacking".as_ptr(),
                (1700.0 + ((*ctx.world).bg_state.rng.random() as f32 * 200.0)) as c_int,
            );
            crate::npc_c::NPC_SetAnim(
                ctx,
                ctx.entity_id_of(npc).unwrap(),
                SETANIM_BOTH,
                BOTH_ATTACK1 as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );

            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(npc), c"attack_dmg".as_ptr(), 200);
        }

        // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks
        if crate::g_timer::TIMER_Done2(ctx, ctx.entity_id_of(npc), c"attack_dmg".as_ptr(), qtrue)
            != 0
        {
            Howler_TryDamage(ctx, (*npc).enemy, 5);
        }

        // Just using this to remove the attacking flag at the right time
        crate::g_timer::TIMER_Done2(ctx, ctx.entity_id_of(npc), c"attacking".as_ptr(), qtrue);
    }
}

/// Raven `Howler_Combat`.
///
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:134-171`
pub fn Howler_Combat(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        let distance: f32;
        let advance: qboolean;

        // If we cannot see our target or we have somewhere to go, then do that
        let enemy_ptr = crate::ent_id::resolve((*ctx.world).g_entities.as_mut_ptr(), (*npc).enemy);
        if crate::NPC_utils::NPC_ClearLOS4(ctx, ctx.entity_id_of(enemy_ptr)) == qfalse
            || !crate::NPC_goal::UpdateGoal(ctx).is_null()
        {
            (*npc_info).combatMove = qtrue;
            (*npc_info).goalEntity = (*npc).enemy;
            (*npc_info).goalRadius = MAX_DISTANCE; // just get us within combat range

            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
            return;
        }

        // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
        crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);

        distance = crate::q_math::DistanceHorizontalSquared(
            (*npc).r.currentOrigin,
            (*enemy_ptr).r.currentOrigin,
        );
        advance = (distance > MIN_DISTANCE_SQR as f32) as qboolean;

        if (advance != 0 || (*npc_info).localState == LSTATE_WAITING)
            && crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"attacking".as_ptr()) != 0
        {
            // waiting monsters can't attack
            if crate::g_timer::TIMER_Done2(
                ctx,
                ctx.entity_id_of(npc),
                c"takingPain".as_ptr(),
                qtrue,
            ) != 0
            {
                (*npc_info).localState = LSTATE_CLEAR;
            } else {
                Howler_Move(ctx, 1 as qboolean);
            }
        } else {
            Howler_Attack(ctx);
        }
    }
}

/// Raven `NPC_Howler_Pain`.
///
/// Raven: pain handler when Howler takes damage >= 10. Sets pain animation
/// and waiting state, cancels current attack.
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:178-194`
pub fn NPC_Howler_Pain(
    ctx: GameContext<'_>,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    // STAGE-1: EntityId params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let attacker: *mut gentity_t = unsafe { ent_resolve_opt(ctx, attacker) };
    unsafe {
        if damage >= 10 {
            crate::g_timer::TIMER_Remove(ctx, ctx.entity_id_of(self_), c"attacking".as_ptr());
            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(self_), c"takingPain".as_ptr(), 2900);

            let npc = (*self_).NPC as *mut gNPC_t;
            if !npc.is_null() {
                crate::q_math::_VectorCopy((*npc).lastPathAngles, &mut (*self_).s.angles);
            }

            crate::npc_c::NPC_SetAnim(
                ctx,
                ctx.entity_id_of(self_).unwrap(),
                SETANIM_BOTH,
                BOTH_PAIN1 as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );

            if !(*self_).NPC.is_null() {
                let npc_mut = (*self_).NPC as *mut gNPC_t;
                (*npc_mut).localState = LSTATE_WAITING;
            }
        }
    }
}

/// Raven `NPC_BSHowler_Default`.
///
/// Default behavior state for Howler NPC — dispatch based on whether the
/// Howler has an enemy target or is in patrol/idle mode.
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:202-218`
pub fn NPC_BSHowler_Default(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        if (*npc).enemy.is_some() {
            Howler_Combat(ctx);
        } else if ((*npc_info).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            Howler_Patrol(ctx);
        } else {
            Howler_Idle();
        }

        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}
