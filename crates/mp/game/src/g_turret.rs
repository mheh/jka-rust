// PORT-COMPLETE: g_turret.c 3/11
//! FAITHFUL port of `oracle/codemp/game/g_turret.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//!
//! Safe-state migration **Stage 2c**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5); bodies reach their entities through the
//! `ctx.world.entity()`/`entity_mut()` accessors instead of re-deriving raw
//! `gentity_t*` at the top. The only residual `unsafe` derefs are the sanctioned
//! seam ones: pool `gclient_t*` reads (an arbitrary target/attacker/enemy may be
//! an NPC carrying a `BG_Alloc`'d pool client, so its `client` value is read via
//! the safe borrow and dereferenced raw — recipe 2b), `m_pVehicle`/
//! `m_pVehicleInfo` (no accessor), and C-string byte checks.
#![allow(non_snake_case, unused, clippy::all)]

use crate::bg_misc::snap_vector;
use crate::bg_misc::{BG_EvaluateTrajectory, BG_FindItemForWeapon};
use crate::ent_fn_enums::{EntDie, EntPain, EntThink, EntUse};
use crate::entity::flags::FL_NOTARGET;
use crate::g_combat::{G_RadiusDamage, ObjectDie};
use crate::g_items::RegisterItem;
use crate::g_items::FRAMETIME;
use crate::g_spawn::{G_SpawnFloat, G_SpawnInt, G_SpawnString};
use crate::g_utils::{
    G_EffectIndex, G_FreeEntity, G_IconIndex, G_ModelIndex, G_PlayEffect, G_PlayEffectID,
    G_RadiusList, G_ScaleNetHealth, G_SetAngles, G_SetOrigin, G_SoundIndex, G_Spawn, G_UseTargets,
    G_UseTargets2,
};
use crate::prelude::*;
use crate::q_math::{
    vectoangles, AngleNormalize180, AngleSubtract, AngleVectors, VectorLengthSquared, PITCH, YAW,
};
use crate::q_shared::Q_stricmp;
use crate::trap;
use crate::NPC_combat::G_SetEnemy;
use mp_bg::public::effect_types::effectTypes_t::{EFFECT_EXPLOSION_TURRET, EFFECT_SPARKS};
use mp_bg::public::means_of_death::meansOfDeath_t::{MOD_TARGET_LASER, MOD_UNKNOWN};
use mp_qshared::common::mp::gentity::MAT_METAL;
use mp_qshared::shared::surface_flags::{CONTENTS_LIGHTSABER, MASK_SHOT};
use mp_qshared::shared::trajectory::trType_t::{TR_LINEAR, TR_LINEAR_STOP, TR_STATIONARY};
use std::ffi::c_char;

/// Raven `q_shared.h:30` `VALIDSTRING(a)` macro.
/// Source: `oracle/codemp/game/q_shared.h:30`
unsafe fn VALIDSTRING(a: *const c_char) -> bool {
    !a.is_null() && *a != 0
}

/// Raven `TurretPain`.
///
/// Source: `oracle/codemp/game/g_turret.c:11-32`
pub fn TurretPain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    let self_health = ctx.world.entity(self_).health;
    if let Some(target_id) = ctx.world.entity(self_).target_ent {
        let t = ctx.world.entity_mut(target_id);
        t.health = self_health;
        let scale = t.maxHealth != 0;
        if scale {
            G_ScaleNetHealth(ctx.world.entity_mut(target_id));
        }
    }

    if let Some(attacker_id) = attacker {
        // CLIENT-POINTER TRAP: attacker may be an NPC → pool client; read the
        // `client` value via the safe borrow and deref it raw (recipe 2b).
        let client = ctx.world.entity(attacker_id).client;
        if !client.is_null() {
            let weapon = unsafe { (*client).ps.weapon };
            if weapon == WP_DEMP2 {
                let time = ctx.world.level.time;
                let random_val = ctx.world.bg_state.rng.random();
                let e = ctx.world.entity_mut(self_);
                e.attackDebounceTime = time + 800 + (random_val * 500.0) as c_int;
                e.painDebounceTime = e.attackDebounceTime;
            }
        }
    }

    if ctx.world.entity(self_).enemy.is_none() {
        G_SetEnemy(ctx, self_, attacker);
    }
}

/// Raven `TurretBasePain`.
///
/// Source: `oracle/codemp/game/g_turret.c:35-48`
pub fn TurretBasePain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    let self_health = ctx.world.entity(self_).health;
    if let Some(target_id) = ctx.world.entity(self_).target_ent {
        let t = ctx.world.entity_mut(target_id);
        t.health = self_health;
        let scale = t.maxHealth != 0;
        if scale {
            G_ScaleNetHealth(ctx.world.entity_mut(target_id));
        }
        TurretPain(ctx, target_id, attacker, damage);
    }
}

/// Raven `auto_turret_die`.
///
/// Source: `oracle/codemp/game/g_turret.c:51-109`
pub fn auto_turret_die(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    meansOfDeath: c_int,
) {
    let owner_num = ctx.world.entity(self_).r.ownerNum as usize;
    if owner_num < ctx.world.g_entities.len() {
        let owner = ctx.world.entity_mut(EntityId(owner_num as u32));
        owner.think = FnId::NONE;
        owner.use_ = FnId::NONE;
    }

    let forward = [0.0, 0.0, 1.0];
    let mut pos = [0.0, 0.0, 0.0];

    {
        let e = ctx.world.entity_mut(self_);
        e.die = FnId::NONE;
        e.takedamage = qfalse;
        e.s.health = 0;
        e.health = 0;
        e.s.loopSound = 0;
        e.s.shouldtarget = qfalse;

        // VectorCopy(self->r.currentOrigin, pos)
        pos[0] = e.r.currentOrigin[0];
        pos[1] = e.r.currentOrigin[1];
        pos[2] = e.r.currentOrigin[2];

        pos[2] += e.r.maxs[2] * 0.5;
    }

    G_PlayEffect(EFFECT_EXPLOSION_TURRET as c_int, pos, forward);
    G_PlayEffectID(G_EffectIndex(c"turret/explode".as_ptr()), pos, forward);

    let splashDamage = ctx.world.entity(self_).splashDamage;
    let splashRadius = ctx.world.entity(self_).splashRadius;
    if splashDamage > 0 && splashRadius > 0 {
        let origin = ctx.world.entity(self_).r.currentOrigin;
        G_RadiusDamage(
            ctx,
            origin,
            attacker,
            splashDamage as f32,
            splashRadius as f32,
            attacker,
            None,
            MOD_UNKNOWN as c_int,
        );
    }

    ctx.world.entity_mut(self_).s.weapon = 0;

    let modelindex2 = ctx.world.entity(self_).s.modelindex2;
    if modelindex2 != 0 {
        ctx.world.entity_mut(self_).s.modelindex = modelindex2;

        if let Some(target_id) = ctx.world.entity(self_).target_ent {
            let target_mi2 = ctx.world.entity(target_id).s.modelindex2;
            if target_mi2 != 0 {
                ctx.world.entity_mut(target_id).s.modelindex = target_mi2;
            }
        }

        {
            let e = ctx.world.entity_mut(self_);
            // VectorCopy(self->r.currentAngles, self->s.apos.trBase)
            e.s.apos.trBase[0] = e.r.currentAngles[0];
            e.s.apos.trBase[1] = e.r.currentAngles[1];
            e.s.apos.trBase[2] = e.r.currentAngles[2];

            // VectorClear(self->s.apos.trDelta)
            e.s.apos.trDelta[0] = 0.0;
            e.s.apos.trDelta[1] = 0.0;
            e.s.apos.trDelta[2] = 0.0;
        }

        if !ctx.world.entity(self_).target.is_null() {
            G_UseTargets(ctx, Some(self_), attacker);
        }
    } else {
        ObjectDie(ctx, self_, inflictor, attacker, damage, meansOfDeath);
    }
}

/// Raven `bottom_die`.
///
/// Source: `oracle/codemp/game/g_turret.c:112-124`
pub fn bottom_die(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    meansOfDeath: c_int,
) {
    let self_health = ctx.world.entity(self_).health;
    if let Some(target_id) = ctx.world.entity(self_).target_ent {
        if ctx.world.entity(target_id).health > 0 {
            let t = ctx.world.entity_mut(target_id);
            t.health = self_health;
            let scale = t.maxHealth != 0;
            if scale {
                G_ScaleNetHealth(ctx.world.entity_mut(target_id));
            }
            auto_turret_die(ctx, target_id, inflictor, attacker, damage, meansOfDeath);
        }
    }
}

/// Raven `turret_fire`.
///
/// Source: `oracle/codemp/game/g_turret.c:129-176`
pub fn turret_fire(ctx: &mut GameContext, ent: EntityId, start: vec3_t, dir: vec3_t) {
    let ent_number = ctx.world.entity(ent).s.number;
    let contents = trap::PointContents(
        ctx.engine,
        mp_abi::game::syscalls::G_POINT_CONTENTS::GPointContentsArgs::new(
            &start as *const vec3_t,
            ent_number,
        ),
    );
    if (contents & MASK_SHOT) != 0 {
        return;
    }

    let mut org = [0.0; 3];
    // VectorMA(start, -START_DIS, dir, org) — org = start + (-15.0) * dir
    org[0] = start[0] - START_DIS * dir[0];
    org[1] = start[1] - START_DIS * dir[1];
    org[2] = start[2] - START_DIS * dir[2];

    let ent_gv13 = ctx.world.entity(ent).genericValue13;
    G_PlayEffectID(ent_gv13, org, dir);

    let bolt = G_Spawn(ctx);
    if bolt.is_null() {
        return;
    }
    let bolt_id = ctx.entity_id_of(bolt).unwrap();

    let level_time = ctx.world.level.time;
    let ent_gv14 = ctx.world.entity(ent).genericValue14;
    let ent_gv15 = ctx.world.entity(ent).genericValue15;
    let ent_damage = ctx.world.entity(ent).damage;
    let ent_alliedTeam = ctx.world.entity(ent).alliedTeam;
    let ent_teamnodmg = ctx.world.entity(ent).teamnodmg;
    let ent_mass = ctx.world.entity(ent).mass;

    let b = ctx.world.entity_mut(bolt_id);
    b.s.otherEntityNum2 = ent_gv14;
    b.s.emplacedOwner = ent_gv15;

    b.classname = c"turret_proj".as_ptr() as *mut c_char;
    b.nextthink = level_time + 10000;
    b.think = Some(EntThink::G_FreeEntity).into();
    b.s.eType = ET_MISSILE as c_int;
    b.s.weapon = WP_EMPLACED_GUN;
    b.r.ownerNum = ent_number;
    b.damage = ent_damage;
    b.alliedTeam = ent_alliedTeam;
    b.teamnodmg = ent_teamnodmg;
    b.splashDamage = ent_damage;
    b.splashRadius = 100;
    b.methodOfDeath = MOD_TARGET_LASER as c_int;
    b.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // VectorSet(maxs, 1.5, 1.5, 1.5)
    b.r.maxs[0] = 1.5;
    b.r.maxs[1] = 1.5;
    b.r.maxs[2] = 1.5;

    // VectorScale(maxs, -1.0, mins)
    b.r.mins[0] = -b.r.maxs[0];
    b.r.mins[1] = -b.r.maxs[1];
    b.r.mins[2] = -b.r.maxs[2];

    b.s.pos.trType = TR_LINEAR;
    b.s.pos.trTime = level_time;

    // VectorCopy(start, trBase)
    b.s.pos.trBase[0] = start[0];
    b.s.pos.trBase[1] = start[1];
    b.s.pos.trBase[2] = start[2];

    // VectorScale(dir, ent->mass, trDelta)
    b.s.pos.trDelta[0] = dir[0] * ent_mass as f32;
    b.s.pos.trDelta[1] = dir[1] * ent_mass as f32;
    b.s.pos.trDelta[2] = dir[2] * ent_mass as f32;

    snap_vector(&mut b.s.pos.trDelta);

    // VectorCopy(start, currentOrigin)
    b.r.currentOrigin[0] = start[0];
    b.r.currentOrigin[1] = start[1];
    b.r.currentOrigin[2] = start[2];

    b.parent = Some(ent);
}

pub const START_DIS: f32 = 15.0;

/// Raven `turret_head_think`.
///
/// Source: `oracle/codemp/game/g_turret.c:179-225`
pub fn turret_head_think(ctx: &mut GameContext, self_: EntityId) {
    let top_num = ctx.world.entity(self_).r.ownerNum as usize;
    if top_num >= ctx.world.g_entities.len() {
        return;
    }
    let top_id = EntityId(top_num as u32);

    let level_time = ctx.world.level.time;

    if ctx.world.entity(self_).painDebounceTime > level_time {
        let cur_org = ctx.world.entity(self_).r.currentOrigin;
        let mut v_up = [0.0, 0.0, 1.0];
        G_PlayEffect(EFFECT_SPARKS as c_int, cur_org, v_up);

        if ctx.world.bg_state.rng.Q_irand(0, 3) != 0 {
            // 25% chance of still firing
            return;
        }
    }

    if !ctx.world.entity(self_).enemy.is_none()
        && ctx.world.entity(self_).setTime < level_time
        && ctx.world.entity(self_).attackDebounceTime < level_time
    {
        let mut fwd = [0.0; 3];
        let mut org = [0.0; 3];

        let wait = ctx.world.entity(self_).wait as c_int;
        ctx.world.entity_mut(self_).setTime = level_time + wait;

        // Get top entity's position and angles
        let top_origin = ctx.world.entity(top_id).r.currentOrigin;
        let top_angles = ctx.world.entity(top_id).r.currentAngles;
        let top_maxs = ctx.world.entity(top_id).r.maxs[2];

        // VectorCopy(top->r.currentOrigin, org)
        org[0] = top_origin[0];
        org[1] = top_origin[1];
        org[2] = top_origin[2];

        // org[2] += top->r.maxs[2] - 8
        org[2] += top_maxs - 8.0;

        // AngleVectors(top->r.currentAngles, fwd, NULL, NULL)
        AngleVectors(top_angles, Some(&mut fwd), None, None);

        // VectorMA(org, START_DIS, fwd, org)
        org[0] = org[0] + START_DIS * fwd[0];
        org[1] = org[1] + START_DIS * fwd[1];
        org[2] = org[2] + START_DIS * fwd[2];

        turret_fire(ctx, top_id, org, fwd);

        ctx.world.entity_mut(self_).fly_sound_debounce_time = level_time;
    }
}

/// Raven `turret_aim`.
///
/// Source: `oracle/codemp/game/g_turret.c:228-352`
pub fn turret_aim(ctx: &mut GameContext, self_: EntityId) {
    let top_num = ctx.world.entity(self_).r.ownerNum as usize;
    if top_num >= ctx.world.g_entities.len() {
        return;
    }
    let top_id = EntityId(top_num as u32);

    let mut enemyDir = [0.0; 3];
    let mut org = [0.0; 3];
    let mut org2 = [0.0; 3];
    let mut desiredAngles = [0.0; 3];
    let mut setAngle = [0.0; 3];
    let mut diffYaw: f32 = 0.0;
    let mut diffPitch: f32 = 0.0;
    let mut turnSpeed: f32;

    const PITCH_CAP: f32 = 40.0;

    let level_time = ctx.world.level.time;

    // Evaluate trajectory for the gun base. `currentAngles` is the entity's
    // persistent field (evaluated + normalized here, written back at the end).
    let apos = ctx.world.entity(top_id).s.apos;
    let mut currentAngles: vec3_t = [0.0; 3];
    BG_EvaluateTrajectory(&apos as *const trajectory_t, level_time, &mut currentAngles);
    currentAngles[PITCH] = AngleNormalize180(currentAngles[PITCH]);
    currentAngles[YAW] = AngleNormalize180(currentAngles[YAW]);
    turnSpeed = ctx.world.entity(top_id).speed;

    if ctx.world.entity(self_).painDebounceTime > level_time {
        // In pain — aim randomly
        // Oracle uses `flrand` (holdrand stream), not the `random()` macro.
        // Source: `oracle/codemp/game/g_turret.c:249-250`
        desiredAngles[YAW] = currentAngles[YAW] + ctx.world.bg_state.rng.flrand(-45.0, 45.0);
        desiredAngles[PITCH] = currentAngles[PITCH] + ctx.world.bg_state.rng.flrand(-10.0, 10.0);

        if desiredAngles[PITCH] < -PITCH_CAP {
            desiredAngles[PITCH] = -PITCH_CAP;
        } else if desiredAngles[PITCH] > PITCH_CAP {
            desiredAngles[PITCH] = PITCH_CAP;
        }

        diffYaw = AngleSubtract(desiredAngles[YAW], currentAngles[YAW]);
        diffPitch = AngleSubtract(desiredAngles[PITCH], currentAngles[PITCH]);
        // Oracle uses `flrand` (holdrand stream), not the `random()` macro.
        // Source: `oracle/codemp/game/g_turret.c:263`
        turnSpeed = ctx.world.bg_state.rng.flrand(-5.0, 5.0);
    } else if let Some(enemy_id) = ctx.world.entity(self_).enemy {
        // Aim at enemy
        let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
        let enemy_maxs2 = ctx.world.entity(enemy_id).r.maxs[2];
        org[0] = enemy_origin[0];
        org[1] = enemy_origin[1];
        org[2] = enemy_origin[2] + enemy_maxs2 * 0.5;

        // Check for walker vehicle
        let enemy_etype = ctx.world.entity(enemy_id).s.eType;
        let enemy_npcclass = ctx.world.entity(enemy_id).s.NPC_class;
        let enemy_veh = ctx.world.entity(enemy_id).m_pVehicle;
        if enemy_etype == ET_NPC as c_int
            && enemy_npcclass == class_t::CLASS_VEHICLE as c_int
            && !enemy_veh.is_null()
        {
            // FLAG: m_pVehicle / m_pVehicleInfo have no accessor; the vehicle
            // derefs stay raw (recipe 2c). C dereferences `m_pVehicleInfo`
            // unconditionally here; the added null guard is a defined-behavior
            // choice for the (always-holding) `m_pVehicle` non-null =>
            // `m_pVehicleInfo` non-null invariant.
            unsafe {
                if (*enemy_veh).m_pVehicleInfo as *const vehicleInfo_t != std::ptr::null() {
                    if (*(*enemy_veh).m_pVehicleInfo).r#type == VH_WALKER {
                        org[2] += 32.0;
                    }
                }
            }
        }

        let top_origin = ctx.world.entity(top_id).r.currentOrigin;
        org2[0] = top_origin[0];
        org2[1] = top_origin[1];
        org2[2] = top_origin[2];

        // enemyDir = org - org2
        enemyDir[0] = org[0] - org2[0];
        enemyDir[1] = org[1] - org2[1];
        enemyDir[2] = org[2] - org2[2];

        vectoangles(enemyDir, &mut desiredAngles);
        desiredAngles[PITCH] = AngleNormalize180(desiredAngles[PITCH]);

        if desiredAngles[PITCH] < -PITCH_CAP {
            desiredAngles[PITCH] = -PITCH_CAP;
        } else if desiredAngles[PITCH] > PITCH_CAP {
            desiredAngles[PITCH] = PITCH_CAP;
        }

        diffYaw = AngleSubtract(desiredAngles[YAW], currentAngles[YAW]);
        diffPitch = AngleSubtract(desiredAngles[PITCH], currentAngles[PITCH]);
    } else {
        // No enemy — pan back and forth
        // C: `sin( level.time * 0.0001f + top->count )` — the sum is float
        // (the `0.0001f` literal and int operands stay float), then promotes
        // to double for the libm `sin`, and the result truncates to float.
        let top_count = ctx.world.entity(top_id).count;
        let self_angles_yaw = ctx.world.entity(self_).s.angles[YAW];
        desiredAngles[YAW] = ((level_time as f32 * 0.0001 + top_count as f32) as f64).sin() as f32;
        desiredAngles[YAW] *= 60.0;
        desiredAngles[YAW] += self_angles_yaw;
        desiredAngles[YAW] = AngleNormalize180(desiredAngles[YAW]);

        diffYaw = AngleSubtract(desiredAngles[YAW], currentAngles[YAW]);
        diffPitch = AngleSubtract(0.0, currentAngles[PITCH]);
        turnSpeed = 1.0;
    }

    // Cap turn speed
    if diffYaw != 0.0 {
        if diffYaw.abs() > turnSpeed {
            diffYaw = if diffYaw >= 0.0 {
                turnSpeed
            } else {
                -turnSpeed
            };
        }
    }
    if diffPitch != 0.0 {
        if diffPitch.abs() > turnSpeed {
            diffPitch = if diffPitch > 0.0 {
                turnSpeed
            } else {
                -turnSpeed
            };
        }
    }

    // Set up desired angles
    setAngle[0] = diffPitch;
    setAngle[1] = diffYaw;
    setAngle[2] = 0.0;

    // Update trajectory + persist the evaluated currentAngles.
    let top = ctx.world.entity_mut(top_id);
    top.r.currentAngles = currentAngles;
    top.s.apos.trBase[0] = currentAngles[0];
    top.s.apos.trBase[1] = currentAngles[1];
    top.s.apos.trBase[2] = currentAngles[2];

    // setAngle * (1000/FRAMETIME)
    top.s.apos.trDelta[0] = setAngle[0] * (1000.0 / FRAMETIME as f32);
    top.s.apos.trDelta[1] = setAngle[1] * (1000.0 / FRAMETIME as f32);
    top.s.apos.trDelta[2] = setAngle[2] * (1000.0 / FRAMETIME as f32);

    top.s.apos.trTime = level_time;
    top.s.apos.trType = TR_LINEAR_STOP;
    top.s.apos.trDuration = FRAMETIME;

    if diffYaw != 0.0 || diffPitch != 0.0 {
        top.s.loopSound = G_SoundIndex(c"sound/vehicles/weapons/hoth_turret/turn.wav".as_ptr());
    } else {
        top.s.loopSound = 0;
    }
}

/// Raven `turret_turnoff`.
///
/// Source: `oracle/codemp/game/g_turret.c:355-374`
pub fn turret_turnoff(ctx: &mut GameContext, self_: EntityId) {
    let top_num = ctx.world.entity(self_).r.ownerNum as usize;
    let level_time = ctx.world.level.time;
    if top_num < ctx.world.g_entities.len() {
        let top = ctx.world.entity_mut(EntityId(top_num as u32));

        // VectorCopy(top->r.currentAngles, top->s.apos.trBase)
        top.s.apos.trBase[0] = top.r.currentAngles[0];
        top.s.apos.trBase[1] = top.r.currentAngles[1];
        top.s.apos.trBase[2] = top.r.currentAngles[2];

        // VectorClear(top->s.apos.trDelta)
        top.s.apos.trDelta[0] = 0.0;
        top.s.apos.trDelta[1] = 0.0;
        top.s.apos.trDelta[2] = 0.0;

        top.s.apos.trTime = level_time;
        top.s.apos.trType = TR_STATIONARY;
    }

    ctx.world.entity_mut(self_).s.loopSound = 0;
    ctx.world.entity_mut(self_).enemy = None;
}

/// Raven `turret_sleep`.
///
/// Source: `oracle/codemp/game/g_turret.c:377-391`
pub fn turret_sleep(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).enemy.is_none() {
        return;
    }

    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(self_).aimDebounceTime = level_time + 5000;
    ctx.world.entity_mut(self_).enemy = None;
}

/// Raven `turret_find_enemies`.
///
/// Source: `oracle/codemp/game/g_turret.c:394-502`
pub fn turret_find_enemies(ctx: &mut GameContext, self_: EntityId) -> qboolean {
    let mut found = qfalse;
    let radius = ctx.world.entity(self_).radius;
    let mut bestDist = radius * radius;
    let mut bestTarget: Option<EntityId> = None;

    let top_num = ctx.world.entity(self_).r.ownerNum as usize;
    if top_num >= ctx.world.g_entities.len() {
        return qfalse;
    }
    let top_id = EntityId(top_num as u32);

    let level_time = ctx.world.level.time;

    if ctx.world.entity(self_).aimDebounceTime > level_time {
        if ctx.world.entity(self_).timestamp < level_time {
            ctx.world.entity_mut(self_).timestamp = level_time + 1000;
        }
    }

    let top_origin = ctx.world.entity(top_id).r.currentOrigin;
    let mut org2 = [0.0; 3];
    org2[0] = top_origin[0];
    org2[1] = top_origin[1];
    org2[2] = top_origin[2];

    let mut entity_list: [*mut gentity_t; MAX_GENTITIES] = [std::ptr::null_mut(); MAX_GENTITIES];
    let count = G_RadiusList(
        ctx,
        org2,
        radius,
        Some(self_),
        qtrue,
        entity_list.as_mut_ptr(),
    );

    let self_number = ctx.world.entity(self_).s.number;
    let self_alliedTeam = ctx.world.entity(self_).alliedTeam;

    for i in 0..count as usize {
        let target = entity_list[i];

        if target.is_null() {
            continue;
        }
        let target_id = match ctx.entity_id_of(target) {
            Some(t) => t,
            None => continue,
        };
        // CLIENT-POINTER TRAP: an arbitrary radius target may be an NPC → pool
        // client; read the `client` value via the safe borrow, deref raw (2b).
        let target_client = ctx.world.entity(target_id).client;
        if target_client.is_null() {
            continue;
        }
        if target_id == self_
            || ctx.world.entity(target_id).takedamage == qfalse
            || ctx.world.entity(target_id).health <= 0
            || (ctx.world.entity(target_id).flags & FL_NOTARGET) != 0
        {
            continue;
        }
        // FLAG: pool-client deref (target may be an NPC).
        if unsafe { (*target_client).sess.sessionTeam } == TEAM_SPECTATOR {
            continue;
        }
        if self_alliedTeam != 0 {
            // `target_client` is provably non-null here (null was filtered
            // above), so the oracle's `else`-teamnodmg arm is dead.
            // FLAG: pool-client deref (target may be an NPC).
            if unsafe { (*target_client).sess.sessionTeam } == self_alliedTeam {
                continue;
            }
        }
        let target_origin = ctx.world.entity(target_id).r.currentOrigin;
        if trap::InPVS(
            ctx.engine,
            mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                &org2 as *const vec3_t,
                &target_origin as *const vec3_t,
            ),
        ) == 0
        {
            continue;
        }

        let target_maxs2 = ctx.world.entity(target_id).r.maxs[2];
        let mut org = [0.0; 3];
        org[0] = target_origin[0];
        org[1] = target_origin[1];
        org[2] = target_origin[2] + target_maxs2 * 0.5;

        let mut tr: trace_t = unsafe { std::mem::zeroed() };
        trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut tr as *mut trace_t,
                &org2 as *const vec3_t,
                std::ptr::null(),
                std::ptr::null(),
                &org as *const vec3_t,
                self_number,
                MASK_SHOT,
            ),
        );

        let target_number = ctx.world.entity(target_id).s.number;
        if tr.allsolid == 0
            && tr.startsolid == 0
            && (tr.fraction == 1.0 || tr.entityNum as c_int == target_number)
        {
            let mut enemyDir = [0.0; 3];
            enemyDir[0] = target_origin[0] - top_origin[0];
            enemyDir[1] = target_origin[1] - top_origin[1];
            enemyDir[2] = target_origin[2] - top_origin[2];

            let enemyDist = VectorLengthSquared(enemyDir);

            let atst_name = c"atst_vehicle".as_ptr();
            let target_is_atst = Q_stricmp(ctx.world.entity(target_id).NPC_type, atst_name) == 0;
            let has_best = bestTarget.is_some();
            let best_is_atst = match bestTarget {
                Some(b) => Q_stricmp(ctx.world.entity(b).NPC_type, atst_name) == 0,
                None => false,
            };

            if enemyDist < bestDist || (target_is_atst && has_best && !best_is_atst) {
                if ctx.world.entity(self_).attackDebounceTime < level_time {
                    ctx.world.entity_mut(self_).attackDebounceTime = level_time + 1400;
                }

                bestTarget = Some(target_id);
                bestDist = enemyDist;
                found = qtrue;
            }
        }
    }

    if found != 0 {
        G_SetEnemy(ctx, self_, bestTarget);
        let target2 = ctx.world.entity(self_).target2;
        if unsafe { VALIDSTRING(target2) } {
            G_UseTargets2(ctx, Some(self_), Some(self_), target2);
        }
    }

    found
}

/// Raven `turret_base_think`.
///
/// Source: `oracle/codemp/game/g_turret.c:505-601`
pub fn turret_base_think(ctx: &mut GameContext, self_: EntityId) {
    let mut turnOff = qtrue;
    let level_time = ctx.world.level.time;

    if (ctx.world.entity(self_).spawnflags & 1) != 0 {
        // Not turned on
        turret_turnoff(ctx, self_);
        ctx.world.entity_mut(self_).flags |= FL_NOTARGET;
        ctx.world.entity_mut(self_).nextthink = -1;
        return;
    } else {
        // All hot and bothered
        ctx.world.entity_mut(self_).flags &= !FL_NOTARGET;
        ctx.world.entity_mut(self_).nextthink = level_time + FRAMETIME;
    }

    if ctx.world.entity(self_).enemy.is_none() {
        if turret_find_enemies(ctx, self_) != 0 {
            turnOff = qfalse;
        }
    } else {
        let enemy_id = ctx.world.entity(self_).enemy.unwrap();
        // CLIENT-POINTER TRAP: enemy may be an NPC → pool client; read the
        // `client` value via the safe borrow, deref raw (recipe 2b).
        let enemy_client = ctx.world.entity(enemy_id).client;
        // FLAG: pool-client deref (enemy may be an NPC).
        let enemy_is_spectator = !enemy_client.is_null()
            && unsafe { (*enemy_client).sess.sessionTeam } == TEAM_SPECTATOR;
        if enemy_is_spectator {
            // Don't keep going after spectators
            ctx.world.entity_mut(self_).enemy = None;
        } else {
            //FIXME: remain single-minded or look for a new enemy every now and then?
            if ctx.world.entity(enemy_id).health > 0 {
                // Enemy is alive
                let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
                let self_origin = ctx.world.entity(self_).r.currentOrigin;
                let mut enemyDir = [0.0; 3];
                enemyDir[0] = enemy_origin[0] - self_origin[0];
                enemyDir[1] = enemy_origin[1] - self_origin[1];
                enemyDir[2] = enemy_origin[2] - self_origin[2];

                let enemyDist = VectorLengthSquared(enemyDir);

                let radius = ctx.world.entity(self_).radius;
                if enemyDist < (radius * radius) {
                    // Was in valid radius
                    if trap::InPVS(
                        ctx.engine,
                        mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                            &self_origin as *const vec3_t,
                            &enemy_origin as *const vec3_t,
                        ),
                    ) != 0
                    {
                        // Every now and then, check if we can trace to enemy
                        let mut tr: trace_t = unsafe { std::mem::zeroed() };
                        let mut org = [0.0; 3];
                        let mut org2 = [0.0; 3];

                        if !enemy_client.is_null() {
                            // FLAG: pool-client deref (enemy may be an NPC).
                            let eye = unsafe { (*enemy_client).renderInfo.eyePoint };
                            org[0] = eye[0];
                            org[1] = eye[1];
                            org[2] = eye[2];
                        } else {
                            org[0] = enemy_origin[0];
                            org[1] = enemy_origin[1];
                            org[2] = enemy_origin[2];
                        }

                        org2[0] = self_origin[0];
                        org2[1] = self_origin[1];
                        org2[2] = self_origin[2];

                        if (ctx.world.entity(self_).spawnflags & 2) != 0 {
                            org2[2] += 10.0;
                        } else {
                            org2[2] -= 10.0;
                        }

                        let self_number = ctx.world.entity(self_).s.number;
                        trap::Trace(
                            ctx.engine,
                            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                &mut tr as *mut trace_t,
                                &org2 as *const vec3_t,
                                std::ptr::null(),
                                std::ptr::null(),
                                &org as *const vec3_t,
                                self_number,
                                MASK_SHOT,
                            ),
                        );

                        let enemy_number = ctx.world.entity(enemy_id).s.number;
                        if tr.allsolid == 0
                            && tr.startsolid == 0
                            && tr.entityNum as c_int == enemy_number
                        {
                            turnOff = qfalse;
                        }
                    }
                }
            }
            // Oracle calls turret_head_think only in this branch (enemy
            // existed and is not a spectator), NOT on the frame a fresh
            // enemy is acquired via turret_find_enemies.
            turret_head_think(ctx, self_);
        }
    }

    if turnOff != 0 {
        if ctx.world.entity(self_).bounceCount < level_time {
            turret_sleep(ctx, self_);
        }
    } else {
        let rnd = ctx.world.bg_state.rng.random();
        ctx.world.entity_mut(self_).bounceCount = level_time + 2000 + (rnd * 150.0) as c_int;
    }

    turret_aim(ctx, self_);
}

/// Raven `turret_base_use`.
///
/// Source: `oracle/codemp/game/g_turret.c:604-620`
pub fn turret_base_use(
    self_: &mut gentity_t,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // Toggle on and off
    self_.spawnflags ^= 1;
    // Raven's commented-out EF_SHADER_ANIM frame toggle (g_turret.c:610-619)
    // is dead code in the oracle source; not ported.
}

/// Raven `SP_misc_turret`.
///
/// Source: `oracle/codemp/game/g_turret.c:663-703`
pub fn SP_misc_turret(ctx: &mut GameContext, base: EntityId) {
    let mi2 = G_ModelIndex(c"models/map_objects/hoth/turret_bottom.md3".as_ptr());
    ctx.world.entity_mut(base).s.modelindex2 = mi2;
    let mi = G_ModelIndex(c"models/map_objects/hoth/turret_base.md3".as_ptr());
    ctx.world.entity_mut(base).s.modelindex = mi;

    let mut s: *mut c_char = std::ptr::null_mut();
    G_SpawnString(ctx, c"icon".as_ptr(), c"".as_ptr(), &mut s);
    if !s.is_null() && unsafe { *s != 0 } {
        let icon = G_IconIndex(ctx, s);
        ctx.world.entity_mut(base).s.genericenemyindex = icon;
    }

    let base_angles = ctx.world.entity(base).s.angles;
    G_SetAngles(ctx.world.entity_mut(base), base_angles);
    let base_origin = ctx.world.entity(base).s.origin;
    G_SetOrigin(ctx.world.entity_mut(base), base_origin);

    {
        let b = ctx.world.entity_mut(base);
        b.r.contents = CONTENTS_BODY;

        // VectorSet(maxs, 32, 32, 128)
        b.r.maxs[0] = 32.0;
        b.r.maxs[1] = 32.0;
        b.r.maxs[2] = 128.0;

        // VectorSet(mins, -32, -32, 0)
        b.r.mins[0] = -32.0;
        b.r.mins[1] = -32.0;
        b.r.mins[2] = 0.0;

        b.use_ = Some(EntUse::turret_base_use).into();
        b.think = Some(EntThink::turret_base_think).into();
    }

    let nextthink = ctx.world.level.time + FRAMETIME * 5;
    ctx.world.entity_mut(base).nextthink = nextthink;

    let base_ptr: *mut gentity_t = ctx.world.entity_mut(base);
    trap::LinkEntity(
        ctx.engine,
        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(base_ptr.cast()),
    );

    if turret_base_spawn_top(ctx, base) == 0 {
        G_FreeEntity(ctx, Some(base));
    }
}

/// Raven `turret_base_spawn_top`.
///
/// Source: `oracle/codemp/game/g_turret.c:706-861`
pub fn turret_base_spawn_top(ctx: &mut GameContext, base: EntityId) -> qboolean {
    let mut org = [0.0; 3];
    let mut t: c_int = 0;

    let top = G_Spawn(ctx);
    if top.is_null() {
        return qfalse;
    }
    let top_id = ctx.entity_id_of(top).unwrap();

    let mi = G_ModelIndex(c"models/map_objects/hoth/turret_top_new.md3".as_ptr());
    ctx.world.entity_mut(top_id).s.modelindex = mi;
    let mi2 = G_ModelIndex(c"models/map_objects/hoth/turret_top.md3".as_ptr());
    ctx.world.entity_mut(top_id).s.modelindex2 = mi2;

    let base_angles = ctx.world.entity(base).s.angles;
    G_SetAngles(ctx.world.entity_mut(top_id), base_angles);

    let base_origin = ctx.world.entity(base).s.origin;
    org[0] = base_origin[0];
    org[1] = base_origin[1];
    org[2] = base_origin[2] + 128.0;
    G_SetOrigin(ctx.world.entity_mut(top_id), org);

    let top_number = ctx.world.entity(top_id).s.number;
    ctx.world.entity_mut(base).r.ownerNum = top_number;
    let base_number = ctx.world.entity(base).s.number;
    ctx.world.entity_mut(top_id).r.ownerNum = base_number;

    let team = ctx.world.entity(base).team;
    let base_teamnodmg0 = ctx.world.entity(base).teamnodmg;
    // FLAG: `team` is a C string; the byte check derefs it raw.
    if !team.is_null() && unsafe { *team != 0 } && base_teamnodmg0 == 0 {
        let v = atoi(team);
        ctx.world.entity_mut(base).teamnodmg = v;
    }
    ctx.world.entity_mut(base).team = std::ptr::null_mut();
    let base_teamnodmg = ctx.world.entity(base).teamnodmg;
    ctx.world.entity_mut(top_id).teamnodmg = base_teamnodmg;
    let base_alliedTeam = ctx.world.entity(base).alliedTeam;
    ctx.world.entity_mut(top_id).alliedTeam = base_alliedTeam;

    ctx.world.entity_mut(base).s.eType = ET_GENERAL as c_int;

    // Set up explosion effects
    G_EffectIndex(c"turret/explode".as_ptr());
    G_EffectIndex(c"sparks/spark_exp_nosnd".as_ptr());
    G_EffectIndex(c"turret/hoth_muzzle_flash".as_ptr());

    // Pitch angle (actually yaw, stored in speed field)
    ctx.world.entity_mut(top_id).speed = 0.0;

    // Random time offset for no-enemy-search-around mode
    let rnd = ctx.world.bg_state.rng.random();
    ctx.world.entity_mut(top_id).count = (rnd * 9000.0) as c_int;

    if ctx.world.entity(base).health == 0 {
        ctx.world.entity_mut(base).health = 3000;
    }
    let base_health = ctx.world.entity(base).health;
    ctx.world.entity_mut(top_id).health = base_health;

    G_SpawnInt(ctx, c"showhealth".as_ptr(), c"0".as_ptr(), &mut t);

    if t != 0 {
        // Show health on HUD
        let base_health2 = ctx.world.entity(base).health;
        ctx.world.entity_mut(top_id).maxHealth = base_health2;
        G_ScaleNetHealth(ctx.world.entity_mut(top_id));

        ctx.world.entity_mut(base).maxHealth = base_health2;
        G_ScaleNetHealth(ctx.world.entity_mut(base));
    }

    {
        let b = ctx.world.entity_mut(base);
        b.takedamage = qtrue;
        b.pain = Some(EntPain::TurretBasePain).into();
        b.die = Some(EntDie::bottom_die).into();
    }

    // Shot speed
    let mut mass = 0.0f32;
    G_SpawnFloat(ctx, c"shotspeed".as_ptr(), c"1100".as_ptr(), &mut mass);
    ctx.world.entity_mut(base).mass = mass;
    ctx.world.entity_mut(top_id).mass = mass;

    // Light crosshair
    if ctx.world.entity(top_id).s.teamowner == 0 {
        let top_alliedTeam = ctx.world.entity(top_id).alliedTeam;
        ctx.world.entity_mut(top_id).s.teamowner = top_alliedTeam;
    }

    let top_alliedTeam = ctx.world.entity(top_id).alliedTeam;
    ctx.world.entity_mut(base).alliedTeam = top_alliedTeam;
    let top_teamowner = ctx.world.entity(top_id).s.teamowner;
    ctx.world.entity_mut(base).s.teamowner = top_teamowner;

    ctx.world.entity_mut(base).s.shouldtarget = qtrue;
    ctx.world.entity_mut(top_id).s.shouldtarget = qtrue;

    // Link them to each other
    ctx.world.entity_mut(base).target_ent = Some(top_id);
    ctx.world.entity_mut(top_id).target_ent = Some(base);

    // Search radius
    if ctx.world.entity(base).radius == 0.0 {
        ctx.world.entity_mut(base).radius = 1024.0;
    }
    let base_radius = ctx.world.entity(base).radius;
    ctx.world.entity_mut(top_id).radius = base_radius;

    // How quickly to fire
    if ctx.world.entity(base).wait == 0.0 {
        let rnd = ctx.world.bg_state.rng.random();
        ctx.world.entity_mut(base).wait = 300.0 + rnd * 55.0;
    }
    let base_wait = ctx.world.entity(base).wait;
    ctx.world.entity_mut(top_id).wait = base_wait;

    if ctx.world.entity(base).splashDamage == 0 {
        ctx.world.entity_mut(base).splashDamage = 300;
    }
    let base_splashDamage = ctx.world.entity(base).splashDamage;
    ctx.world.entity_mut(top_id).splashDamage = base_splashDamage;

    if ctx.world.entity(base).splashRadius == 0 {
        ctx.world.entity_mut(base).splashRadius = 128;
    }
    let base_splashRadius = ctx.world.entity(base).splashRadius;
    ctx.world.entity_mut(top_id).splashRadius = base_splashRadius;

    // Damage per shot
    if ctx.world.entity(base).damage == 0 {
        ctx.world.entity_mut(base).damage = 100;
    }
    let base_damage = ctx.world.entity(base).damage;
    ctx.world.entity_mut(top_id).damage = base_damage;

    // How fast it turns
    if ctx.world.entity(base).speed == 0.0 {
        ctx.world.entity_mut(base).speed = 20.0;
    }
    let base_speed = ctx.world.entity(base).speed;
    ctx.world.entity_mut(top_id).speed = base_speed;

    {
        let tp = ctx.world.entity_mut(top_id);
        // VectorSet(maxs, 48, 48, 16)
        tp.r.maxs[0] = 48.0;
        tp.r.maxs[1] = 48.0;
        tp.r.maxs[2] = 16.0;

        // VectorSet(mins, -48, -48, 0)
        tp.r.mins[0] = -48.0;
        tp.r.mins[1] = -48.0;
        tp.r.mins[2] = 0.0;
    }

    G_SoundIndex(c"sound/vehicles/weapons/hoth_turret/turn.wav".as_ptr());
    let gv13 = G_EffectIndex(c"turret/hoth_muzzle_flash".as_ptr());
    ctx.world.entity_mut(top_id).genericValue13 = gv13;
    let gv14 = G_EffectIndex(c"turret/hoth_shot".as_ptr());
    ctx.world.entity_mut(top_id).genericValue14 = gv14;
    let gv15 = G_EffectIndex(c"turret/hoth_impact".as_ptr());
    ctx.world.entity_mut(top_id).genericValue15 = gv15;

    {
        let tp = ctx.world.entity_mut(top_id);
        tp.r.contents = CONTENTS_BODY;

        tp.takedamage = qtrue;
        tp.pain = Some(EntPain::TurretPain).into();
        tp.die = Some(EntDie::auto_turret_die).into();

        tp.material = MAT_METAL;
    }

    // Register item for missile effect
    RegisterItem(ctx, BG_FindItemForWeapon(WP_EMPLACED_GUN));

    // Set as turret
    ctx.world.entity_mut(top_id).s.weapon = WP_EMPLACED_GUN;

    let top_ptr: *mut gentity_t = ctx.world.entity_mut(top_id);
    trap::LinkEntity(
        ctx.engine,
        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(top_ptr.cast()),
    );
    qtrue
}

// `atoi` is the libc-parity helper reached via the prelude
// (`crate::cstr_util::atoi`); no local extern shim.
