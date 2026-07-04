// PORT-COMPLETE: NPC_AI_Seeker.c 10/10
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Seeker.c`.
//!
//! All 10 functions ported. Nearly every function in this file relies on
//! file-scope globals set up by `SetNPCGlobals()` (NPC, NPCInfo, ucmd, etc.)
//! or reads other ambient state (level, g_entities, g_spskill cvars). These
//! globals are now threaded through GameContext and accessed via ctx.world.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// Raven `#define VELOCITY_DECAY 0.7f` (oracle/oracle/codemp/game/NPC_AI_Seeker.c:8).
const VELOCITY_DECAY: f32 = 0.7f32;

// Raven `#define MIN_MELEE_RANGE 320` / `MIN_MELEE_RANGE_SQR`.
const MIN_MELEE_RANGE: c_int = 320;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;

// Raven `#define MIN_DISTANCE 80` / `MIN_DISTANCE_SQR`.
const MIN_DISTANCE: c_int = 80;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

// Raven `#define SEEKER_STRAFE_VEL 100` / `SEEKER_STRAFE_DIS 200` / `SEEKER_UPWARD_PUSH 32`.
const SEEKER_STRAFE_VEL: f32 = 100.0f32;
const SEEKER_STRAFE_DIS: f32 = 200.0f32;
const SEEKER_UPWARD_PUSH: f32 = 32.0f32;

// Raven `#define SEEKER_FORWARD_BASE_SPEED 10` / `SEEKER_FORWARD_MULTIPLIER 2`.
const SEEKER_FORWARD_BASE_SPEED: f32 = 10.0f32;
const SEEKER_FORWARD_MULTIPLIER: f32 = 2.0f32;

// Raven `#define SEEKER_SEEK_RADIUS 1024`.
const SEEKER_SEEK_RADIUS: f32 = 1024.0f32;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// Local constants for Seeker AI.
// Source: oracle/oracle/codemp/game/NPC_AI_Seeker.c / q_shared.h / g_local.h
const CONTENTS_LIGHTSABER: c_int = 0x00040000;
const MASK_SHOT: c_int = CONTENTS_LIGHTSABER | 0x00000001 | 0x00000100 | 0x00000200; // CONTENTS_LIGHTSABER | CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_CORPSE
const MOD_FALLING: c_int = 9;
const MOD_BLASTER: c_int = 5;
const MOD_UNKNOWN: c_int = 46;
const MOD_TELEFRAG: c_int = 13;
const SCF_CHASE_ENEMIES: i32 = 0x00000400;


/// Raven `NPC_Seeker_Precache`.
///
/// Caches sound and effect resources for Seeker NPCs at map load time.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:26-31`
pub fn NPC_Seeker_Precache(ctx: GameContext<'_>) {
    crate::g_utils::G_SoundIndex(c"sound/chars/seeker/misc/fire.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/seeker/misc/hiss.wav".as_ptr());
    crate::g_utils::G_EffectIndex(c"env/small_explode".as_ptr());
}

/// Raven `NPC_Seeker_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:34-46`
pub fn NPC_Seeker_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    unsafe {
        if !((*(*self_).NPC).aiFlags & crate::npc::ai_flags::NPCAI_CUSTOM_GRAVITY) {
            crate::g_combat::G_Damage(
                self_,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                crate::q_math::vec3_origin,
                crate::q_math::vec3_origin,
                999,
                0,
                MOD_FALLING,
            );
        }

        crate::npc_c::SaveNPCGlobals(ctx);
        crate::npc_c::SetNPCGlobals(ctx, self_);
        Seeker_Strafe(ctx);
        crate::npc_c::RestoreNPCGlobals(ctx);
        crate::NPC_reactions::NPC_Pain(ctx, self_, attacker, damage);
    }
}

/// Raven `Seeker_MaintainHeight`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:49-148`
pub fn Seeker_MaintainHeight(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC;
        let NPCInfo = world.globals.NPCInfo;

        // Update our angles regardless
        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);

        // If we have an enemy, we should try to hover at or a little below enemy eye level
        if !(*NPC).enemy.is_null() {
            if crate::g_timer::TIMER_Done(ctx, NPC, c"heightChange".as_ptr()) != 0 {
                let mut difFactor: f32 = 1.0f32;

                crate::g_timer::TIMER_Set(ctx, NPC, c"heightChange".as_ptr(), crate::q_math::Q_irand(1000, 3000));

                // Find the height difference
                let dif = ((*(*NPC).enemy).r.currentOrigin[2]
                    + crate::q_math::flrand(
                        (*(*NPC).enemy).r.maxs[2] / 2.0f32,
                        (*(*NPC).enemy).r.maxs[2] + 8.0f32,
                    ))
                    - (*NPC).r.currentOrigin[2];

                if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                    if crate::g_timer::TIMER_Done(ctx, NPC, c"flameTime".as_ptr()) != 0 {
                        difFactor = 10.0f32;
                    }
                }

                // cap to prevent dramatic height shifts
                if dif.abs() > 2.0f32 * difFactor {
                    let mut dif_capped = dif;
                    if dif_capped.abs() > 24.0f32 * difFactor {
                        dif_capped = if dif < 0.0f32 { -24.0f32 * difFactor } else { 24.0f32 * difFactor };
                    }

                    (*(*NPC).client).ps.velocity[2] = ((*(*NPC).client).ps.velocity[2] + dif_capped) / 2.0f32;
                }
                if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                    (*(*NPC).client).ps.velocity[2] *= crate::q_math::flrand(0.85f32, 3.0f32);
                }
            }
        } else {
            let mut goal: *mut gentity_t = core::ptr::null_mut();

            if !(*NPCInfo).goalEntity.is_null() {
                // Is there a goal?
                goal = (*NPCInfo).goalEntity;
            } else {
                goal = (*NPCInfo).lastGoalEntity;
            }
            if !goal.is_null() {
                let dif = (*goal).r.currentOrigin[2] - (*NPC).r.currentOrigin[2];

                if dif.abs() > 24.0f32 {
                    world.globals.ucmd.upmove = if world.globals.ucmd.upmove < 0 { -4 } else { 4 };
                } else {
                    if (*(*NPC).client).ps.velocity[2] != 0.0f32 {
                        (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

                        if (*(*NPC).client).ps.velocity[2].abs() < 2.0f32 {
                            (*(*NPC).client).ps.velocity[2] = 0.0f32;
                        }
                    }
                }
            }
        }

        // Apply friction
        if (*(*NPC).client).ps.velocity[0] != 0.0f32 {
            (*(*NPC).client).ps.velocity[0] *= VELOCITY_DECAY;

            if (*(*NPC).client).ps.velocity[0].abs() < 1.0f32 {
                (*(*NPC).client).ps.velocity[0] = 0.0f32;
            }
        }

        if (*(*NPC).client).ps.velocity[1] != 0.0f32 {
            (*(*NPC).client).ps.velocity[1] *= VELOCITY_DECAY;

            if (*(*NPC).client).ps.velocity[1].abs() < 1.0f32 {
                (*(*NPC).client).ps.velocity[1] = 0.0f32;
            }
        }
    }
}

/// Raven `Seeker_Strafe`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:151-239`
pub fn Seeker_Strafe(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC;
        let NPCInfo = world.globals.NPCInfo;

        let mut side: c_int;
        let mut end: vec3_t = [0.0f32; 3];
        let mut right: vec3_t = [0.0f32; 3];
        let mut dir: vec3_t = [0.0f32; 3];
        let mut tr: trace_t = core::mem::zeroed();

        if crate::bg_lib::random() > 0.7f32 || (*NPC).enemy.is_null() || (*(*NPC).enemy).client.is_null() {
            // Do a regular style strafe
            crate::q_math::AngleVectors(
                (*(*NPC).client).renderInfo.eyeAngles,
                None,
                Some(&mut right),
                None,
            );

            // Pick a random strafe direction, then check to see if doing a strafe would be
            // reasonably valid
            side = if (crate::bg_lib::rand() & 1) != 0 { -1 } else { 1 };
            // Inline VectorMA: end = origin + scalar * right
            for i in 0..3 {
                end[i] = (*NPC).r.currentOrigin[i] + SEEKER_STRAFE_DIS * side as f32 * right[i];
            }

            crate::trap::Trace(ctx.engine, &mut tr, (*NPC).r.currentOrigin, core::ptr::null(), core::ptr::null(), end, (*NPC).s.number, MASK_SOLID);

            // Close enough
            if tr.fraction > 0.9f32 {
                let mut vel = SEEKER_STRAFE_VEL;
                let mut upPush = SEEKER_UPWARD_PUSH;
                if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
                    crate::g_utils::G_Sound(
                        ctx,
                        NPC,
                        CHAN_AUTO,
                        crate::g_utils::G_SoundIndex(c"sound/chars/seeker/misc/hiss".as_ptr()),
                    );
                } else {
                    vel *= 3.0f32;
                    upPush *= 4.0f32;
                }
                // Inline VectorMA: velocity += vel * side * right
                for i in 0..3 {
                    (*(*NPC).client).ps.velocity[i] += vel * side as f32 * right[i];
                }
                // Add a slight upward push
                (*(*NPC).client).ps.velocity[2] += upPush;

                (*NPCInfo).standTime = world.level.time + 1000 + (crate::bg_lib::random() * 500.0f32) as c_int;
            }
        } else {
            let mut stDis: f32;

            // Do a strafe to try and keep on the side of their enemy
            crate::q_math::AngleVectors(
                (*(*(*NPC).enemy).client).renderInfo.eyeAngles,
                Some(&mut dir),
                Some(&mut right),
                None,
            );

            // Pick a random side
            side = if (crate::bg_lib::rand() & 1) != 0 { -1 } else { 1 };
            stDis = SEEKER_STRAFE_DIS;
            if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                stDis *= 2.0f32;
            }
            // Inline VectorMA: end = enemy_origin + stDis * side * right
            for i in 0..3 {
                end[i] = (*(*NPC).enemy).r.currentOrigin[i] + stDis * side as f32 * right[i];
            }

            // then add a very small bit of random in front of/behind the player action
            // Inline VectorMA: end += crandom * 25 * dir
            for i in 0..3 {
                end[i] += crate::q_math::crandom() * 25.0f32 * dir[i];
            }

            crate::trap::Trace(ctx.engine, &mut tr, (*NPC).r.currentOrigin, core::ptr::null(), core::ptr::null(), end, (*NPC).s.number, MASK_SOLID);

            // Close enough
            if tr.fraction > 0.9f32 {
                let mut upPush: f32;

                // Inline VectorSubtract: dir = endpos - origin
                for i in 0..3 {
                    dir[i] = tr.endpos[i] - (*NPC).r.currentOrigin[i];
                }
                dir[2] *= 0.25f32; // do less upward change
                let dis = crate::q_math::VectorNormalize(&mut dir);

                // Inline VectorMA: velocity += dis * dir
                for i in 0..3 {
                    (*(*NPC).client).ps.velocity[i] += dis * dir[i];
                }

                upPush = SEEKER_UPWARD_PUSH;
                if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
                    crate::g_utils::G_Sound(
                        ctx,
                        NPC,
                        CHAN_AUTO,
                        crate::g_utils::G_SoundIndex(c"sound/chars/seeker/misc/hiss".as_ptr()),
                    );
                } else {
                    upPush *= 4.0f32;
                }

                // Add a slight upward push
                (*(*NPC).client).ps.velocity[2] += upPush;

                (*NPCInfo).standTime = world.level.time + 2500 + (crate::bg_lib::random() * 500.0f32) as c_int;
            }
        }
    }
}

/// Raven `Seeker_Hunt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:242-287`
pub fn Seeker_Hunt(
    ctx: GameContext<'_>,
    visible: qboolean,
    advance: qboolean,
) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC;
        let NPCInfo = world.globals.NPCInfo;

        crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);

        // If we're not supposed to stand still, pursue the player
        if (*NPCInfo).standTime < world.level.time {
            // Only strafe when we can see the player
            if visible != 0 {
                Seeker_Strafe(ctx);
                return;
            }
        }

        // If we don't want to advance, stop here
        if advance == qfalse {
            return;
        }

        // Only try and navigate if the player is visible
        if visible == qfalse {
            // Move towards our goal
            (*NPCInfo).goalEntity = (*NPC).enemy;
            (*NPCInfo).goalRadius = 24;

            // Get our direction from the navigator if we can't see our target
            let mut forward: vec3_t = [0.0f32; 3];
            let mut distance: f32 = 0.0f32;
            if crate::NPC_move::NPC_GetMoveDirection(ctx, &mut forward, &mut distance) == qfalse {
                return;
            }

            let speed = SEEKER_FORWARD_BASE_SPEED + SEEKER_FORWARD_MULTIPLIER * world.cvars.g_spskill.integer as f32;
            for i in 0..3 {
                (*(*NPC).client).ps.velocity[i] += speed * forward[i];
            }
        } else {
            let mut forward: vec3_t = [0.0f32; 3];
            forward[0] = (*(*NPC).enemy).r.currentOrigin[0] - (*NPC).r.currentOrigin[0];
            forward[1] = (*(*NPC).enemy).r.currentOrigin[1] - (*NPC).r.currentOrigin[1];
            forward[2] = (*(*NPC).enemy).r.currentOrigin[2] - (*NPC).r.currentOrigin[2];
            let _distance = crate::q_math::VectorNormalize(&mut forward);

            let speed = SEEKER_FORWARD_BASE_SPEED + SEEKER_FORWARD_MULTIPLIER * world.cvars.g_spskill.integer as f32;
            for i in 0..3 {
                (*(*NPC).client).ps.velocity[i] += speed * forward[i];
            }
        }
    }
}

/// Raven `Seeker_Fire`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:290-317`
pub fn Seeker_Fire(ctx: GameContext<'_>) {
    unsafe {
        let NPC = (*ctx.world).globals.NPC;

        let mut dir: vec3_t = [0.0f32; 3];
        let mut enemy_org: vec3_t = [0.0f32; 3];
        let mut muzzle: vec3_t = [0.0f32; 3];

        crate::NPC_utils::CalcEntitySpot(ctx, (*NPC).enemy, spot_t::SPOT_HEAD, &mut enemy_org);
        // Inline VectorSubtract: dir = enemy_org - origin
        for i in 0..3 {
            dir[i] = enemy_org[i] - (*NPC).r.currentOrigin[i];
        }
        crate::q_math::VectorNormalize(&mut dir);

        // move a bit forward in the direction we shall shoot in so that the bolt doesn't poke out the other side of the seeker
        // Inline VectorMA: muzzle = origin + 15 * dir
        for i in 0..3 {
            muzzle[i] = (*NPC).r.currentOrigin[i] + 15.0f32 * dir[i];
        }

        let missile = crate::g_missile::CreateMissile(ctx, muzzle, dir, 1000.0f32, 10000, NPC, qfalse);

        crate::g_utils::G_PlayEffectID(crate::g_utils::G_EffectIndex(c"blaster/muzzle_flash".as_ptr()), (*NPC).r.currentOrigin, dir);

        (*missile).classname = c"blaster".as_ptr();
        (*missile).s.weapon = WP_BLASTER;

        (*missile).damage = 5;
        (*missile).dflags = crate::level::damage_flags::DAMAGE_DEATH_KNOCKBACK;
        (*missile).methodOfDeath = MOD_BLASTER;
        (*missile).clipmask = MASK_SHOT;
        if (*NPC).r.ownerNum < ENTITYNUM_NONE {
            (*missile).r.ownerNum = (*NPC).r.ownerNum;
        }
    }
}

/// Raven `Seeker_Ranged`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:320-347`
pub fn Seeker_Ranged(
    ctx: GameContext<'_>,
    visible: qboolean,
    advance: qboolean,
) {
    unsafe {
        let NPC = (*ctx.world).globals.NPC;
        let NPCInfo = (*ctx.world).globals.NPCInfo;

        if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
            if (*NPC).count > 0 {
                if crate::g_timer::TIMER_Done(ctx, NPC, c"attackDelay".as_ptr()) != 0 {
                    crate::g_timer::TIMER_Set(ctx, NPC, c"attackDelay".as_ptr(), crate::q_math::Q_irand(250, 2500));
                    Seeker_Fire(ctx);
                    (*NPC).count -= 1;
                }
            } else {
                // out of ammo, so let it die...give it a push up so it can fall more and blow up on impact
                crate::g_combat::G_Damage(
                    NPC,
                    NPC,
                    NPC,
                    core::ptr::null(),
                    core::ptr::null(),
                    999,
                    0,
                    MOD_UNKNOWN,
                );
            }
        }

        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            Seeker_Hunt(ctx, visible, advance);
        }
    }
}

/// Raven `Seeker_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:350-380`
pub fn Seeker_Attack(ctx: GameContext<'_>) {
    unsafe {
        let NPC = (*ctx.world).globals.NPC;
        let NPCInfo = (*ctx.world).globals.NPCInfo;

        // Always keep a good height off the ground
        Seeker_MaintainHeight(ctx);

        // Rate our distance to the target, and our visibilty
        let distance = crate::q_math::DistanceHorizontalSquared((*NPC).r.currentOrigin, (*(*NPC).enemy).r.currentOrigin);
        let visible = crate::NPC_utils::NPC_ClearLOS4(ctx, (*NPC).enemy);
        let mut advance = if distance > MIN_DISTANCE_SQR as f32 { qtrue } else { qfalse };

        if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
            advance = if distance > (200.0f32 * 200.0f32) { qtrue } else { qfalse };
        }

        // If we cannot see our target, move to see it
        if visible == qfalse {
            if (*NPCInfo).scriptFlags & (SCF_CHASE_ENEMIES as c_int) != 0 {
                Seeker_Hunt(ctx, visible, advance);
                return;
            }
        }

        Seeker_Ranged(ctx, visible, advance);
    }
}

/// Raven `Seeker_FindEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:383-436`
pub fn Seeker_FindEnemy(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC;

        let mut numFound: c_int;
        let mut dis: f32;
        let mut bestDis: f32 = SEEKER_SEEK_RADIUS * SEEKER_SEEK_RADIUS + 1.0f32;
        let mut mins: vec3_t = [-SEEKER_SEEK_RADIUS; 3];
        let mut maxs: vec3_t = [SEEKER_SEEK_RADIUS; 3];
        let mut entityList: [c_int; mp_qshared::shared::MAX_GENTITIES] = [0; mp_qshared::shared::MAX_GENTITIES];
        let mut best: *mut gentity_t = core::ptr::null_mut();

        numFound = crate::trap::EntitiesInBox(ctx.engine, mins, maxs, entityList.as_mut_ptr(), mp_qshared::shared::MAX_GENTITIES as c_int);

        for i in 0..numFound {
            let ent = &mut world.entities[entityList[i as usize] as usize];

            if ent.s.number == (*NPC).s.number || ent.client.is_null() || ent.health <= 0 || ent.inuse == 0 {
                continue;
            }

            if (*ent.client).playerTeam == (*(*NPC).client).playerTeam || (*ent.client).playerTeam == crate::teams::npcteam::NPCTEAM_NEUTRAL {
                // don't attack same team or bots
                continue;
            }

            // try to find the closest visible one
            if crate::NPC_utils::NPC_ClearLOS4(ctx, ent) == 0 {
                continue;
            }

            dis = crate::q_math::DistanceHorizontalSquared((*NPC).r.currentOrigin, ent.r.currentOrigin);

            if dis <= bestDis {
                bestDis = dis;
                best = ent;
            }
        }

        if !best.is_null() {
            // used to offset seekers around a circle so they don't occupy the same spot.  This is not a fool-proof method.
            (*NPC).random = crate::bg_lib::random() * 6.3f32; // roughly 2pi

            (*NPC).enemy = best;
        }
    }
}

/// Raven `Seeker_FollowOwner`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:439-520`
pub fn Seeker_FollowOwner(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC;
        let NPCInfo = world.globals.NPCInfo;

        let mut dis: f32;
        let mut minDistSqr: f32;
        let mut pt: vec3_t = [0.0f32; 3];
        let mut dir: vec3_t = [0.0f32; 3];
        let mut owner: *mut gentity_t = &mut world.entities[(*NPC).s.owner as usize];

        Seeker_MaintainHeight(ctx);

        if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
            owner = (*NPC).enemy;
            if owner.is_null() {
                return;
            }
        }
        if owner.is_null() || owner == NPC || (*owner).client.is_null() {
            return;
        }
        //rwwFIXMEFIXME: Care about all clients not just 0
        dis = crate::q_math::DistanceHorizontalSquared((*NPC).r.currentOrigin, (*owner).r.currentOrigin);

        minDistSqr = MIN_DISTANCE_SQR as f32;

        if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
            if crate::g_timer::TIMER_Done(ctx, NPC, c"flameTime".as_ptr()) != 0 {
                minDistSqr = 200.0f32 * 200.0f32;
            }
        }

        if dis < minDistSqr {
            // generally circle the player closely till we take an enemy..this is our target point
            if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                pt[0] = (*owner).r.currentOrigin[0] + (world.level.time as f32 * 0.001f32 + (*NPC).random).cos() * 250.0f32;
                pt[1] = (*owner).r.currentOrigin[1] + (world.level.time as f32 * 0.001f32 + (*NPC).random).sin() * 250.0f32;
                if (*(*NPC).client).jetPackTime < world.level.time {
                    pt[2] = (*NPC).r.currentOrigin[2] - 64.0f32;
                } else {
                    pt[2] = (*owner).r.currentOrigin[2] + 200.0f32;
                }
            } else {
                pt[0] = (*owner).r.currentOrigin[0] + (world.level.time as f32 * 0.001f32 + (*NPC).random).cos() * 56.0f32;
                pt[1] = (*owner).r.currentOrigin[1] + (world.level.time as f32 * 0.001f32 + (*NPC).random).sin() * 56.0f32;
                pt[2] = (*owner).r.currentOrigin[2] + 40.0f32;
            }

            // Inline VectorSubtract: dir = pt - origin
            for i in 0..3 {
                dir[i] = pt[i] - (*NPC).r.currentOrigin[i];
            }
            // Inline VectorMA: velocity += 0.8 * dir
            for i in 0..3 {
                (*(*NPC).client).ps.velocity[i] += 0.8f32 * dir[i];
            }
        } else {
            if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
                if crate::g_timer::TIMER_Done(ctx, NPC, c"seekerhiss".as_ptr()) != 0 {
                    crate::g_timer::TIMER_Set(ctx, NPC, c"seekerhiss".as_ptr(), (1000 + crate::bg_lib::random() * 1000.0f32) as c_int);
                    crate::g_utils::G_Sound(
                        ctx,
                        NPC,
                        CHAN_AUTO,
                        crate::g_utils::G_SoundIndex(c"sound/chars/seeker/misc/hiss".as_ptr()),
                    );
                }
            }

            // Hey come back!
            (*NPCInfo).goalEntity = owner;
            (*NPCInfo).goalRadius = 32;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
            (*NPC).parent = owner;
        }

        if (*NPCInfo).enemyCheckDebounceTime < world.level.time {
            // check twice a second to find a new enemy
            Seeker_FindEnemy(ctx);
            (*NPCInfo).enemyCheckDebounceTime = world.level.time + 500;
        }

        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}

/// Raven `NPC_BSSeeker_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:523-574`
pub fn NPC_BSSeeker_Default(ctx: GameContext<'_>) {
    unsafe {
        let world = &mut *ctx.world;
        let NPC = world.globals.NPC;
        let NPCInfo = world.globals.NPCInfo;

        //N/A for MP.
        if (*NPC).r.ownerNum < ENTITYNUM_NONE {
            let owner = &mut world.entities[0];
            if (*owner).health <= 0 || (!(*owner).client.is_null() && (*(*owner).client).pers.connected == crate::client::client_connected::CON_DISCONNECTED) {
                //owner is dead or gone
                //remove me
                crate::g_combat::G_Damage(
                    NPC,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null(),
                    core::ptr::null(),
                    10000,
                    crate::level::damage_flags::DAMAGE_NO_PROTECTION,
                    MOD_TELEFRAG,
                );
                return;
            }
        }

        if (*NPC).random == 0.0f32 {
            // used to offset seekers around a circle so they don't occupy the same spot.  This is not a fool-proof method.
            (*NPC).random = crate::bg_lib::random() * 6.3f32; // roughly 2pi
        }

        if !(*NPC).enemy.is_null() && (*(*NPC).enemy).health > 0 && (*(*NPC).enemy).inuse != 0 {
            if (*(*NPC).client).NPC_class != CLASS_BOBAFETT && ((*(*NPC).enemy).s.number == 0 || (!(*(*NPC).enemy).client.is_null() && (*(*(*NPC).enemy).client).NPC_class == CLASS_SEEKER)) {
                //hacked to never take the player as an enemy, even if the player shoots at it
                (*NPC).enemy = core::ptr::null_mut();
            } else {
                Seeker_Attack(ctx);
                if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                    crate::NPC_AI_Jedi::Boba_FireDecide(ctx);
                }
                return;
            }
        }

        // In all other cases, follow the player and look for enemies to take on
        Seeker_FollowOwner(ctx);
    }
}
