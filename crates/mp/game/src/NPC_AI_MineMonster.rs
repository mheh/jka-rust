// PORT-COMPLETE: NPC_AI_MineMonster.c
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_MineMonster.c` (MP `_JK2MP` +
//! `QAGAME` compile path).
//!
//! Filled by the jampgame mega-pass.
//!
//! Parking pattern (mirrors `NPC_AI_Stormtrooper.rs`):
//! - `ai-context`: reads the file-static ambient globals `NPC`, `NPCInfo`,
//!   `ucmd` (these become GameWorld fields, but no `GameContext` is
//!   threaded into this faithful skeleton signature to access them). Also
//!   reads `level.time` for timer operations and the LCG-based `random()`
//!   (owned threaded Rng, unavailable here).
//!
//! Safe-state campaign 2c: entity (`gentity_t`) derefs of the ambient `NPC`
//! (and the `self_` handle) reads/writes route through the `GameWorld`/`GameContext`
//! accessors (`ctx.world.entity()`/`entity_mut()`) instead of raw pointers. The
//! `NPCInfo` (`gNPC_t`) and `.client` (`gclient_t`) derefs stay raw — those two
//! regimes are task #7 territory and remain in isolated `unsafe` blocks.
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
/// Source: `oracle/codemp/game/NPC_AI_MineMonster.c:18-27`
pub fn NPC_MineMonster_Precache(ctx: &mut GameContext) {
    for i in 0..4 {
        let bite_sound = cstr(&format!("sound/chars/mine/misc/bite{}.wav", i + 1));
        G_SoundIndex(bite_sound.as_ptr());
        let miss_sound = cstr(&format!("sound/chars/mine/misc/miss{}.wav", i + 1));
        G_SoundIndex(miss_sound.as_ptr());
    }
}

/// Raven `MineMonster_Idle`.
///
/// Source: `oracle/codemp/game/NPC_AI_MineMonster.c:35-42`
pub fn MineMonster_Idle(ctx: &mut GameContext) {
    if !UpdateGoal(ctx).is_null() {
        ctx.world.globals.ucmd.buttons &= !BUTTON_WALKING;
        NPC_MoveToGoal(ctx, qtrue);
    }
}

/// Raven `MineMonster_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_MineMonster.c:50-83`
pub fn MineMonster_Patrol(ctx: &mut GameContext) {
    let mut dif: vec3_t = [0.0; 3];
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
    unsafe {
        (*npc_info).localState = LSTATE_CLEAR;
    }

    if !UpdateGoal(ctx).is_null() {
        ctx.world.globals.ucmd.buttons &= !BUTTON_WALKING;
        NPC_MoveToGoal(ctx, qtrue);
    } else {
        let patrol_timer_id = cstr("patrolTime");
        if TIMER_Done(ctx, Some(npc_id), patrol_timer_id.as_ptr()) != 0 {
            let dur = (ctx.world.bg_state.rng.crandom() * 5000.0 + 5000.0) as c_int;
            TIMER_Set(ctx, Some(npc_id), patrol_timer_id.as_ptr(), dur);
        }
    }

    let e0_origin = ctx.world.g_entities[0].r.currentOrigin;
    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    _VectorSubtract(e0_origin, npc_origin, &mut dif);

    if VectorLengthSquared(dif) < 65536.0 {
        G_SetEnemy(ctx, npc_id, EntityId::from_num(0));
    }

    if NPC_CheckEnemyExt(ctx, qtrue) == 0 {
        MineMonster_Idle(ctx);
        return;
    }
}

/// Raven `MineMonster_Move`.
///
/// Source: `oracle/codemp/game/NPC_AI_MineMonster.c:90-98`
pub fn MineMonster_Move(ctx: &mut GameContext, visible: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    // gNPC_t derefs stay raw (NPCInfo deref regime, task #7) — FLAG.
    if unsafe { (*npc_info).localState } != LSTATE_WAITING {
        let npc_enemy = ctx.world.entity(npc_id).enemy;
        unsafe {
            (*npc_info).goalEntity = npc_enemy;
        }
        NPC_MoveToGoal(ctx, qtrue);
        unsafe {
            (*npc_info).goalRadius = MAX_DISTANCE;
        }
    }
}

/// Raven `MineMonster_TryDamage`.
///
/// Source: `oracle/codemp/game/NPC_AI_MineMonster.c:101-126`
pub fn MineMonster_TryDamage(ctx: &mut GameContext, enemy: Option<EntityId>, damage: c_int) {
    if enemy.is_none() {
        return;
    }

    let mut end: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    // trace_t POD zero-init (not part of the entity deref regime).
    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let origin = vec3_origin;
    let start = ctx.world.entity(npc_id).r.currentOrigin;

    // gclient deref stays raw (client deref regime, task #7) — FLAG.
    let client = ctx.world.entity(npc_id).client;
    let viewangles = unsafe { (*client).ps.viewangles };
    AngleVectors(viewangles, Some(&mut dir), None, None);

    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    _VectorMA(npc_origin, MIN_DISTANCE as f32, dir, &mut end);

    let npc_number = ctx.world.entity(npc_id).s.number;
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            core::ptr::addr_of_mut!(tr) as *mut trace_t,
            core::ptr::addr_of!(start) as *const vec3_t,
            core::ptr::addr_of!(origin) as *const vec3_t,
            core::ptr::addr_of!(origin) as *const vec3_t,
            core::ptr::addr_of!(end) as *const vec3_t,
            npc_number,
            MASK_SHOT,
        ),
    );

    let npc_opt = Some(npc_id);
    if tr.entityNum >= 0 && (tr.entityNum as c_uint) < ENTITYNUM_NONE as c_uint {
        let mut dir_copy = dir;
        G_Damage(
            ctx,
            EntityId::from_num(tr.entityNum as c_int),
            npc_opt,
            npc_opt,
            Some(&mut dir_copy),
            tr.endpos,
            damage,
            DAMAGE_NO_KNOCKBACK,
            MOD_MELEE as c_int,
        );
        let idx = ctx.world.bg_state.rng.Q_irand(1, 4);
        let bite_str = cstr(&format!("sound/chars/mine/misc/bite{}.wav", idx));
        let sound_idx = G_EffectIndex(bite_str.as_ptr());
        G_Sound(ctx, npc_opt, CHAN_AUTO, sound_idx);
    } else {
        let idx = ctx.world.bg_state.rng.Q_irand(1, 4);
        let miss_str = cstr(&format!("sound/chars/mine/misc/miss{}.wav", idx));
        let sound_idx = G_EffectIndex(miss_str.as_ptr());
        G_Sound(ctx, npc_opt, CHAN_AUTO, sound_idx);
    }
}

/// Raven `MineMonster_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_MineMonster.c:129-186`
pub fn MineMonster_Attack(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let attacking_id = cstr("attacking");

    if TIMER_Exists(ctx, Some(npc_id), attacking_id.as_ptr()) == 0 {
        let npc_enemy = ctx.world.entity(npc_id).enemy;
        let enemy_height_diff = if let Some(eid) = npc_enemy {
            ctx.world.entity(eid).r.currentOrigin[2] - ctx.world.entity(npc_id).r.currentOrigin[2]
        } else {
            0.0
        };

        let rng = &mut ctx.world.bg_state.rng;

        let do_attack4 = npc_enemy.is_some()
            && ((enemy_height_diff > 10.0 && rng.random() as f32 > 0.1f32)
                || rng.random() as f32 > 0.8f32);

        if do_attack4 {
            let dur = (1750.0 + rng.random() as f32 * 200.0) as c_int;
            TIMER_Set(ctx, Some(npc_id), attacking_id.as_ptr(), dur);
            NPC_SetAnim(
                ctx,
                npc_id,
                SETANIM_BOTH,
                BOTH_ATTACK4 as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            TIMER_Set(ctx, Some(npc_id), cstr("attack2_dmg").as_ptr(), 950);
        } else if rng.random() as f32 > 0.5f32 {
            if rng.random() as f32 > 0.8f32 {
                TIMER_Set(ctx, Some(npc_id), attacking_id.as_ptr(), 850);
                NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    BOTH_ATTACK3 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                TIMER_Set(ctx, Some(npc_id), cstr("attack2_dmg").as_ptr(), 400);
            } else {
                TIMER_Set(ctx, Some(npc_id), attacking_id.as_ptr(), 850);
                NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    BOTH_ATTACK1 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                TIMER_Set(ctx, Some(npc_id), cstr("attack1_dmg").as_ptr(), 450);
            }
        } else {
            TIMER_Set(ctx, Some(npc_id), attacking_id.as_ptr(), 1250);
            NPC_SetAnim(
                ctx,
                npc_id,
                SETANIM_BOTH,
                BOTH_ATTACK2 as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            TIMER_Set(ctx, Some(npc_id), cstr("attack1_dmg").as_ptr(), 700);
        }
    } else {
        if TIMER_Done2(ctx, Some(npc_id), cstr("attack1_dmg").as_ptr(), qtrue) != 0 {
            let npc_enemy = ctx.world.entity(npc_id).enemy;
            if let Some(enemy_id) = npc_enemy {
                MineMonster_TryDamage(ctx, Some(enemy_id), 5);
            }
        } else if TIMER_Done2(ctx, Some(npc_id), cstr("attack2_dmg").as_ptr(), qtrue) != 0 {
            let npc_enemy = ctx.world.entity(npc_id).enemy;
            if let Some(enemy_id) = npc_enemy {
                MineMonster_TryDamage(ctx, Some(enemy_id), 10);
            }
        }
    }

    TIMER_Done2(ctx, Some(npc_id), cstr("attacking").as_ptr(), qtrue);
}

/// Raven `MineMonster_Combat`.
///
/// Source: `oracle/codemp/game/NPC_AI_MineMonster.c:189-227`
pub fn MineMonster_Combat(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let npc_enemy = ctx.world.entity(npc_id).enemy;
    let can_see = if let Some(enemy_id) = npc_enemy {
        NPC_ClearLOS4(ctx, Some(enemy_id)) != 0
    } else {
        false
    };

    if !can_see || !UpdateGoal(ctx).is_null() {
        let e = ctx.world.entity(npc_id).enemy;
        // gNPC_t derefs stay raw (NPCInfo deref regime, task #7) — FLAG.
        unsafe {
            (*npc_info).combatMove = qtrue;
            (*npc_info).goalEntity = e;
            (*npc_info).goalRadius = MAX_DISTANCE;
        }
        NPC_MoveToGoal(ctx, qtrue);
        return;
    }

    NPC_FaceEnemy(ctx, qtrue);

    let npc_enemy2 = ctx.world.entity(npc_id).enemy;
    let distance = if let Some(enemy_id) = npc_enemy2 {
        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
        let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
        DistanceHorizontalSquared(npc_origin, enemy_origin)
    } else {
        0.0
    };

    let advance = distance > (MIN_DISTANCE_SQR as f32);

    // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
    if (advance || unsafe { (*npc_info).localState } == LSTATE_WAITING)
        && TIMER_Done(ctx, Some(npc_id), cstr("attacking").as_ptr()) != 0
    {
        if TIMER_Done2(ctx, Some(npc_id), cstr("takingPain").as_ptr(), qtrue) != 0 {
            // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
            unsafe {
                (*npc_info).localState = LSTATE_CLEAR;
            }
        } else {
            MineMonster_Move(ctx, 1);
        }
    } else {
        MineMonster_Attack(ctx);
    }
}

/// Raven `NPC_MineMonster_Pain`.
///
/// Handles pain/damage response for the MineMonster.
/// Source: `oracle/codemp/game/NPC_AI_MineMonster.c:234-254`
pub fn NPC_MineMonster_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    let health = ctx.world.entity(self_).health;
    // gclient deref stays raw (client deref regime, task #7) — FLAG.
    let client = ctx.world.entity(self_).client;
    let max_health = unsafe { (*client).pers.maxHealth };
    let parm = (((health as f32) / (max_health as f32)) * 100.0).floor() as c_int;
    G_AddEvent(ctx.world.entity_mut(self_), EV_PAIN as c_int, parm);

    if damage >= 10 {
        TIMER_Remove(ctx, Some(self_), cstr("attacking").as_ptr());
        TIMER_Remove(ctx, Some(self_), cstr("attacking1_dmg").as_ptr());
        TIMER_Remove(ctx, Some(self_), cstr("attacking2_dmg").as_ptr());
        TIMER_Set(ctx, Some(self_), cstr("takingPain").as_ptr(), 1350);

        // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
        let npc_ptr = ctx.world.entity(self_).NPC;
        let last_path_angles = unsafe { (*npc_ptr).lastPathAngles };
        _VectorCopy(last_path_angles, &mut ctx.world.entity_mut(self_).s.angles);

        NPC_SetAnim(
            ctx,
            self_,
            SETANIM_BOTH,
            BOTH_PAIN1 as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );

        let npc_ptr = ctx.world.entity(self_).NPC;
        if !npc_ptr.is_null() {
            // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
            unsafe {
                (*npc_ptr).localState = LSTATE_WAITING;
            }
        }
    }
}

/// Raven `NPC_BSMineMonster_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_MineMonster.c:262-278`
pub fn NPC_BSMineMonster_Default(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if ctx.world.entity(npc_id).enemy.is_some() {
        MineMonster_Combat(ctx);
    // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
    } else if (unsafe { (*npc_info).scriptFlags } & SCF_LOOK_FOR_ENEMIES) != 0 {
        MineMonster_Patrol(ctx);
    } else {
        MineMonster_Idle(ctx);
    }

    NPC_UpdateAngles(ctx, qtrue, qtrue);
}
