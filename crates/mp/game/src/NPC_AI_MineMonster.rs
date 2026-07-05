// PORT-COMPLETE: NPC_AI_MineMonster.c 1/9
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_MineMonster.c` (MP `_JK2MP` +
//! `QAGAME` compile path).
//!
//! Generated from the `fnskel.py` signature skeleton; bodies transcribed per
//! the settled jampgame fork rulings. STAGING ONLY — not yet wired into
//! crates/.
//!
//! Parking pattern (mirrors `NPC_AI_Stormtrooper.rs`):
//! - `ai-context`: reads the file-static ambient globals `NPC`, `NPCInfo`,
//!   `ucmd` (these become GameWorld fields, but no `GameContext` is
//!   threaded into this faithful skeleton signature to access them). Also
//!   reads `level.time` for timer operations and the LCG-based `random()`
//!   (owned threaded Rng, unavailable here).
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_combat::G_Damage;
use crate::g_timer::{TIMER_Done, TIMER_Done2, TIMER_Exists, TIMER_Remove, TIMER_Set};
use crate::g_utils::{G_AddEvent, G_EffectIndex, G_Sound, G_SoundIndex};
use crate::npc_c::NPC_SetAnim;
use crate::prelude::*;
use crate::q_math::{
    _VectorCopy, _VectorMA, _VectorSubtract, AngleVectors, DistanceHorizontalSquared,
    VectorLengthSquared,
};
use crate::trap;
use crate::NPC_combat::G_SetEnemy;
use crate::NPC_move::NPC_MoveToGoal;
use crate::NPC_utils::{
    NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_UpdateAngles, UpdateGoal,
};
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::public::entity_event::entity_event_t;

// Raven's working combat range defines (NPC_AI_MineMonster.c:3-8):
// These define the working combat range for these suckers
const MIN_DISTANCE: c_int = 54;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

pub const MAX_DISTANCE: c_int = 128;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;

// Raven's file-scope local state (NPC_AI_MineMonster.c:10-11):
const LSTATE_CLEAR: i32 = 0;
const LSTATE_WAITING: i32 = 1;

// `VectorLengthSquared` is the canonical `crate::q_math::VectorLengthSquared`,
// reached via the prelude glob (the former per-file copy was unused).

/// Raven `NPC_MineMonster_Precache`.
///
/// Precaches the MineMonster's sound effects.
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:18-27`
pub fn NPC_MineMonster_Precache(ctx: GameContext<'_>) {
    unsafe {
        for i in 0..4 {
            let bite_sound = cstr(&format!("sound/chars/mine/misc/bite{}.wav", i + 1));
            G_SoundIndex(bite_sound.as_ptr());
            let miss_sound = cstr(&format!("sound/chars/mine/misc/miss{}.wav", i + 1));
            G_SoundIndex(miss_sound.as_ptr());
        }
    }
}

/// Raven `MineMonster_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:35-42`
pub fn MineMonster_Idle(ctx: GameContext<'_>) {
    unsafe {
        if !UpdateGoal(ctx).is_null() {
            (*ctx.world).globals.ucmd.buttons &= !BUTTON_WALKING;
            NPC_MoveToGoal(ctx, qtrue);
        }
    }
}

/// Raven `MineMonster_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:50-83`
pub fn MineMonster_Patrol(ctx: GameContext<'_>) {
    unsafe {
        let mut dif: vec3_t = [0.0; 3];
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        (*npc_info).localState = LSTATE_CLEAR;

        if !UpdateGoal(ctx).is_null() {
            (*ctx.world).globals.ucmd.buttons &= !BUTTON_WALKING;
            NPC_MoveToGoal(ctx, qtrue);
        } else {
            let patrol_timer_id = cstr("patrolTime");
            if TIMER_Done(ctx, npc, patrol_timer_id.as_ptr()) != 0 {
                let dur = ((*ctx.world).bg_state.rng.crandom() * 5000.0 + 5000.0) as c_int;
                TIMER_Set(ctx, npc, patrol_timer_id.as_ptr(), dur);
            }
        }

        _VectorSubtract(
            (*ctx.world).g_entities[0].r.currentOrigin,
            (*npc).r.currentOrigin,
            &mut dif,
        );

        if VectorLengthSquared(dif) < 65536.0 {
            G_SetEnemy(ctx, npc, &mut (*ctx.world).g_entities[0] as *mut gentity_t);
        }

        if NPC_CheckEnemyExt(ctx, qtrue) == 0 {
            MineMonster_Idle(ctx);
            return;
        }
    }
}

/// Raven `MineMonster_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:90-98`
pub fn MineMonster_Move(ctx: GameContext<'_>, visible: qboolean) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        if (*npc_info).localState != LSTATE_WAITING {
            (*npc_info).goalEntity = (*npc).enemy;
            NPC_MoveToGoal(ctx, qtrue);
            (*npc_info).goalRadius = MAX_DISTANCE;
        }
    }
}

/// Raven `MineMonster_TryDamage`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:101-126`
pub fn MineMonster_TryDamage(ctx: GameContext<'_>, enemy: *mut gentity_t, damage: c_int) {
    unsafe {
        if enemy.is_null() {
            return;
        }

        let mut end: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let mut tr: trace_t = unsafe { core::mem::zeroed() };
        let npc = (*ctx.world).globals.NPC;
        let origin = vec3_origin;
        let start = (*npc).r.currentOrigin;

        AngleVectors(
            (*((*npc).client as *mut gclient_t)).ps.viewangles,
            Some(&mut dir),
            None,
            None,
        );
        _VectorMA((*npc).r.currentOrigin, MIN_DISTANCE as f32, dir, &mut end);

        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                core::ptr::addr_of_mut!(tr) as *mut trace_t,
                core::ptr::addr_of!(start) as *const vec3_t,
                core::ptr::addr_of!(origin) as *const vec3_t,
                core::ptr::addr_of!(origin) as *const vec3_t,
                core::ptr::addr_of!(end) as *const vec3_t,
                (*npc).s.number,
                MASK_SHOT,
            ),
        );

        if tr.entityNum >= 0 && (tr.entityNum as c_uint) < ENTITYNUM_NONE as c_uint {
            let damage_entity =
                &mut (*ctx.world).g_entities[tr.entityNum as usize] as *mut gentity_t;
            let mut dir_copy = dir;
            G_Damage(
                ctx,
                damage_entity,
                npc,
                npc,
                Some(&mut dir_copy),
                tr.endpos,
                damage,
                DAMAGE_NO_KNOCKBACK,
                MOD_MELEE as c_int,
            );
            let idx = (*ctx.world).bg_state.rng.Q_irand(1, 4);
            let bite_str = cstr(&format!("sound/chars/mine/misc/bite{}.wav", idx));
            let sound_idx = G_EffectIndex(bite_str.as_ptr());
            G_Sound(ctx, npc, CHAN_AUTO, sound_idx);
        } else {
            let idx = (*ctx.world).bg_state.rng.Q_irand(1, 4);
            let miss_str = cstr(&format!("sound/chars/mine/misc/miss{}.wav", idx));
            let sound_idx = G_EffectIndex(miss_str.as_ptr());
            G_Sound(ctx, npc, CHAN_AUTO, sound_idx);
        }
    }
}

/// Raven `MineMonster_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:129-186`
pub fn MineMonster_Attack(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let attacking_id = cstr("attacking");

        if TIMER_Exists(ctx, npc, attacking_id.as_ptr()) == 0 {
            let rng = &mut (*ctx.world).bg_state.rng;

            let enemy_height_diff = if let Some(eid) = (*npc).enemy {
                (*ctx.world).g_entities[eid.0 as usize].r.currentOrigin[2]
                    - (*npc).r.currentOrigin[2]
            } else {
                0.0
            };

            let do_attack4 = (*npc).enemy.is_some()
                && ((enemy_height_diff > 10.0 && rng.random() as f32 > 0.1f32)
                    || rng.random() as f32 > 0.8f32);

            if do_attack4 {
                let dur = (1750.0 + rng.random() as f32 * 200.0) as c_int;
                TIMER_Set(ctx, npc, attacking_id.as_ptr(), dur);
                NPC_SetAnim(
                    ctx,
                    npc,
                    SETANIM_BOTH,
                    BOTH_ATTACK4 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                TIMER_Set(ctx, npc, cstr("attack2_dmg").as_ptr(), 950);
            } else if rng.random() as f32 > 0.5f32 {
                if rng.random() as f32 > 0.8f32 {
                    TIMER_Set(ctx, npc, attacking_id.as_ptr(), 850);
                    NPC_SetAnim(
                        ctx,
                        npc,
                        SETANIM_BOTH,
                        BOTH_ATTACK3 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    TIMER_Set(ctx, npc, cstr("attack2_dmg").as_ptr(), 400);
                } else {
                    TIMER_Set(ctx, npc, attacking_id.as_ptr(), 850);
                    NPC_SetAnim(
                        ctx,
                        npc,
                        SETANIM_BOTH,
                        BOTH_ATTACK1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    TIMER_Set(ctx, npc, cstr("attack1_dmg").as_ptr(), 450);
                }
            } else {
                TIMER_Set(ctx, npc, attacking_id.as_ptr(), 1250);
                NPC_SetAnim(
                    ctx,
                    npc,
                    SETANIM_BOTH,
                    BOTH_ATTACK2 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                TIMER_Set(ctx, npc, cstr("attack1_dmg").as_ptr(), 700);
            }
        } else {
            if TIMER_Done2(ctx, npc, cstr("attack1_dmg").as_ptr(), qtrue) != 0 {
                if let Some(enemy_id) = (*npc).enemy {
                    let enemy_ptr =
                        &mut (*ctx.world).g_entities[enemy_id.0 as usize] as *mut gentity_t;
                    MineMonster_TryDamage(ctx, enemy_ptr, 5);
                }
            } else if TIMER_Done2(ctx, npc, cstr("attack2_dmg").as_ptr(), qtrue) != 0 {
                if let Some(enemy_id) = (*npc).enemy {
                    let enemy_ptr =
                        &mut (*ctx.world).g_entities[enemy_id.0 as usize] as *mut gentity_t;
                    MineMonster_TryDamage(ctx, enemy_ptr, 10);
                }
            }
        }

        TIMER_Done2(ctx, npc, cstr("attacking").as_ptr(), qtrue);
    }
}

/// Raven `MineMonster_Combat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:189-227`
pub fn MineMonster_Combat(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        let can_see = if let Some(enemy_id) = (*npc).enemy {
            let enemy_ptr = &mut (*ctx.world).g_entities[enemy_id.0 as usize] as *mut gentity_t;
            NPC_ClearLOS4(ctx, enemy_ptr) != 0
        } else {
            false
        };

        if !can_see || !UpdateGoal(ctx).is_null() {
            (*npc_info).combatMove = qtrue;
            (*npc_info).goalEntity = (*npc).enemy;
            (*npc_info).goalRadius = MAX_DISTANCE;
            NPC_MoveToGoal(ctx, qtrue);
            return;
        }

        NPC_FaceEnemy(ctx, qtrue);

        let distance = if let Some(enemy_id) = (*npc).enemy {
            let enemy_ptr = &mut (*ctx.world).g_entities[enemy_id.0 as usize] as *mut gentity_t;
            DistanceHorizontalSquared((*npc).r.currentOrigin, (*enemy_ptr).r.currentOrigin)
        } else {
            0.0
        };

        let advance = distance > (MIN_DISTANCE_SQR as f32);

        if (advance || (*npc_info).localState == LSTATE_WAITING)
            && TIMER_Done(ctx, npc, cstr("attacking").as_ptr()) != 0
        {
            if TIMER_Done2(ctx, npc, cstr("takingPain").as_ptr(), qtrue) != 0 {
                (*npc_info).localState = LSTATE_CLEAR;
            } else {
                MineMonster_Move(ctx, 1);
            }
        } else {
            MineMonster_Attack(ctx);
        }
    }
}

/// Raven `NPC_MineMonster_Pain`.
///
/// Handles pain/damage response for the MineMonster.
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:234-254`
pub fn NPC_MineMonster_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    unsafe {
        let parm = ((((*self_).health as f32)
            / ((*((*self_).client as *mut gclient_t)).pers.maxHealth as f32))
            * 100.0)
            .floor() as c_int;
        G_AddEvent(self_, EV_PAIN as c_int, parm);

        if damage >= 10 {
            TIMER_Remove(ctx, self_, cstr("attacking").as_ptr());
            TIMER_Remove(ctx, self_, cstr("attacking1_dmg").as_ptr());
            TIMER_Remove(ctx, self_, cstr("attacking2_dmg").as_ptr());
            TIMER_Set(ctx, self_, cstr("takingPain").as_ptr(), 1350);

            _VectorCopy(
                (*((*self_).NPC as *mut gNPC_t)).lastPathAngles,
                &mut (*self_).s.angles,
            );

            NPC_SetAnim(
                ctx,
                self_,
                SETANIM_BOTH,
                BOTH_PAIN1 as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );

            if !(*self_).NPC.is_null() {
                (*((*self_).NPC as *mut gNPC_t)).localState = LSTATE_WAITING;
            }
        }
    }
}

/// Raven `NPC_BSMineMonster_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_MineMonster.c:262-278`
pub fn NPC_BSMineMonster_Default(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        if (*npc).enemy.is_some() {
            MineMonster_Combat(ctx);
        } else if ((*npc_info).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            MineMonster_Patrol(ctx);
        } else {
            MineMonster_Idle(ctx);
        }

        NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}
