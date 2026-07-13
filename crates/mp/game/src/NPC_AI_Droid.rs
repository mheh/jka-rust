// PORT-COMPLETE: NPC_AI_Droid.c 14/14
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_Droid.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
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
use std::ffi::CStr;

// EntityId seam helper: resolve `Option<EntityId>` back to the raw pointer the
// verbatim body still expects (`None` -> null), per the `NPC_AI_Stormtrooper.rs`
// precedent.
#[inline]
unsafe fn ent_resolve_opt(ctx: &mut GameContext, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => unsafe { &mut (*ctx.world_raw()).g_entities[i.index()] as *mut gentity_t },
        None => core::ptr::null_mut(),
    }
}

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
    // PORT-NOTE(globals-access): NPC accessed as (*ctx.world_raw()).globals.NPC per threading digest
    unsafe {
        let npc = (*ctx.world_raw()).globals.NPC;
        if npc.is_null() {
            return;
        }

        if TIMER_Done(
            ctx,
            ctx.entity_id_of(npc),
            b"eyeDelay\0".as_ptr() as *const c_char,
        ) != 0
        {
            (*npc).pos1[1] = AngleNormalize360((*npc).pos1[1]);

            (*npc).pos1[0] += (*ctx.world_raw()).bg_state.rng.Q_irand(-20, 20) as f32;
            (*npc).pos1[1] = (*ctx.world_raw()).bg_state.rng.Q_irand(-20, 20) as f32;
            (*npc).pos1[2] = (*ctx.world_raw()).bg_state.rng.Q_irand(-20, 20) as f32;

            NPC_SetBoneAngles(
                ctx,
                ctx.entity_id_of(npc).unwrap(),
                b"f_eye\0".as_ptr() as *mut c_char,
                (*npc).pos1,
            );

            let __h14 = ctx.entity_id_of(npc);
            let __h15 = (*ctx.world_raw()).bg_state.rng.Q_irand(100, 1000);
            TIMER_Set(ctx, __h14, b"eyeDelay\0".as_ptr() as *const c_char, __h15);
        }
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
    unsafe {
        let npc = (*ctx.world_raw()).globals.NPC;
        let npc_info = (*ctx.world_raw()).globals.NPCInfo;
        if npc.is_null() || npc_info.is_null() {
            return;
        }

        let turndelta = AngleDelta((*npc).r.currentAngles[1], (*npc_info).desiredYaw); // YAW = 1
        let anim: c_int;

        if (turndelta.abs() > 20.0)
            && (((*((*npc).client as *mut gclient_t)).NPC_class == class_t::CLASS_R2D2)
                || ((*((*npc).client as *mut gclient_t)).NPC_class == class_t::CLASS_R5D2))
        {
            // CLASS_R2D2 = 2, CLASS_R5D2 = 3 (or check from globals)
            anim = (*((*npc).client as *mut gclient_t)).ps.legsAnim;
            if turndelta < 0.0 {
                if anim != BOTH_TURN_LEFT1 as c_int {
                    NPC_SetAnim(
                        ctx,
                        ctx.entity_id_of(npc).unwrap(),
                        SETANIM_BOTH,
                        BOTH_TURN_LEFT1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                }
            } else {
                if anim != BOTH_TURN_RIGHT1 as c_int {
                    NPC_SetAnim(
                        ctx,
                        ctx.entity_id_of(npc).unwrap(),
                        SETANIM_BOTH,
                        BOTH_TURN_RIGHT1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                }
            }
        } else {
            NPC_SetAnim(
                ctx,
                ctx.entity_id_of(npc).unwrap(),
                SETANIM_BOTH,
                BOTH_RUN1 as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
        }
    }
}

/// Raven `Droid_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:102-168`
pub fn Droid_Patrol(ctx: &mut GameContext) {
    unsafe {
        let npc = (*ctx.world_raw()).globals.NPC;
        let npc_info = (*ctx.world_raw()).globals.NPCInfo;
        let ucmd = &mut (*ctx.world_raw()).globals.ucmd;
        if npc.is_null() || npc_info.is_null() {
            return;
        }

        (*npc).pos1[1] = AngleNormalize360((*npc).pos1[1]);

        if !(*npc).client.is_null()
            && (*((*npc).client as *mut gclient_t)).NPC_class != class_t::CLASS_GONK
        {
            // CLASS_GONK
            if (*((*npc).client as *mut gclient_t)).NPC_class != class_t::CLASS_R5D2 {
                // CLASS_R5D2
                R2D2_PartsMove(ctx);
            }
            R2D2_TurnAnims(ctx);
        }

        if !UpdateGoal(ctx).is_null() {
            ucmd.buttons |= 1; // BUTTON_WALKING
            NPC_MoveToGoal(ctx, 1 as qboolean); // qtrue

            if !(*npc).client.is_null()
                && (*((*npc).client as *mut gclient_t)).NPC_class == class_t::CLASS_MOUSE
            {
                // CLASS_MOUSE
                // `.5` is a double literal and `sin` is the double libm: the whole
                // term is evaluated in f64 and narrowed only on store to the float.
                (*npc_info).desiredYaw = ((*npc_info).desiredYaw as f64
                    + ((*ctx.world_raw()).level.time as f64 * 0.5).sin() * 25.0)
                    as f32;

                if TIMER_Done(
                    ctx,
                    ctx.entity_id_of(npc),
                    b"patrolNoise\0".as_ptr() as *const c_char,
                ) != 0
                {
                    let idx = (*ctx.world_raw()).bg_state.rng.Q_irand(1, 3);
                    let sound_path = format!("sound/chars/mouse/misc/mousego{}.wav", idx);
                    let __h15 = ctx.entity_id_of(npc).unwrap();
                    G_SoundOnEnt(ctx, __h15, 0, cstr(&sound_path).as_ptr()); // CHAN_AUTO = 0
                    let __h16 = ctx.entity_id_of(npc);
                    let __h17 = (*ctx.world_raw()).bg_state.rng.Q_irand(2000, 4000);

                    TIMER_Set(
                        ctx,
                        __h16,
                        b"patrolNoise\0".as_ptr() as *const c_char,
                        __h17,
                    );
                }
            } else if !(*npc).client.is_null()
                && (*((*npc).client as *mut gclient_t)).NPC_class == class_t::CLASS_R2D2
            {
                // CLASS_R2D2
                if TIMER_Done(
                    ctx,
                    ctx.entity_id_of(npc),
                    b"patrolNoise\0".as_ptr() as *const c_char,
                ) != 0
                {
                    let idx = (*ctx.world_raw()).bg_state.rng.Q_irand(1, 3);
                    let sound_path = format!("sound/chars/r2d2/misc/r2d2talk0{}.wav", idx);
                    G_SoundOnEnt(
                        ctx,
                        ctx.entity_id_of(npc).unwrap(),
                        0,
                        cstr(&sound_path).as_ptr(),
                    );

                    let __h18 = ctx.entity_id_of(npc);
                    let __h19 = (*ctx.world_raw()).bg_state.rng.Q_irand(2000, 4000);
                    TIMER_Set(
                        ctx,
                        __h18,
                        b"patrolNoise\0".as_ptr() as *const c_char,
                        __h19,
                    );
                }
            } else if !(*npc).client.is_null()
                && (*((*npc).client as *mut gclient_t)).NPC_class == class_t::CLASS_R5D2
            {
                // CLASS_R5D2
                if TIMER_Done(
                    ctx,
                    ctx.entity_id_of(npc),
                    b"patrolNoise\0".as_ptr() as *const c_char,
                ) != 0
                {
                    let idx = (*ctx.world_raw()).bg_state.rng.Q_irand(1, 4);
                    let sound_path = format!("sound/chars/r5d2/misc/r5talk{}.wav", idx);
                    G_SoundOnEnt(
                        ctx,
                        ctx.entity_id_of(npc).unwrap(),
                        0,
                        cstr(&sound_path).as_ptr(),
                    );

                    let __h20 = ctx.entity_id_of(npc);
                    let __h21 = (*ctx.world_raw()).bg_state.rng.Q_irand(2000, 4000);
                    TIMER_Set(
                        ctx,
                        __h20,
                        b"patrolNoise\0".as_ptr() as *const c_char,
                        __h21,
                    );
                }
            }
            if !(*npc).client.is_null()
                && (*((*npc).client as *mut gclient_t)).NPC_class == class_t::CLASS_GONK
            {
                // CLASS_GONK
                if TIMER_Done(
                    ctx,
                    ctx.entity_id_of(npc),
                    b"patrolNoise\0".as_ptr() as *const c_char,
                ) != 0
                {
                    let idx = (*ctx.world_raw()).bg_state.rng.Q_irand(1, 2);
                    let sound_path = format!("sound/chars/gonk/misc/gonktalk{}.wav", idx);
                    G_SoundOnEnt(
                        ctx,
                        ctx.entity_id_of(npc).unwrap(),
                        0,
                        cstr(&sound_path).as_ptr(),
                    );

                    let __h22 = ctx.entity_id_of(npc);
                    let __h23 = (*ctx.world_raw()).bg_state.rng.Q_irand(2000, 4000);
                    TIMER_Set(
                        ctx,
                        __h22,
                        b"patrolNoise\0".as_ptr() as *const c_char,
                        __h23,
                    );
                }
            }
        }

        NPC_UpdateAngles(ctx, 1 as qboolean, 1 as qboolean); // qtrue, qtrue
    }
}

/// Raven `Droid_Run`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:175-200`
pub fn Droid_Run(ctx: &mut GameContext) {
    unsafe {
        let npc = (*ctx.world_raw()).globals.NPC;
        let npc_info = (*ctx.world_raw()).globals.NPCInfo;
        let ucmd = &mut (*ctx.world_raw()).globals.ucmd;
        if npc.is_null() || npc_info.is_null() {
            return;
        }

        R2D2_PartsMove(ctx);

        if (*npc_info).localState == LSTATE_BACKINGUP {
            ucmd.forwardmove = -127;
            (*npc_info).desiredYaw += 5.0;

            (*npc_info).localState = LSTATE_NONE;
        } else {
            ucmd.forwardmove = 64;
            if !UpdateGoal(ctx).is_null() {
                if NPC_MoveToGoal(ctx, 0 as qboolean) != 0 {
                    // qfalse
                    // `.5` is a double literal and `sin` is the double libm: the whole
                    // term is evaluated in f64 and narrowed only on store to the float.
                    (*npc_info).desiredYaw = ((*npc_info).desiredYaw as f64
                        + ((*ctx.world_raw()).level.time as f64 * 0.5).sin() * 5.0)
                        as f32;
                }
            }
        }

        NPC_UpdateAngles(ctx, 1 as qboolean, 1 as qboolean); // qtrue, qtrue
    }
}

/// Raven `Droid_Spin`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:207-266`
pub fn Droid_Spin(ctx: &mut GameContext) {
    unsafe {
        let npc = (*ctx.world_raw()).globals.NPC;
        let npc_info = (*ctx.world_raw()).globals.NPCInfo;
        let ucmd = &mut (*ctx.world_raw()).globals.ucmd;
        if npc.is_null() || npc_info.is_null() {
            return;
        }

        let mut dir = [0.0f32, 0.0f32, 1.0f32];

        R2D2_TurnAnims(ctx);

        if (*((*npc).client as *mut gclient_t)).NPC_class == class_t::CLASS_R5D2
            || (*((*npc).client as *mut gclient_t)).NPC_class == class_t::CLASS_R2D2
        {
            // CLASS_R5D2, CLASS_R2D2
            // No head?
            if trap::G2API_GetSurfaceRenderStatus(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                    (*npc).ghoul2,
                    0,
                    c"head".to_owned(),
                ),
            ) > 0 {
                if TIMER_Done(ctx, ctx.entity_id_of(npc), b"smoke\0".as_ptr() as *const c_char) != 0 && TIMER_Done(ctx, ctx.entity_id_of(npc), b"droidsmoketotal\0".as_ptr() as *const c_char) == 0 {
                    TIMER_Set(ctx, ctx.entity_id_of(npc), b"smoke\0".as_ptr() as *const c_char, 100);
                    G_PlayEffectID(G_EffectIndex(b"volumetric/droid_smoke\0".as_ptr() as *const c_char), (*npc).r.currentOrigin, dir);
                }

                if TIMER_Done(ctx, ctx.entity_id_of(npc), b"droidspark\0".as_ptr() as *const c_char) != 0 {
                    let __h24 = ctx.entity_id_of(npc);
                    let __h25 = (*ctx.world_raw()).bg_state.rng.Q_irand(100, 500);
                    TIMER_Set(ctx, __h24, b"droidspark\0".as_ptr() as *const c_char, __h25);
                    G_PlayEffectID(G_EffectIndex(b"sparks/spark\0".as_ptr() as *const c_char), (*npc).r.currentOrigin, dir);
                }

                ucmd.forwardmove = (*ctx.world_raw()).bg_state.rng.Q_irand(-64, 64) as c_int as i8;

                if TIMER_Done(ctx, ctx.entity_id_of(npc), b"roam\0".as_ptr() as *const c_char) != 0 {
                    let __h26 = ctx.entity_id_of(npc);
                    let __h27 = (*ctx.world_raw()).bg_state.rng.Q_irand(250, 1000);
                    TIMER_Set(ctx, __h26, b"roam\0".as_ptr() as *const c_char, __h27);
                    (*npc_info).desiredYaw = (*ctx.world_raw()).bg_state.rng.Q_irand(0, 360) as f32;
                }
            } else {
                if TIMER_Done(ctx, ctx.entity_id_of(npc), b"roam\0".as_ptr() as *const c_char) != 0 {
                    (*npc_info).localState = LSTATE_NONE;
                } else {
                    (*npc_info).desiredYaw = AngleNormalize360((*npc_info).desiredYaw + 40.0);
                }
            }
        } else {
            if TIMER_Done(
                ctx,
                ctx.entity_id_of(npc),
                b"roam\0".as_ptr() as *const c_char,
            ) != 0
            {
                (*npc_info).localState = LSTATE_NONE;
            } else {
                (*npc_info).desiredYaw = AngleNormalize360((*npc_info).desiredYaw + 40.0);
            }
        }

        NPC_UpdateAngles(ctx, 1 as qboolean, 1 as qboolean); // qtrue, qtrue
    }
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
    // STAGE-1: EntityId params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let attacker: *mut gentity_t = unsafe { ent_resolve_opt(ctx, attacker) };
    unsafe {
        let mod_: c_int = (*ctx.world_raw()).globals.gPainMOD;
        let mut pain_chance: f32;

        // VectorCopy( self->NPC->lastPathAngles, self->s.angles )
        crate::q_math::_VectorCopy(
            (*((*self_).NPC as *mut gNPC_t)).lastPathAngles,
            &mut (*self_).s.angles,
        );

        if (*((*self_).client as *mut gclient_t)).NPC_class == class_t::CLASS_R5D2 {
            // CLASS_R5D2
            pain_chance = NPC_GetPainChance(ctx, ctx.entity_id_of(self_).unwrap(), damage);

            if mod_ == MOD_DEMP2 as c_int
                || mod_ == MOD_DEMP2_ALT as c_int
                || (*ctx.world_raw()).bg_state.rng.random() < pain_chance
            {
                if (*self_).s.m_iVehicleNum == 0
                    && ((*self_).health < 30
                        || mod_ == MOD_DEMP2 as c_int
                        || mod_ == MOD_DEMP2_ALT as c_int)
                {
                    if ((*self_).spawnflags & 2) == 0 {
                        if ((*((*self_).NPC as *mut gNPC_t)).localState != LSTATE_SPINNING) &&
                           (trap::G2API_GetSurfaceRenderStatus(
                                ctx.engine,
                                mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                                    (*self_).ghoul2,
                                    0,
                                    c"head".to_owned(),
                                ),
                            ) == 0) {
                            NPC_SetSurfaceOnOff(ctx, ctx.entity_id_of(self_).unwrap(), b"head\0".as_ptr() as *const c_char, TURN_OFF);

                            if (*((*self_).client as *mut gclient_t)).ps.m_iVehicleNum != 0 {
                                let mut up = [0.0f32; 3];
                                AngleVectors((*self_).r.currentAngles, None, None, Some(&mut up));
                                G_PlayEffectID(G_EffectIndex(b"chunks/r5d2head_veh\0".as_ptr() as *const c_char), (*self_).r.currentOrigin, up);
                            } else {
                                G_PlayEffectID(G_EffectIndex(b"small_chunks\0".as_ptr() as *const c_char), (*self_).r.currentOrigin, [0.0, 0.0, 0.0]);
                                G_PlayEffectID(G_EffectIndex(b"chunks/r5d2head\0".as_ptr() as *const c_char), (*self_).r.currentOrigin, [0.0, 0.0, 0.0]);
                            }

                            (*((*self_).client as *mut gclient_t)).ps.electrifyTime = (*ctx.world_raw()).level.time + 3000;

                            TIMER_Set(ctx, ctx.entity_id_of(self_), b"droidsmoketotal\0".as_ptr() as *const c_char, 5000);
                            TIMER_Set(ctx, ctx.entity_id_of(self_), b"droidspark\0".as_ptr() as *const c_char, 100);
                            (*((*self_).NPC as *mut gNPC_t)).localState = LSTATE_SPINNING;
                        }
                    }
                } else {
                    let anim = (*((*self_).client as *mut gclient_t)).ps.legsAnim;

                    if anim == BOTH_STAND2 as c_int {
                        NPC_SetAnim(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            SETANIM_BOTH,
                            BOTH_PAIN1 as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    } else {
                        NPC_SetAnim(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            SETANIM_BOTH,
                            BOTH_PAIN2 as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    }

                    (*((*self_).NPC as *mut gNPC_t)).localState = LSTATE_SPINNING;
                    let __h28 = ctx.entity_id_of(self_);
                    let __h29 = (*ctx.world_raw()).bg_state.rng.Q_irand(1000, 2000);
                    TIMER_Set(ctx, __h28, b"roam\0".as_ptr() as *const c_char, __h29);
                }
            }
        } else if (*((*self_).client as *mut gclient_t)).NPC_class == class_t::CLASS_MOUSE {
            // CLASS_MOUSE
            if mod_ == MOD_DEMP2 as c_int || mod_ == MOD_DEMP2_ALT as c_int {
                (*((*self_).NPC as *mut gNPC_t)).localState = LSTATE_SPINNING;
                (*((*self_).client as *mut gclient_t)).ps.electrifyTime =
                    (*ctx.world_raw()).level.time + 3000;
            } else {
                (*((*self_).NPC as *mut gNPC_t)).localState = LSTATE_BACKINGUP;
            }

            (*((*self_).NPC as *mut gNPC_t)).scriptFlags &= !SCF_LOOK_FOR_ENEMIES;
        } else if (*((*self_).client as *mut gclient_t)).NPC_class == class_t::CLASS_R2D2 {
            // CLASS_R2D2
            pain_chance = NPC_GetPainChance(ctx, ctx.entity_id_of(self_).unwrap(), damage);

            if mod_ == MOD_DEMP2 as c_int
                || mod_ == MOD_DEMP2_ALT as c_int
                || (*ctx.world_raw()).bg_state.rng.random() < pain_chance
            {
                if (*self_).s.m_iVehicleNum == 0
                    && ((*self_).health < 30
                        || mod_ == MOD_DEMP2 as c_int
                        || mod_ == MOD_DEMP2_ALT as c_int)
                {
                    if ((*self_).spawnflags & 2) == 0 {
                        if ((*((*self_).NPC as *mut gNPC_t)).localState != LSTATE_SPINNING) &&
                           (trap::G2API_GetSurfaceRenderStatus(
                                ctx.engine,
                                mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                                    (*self_).ghoul2,
                                    0,
                                    c"head".to_owned(),
                                ),
                            ) == 0) {
                            NPC_SetSurfaceOnOff(ctx, ctx.entity_id_of(self_).unwrap(), b"head\0".as_ptr() as *const c_char, TURN_OFF);

                            if (*((*self_).client as *mut gclient_t)).ps.m_iVehicleNum != 0 {
                                let mut up = [0.0f32; 3];
                                AngleVectors((*self_).r.currentAngles, None, None, Some(&mut up));
                                G_PlayEffectID(G_EffectIndex(b"chunks/r2d2head_veh\0".as_ptr() as *const c_char), (*self_).r.currentOrigin, up);
                            } else {
                                G_PlayEffectID(G_EffectIndex(b"small_chunks\0".as_ptr() as *const c_char), (*self_).r.currentOrigin, [0.0, 0.0, 0.0]);
                                G_PlayEffectID(G_EffectIndex(b"chunks/r2d2head\0".as_ptr() as *const c_char), (*self_).r.currentOrigin, [0.0, 0.0, 0.0]);
                            }

                            (*((*self_).client as *mut gclient_t)).ps.electrifyTime = (*ctx.world_raw()).level.time + 3000;

                            TIMER_Set(ctx, ctx.entity_id_of(self_), b"droidsmoketotal\0".as_ptr() as *const c_char, 5000);
                            TIMER_Set(ctx, ctx.entity_id_of(self_), b"droidspark\0".as_ptr() as *const c_char, 100);
                            (*((*self_).NPC as *mut gNPC_t)).localState = LSTATE_SPINNING;
                        }
                    }
                } else {
                    let anim = (*((*self_).client as *mut gclient_t)).ps.legsAnim;

                    if anim == BOTH_STAND2 as c_int {
                        NPC_SetAnim(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            SETANIM_BOTH,
                            BOTH_PAIN1 as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    } else {
                        NPC_SetAnim(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            SETANIM_BOTH,
                            BOTH_PAIN2 as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    }

                    (*((*self_).NPC as *mut gNPC_t)).localState = LSTATE_SPINNING;
                    let __h30 = ctx.entity_id_of(self_);
                    let __h31 = (*ctx.world_raw()).bg_state.rng.Q_irand(1000, 2000);
                    TIMER_Set(ctx, __h30, b"roam\0".as_ptr() as *const c_char, __h31);
                }
            }
        } else if (*((*self_).client as *mut gclient_t)).NPC_class == class_t::CLASS_INTERROGATOR
            && (mod_ == MOD_DEMP2 as c_int || mod_ == MOD_DEMP2_ALT as c_int)
            && !attacker.is_null()
        {
            // CLASS_INTERROGATOR
            let mut dir = [0.0f32; 3];
            crate::q_math::_VectorSubtract(
                (*self_).r.currentOrigin,
                (*attacker).r.currentOrigin,
                &mut dir,
            );
            VectorNormalize(&mut dir);

            crate::q_math::_VectorMA(
                (*((*self_).client as *mut gclient_t)).ps.velocity,
                550.0,
                dir,
                &mut (*((*self_).client as *mut gclient_t)).ps.velocity,
            );
            (*((*self_).client as *mut gclient_t)).ps.velocity[2] -= 127.0;
        }

        NPC_Pain(
            ctx,
            ctx.entity_id_of(self_).unwrap(),
            ctx.entity_id_of(attacker),
            damage,
        );
    }
}

/// Raven `Droid_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:442-448`
pub fn Droid_Pain(ctx: &mut GameContext) {
    unsafe {
        let npc = (*ctx.world_raw()).globals.NPC;
        let npc_info = (*ctx.world_raw()).globals.NPCInfo;
        if npc.is_null() || npc_info.is_null() {
            return;
        }

        if TIMER_Done(
            ctx,
            ctx.entity_id_of(npc),
            b"droidpain\0".as_ptr() as *const c_char,
        ) != 0
        {
            (*npc_info).localState = LSTATE_NONE;
        }
    }
}

/// Raven `NPC_Mouse_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:455-467`
pub fn NPC_Mouse_Precache(ctx: &mut GameContext) {
    unsafe {
        for i in 1..4 {
            let sound_path = format!("sound/chars/mouse/misc/mousego{}.wav", i);
            G_SoundIndex(cstr(&sound_path).as_ptr());
        }

        G_EffectIndex(b"env/small_explode\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/mouse/misc/death1\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/mouse/misc/mouse_lp\0".as_ptr() as *const c_char);
    }
}

/// Raven `NPC_R5D2_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:474-490`
pub fn NPC_R5D2_Precache(ctx: &mut GameContext) {
    unsafe {
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
}

/// Raven `NPC_R2D2_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:497-513`
pub fn NPC_R2D2_Precache(ctx: &mut GameContext) {
    unsafe {
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
}

/// Raven `NPC_Gonk_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:520-530`
pub fn NPC_Gonk_Precache(ctx: &mut GameContext) {
    unsafe {
        // SAFETY: string literals valid.
        G_SoundIndex(b"sound/chars/gonk/misc/gonktalk1.wav\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/gonk/misc/gonktalk2.wav\0".as_ptr() as *const c_char);

        G_SoundIndex(b"sound/chars/gonk/misc/death1.wav\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/gonk/misc/death2.wav\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/gonk/misc/death3.wav\0".as_ptr() as *const c_char);

        G_EffectIndex(b"env/med_explode\0".as_ptr() as *const c_char);
    }
}

/// Raven `NPC_Protocol_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:537-541`
pub fn NPC_Protocol_Precache(ctx: &mut GameContext) {
    unsafe {
        // SAFETY: string literals valid.
        G_SoundIndex(b"sound/chars/mark2/misc/mark2_explo\0".as_ptr() as *const c_char);
        G_EffectIndex(b"env/med_explode\0".as_ptr() as *const c_char);
    }
}

/// Raven `NPC_BSDroid_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_Droid.c:597-621`
pub fn NPC_BSDroid_Default(ctx: &mut GameContext) {
    unsafe {
        let npc_info = (*ctx.world_raw()).globals.NPCInfo;
        if npc_info.is_null() {
            return;
        }

        if (*npc_info).localState == LSTATE_SPINNING {
            Droid_Spin(ctx);
        } else if (*npc_info).localState == LSTATE_PAIN {
            Droid_Pain(ctx);
        } else if (*npc_info).localState == LSTATE_DROP {
            NPC_UpdateAngles(ctx, 1 as qboolean, 1 as qboolean); // qtrue, qtrue
            (*ctx.world_raw()).globals.ucmd.upmove =
                ((*ctx.world_raw()).bg_state.rng.crandom() * 64.0) as c_int as i8;
        } else if ((*npc_info).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            Droid_Patrol(ctx);
        } else {
            Droid_Run(ctx);
        }
    }
}
