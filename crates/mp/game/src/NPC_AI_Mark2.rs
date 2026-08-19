//! Port of `oracle/codemp/game/NPC_AI_Mark2.c`.
//!
//! Functions reach file-scope game state (`level`, `g_entities`, cvars) and engine traps through the threaded `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::entity::hit_location::HL_GENERIC1;
use crate::g_combat::G_Damage;
use crate::g_items::RegisterItem;
use crate::g_missile::CreateMissile;
use crate::g_utils::{G_EffectIndex, G_PlayEffectID, G_Sound, G_SoundIndex};
use crate::level::damage_flags::DAMAGE_NO_PROTECTION;
use crate::prelude::*;
use crate::q_shared::va;
use crate::trap;
use crate::NPC_reactions::NPC_Pain;
use crate::NPC_utils::NPC_SetSurfaceOnOff;
use mp_bg::bg_misc::{BG_FindItemForAmmo, BG_FindItemForWeapon};

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
    G_SoundIndex(ctx, "sound/chars/mark2/misc/mark2_explo");
    G_SoundIndex(ctx, "sound/chars/mark2/misc/mark2_pain");
    G_SoundIndex(ctx, "sound/chars/mark2/misc/mark2_fire");
    G_SoundIndex(ctx, "sound/chars/mark2/misc/mark2_move_lp");

    G_EffectIndex(ctx, "explosions/droidexplosion1");
    G_EffectIndex(ctx, "env/med_explode2");
    G_EffectIndex(ctx, "blaster/smoke_bolton");
    G_EffectIndex(ctx, "bryar/muzzle_flash");

    RegisterItem(ctx, BG_FindItemForWeapon(WP_BRYAR_PISTOL));
    RegisterItem(ctx, BG_FindItemForAmmo(AMMO_METAL_BOLTS));
    RegisterItem(ctx, BG_FindItemForAmmo(AMMO_POWERCELL));
    RegisterItem(ctx, BG_FindItemForAmmo(AMMO_BLASTER));
}

/// Raven `NPC_Mark2_Part_Explode`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:50-72`
pub fn NPC_Mark2_Part_Explode(ctx: &mut GameContext, self_: EntityId, bolt: c_int) {
    if bolt >= 0 {
        // mdxaBone_t POD zero-init (not part of the entity deref regime).
        let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
        let mut org: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];

        let ghoul2 = ctx.world.entity(self_).ghoul2;
        let angles = ctx.world.entity(self_).r.currentAngles;
        let origin = ctx.world.entity(self_).r.currentOrigin;
        let scale = ctx.world.entity(self_).modelScale;
        let time = ctx.world.level.time;

        trap::G2API_GetBoltMatrix(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                ghoul2,
                0,
                bolt,
                &mut boltMatrix as *mut mdxaBone_t,
                &angles as *const vec3_t,
                &origin as *const vec3_t,
                time,
                core::ptr::null_mut(),
                &scale as *const vec3_t,
            ),
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut org);
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::NEGATIVE_Y as c_int, &mut dir);

        G_PlayEffectID(
            G_EffectIndex(ctx, "env/med_explode2"),
            org,
            dir,
        );
        G_PlayEffectID(
            G_EffectIndex(ctx, "blaster/smoke_bolton"),
            org,
            dir,
        );
    }

    ctx.world.entity_mut(self_).count += 1;
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
    let hit_loc = ctx.world.globals.gPainHitLoc;

    NPC_Pain(ctx, self_, attacker, damage);

    for i in 0..3 {
        if hit_loc == HL_GENERIC1 + i
            && ctx.world.entity(self_).locationDamage[(HL_GENERIC1 + i) as usize] > AMMO_POD_HEALTH
        {
            if ctx.world.entity(self_).locationDamage[hit_loc as usize] >= AMMO_POD_HEALTH {
                let surface_name = cstr(&format!("torso_canister{}", (i + 1) as c_int));
                let ghoul2 = ctx.world.entity(self_).ghoul2;
                let new_bolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, surface_name.to_str().unwrap());
                if new_bolt != -1 {
                    NPC_Mark2_Part_Explode(ctx, self_, new_bolt);
                }
                NPC_SetSurfaceOnOff(ctx, self_, surface_name.as_ptr(), TURN_OFF);
                break;
            }
        }
    }

    let sound = G_SoundIndex(ctx, "sound/chars/mark2/misc/mark2_pain");
    G_Sound(ctx, Some(self_), CHAN_AUTO, sound);

    if ctx.world.entity(self_).count > 0 {
        let health = ctx.world.entity(self_).health;
        G_Damage(
            ctx,
            Some(self_),
            None,
            None,
            None,
            [0.0; 3],
            health,
            crate::level::damage_flags::DAMAGE_NO_PROTECTION,
            MOD_UNKNOWN as c_int,
        );
    }
}

/// Raven `Mark2_Hunt`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:118-130`
pub fn Mark2_Hunt(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    // FLAG: NPCInfo (gNPC_t) goalEntity, raw read.
    let goal = unsafe { (*npc_info).goalEntity };
    if goal.is_none() {
        let enemy = ctx.world.entity(npc_id).enemy;
        unsafe {
            (*npc_info).goalEntity = enemy;
        }
    }

    NPC_FaceEnemy(ctx, qtrue);

    // FLAG: NPCInfo (gNPC_t) combatMove, raw write.
    unsafe {
        (*npc_info).combatMove = qtrue;
    }
    NPC_MoveToGoal(ctx, qtrue);
}

/// Raven `Mark2_FireBlaster`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:137-179`
pub fn Mark2_FireBlaster(ctx: &mut GameContext, advance: qboolean) {
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut enemy_org1: vec3_t = [0.0; 3];
    let mut delta1: vec3_t = [0.0; 3];
    let mut angleToEnemy1: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];

    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // mdxaBone_t POD zero-init (not part of the entity deref regime).
    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };

    let ghoul2 = ctx.world.entity(npc_id).ghoul2;
    let bolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*flash");

    let angles = ctx.world.entity(npc_id).r.currentAngles;
    let origin = ctx.world.entity(npc_id).r.currentOrigin;
    let scale = ctx.world.entity(npc_id).modelScale;
    let time = ctx.world.level.time;
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
            ghoul2,
            0,
            bolt,
            &mut boltMatrix as *mut mdxaBone_t,
            &angles as *const vec3_t,
            &origin as *const vec3_t,
            time,
            core::ptr::null_mut(),
            &scale as *const vec3_t,
        ),
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut muzzle1);

    if ctx.world.entity(npc_id).health != 0 {
        let enemy = ctx.world.entity(npc_id).enemy;
        CalcEntitySpot(
            ctx,
            enemy,
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
        let cur_angles = ctx.world.entity(npc_id).r.currentAngles;
        AngleVectors(
            cur_angles,
            Some(&mut forward),
            Some(&mut vright),
            Some(&mut up),
        );
    }

    G_PlayEffectID(
        G_EffectIndex(ctx, "bryar/muzzle_flash"),
        muzzle1,
        forward,
    );

    let sound = G_SoundIndex(ctx, "sound/chars/mark2/misc/mark2_fire");
    G_Sound(ctx, Some(npc_id), CHAN_AUTO, sound);

    let missile_id = CreateMissile(ctx, muzzle1, forward, 1600.0, 10000, npc_id, false);
    ctx.ent_set(missile_id, PrefixSet::ClassnameStatic(c"bryar_proj"));
    let m = ctx.world.entity_mut(missile_id);
    m.s.weapon = WP_BRYAR_PISTOL as c_int;
    m.damage = 1;
    m.dflags = crate::level::damage_flags::DAMAGE_DEATH_KNOCKBACK;
    m.methodOfDeath = MOD_BRYAR_PISTOL as c_int;
    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/// Raven `Mark2_BlasterAttack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:186-205`
pub fn Mark2_BlasterAttack(ctx: &mut GameContext, advance: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if TIMER_Done(
        ctx,
        Some(npc_id),
        b"attackDelay\0".as_ptr() as *const c_char,
    ) == qtrue
    {
        // FLAG: NPCInfo (gNPC_t) localState, raw read.
        if unsafe { (*npc_info).localState } == LSTATE_NONE {
            let delay = ctx.world.bg_state.rng.Q_irand(500, 2000);
            TIMER_Set(
                ctx,
                Some(npc_id),
                b"attackDelay\0".as_ptr() as *const c_char,
                delay,
            );
        } else {
            let delay = ctx.world.bg_state.rng.Q_irand(100, 500);
            TIMER_Set(
                ctx,
                Some(npc_id),
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

/// Raven `Mark2_AttackDecision`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:212-295`
pub fn Mark2_AttackDecision(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    NPC_FaceEnemy(ctx, qtrue);

    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    let npc_enemy = ctx.world.entity(npc_id).enemy;
    let enemy_origin = npc_enemy
        .map(|eid| ctx.world.entity(eid).r.currentOrigin)
        .unwrap_or([0.0; 3]);
    let distance =
        crate::q_math::DistanceHorizontalSquared(npc_origin, enemy_origin) as c_int as i64;
    let visible = NPC_ClearLOS4(ctx, npc_enemy);
    let advance = (distance > MIN_DISTANCE_SQR as i64) as qboolean;

    // He's been ordered to get up
    // FLAG: NPCInfo (gNPC_t) localState, raw read.
    if unsafe { (*npc_info).localState } == LSTATE_RISINGUP {
        ctx.world.entity_mut(npc_id).flags &= !FL_SHIELDED;
        NPC_SetAnim(
            ctx,
            npc_id,
            SETANIM_BOTH,
            BOTH_RUN1START as c_int,
            SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        );
        // FLAG: NPC pool `gclient_t`, raw deref for ps.legsTimer / ps.torsoAnim.
        let client = ctx.world.entity(npc_id).client;
        let (legs_timer, torso_anim) = unsafe { ((*client).ps.legsTimer, (*client).ps.torsoAnim) };
        if legs_timer <= 0 && torso_anim == BOTH_RUN1START as c_int {
            unsafe {
                (*npc_info).localState = LSTATE_NONE;
            }
        }
        return;
    }

    // If we cannot see our target, move to see it
    if visible == qfalse || NPC_FaceEnemy(ctx, qtrue) == qfalse {
        // FLAG: NPCInfo (gNPC_t) localState, raw read.
        let local_state = unsafe { (*npc_info).localState };
        if local_state == LSTATE_DOWN || local_state == LSTATE_DROPPINGDOWN {
            if TIMER_Done(ctx, Some(npc_id), b"downTime\0".as_ptr() as *const c_char) == qtrue {
                unsafe {
                    (*npc_info).localState = LSTATE_RISINGUP;
                }
                NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    BOTH_RUN1STOP as c_int,
                    SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
                );
                let delay = ctx.world.bg_state.rng.Q_irand(3000, 8000);
                TIMER_Set(
                    ctx,
                    Some(npc_id),
                    b"runTime\0".as_ptr() as *const c_char,
                    delay,
                );
            }
        } else {
            Mark2_Hunt(ctx);
        }
        return;
    }

    // He's down but he could advance if he wants to.
    // FLAG: NPCInfo (gNPC_t) localState, raw read.
    if advance == qtrue
        && TIMER_Done(ctx, Some(npc_id), b"downTime\0".as_ptr() as *const c_char) == qtrue
        && unsafe { (*npc_info).localState } == LSTATE_DOWN
    {
        unsafe {
            (*npc_info).localState = LSTATE_RISINGUP;
        }
        NPC_SetAnim(
            ctx,
            npc_id,
            SETANIM_BOTH,
            BOTH_RUN1STOP as c_int,
            SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        );
        let delay = ctx.world.bg_state.rng.Q_irand(3000, 8000);
        TIMER_Set(
            ctx,
            Some(npc_id),
            b"runTime\0".as_ptr() as *const c_char,
            delay,
        );
    }

    NPC_FaceEnemy(ctx, qtrue);

    // FLAG: NPCInfo (gNPC_t) localState, raw read.
    let local_state = unsafe { (*npc_info).localState };
    if local_state == LSTATE_DROPPINGDOWN {
        NPC_SetAnim(
            ctx,
            npc_id,
            SETANIM_BOTH,
            BOTH_RUN1STOP as c_int,
            SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        );
        let delay = ctx.world.bg_state.rng.Q_irand(3000, 9000);
        TIMER_Set(
            ctx,
            Some(npc_id),
            b"downTime\0".as_ptr() as *const c_char,
            delay,
        );

        // FLAG: NPC pool `gclient_t`, raw deref for ps.legsTimer / ps.torsoAnim.
        let client = ctx.world.entity(npc_id).client;
        let (legs_timer, torso_anim) = unsafe { ((*client).ps.legsTimer, (*client).ps.torsoAnim) };
        if legs_timer <= 0 && torso_anim == BOTH_RUN1STOP as c_int {
            ctx.world.entity_mut(npc_id).flags |= FL_SHIELDED;
            unsafe {
                (*npc_info).localState = LSTATE_DOWN;
            }
        }
    } else if local_state == LSTATE_DOWN {
        ctx.world.entity_mut(npc_id).flags |= FL_SHIELDED;
        Mark2_BlasterAttack(ctx, qfalse);
    } else if TIMER_Done(ctx, Some(npc_id), b"runTime\0".as_ptr() as *const c_char) == qtrue {
        unsafe {
            (*npc_info).localState = LSTATE_DROPPINGDOWN;
        }
    } else if advance == qtrue {
        Mark2_BlasterAttack(ctx, advance);
    }
}

/// Raven `Mark2_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark2.c:303-330`
pub fn Mark2_Patrol(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if NPC_CheckPlayerTeamStealth(ctx) == qtrue {
        NPC_UpdateAngles(ctx, qtrue, qtrue);
        return;
    }

    if ctx.world.entity(npc_id).enemy.is_none() {
        if UpdateGoal(ctx) != core::ptr::null_mut() {
            ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(ctx, qtrue);
            NPC_UpdateAngles(ctx, qtrue, qtrue);
        }

        if TIMER_Done(
            ctx,
            Some(npc_id),
            b"patrolNoise\0".as_ptr() as *const c_char,
        ) == qtrue
        {
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
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if ctx.world.entity(npc_id).enemy.is_some() {
        let enemy = ctx.world.entity(npc_id).enemy;
        // FLAG: NPCInfo (gNPC_t) goalEntity, raw write.
        unsafe {
            (*npc_info).goalEntity = enemy;
        }
        Mark2_AttackDecision(ctx);
    // FLAG: NPCInfo (gNPC_t) scriptFlags, raw read.
    } else if (unsafe { (*npc_info).scriptFlags } & SCF_LOOK_FOR_ENEMIES) != 0 {
        Mark2_Patrol(ctx);
    } else {
        Mark2_Idle(ctx);
    }
}
