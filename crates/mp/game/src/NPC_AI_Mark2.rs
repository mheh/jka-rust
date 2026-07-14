// PORT-COMPLETE: NPC_AI_Mark2.c 10/10
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_Mark2.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::bg_misc::{BG_FindItemForAmmo, BG_FindItemForWeapon};
use crate::entity::hit_location::HL_GENERIC1;
use crate::g_combat::G_Damage;
use crate::g_items::RegisterItem;
use crate::g_utils::{G_EffectIndex, G_PlayEffectID, G_Sound, G_SoundIndex};
use crate::level::damage_flags::DAMAGE_NO_PROTECTION;
use crate::prelude::*;
use crate::q_shared::va;
use crate::trap;
use crate::NPC_reactions::NPC_Pain;
use crate::NPC_utils::NPC_SetSurfaceOnOff;

// EntityId seam helper: resolve `Option<EntityId>` back to the raw pointer the
// verbatim body still expects (`None` -> null), per the `NPC_AI_Stormtrooper.rs`
// precedent.
#[inline]
unsafe fn ent_resolve_opt(ctx: &mut GameContext, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => &mut ctx.world.g_entities[i.index()] as *mut gentity_t,
        None => core::ptr::null_mut(),
    }
}

/// Raven ammo pod health.
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:4-5`
const AMMO_POD_HEALTH: c_int = 1;

/// Surface render status flag: turn off.
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:6`
const TURN_OFF: c_int = 0x00000100;

/// Local state enums.
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:19-25`
const LSTATE_NONE: c_int = 0;
pub const LSTATE_DROPPINGDOWN: c_int = 1;
pub const LSTATE_DOWN: c_int = 2;
pub const LSTATE_RISINGUP: c_int = 3;

/// Distance constants for Mark2.
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:8-12`
const MIN_DISTANCE: c_int = 24;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

/// Raven `NPC_Mark2_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:27-43`
pub fn NPC_Mark2_Precache(ctx: &mut GameContext) {
    // SAFETY: G_SoundIndex, G_EffectIndex, RegisterItem accessed through game context.
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_explo\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_pain\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_fire\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_move_lp\0".as_ptr() as *const c_char);

    G_EffectIndex(b"explosions/droidexplosion1\0".as_ptr() as *const c_char);
    G_EffectIndex(b"env/med_explode2\0".as_ptr() as *const c_char);
    G_EffectIndex(b"blaster/smoke_bolton\0".as_ptr() as *const c_char);
    G_EffectIndex(b"bryar/muzzle_flash\0".as_ptr() as *const c_char);

    RegisterItem(ctx, BG_FindItemForWeapon(WP_BRYAR_PISTOL));
    RegisterItem(ctx, BG_FindItemForAmmo(AMMO_METAL_BOLTS));
    RegisterItem(ctx, BG_FindItemForAmmo(AMMO_POWERCELL));
    RegisterItem(ctx, BG_FindItemForAmmo(AMMO_BLASTER));
}

/// Raven `NPC_Mark2_Part_Explode`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:50-72`
pub fn NPC_Mark2_Part_Explode(ctx: &mut GameContext, self_: EntityId, bolt: c_int) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        if bolt >= 0 {
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let mut org: vec3_t = [0.0; 3];
            let mut dir: vec3_t = [0.0; 3];

            trap::G2API_GetBoltMatrix(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                    (*self_).ghoul2,
                    0,
                    bolt,
                    &mut boltMatrix as *mut mdxaBone_t,
                    &(*self_).r.currentAngles as *const vec3_t,
                    &(*self_).r.currentOrigin as *const vec3_t,
                    ctx.world.level.time,
                    core::ptr::null_mut(),
                    &(*self_).modelScale as *const vec3_t,
                ),
            );

            BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut org);
            BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::NEGATIVE_Y as c_int, &mut dir);

            G_PlayEffectID(
                G_EffectIndex(b"env/med_explode2\0".as_ptr() as *const c_char),
                org,
                dir,
            );
            G_PlayEffectID(
                G_EffectIndex(b"blaster/smoke_bolton\0".as_ptr() as *const c_char),
                org,
                dir,
            );
        }

        (*self_).count += 1;
    }
}

/// Raven `NPC_Mark2_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:80-111`
pub fn NPC_Mark2_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    // STAGE-1: EntityId params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let attacker: *mut gentity_t = unsafe { ent_resolve_opt(ctx, attacker) };
    unsafe {
        let hit_loc = ctx.world.globals.gPainHitLoc;

        NPC_Pain(
            ctx,
            ctx.entity_id_of(self_).unwrap(),
            ctx.entity_id_of(attacker),
            damage,
        );

        for i in 0..3 {
            if hit_loc == HL_GENERIC1 + i
                && (*self_).locationDamage[(HL_GENERIC1 + i) as usize] > AMMO_POD_HEALTH
            {
                if (*self_).locationDamage[hit_loc as usize] >= AMMO_POD_HEALTH {
                    let surface_name = cstr(&format!("torso_canister{}", (i + 1) as c_int));
                    let new_bolt = trap::G2API_AddBolt(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                            (*self_).ghoul2,
                            0,
                            surface_name.clone(),
                        ),
                    );
                    if new_bolt != -1 {
                        NPC_Mark2_Part_Explode(ctx, ctx.entity_id_of(self_).unwrap(), new_bolt);
                    }
                    NPC_SetSurfaceOnOff(
                        ctx,
                        ctx.entity_id_of(self_).unwrap(),
                        surface_name.as_ptr(),
                        TURN_OFF,
                    );
                    break;
                }
            }
        }

        G_Sound(
            ctx,
            ctx.entity_id_of(self_),
            CHAN_AUTO,
            G_SoundIndex(b"sound/chars/mark2/misc/mark2_pain\0".as_ptr() as *const c_char),
        );

        if (*self_).count > 0 {
            G_Damage(
                ctx,
                ctx.entity_id_of(self_),
                ctx.entity_id_of(core::ptr::null_mut()),
                ctx.entity_id_of(core::ptr::null_mut()),
                None,
                [0.0; 3],
                (*self_).health,
                crate::level::damage_flags::DAMAGE_NO_PROTECTION,
                MOD_UNKNOWN as c_int,
            );
        }
    }
}

/// Raven `Mark2_Hunt`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:118-130`
pub fn Mark2_Hunt(ctx: &mut GameContext) {
    unsafe {
        let npc_ptr = ctx.world.globals.NPC;
        let npc_info_ptr = ctx.world.globals.NPCInfo;

        if (*npc_info_ptr).goalEntity.is_none() {
            (*npc_info_ptr).goalEntity = (*npc_ptr).enemy;
        }

        NPC_FaceEnemy(ctx, qtrue);

        (*npc_info_ptr).combatMove = qtrue;
        NPC_MoveToGoal(ctx, qtrue);
    }
}

/// Raven `Mark2_FireBlaster`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:137-179`
pub fn Mark2_FireBlaster(ctx: &mut GameContext, advance: qboolean) {
    unsafe {
        let mut muzzle1: vec3_t = [0.0; 3];
        let mut enemy_org1: vec3_t = [0.0; 3];
        let mut delta1: vec3_t = [0.0; 3];
        let mut angleToEnemy1: vec3_t = [0.0; 3];
        let mut forward: vec3_t = [0.0; 3];
        let mut vright: vec3_t = [0.0; 3];
        let mut up: vec3_t = [0.0; 3];

        let npc_ptr = ctx.world.globals.NPC;
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();

        let bolt = trap::G2API_AddBolt(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                (*npc_ptr).ghoul2,
                0,
                c"*flash".to_owned(),
            ),
        );

        trap::G2API_GetBoltMatrix(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                (*npc_ptr).ghoul2,
                0,
                bolt,
                &mut boltMatrix as *mut mdxaBone_t,
                &(*npc_ptr).r.currentAngles as *const vec3_t,
                &(*npc_ptr).r.currentOrigin as *const vec3_t,
                ctx.world.level.time,
                core::ptr::null_mut(),
                &(*npc_ptr).modelScale as *const vec3_t,
            ),
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut muzzle1);

        if (*npc_ptr).health != 0 {
            let enemy_ptr = if let Some(eid) = (*npc_ptr).enemy {
                &ctx.world.g_entities[eid.0 as usize] as *const gentity_t
            } else {
                core::ptr::null()
            };
            CalcEntitySpot(
                ctx,
                ctx.entity_id_of(enemy_ptr),
                crate::npc::spot_t::spot_t::SPOT_HEAD,
                &mut enemy_org1,
            );
            crate::q_math::_VectorSubtract(enemy_org1, muzzle1, &mut delta1);
            vectoangles(delta1, &mut angleToEnemy1);
            AngleVectors(
                angleToEnemy1,
                Some(&mut forward),
                Some(&mut vright),
                Some(&mut up),
            );
        } else {
            AngleVectors(
                (*npc_ptr).r.currentAngles,
                Some(&mut forward),
                Some(&mut vright),
                Some(&mut up),
            );
        }

        G_PlayEffectID(
            G_EffectIndex(b"bryar/muzzle_flash\0".as_ptr() as *const c_char),
            muzzle1,
            forward,
        );

        G_Sound(
            ctx,
            ctx.entity_id_of(npc_ptr),
            CHAN_AUTO,
            G_SoundIndex(b"sound/chars/mark2/misc/mark2_fire\0".as_ptr() as *const c_char),
        );

        let missile = crate::g_missile::CreateMissile(
            ctx,
            muzzle1,
            forward,
            1600.0,
            10000,
            ctx.entity_id_of(npc_ptr).unwrap(),
            qfalse,
        );

        (*missile).classname = b"bryar_proj\0".as_ptr() as *mut c_char;
        (*missile).s.weapon = WP_BRYAR_PISTOL as c_int;

        (*missile).damage = 1;
        (*missile).dflags = crate::level::damage_flags::DAMAGE_DEATH_KNOCKBACK;
        (*missile).methodOfDeath = MOD_BRYAR_PISTOL as c_int;
        (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    }
}

/// Raven `Mark2_BlasterAttack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:186-205`
pub fn Mark2_BlasterAttack(ctx: &mut GameContext, advance: qboolean) {
    unsafe {
        let npc_ptr = ctx.world.globals.NPC;
        let npc_info_ptr = ctx.world.globals.NPCInfo;

        if TIMER_Done(
            ctx,
            ctx.entity_id_of(npc_ptr),
            b"attackDelay\0".as_ptr() as *const c_char,
        ) == qtrue
        {
            if (*npc_info_ptr).localState == LSTATE_NONE {
                let npc_id = ctx.entity_id_of(npc_ptr);
                let delay = ctx.world.bg_state.rng.Q_irand(500, 2000);
                TIMER_Set(
                    ctx,
                    npc_id,
                    b"attackDelay\0".as_ptr() as *const c_char,
                    delay,
                );
            } else {
                let npc_id = ctx.entity_id_of(npc_ptr);
                let delay = ctx.world.bg_state.rng.Q_irand(100, 500);
                TIMER_Set(
                    ctx,
                    npc_id,
                    b"attackDelay\0".as_ptr() as *const c_char,
                    delay,
                );
            }
            Mark2_FireBlaster(ctx, advance);
            return;
        } else if advance == qtrue {
            Mark2_Hunt(ctx);
        }
    }
}

/// Raven `Mark2_AttackDecision`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:212-295`
pub fn Mark2_AttackDecision(ctx: &mut GameContext) {
    unsafe {
        let npc_ptr = ctx.world.globals.NPC;
        let npc_info_ptr = ctx.world.globals.NPCInfo;

        NPC_FaceEnemy(ctx, qtrue);

        let distance = {
            let d = crate::q_math::DistanceHorizontalSquared(
                (*npc_ptr).r.currentOrigin,
                (*npc_ptr)
                    .enemy
                    .map(|eid| ctx.world.g_entities[eid.0 as usize].r.currentOrigin)
                    .unwrap_or([0.0; 3]),
            );
            d as c_int
        } as i64;
        let enemy_ptr = if let Some(eid) = (*npc_ptr).enemy {
            &mut ctx.world.g_entities[eid.0 as usize] as *mut gentity_t
        } else {
            core::ptr::null_mut()
        };
        let visible = NPC_ClearLOS4(ctx, ctx.entity_id_of(enemy_ptr));
        let advance = (distance > MIN_DISTANCE_SQR as i64) as qboolean;

        if (*npc_info_ptr).localState == LSTATE_RISINGUP {
            (*npc_ptr).flags &= !FL_SHIELDED;
            NPC_SetAnim(
                ctx,
                ctx.entity_id_of(npc_ptr).unwrap(),
                SETANIM_BOTH,
                BOTH_RUN1START as c_int,
                SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
            );
            if (*((*npc_ptr).client as *mut gclient_t)).ps.legsTimer <= 0
                && (*((*npc_ptr).client as *mut gclient_t)).ps.torsoAnim == BOTH_RUN1START as c_int
            {
                (*npc_info_ptr).localState = LSTATE_NONE;
            }
            return;
        }

        if visible == qfalse || NPC_FaceEnemy(ctx, qtrue) == qfalse {
            if (*npc_info_ptr).localState == LSTATE_DOWN
                || (*npc_info_ptr).localState == LSTATE_DROPPINGDOWN
            {
                if TIMER_Done(
                    ctx,
                    ctx.entity_id_of(npc_ptr),
                    b"downTime\0".as_ptr() as *const c_char,
                ) == qtrue
                {
                    (*npc_info_ptr).localState = LSTATE_RISINGUP;
                    NPC_SetAnim(
                        ctx,
                        ctx.entity_id_of(npc_ptr).unwrap(),
                        SETANIM_BOTH,
                        BOTH_RUN1STOP as c_int,
                        SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
                    );
                    let npc_id = ctx.entity_id_of(npc_ptr);
                    let delay = ctx.world.bg_state.rng.Q_irand(3000, 8000);
                    TIMER_Set(ctx, npc_id, b"runTime\0".as_ptr() as *const c_char, delay);
                }
            } else {
                Mark2_Hunt(ctx);
            }
            return;
        }

        if advance == qtrue
            && TIMER_Done(
                ctx,
                ctx.entity_id_of(npc_ptr),
                b"downTime\0".as_ptr() as *const c_char,
            ) == qtrue
            && (*npc_info_ptr).localState == LSTATE_DOWN
        {
            (*npc_info_ptr).localState = LSTATE_RISINGUP;
            NPC_SetAnim(
                ctx,
                ctx.entity_id_of(npc_ptr).unwrap(),
                SETANIM_BOTH,
                BOTH_RUN1STOP as c_int,
                SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
            );
            let npc_id = ctx.entity_id_of(npc_ptr);
            let delay = ctx.world.bg_state.rng.Q_irand(3000, 8000);
            TIMER_Set(ctx, npc_id, b"runTime\0".as_ptr() as *const c_char, delay);
        }

        NPC_FaceEnemy(ctx, qtrue);

        if (*npc_info_ptr).localState == LSTATE_DROPPINGDOWN {
            NPC_SetAnim(
                ctx,
                ctx.entity_id_of(npc_ptr).unwrap(),
                SETANIM_BOTH,
                BOTH_RUN1STOP as c_int,
                SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
            );
            let npc_id = ctx.entity_id_of(npc_ptr);
            let delay = ctx.world.bg_state.rng.Q_irand(3000, 9000);
            TIMER_Set(ctx, npc_id, b"downTime\0".as_ptr() as *const c_char, delay);

            if (*((*npc_ptr).client as *mut gclient_t)).ps.legsTimer <= 0
                && (*((*npc_ptr).client as *mut gclient_t)).ps.torsoAnim == BOTH_RUN1STOP as c_int
            {
                (*npc_ptr).flags |= FL_SHIELDED;
                (*npc_info_ptr).localState = LSTATE_DOWN;
            }
        } else if (*npc_info_ptr).localState == LSTATE_DOWN {
            (*npc_ptr).flags |= FL_SHIELDED;
            Mark2_BlasterAttack(ctx, qfalse);
        } else if TIMER_Done(
            ctx,
            ctx.entity_id_of(npc_ptr),
            b"runTime\0".as_ptr() as *const c_char,
        ) == qtrue
        {
            (*npc_info_ptr).localState = LSTATE_DROPPINGDOWN;
        } else if advance == qtrue {
            Mark2_BlasterAttack(ctx, advance);
        }
    }
}

/// Raven `Mark2_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:303-330`
pub fn Mark2_Patrol(ctx: &mut GameContext) {
    unsafe {
        let npc_ptr = ctx.world.globals.NPC;

        if NPC_CheckPlayerTeamStealth(ctx) == qtrue {
            NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        if (*npc_ptr).enemy.is_none() {
            if UpdateGoal(ctx) != core::ptr::null_mut() {
                ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
                NPC_MoveToGoal(ctx, qtrue);
                NPC_UpdateAngles(ctx, qtrue, qtrue);
            }

            if TIMER_Done(
                ctx,
                ctx.entity_id_of(npc_ptr),
                b"patrolNoise\0".as_ptr() as *const c_char,
            ) == qtrue
            {
                let npc_id = ctx.entity_id_of(npc_ptr);
                let delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                TIMER_Set(
                    ctx,
                    npc_id,
                    b"patrolNoise\0".as_ptr() as *const c_char,
                    delay,
                );
            }
        }
    }
}

/// Raven `Mark2_Idle`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:337-340`
pub fn Mark2_Idle(ctx: &mut GameContext) {
    crate::NPC_AI_Default::NPC_BSIdle(ctx);
}

/// Raven `NPC_BSMark2_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:347-362`
pub fn NPC_BSMark2_Default(ctx: &mut GameContext) {
    unsafe {
        let npc_ptr = ctx.world.globals.NPC;
        let npc_info_ptr = ctx.world.globals.NPCInfo;

        if (*npc_ptr).enemy.is_some() {
            (*npc_info_ptr).goalEntity = (*npc_ptr).enemy;
            Mark2_AttackDecision(ctx);
        } else if ((*npc_info_ptr).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            Mark2_Patrol(ctx);
        } else {
            Mark2_Idle(ctx);
        }
    }
}
