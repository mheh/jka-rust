// PORT-COMPLETE: NPC_AI_Droid.c
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_Droid.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//!
//! Safe-state campaign 2c: entity (`gentity_t`) derefs of the ambient `NPC`
//! (and the `self_` handle) reads/writes route through the `GameWorld`/`GameContext`
//! accessors (`ctx.world.entity()`/`entity_mut()`) instead of raw pointers. The
//! `NPCInfo` (`gNPC_t`) and `.client` (`gclient_t`) derefs stay raw — those two
//! regimes are task #7 territory and remain in isolated `unsafe` blocks.
#![allow(non_snake_case, unused, clippy::all)]

use crate::cstr_util::cstr;
use crate::g_timer::{TIMER_Done, TIMER_Set};
use crate::g_utils::{G_EffectIndex, G_PlayEffectID, G_SoundIndex, G_SoundOnEnt};
use crate::npc_c::NPC_SetAnim;
use crate::prelude::*;
use crate::q_math::{
    _VectorMA, _VectorSubtract, AngleDelta, AngleNormalize360, AngleVectors, VectorNormalize,
};
use crate::q_shared::va;
use crate::NPC_goal::UpdateGoal;
use crate::NPC_move::NPC_MoveToGoal;
use crate::NPC_reactions::{NPC_GetPainChance, NPC_Pain};
use crate::NPC_utils::NPC_SetSurfaceOnOff;
use crate::NPC_utils::{NPC_SetBoneAngles, NPC_UpdateAngles};

/// Local state enums.
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:10-17`
pub const LSTATE_NONE: c_int = 0;
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:13`
pub const LSTATE_BACKINGUP: c_int = 1;
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:14`
pub const LSTATE_SPINNING: c_int = 2;
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:15`
pub const LSTATE_PAIN: c_int = 3;
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:16`
pub const LSTATE_DROP: c_int = 4;

/// Surface render status flag: turn off.
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:7`
const TURN_OFF: c_int = 0x00000100;

/// Raven `R2D2_PartsMove`.
///
/// Raven: Front 'eye' lense animation.
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:24-46`
pub fn R2D2_PartsMove(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    if npc.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if TIMER_Done(ctx, Some(npc_id), b"eyeDelay\0".as_ptr() as *const c_char) != 0 {
        let normalized = AngleNormalize360(ctx.world.entity(npc_id).pos1[1]);
        ctx.world.entity_mut(npc_id).pos1[1] = normalized;

        let r0 = ctx.world.bg_state.rng.Q_irand(-20, 20) as f32;
        ctx.world.entity_mut(npc_id).pos1[0] += r0;
        let r1 = ctx.world.bg_state.rng.Q_irand(-20, 20) as f32;
        ctx.world.entity_mut(npc_id).pos1[1] = r1;
        let r2 = ctx.world.bg_state.rng.Q_irand(-20, 20) as f32;
        ctx.world.entity_mut(npc_id).pos1[2] = r2;

        let pos1 = ctx.world.entity(npc_id).pos1;
        NPC_SetBoneAngles(ctx, npc_id, b"f_eye\0".as_ptr() as *mut c_char, pos1);

        let delay = ctx.world.bg_state.rng.Q_irand(100, 1000);
        TIMER_Set(
            ctx,
            Some(npc_id),
            b"eyeDelay\0".as_ptr() as *const c_char,
            delay,
        );
    }
}

/// Raven `Droid_Idle`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:53-58`
pub fn Droid_Idle() {
    // Empty function — Raven code has only commented-out code
}

/// Raven `R2D2_TurnAnims`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:65-95`
pub fn R2D2_TurnAnims(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    if npc.is_null() || npc_info.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let current_yaw = ctx.world.entity(npc_id).r.currentAngles[1];
    // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
    let desired_yaw = unsafe { (*npc_info).desiredYaw };
    let turndelta = AngleDelta(current_yaw, desired_yaw); // YAW = 1
    let anim: c_int;

    // gclient deref stays raw (client deref regime, task #7) — FLAG.
    let client = ctx.world.entity(npc_id).client;
    let npc_class = unsafe { (*client).NPC_class };

    if (turndelta.abs() > 20.0)
        && (npc_class == class_t::CLASS_R2D2 || npc_class == class_t::CLASS_R5D2)
    {
        // CLASS_R2D2 = 2, CLASS_R5D2 = 3 (or check from globals)
        // gclient deref stays raw (client deref regime, task #7) — FLAG.
        anim = unsafe { (*client).ps.legsAnim };
        if turndelta < 0.0 {
            if anim != BOTH_TURN_LEFT1 as c_int {
                NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    BOTH_TURN_LEFT1 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
            }
        } else {
            if anim != BOTH_TURN_RIGHT1 as c_int {
                NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    BOTH_TURN_RIGHT1 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
            }
        }
    } else {
        NPC_SetAnim(
            ctx,
            npc_id,
            SETANIM_BOTH,
            BOTH_RUN1 as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
    }
}

/// Raven `Droid_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:102-168`
pub fn Droid_Patrol(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    if npc.is_null() || npc_info.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let normalized = AngleNormalize360(ctx.world.entity(npc_id).pos1[1]);
    ctx.world.entity_mut(npc_id).pos1[1] = normalized;

    // gclient deref stays raw (client deref regime, task #7) — FLAG.
    let client = ctx.world.entity(npc_id).client;
    if !client.is_null() && unsafe { (*client).NPC_class } != class_t::CLASS_GONK {
        // CLASS_GONK
        if unsafe { (*client).NPC_class } != class_t::CLASS_R5D2 {
            // CLASS_R5D2
            R2D2_PartsMove(ctx);
        }
        R2D2_TurnAnims(ctx);
    }

    if !UpdateGoal(ctx).is_null() {
        ctx.world.globals.ucmd.buttons |= 1; // BUTTON_WALKING
        NPC_MoveToGoal(ctx, 1 as qboolean); // qtrue

        // gclient deref stays raw (client deref regime, task #7) — FLAG.
        let client = ctx.world.entity(npc_id).client;
        if !client.is_null() && unsafe { (*client).NPC_class } == class_t::CLASS_MOUSE {
            // CLASS_MOUSE
            // `.5` is a double literal and `sin` is the double libm: the whole
            // term is evaluated in f64 and narrowed only on store to the float.
            let time = ctx.world.level.time;
            // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
            unsafe {
                (*npc_info).desiredYaw =
                    ((*npc_info).desiredYaw as f64 + (time as f64 * 0.5).sin() * 25.0) as f32;
            }

            if TIMER_Done(
                ctx,
                Some(npc_id),
                b"patrolNoise\0".as_ptr() as *const c_char,
            ) != 0
            {
                let idx = ctx.world.bg_state.rng.Q_irand(1, 3);
                let sound_path = format!("sound/chars/mouse/misc/mousego{}.wav", idx);
                G_SoundOnEnt(ctx, npc_id, 0, cstr(&sound_path).as_ptr()); // CHAN_AUTO = 0
                let delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);

                TIMER_Set(
                    ctx,
                    Some(npc_id),
                    b"patrolNoise\0".as_ptr() as *const c_char,
                    delay,
                );
            }
        } else if !client.is_null() && unsafe { (*client).NPC_class } == class_t::CLASS_R2D2 {
            // CLASS_R2D2
            if TIMER_Done(
                ctx,
                Some(npc_id),
                b"patrolNoise\0".as_ptr() as *const c_char,
            ) != 0
            {
                let idx = ctx.world.bg_state.rng.Q_irand(1, 3);
                let sound_path = format!("sound/chars/r2d2/misc/r2d2talk0{}.wav", idx);
                G_SoundOnEnt(ctx, npc_id, 0, cstr(&sound_path).as_ptr());

                let delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                TIMER_Set(
                    ctx,
                    Some(npc_id),
                    b"patrolNoise\0".as_ptr() as *const c_char,
                    delay,
                );
            }
        } else if !client.is_null() && unsafe { (*client).NPC_class } == class_t::CLASS_R5D2 {
            // CLASS_R5D2
            if TIMER_Done(
                ctx,
                Some(npc_id),
                b"patrolNoise\0".as_ptr() as *const c_char,
            ) != 0
            {
                let idx = ctx.world.bg_state.rng.Q_irand(1, 4);
                let sound_path = format!("sound/chars/r5d2/misc/r5talk{}.wav", idx);
                G_SoundOnEnt(ctx, npc_id, 0, cstr(&sound_path).as_ptr());

                let delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                TIMER_Set(
                    ctx,
                    Some(npc_id),
                    b"patrolNoise\0".as_ptr() as *const c_char,
                    delay,
                );
            }
        }
        if !client.is_null() && unsafe { (*client).NPC_class } == class_t::CLASS_GONK {
            // CLASS_GONK
            if TIMER_Done(
                ctx,
                Some(npc_id),
                b"patrolNoise\0".as_ptr() as *const c_char,
            ) != 0
            {
                let idx = ctx.world.bg_state.rng.Q_irand(1, 2);
                let sound_path = format!("sound/chars/gonk/misc/gonktalk{}.wav", idx);
                G_SoundOnEnt(ctx, npc_id, 0, cstr(&sound_path).as_ptr());

                let delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                TIMER_Set(
                    ctx,
                    Some(npc_id),
                    b"patrolNoise\0".as_ptr() as *const c_char,
                    delay,
                );
            }
        }
    }

    NPC_UpdateAngles(ctx, 1 as qboolean, 1 as qboolean); // qtrue, qtrue
}

/// Raven `Droid_Run`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:175-200`
pub fn Droid_Run(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    if npc.is_null() || npc_info.is_null() {
        return;
    }

    R2D2_PartsMove(ctx);

    // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
    if unsafe { (*npc_info).localState } == LSTATE_BACKINGUP {
        ctx.world.globals.ucmd.forwardmove = -127;
        // gNPC_t derefs stay raw (NPCInfo deref regime, task #7) — FLAG.
        unsafe {
            (*npc_info).desiredYaw += 5.0;
            (*npc_info).localState = LSTATE_NONE;
        }
    } else {
        ctx.world.globals.ucmd.forwardmove = 64;
        if !UpdateGoal(ctx).is_null() {
            if NPC_MoveToGoal(ctx, 0 as qboolean) != 0 {
                // qfalse
                // `.5` is a double literal and `sin` is the double libm: the whole
                // term is evaluated in f64 and narrowed only on store to the float.
                let time = ctx.world.level.time;
                // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
                unsafe {
                    (*npc_info).desiredYaw =
                        ((*npc_info).desiredYaw as f64 + (time as f64 * 0.5).sin() * 5.0) as f32;
                }
            }
        }
    }

    NPC_UpdateAngles(ctx, 1 as qboolean, 1 as qboolean); // qtrue, qtrue
}

/// Raven `Droid_Spin`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:207-266`
pub fn Droid_Spin(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    if npc.is_null() || npc_info.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let dir = [0.0f32, 0.0f32, 1.0f32];

    R2D2_TurnAnims(ctx);

    // gclient deref stays raw (client deref regime, task #7) — FLAG.
    let client = ctx.world.entity(npc_id).client;
    let npc_class = unsafe { (*client).NPC_class };

    if npc_class == class_t::CLASS_R5D2 || npc_class == class_t::CLASS_R2D2 {
        // CLASS_R5D2, CLASS_R2D2
        // No head?
        let ghoul2 = ctx.world.entity(npc_id).ghoul2;
        if trap::G2API_GetSurfaceRenderStatus(ctx.engine, ghoul2, 0, "head") > 0
        {
            if TIMER_Done(ctx, Some(npc_id), b"smoke\0".as_ptr() as *const c_char) != 0
                && TIMER_Done(
                    ctx,
                    Some(npc_id),
                    b"droidsmoketotal\0".as_ptr() as *const c_char,
                ) == 0
            {
                TIMER_Set(ctx, Some(npc_id), b"smoke\0".as_ptr() as *const c_char, 100);
                let origin = ctx.world.entity(npc_id).r.currentOrigin;
                G_PlayEffectID(
                    G_EffectIndex(b"volumetric/droid_smoke\0".as_ptr() as *const c_char),
                    origin,
                    dir,
                );
            }

            if TIMER_Done(ctx, Some(npc_id), b"droidspark\0".as_ptr() as *const c_char) != 0 {
                let delay = ctx.world.bg_state.rng.Q_irand(100, 500);
                TIMER_Set(
                    ctx,
                    Some(npc_id),
                    b"droidspark\0".as_ptr() as *const c_char,
                    delay,
                );
                let origin = ctx.world.entity(npc_id).r.currentOrigin;
                G_PlayEffectID(
                    G_EffectIndex(b"sparks/spark\0".as_ptr() as *const c_char),
                    origin,
                    dir,
                );
            }

            ctx.world.globals.ucmd.forwardmove =
                ctx.world.bg_state.rng.Q_irand(-64, 64) as c_int as i8;

            if TIMER_Done(ctx, Some(npc_id), b"roam\0".as_ptr() as *const c_char) != 0 {
                let delay = ctx.world.bg_state.rng.Q_irand(250, 1000);
                TIMER_Set(
                    ctx,
                    Some(npc_id),
                    b"roam\0".as_ptr() as *const c_char,
                    delay,
                );
                let dy = ctx.world.bg_state.rng.Q_irand(0, 360) as f32;
                // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
                unsafe {
                    (*npc_info).desiredYaw = dy;
                }
            }
        } else {
            if TIMER_Done(ctx, Some(npc_id), b"roam\0".as_ptr() as *const c_char) != 0 {
                // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
                unsafe {
                    (*npc_info).localState = LSTATE_NONE;
                }
            } else {
                // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
                unsafe {
                    (*npc_info).desiredYaw = AngleNormalize360((*npc_info).desiredYaw + 40.0);
                }
            }
        }
    } else {
        if TIMER_Done(ctx, Some(npc_id), b"roam\0".as_ptr() as *const c_char) != 0 {
            // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
            unsafe {
                (*npc_info).localState = LSTATE_NONE;
            }
        } else {
            // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
            unsafe {
                (*npc_info).desiredYaw = AngleNormalize360((*npc_info).desiredYaw + 40.0);
            }
        }
    }

    NPC_UpdateAngles(ctx, 1 as qboolean, 1 as qboolean); // qtrue, qtrue
}

/// Raven `NPC_Droid_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:273-434`
pub fn NPC_Droid_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    let mod_: c_int = ctx.world.globals.gPainMOD;
    let mut pain_chance: f32;

    // VectorCopy( self->NPC->lastPathAngles, self->s.angles )
    // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
    let npc_ptr = ctx.world.entity(self_).NPC;
    let last_path_angles = unsafe { (*npc_ptr).lastPathAngles };
    crate::q_math::_VectorCopy(last_path_angles, &mut ctx.world.entity_mut(self_).s.angles);

    // gclient deref stays raw (client deref regime, task #7) — FLAG.
    let client = ctx.world.entity(self_).client;
    let npc_class = unsafe { (*client).NPC_class };

    if npc_class == class_t::CLASS_R5D2 {
        // CLASS_R5D2
        pain_chance = NPC_GetPainChance(ctx, self_, damage);

        if mod_ == MOD_DEMP2 as c_int
            || mod_ == MOD_DEMP2_ALT as c_int
            || ctx.world.bg_state.rng.random() < pain_chance
        {
            let vehicle_num = ctx.world.entity(self_).s.m_iVehicleNum;
            let health = ctx.world.entity(self_).health;
            if vehicle_num == 0
                && (health < 30 || mod_ == MOD_DEMP2 as c_int || mod_ == MOD_DEMP2_ALT as c_int)
            {
                if (ctx.world.entity(self_).spawnflags & 2) == 0 {
                    // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
                    let local_state = unsafe { (*npc_ptr).localState };
                    let ghoul2 = ctx.world.entity(self_).ghoul2;
                    if (local_state != LSTATE_SPINNING)
                        && (trap::G2API_GetSurfaceRenderStatus(ctx.engine, ghoul2, 0, "head") == 0)
                    {
                        NPC_SetSurfaceOnOff(
                            ctx,
                            self_,
                            b"head\0".as_ptr() as *const c_char,
                            TURN_OFF,
                        );

                        // gclient deref stays raw (client deref regime, task #7) — FLAG.
                        let veh = unsafe { (*client).ps.m_iVehicleNum };
                        if veh != 0 {
                            let mut up = [0.0f32; 3];
                            let current_angles = ctx.world.entity(self_).r.currentAngles;
                            AngleVectors(current_angles, None, None, Some(&mut up));
                            let origin = ctx.world.entity(self_).r.currentOrigin;
                            G_PlayEffectID(
                                G_EffectIndex(b"chunks/r5d2head_veh\0".as_ptr() as *const c_char),
                                origin,
                                up,
                            );
                        } else {
                            let origin = ctx.world.entity(self_).r.currentOrigin;
                            G_PlayEffectID(
                                G_EffectIndex(b"small_chunks\0".as_ptr() as *const c_char),
                                origin,
                                [0.0, 0.0, 0.0],
                            );
                            G_PlayEffectID(
                                G_EffectIndex(b"chunks/r5d2head\0".as_ptr() as *const c_char),
                                origin,
                                [0.0, 0.0, 0.0],
                            );
                        }

                        let time = ctx.world.level.time;
                        // gclient deref stays raw (client deref regime, task #7) — FLAG.
                        unsafe {
                            (*client).ps.electrifyTime = time + 3000;
                        }

                        TIMER_Set(
                            ctx,
                            Some(self_),
                            b"droidsmoketotal\0".as_ptr() as *const c_char,
                            5000,
                        );
                        TIMER_Set(ctx, Some(self_), b"droidspark\0".as_ptr() as *const c_char, 100);
                        // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
                        unsafe {
                            (*npc_ptr).localState = LSTATE_SPINNING;
                        }
                    }
                }
            } else {
                // gclient deref stays raw (client deref regime, task #7) — FLAG.
                let anim = unsafe { (*client).ps.legsAnim };

                if anim == BOTH_STAND2 as c_int {
                    NPC_SetAnim(
                        ctx,
                        self_,
                        SETANIM_BOTH,
                        BOTH_PAIN1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                } else {
                    NPC_SetAnim(
                        ctx,
                        self_,
                        SETANIM_BOTH,
                        BOTH_PAIN2 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                }

                // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
                unsafe {
                    (*npc_ptr).localState = LSTATE_SPINNING;
                }
                let delay = ctx.world.bg_state.rng.Q_irand(1000, 2000);
                TIMER_Set(ctx, Some(self_), b"roam\0".as_ptr() as *const c_char, delay);
            }
        }
    } else if npc_class == class_t::CLASS_MOUSE {
        // CLASS_MOUSE
        if mod_ == MOD_DEMP2 as c_int || mod_ == MOD_DEMP2_ALT as c_int {
            let time = ctx.world.level.time;
            // gNPC_t + gclient derefs stay raw (task #7 regimes) — FLAG.
            unsafe {
                (*npc_ptr).localState = LSTATE_SPINNING;
                (*client).ps.electrifyTime = time + 3000;
            }
        } else {
            // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
            unsafe {
                (*npc_ptr).localState = LSTATE_BACKINGUP;
            }
        }

        // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
        unsafe {
            (*npc_ptr).scriptFlags &= !SCF_LOOK_FOR_ENEMIES;
        }
    } else if npc_class == class_t::CLASS_R2D2 {
        // CLASS_R2D2
        pain_chance = NPC_GetPainChance(ctx, self_, damage);

        if mod_ == MOD_DEMP2 as c_int
            || mod_ == MOD_DEMP2_ALT as c_int
            || ctx.world.bg_state.rng.random() < pain_chance
        {
            let vehicle_num = ctx.world.entity(self_).s.m_iVehicleNum;
            let health = ctx.world.entity(self_).health;
            if vehicle_num == 0
                && (health < 30 || mod_ == MOD_DEMP2 as c_int || mod_ == MOD_DEMP2_ALT as c_int)
            {
                if (ctx.world.entity(self_).spawnflags & 2) == 0 {
                    // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
                    let local_state = unsafe { (*npc_ptr).localState };
                    let ghoul2 = ctx.world.entity(self_).ghoul2;
                    if (local_state != LSTATE_SPINNING)
                        && (trap::G2API_GetSurfaceRenderStatus(ctx.engine, ghoul2, 0, "head") == 0)
                    {
                        NPC_SetSurfaceOnOff(
                            ctx,
                            self_,
                            b"head\0".as_ptr() as *const c_char,
                            TURN_OFF,
                        );

                        // gclient deref stays raw (client deref regime, task #7) — FLAG.
                        let veh = unsafe { (*client).ps.m_iVehicleNum };
                        if veh != 0 {
                            let mut up = [0.0f32; 3];
                            let current_angles = ctx.world.entity(self_).r.currentAngles;
                            AngleVectors(current_angles, None, None, Some(&mut up));
                            let origin = ctx.world.entity(self_).r.currentOrigin;
                            G_PlayEffectID(
                                G_EffectIndex(b"chunks/r2d2head_veh\0".as_ptr() as *const c_char),
                                origin,
                                up,
                            );
                        } else {
                            let origin = ctx.world.entity(self_).r.currentOrigin;
                            G_PlayEffectID(
                                G_EffectIndex(b"small_chunks\0".as_ptr() as *const c_char),
                                origin,
                                [0.0, 0.0, 0.0],
                            );
                            G_PlayEffectID(
                                G_EffectIndex(b"chunks/r2d2head\0".as_ptr() as *const c_char),
                                origin,
                                [0.0, 0.0, 0.0],
                            );
                        }

                        let time = ctx.world.level.time;
                        // gclient deref stays raw (client deref regime, task #7) — FLAG.
                        unsafe {
                            (*client).ps.electrifyTime = time + 3000;
                        }

                        TIMER_Set(
                            ctx,
                            Some(self_),
                            b"droidsmoketotal\0".as_ptr() as *const c_char,
                            5000,
                        );
                        TIMER_Set(ctx, Some(self_), b"droidspark\0".as_ptr() as *const c_char, 100);
                        // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
                        unsafe {
                            (*npc_ptr).localState = LSTATE_SPINNING;
                        }
                    }
                }
            } else {
                // gclient deref stays raw (client deref regime, task #7) — FLAG.
                let anim = unsafe { (*client).ps.legsAnim };

                if anim == BOTH_STAND2 as c_int {
                    NPC_SetAnim(
                        ctx,
                        self_,
                        SETANIM_BOTH,
                        BOTH_PAIN1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                } else {
                    NPC_SetAnim(
                        ctx,
                        self_,
                        SETANIM_BOTH,
                        BOTH_PAIN2 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                }

                // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
                unsafe {
                    (*npc_ptr).localState = LSTATE_SPINNING;
                }
                let delay = ctx.world.bg_state.rng.Q_irand(1000, 2000);
                TIMER_Set(ctx, Some(self_), b"roam\0".as_ptr() as *const c_char, delay);
            }
        }
    } else if npc_class == class_t::CLASS_INTERROGATOR
        && (mod_ == MOD_DEMP2 as c_int || mod_ == MOD_DEMP2_ALT as c_int)
        && attacker.is_some()
    {
        // CLASS_INTERROGATOR
        let attacker_id = attacker.unwrap();
        let mut dir = [0.0f32; 3];
        let self_origin = ctx.world.entity(self_).r.currentOrigin;
        let attacker_origin = ctx.world.entity(attacker_id).r.currentOrigin;
        crate::q_math::_VectorSubtract(self_origin, attacker_origin, &mut dir);
        VectorNormalize(&mut dir);

        // gclient deref stays raw (client deref regime, task #7) — FLAG.
        unsafe {
            let velocity = (*client).ps.velocity;
            crate::q_math::_VectorMA(velocity, 550.0, dir, &mut (*client).ps.velocity);
            (*client).ps.velocity[2] -= 127.0;
        }
    }

    NPC_Pain(ctx, self_, attacker, damage);
}

/// Raven `Droid_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:442-448`
pub fn Droid_Pain(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    if npc.is_null() || npc_info.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if TIMER_Done(ctx, Some(npc_id), b"droidpain\0".as_ptr() as *const c_char) != 0 {
        // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
        unsafe {
            (*npc_info).localState = LSTATE_NONE;
        }
    }
}

/// Raven `NPC_Mouse_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:455-467`
pub fn NPC_Mouse_Precache(ctx: &mut GameContext) {
    for i in 1..4 {
        let sound_path = format!("sound/chars/mouse/misc/mousego{}.wav", i);
        G_SoundIndex(cstr(&sound_path).as_ptr());
    }

    G_EffectIndex(b"env/small_explode\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/mouse/misc/death1\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/mouse/misc/mouse_lp\0".as_ptr() as *const c_char);
}

/// Raven `NPC_R5D2_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:474-490`
pub fn NPC_R5D2_Precache(ctx: &mut GameContext) {
    for i in 1..5 {
        let sound_path = format!("sound/chars/r5d2/misc/r5talk{}.wav", i);
        G_SoundIndex(cstr(&sound_path).as_ptr());
    }

    G_SoundIndex(b"sound/chars/mark2/misc/mark2_explo\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/r2d2/misc/r2_move_lp2.wav\0".as_ptr() as *const c_char);
    G_EffectIndex(b"env/med_explode\0".as_ptr() as *const c_char);
    G_EffectIndex(b"volumetric/droid_smoke\0".as_ptr() as *const c_char);
    G_EffectIndex(b"sparks/spark\0".as_ptr() as *const c_char);
    G_EffectIndex(b"chunks/r5d2head\0".as_ptr() as *const c_char);
    G_EffectIndex(b"chunks/r5d2head_veh\0".as_ptr() as *const c_char);
}

/// Raven `NPC_R2D2_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:497-513`
pub fn NPC_R2D2_Precache(ctx: &mut GameContext) {
    for i in 1..4 {
        let sound_path = format!("sound/chars/r2d2/misc/r2d2talk0{}.wav", i);
        G_SoundIndex(cstr(&sound_path).as_ptr());
    }

    G_SoundIndex(b"sound/chars/mark2/misc/mark2_explo\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/r2d2/misc/r2_move_lp.wav\0".as_ptr() as *const c_char);
    G_EffectIndex(b"env/med_explode\0".as_ptr() as *const c_char);
    G_EffectIndex(b"volumetric/droid_smoke\0".as_ptr() as *const c_char);
    G_EffectIndex(b"sparks/spark\0".as_ptr() as *const c_char);
    G_EffectIndex(b"chunks/r2d2head\0".as_ptr() as *const c_char);
    G_EffectIndex(b"chunks/r2d2head_veh\0".as_ptr() as *const c_char);
}

/// Raven `NPC_Gonk_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:520-530`
pub fn NPC_Gonk_Precache(ctx: &mut GameContext) {
    // SAFETY: string literals valid.
    G_SoundIndex(b"sound/chars/gonk/misc/gonktalk1.wav\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/gonk/misc/gonktalk2.wav\0".as_ptr() as *const c_char);

    G_SoundIndex(b"sound/chars/gonk/misc/death1.wav\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/gonk/misc/death2.wav\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/gonk/misc/death3.wav\0".as_ptr() as *const c_char);

    G_EffectIndex(b"env/med_explode\0".as_ptr() as *const c_char);
}

/// Raven `NPC_Protocol_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:537-541`
pub fn NPC_Protocol_Precache(ctx: &mut GameContext) {
    // SAFETY: string literals valid.
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_explo\0".as_ptr() as *const c_char);
    G_EffectIndex(b"env/med_explode\0".as_ptr() as *const c_char);
}

/// Raven `NPC_BSDroid_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:597-621`
pub fn NPC_BSDroid_Default(ctx: &mut GameContext) {
    let npc_info = ctx.world.globals.NPCInfo;
    if npc_info.is_null() {
        return;
    }

    // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
    let local_state = unsafe { (*npc_info).localState };
    if local_state == LSTATE_SPINNING {
        Droid_Spin(ctx);
    } else if local_state == LSTATE_PAIN {
        Droid_Pain(ctx);
    } else if local_state == LSTATE_DROP {
        NPC_UpdateAngles(ctx, 1 as qboolean, 1 as qboolean); // qtrue, qtrue
        ctx.world.globals.ucmd.upmove = (ctx.world.bg_state.rng.crandom() * 64.0) as c_int as i8;
    // gNPC_t deref stays raw (NPCInfo deref regime, task #7) — FLAG.
    } else if (unsafe { (*npc_info).scriptFlags } & SCF_LOOK_FOR_ENEMIES) != 0 {
        Droid_Patrol(ctx);
    } else {
        Droid_Run(ctx);
    }
}
