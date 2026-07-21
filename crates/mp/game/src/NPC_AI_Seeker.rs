// PORT-COMPLETE: NPC_AI_Seeker.c
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_Seeker.c`.
//!
//! All 10 functions ported. Nearly every function in this file relies on
//! file-scope globals set up by `SetNPCGlobals()` (NPC, NPCInfo, ucmd, etc.)
//! or reads other ambient state (level, g_entities, g_spskill cvars). These
//! globals are now threaded through GameContext and accessed via ctx.world.
//!
//! Safe-state 2c: the NPC entity half is converted to `ctx.world.entity(npc_id)`
//! / `entity_mut` accessor borrows. Two irreducible raw-deref regimes remain
//! (FLAGged inline, task #7): `NPCInfo` is a `*mut gNPC_t` with no safe
//! accessor, and NPCs carry a `BG_Alloc`'d pool `gclient_t` (`gClPtrs`,
//! g_utils.c:430) that is not a `level.clients` slot, so `NPC->client` (and
//! other entities' `->client` in this AI) is dereffed raw exactly as Raven does.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_missile::CreateMissile;
use crate::npc::script_flags::SCF_CHASE_ENEMIES;
use crate::prelude::*;
use crate::g_utils::G_EffectIndex;
use crate::g_utils::G_SoundIndex;
use mp_qshared::shared::{CONTENTS_LIGHTSABER, MASK_SHOT};

// Raven `#define VELOCITY_DECAY 0.7f32` (oracle/codemp/game/NPC_AI_Seeker.c:8).
const VELOCITY_DECAY: f32 = 0.7f32;

// Raven `#define MIN_MELEE_RANGE 320` / `MIN_MELEE_RANGE_SQR`.
// Source: `oracle/codemp/game/NPC_AI_Seeker.c:10-11`
const MIN_MELEE_RANGE: c_int = 320;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;

// Raven `#define MIN_DISTANCE 80` / `MIN_DISTANCE_SQR`.
// Source: `oracle/codemp/game/NPC_AI_Seeker.c:13-14`
const MIN_DISTANCE: c_int = 80;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

// Raven `#define SEEKER_STRAFE_VEL 100` / `SEEKER_STRAFE_DIS 200` / `SEEKER_UPWARD_PUSH 32`.
pub const SEEKER_STRAFE_VEL: f32 = 100.0f32;
pub const SEEKER_STRAFE_DIS: f32 = 200.0f32;
pub const SEEKER_UPWARD_PUSH: f32 = 32.0f32;

// Raven `#define SEEKER_FORWARD_BASE_SPEED 10` / `SEEKER_FORWARD_MULTIPLIER 2`.
pub const SEEKER_FORWARD_BASE_SPEED: f32 = 10.0f32;
pub const SEEKER_FORWARD_MULTIPLIER: f32 = 2.0f32;

// Raven `#define SEEKER_SEEK_RADIUS 1024`.
pub const SEEKER_SEEK_RADIUS: f32 = 1024.0f32;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.

// Local constants for Seeker AI.
// Source: oracle/codemp/game/NPC_AI_Seeker.c / g_local.h
const MOD_FALLING: c_int = 38;
const MOD_BLASTER: c_int = 6;
const MOD_UNKNOWN: c_int = 0;
const MOD_TELEFRAG: c_int = 37;

/// Raven `NPC_Seeker_Precache`.
///
/// Caches sound and effect resources for Seeker NPCs at map load time.
///
/// Source: `oracle/codemp/game/NPC_AI_Seeker.c:26-31`
pub fn NPC_Seeker_Precache(ctx: &mut GameContext) {
    G_SoundIndex("sound/chars/seeker/misc/fire.wav");
    G_SoundIndex("sound/chars/seeker/misc/hiss.wav");
    G_EffectIndex("env/small_explode");
}

/// Raven `NPC_Seeker_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_Seeker.c:34-46`
pub fn NPC_Seeker_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    // FLAG (task #7): NPCInfo (gNPC_t) has no safe accessor; `aiFlags` read
    // stays a raw deref through the pointer read via the safe entity borrow.
    let npc_info = ctx.world.entity(self_).NPC;
    if unsafe { (*npc_info).aiFlags } & crate::npc::ai_flags::NPCAI_CUSTOM_GRAVITY == 0 {
        // Raven passes the global `vec3_origin` as `dir`; G_Damage normalizes
        // `dir` in place (a no-op on the zero vector), so a fresh local copy
        // is behaviorally identical.
        let mut origin = vec3_origin;
        crate::g_combat::G_Damage(
            ctx,
            Some(self_),
            None,
            None,
            Some(&mut origin),
            crate::q_math::vec3_origin,
            999,
            0,
            MOD_FALLING as c_int,
        );
    }

    crate::npc_c::SaveNPCGlobals(ctx);
    crate::npc_c::SetNPCGlobals(ctx, self_);
    Seeker_Strafe(ctx);
    crate::npc_c::RestoreNPCGlobals(ctx);
    crate::NPC_reactions::NPC_Pain(ctx, self_, attacker, damage);
}

/// Raven `Seeker_MaintainHeight`.
///
/// Source: `oracle/codemp/game/NPC_AI_Seeker.c:49-148`
pub fn Seeker_MaintainHeight(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    // Update our angles regardless
    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);

    // FLAG (task #7): NPC pool `gclient_t` — raw deref for every velocity op.
    let client = ctx.world.entity(npc_id).client;
    let enemy = ctx.world.entity(npc_id).enemy;

    // If we have an enemy, we should try to hover at or a little below enemy eye level
    if let Some(enemy_id) = enemy {
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"heightChange".as_ptr()) != 0 {
            let mut difFactor: f32 = 1.0f32;

            let delay = ctx.world.bg_state.rng.Q_irand(1000, 3000);
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"heightChange".as_ptr(), delay);

            // Find the height difference
            let (enemy_org_z, enemy_maxs_z) = {
                let e = ctx.world.entity(enemy_id);
                (e.r.currentOrigin[2], e.r.maxs[2])
            };
            let draw = ctx
                .world
                .bg_state
                .rng
                .flrand(enemy_maxs_z / 2.0f32, enemy_maxs_z + 8.0f32);
            let npc_org_z = ctx.world.entity(npc_id).r.currentOrigin[2];
            let dif = (enemy_org_z + draw) - npc_org_z;

            if unsafe { (*client).NPC_class } == CLASS_BOBAFETT {
                if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"flameTime".as_ptr()) != 0 {
                    difFactor = 10.0f32;
                }
            }

            // cap to prevent dramatic height shifts
            if dif.abs() > 2.0f32 * difFactor {
                let mut dif_capped = dif;
                if dif_capped.abs() > 24.0f32 * difFactor {
                    dif_capped = if dif < 0.0f32 {
                        -24.0f32 * difFactor
                    } else {
                        24.0f32 * difFactor
                    };
                }

                unsafe {
                    (*client).ps.velocity[2] = ((*client).ps.velocity[2] + dif_capped) / 2.0f32;
                }
            }
            if unsafe { (*client).NPC_class } == CLASS_BOBAFETT {
                let draw2 = ctx.world.bg_state.rng.flrand(0.85f32, 3.0f32);
                unsafe {
                    (*client).ps.velocity[2] *= draw2;
                }
            }
        }
    } else {
        // FLAG (task #7): NPCInfo (gNPC_t) goalEntity/lastGoalEntity — raw reads.
        let goal: Option<EntityId> = unsafe {
            if (*npc_info).goalEntity.is_some() {
                // Is there a goal?
                (*npc_info).goalEntity
            } else {
                (*npc_info).lastGoalEntity
            }
        };
        if let Some(goal_id) = goal {
            let goal_z = ctx.world.entity(goal_id).r.currentOrigin[2];
            let npc_z = ctx.world.entity(npc_id).r.currentOrigin[2];
            let dif = goal_z - npc_z;

            if dif.abs() > 24.0f32 {
                ctx.world.globals.ucmd.upmove = if ctx.world.globals.ucmd.upmove < 0 {
                    -4
                } else {
                    4
                };
            } else {
                unsafe {
                    if (*client).ps.velocity[2] != 0.0f32 {
                        (*client).ps.velocity[2] *= VELOCITY_DECAY;

                        if (*client).ps.velocity[2].abs() < 2.0f32 {
                            (*client).ps.velocity[2] = 0.0f32;
                        }
                    }
                }
            }
        }
    }

    // Apply friction
    unsafe {
        if (*client).ps.velocity[0] != 0.0f32 {
            (*client).ps.velocity[0] *= VELOCITY_DECAY;

            if (*client).ps.velocity[0].abs() < 1.0f32 {
                (*client).ps.velocity[0] = 0.0f32;
            }
        }

        if (*client).ps.velocity[1] != 0.0f32 {
            (*client).ps.velocity[1] *= VELOCITY_DECAY;

            if (*client).ps.velocity[1].abs() < 1.0f32 {
                (*client).ps.velocity[1] = 0.0f32;
            }
        }
    }
}

/// Raven `Seeker_Strafe`.
///
/// Source: `oracle/codemp/game/NPC_AI_Seeker.c:151-239`
pub fn Seeker_Strafe(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    let mut end: vec3_t = [0.0f32; 3];
    let mut right: vec3_t = [0.0f32; 3];
    let mut dir: vec3_t = [0.0f32; 3];
    let mut tr: trace_t = unsafe { core::mem::zeroed() };

    // FLAG (task #7): NPC pool `gclient_t` — raw deref for eyeAngles / velocity.
    let client = ctx.world.entity(npc_id).client;

    // Read `NPC->enemy` for the branch predicate; the RNG draw is the first
    // operand and must be evaluated regardless (Raven `||` short-circuit).
    let enemy = ctx.world.entity(npc_id).enemy;
    let regular = ctx.world.bg_state.rng.random() > 0.7f32
        || enemy.is_none()
        || ctx.world.entity(enemy.unwrap()).client.is_null();

    if regular {
        // Do a regular style strafe
        let eye = unsafe { (*client).renderInfo.eyeAngles };
        crate::q_math::AngleVectors(eye, None, Some(&mut right), None);

        let roll = ctx.world.bg_state.rng.rand() & 1;
        // Pick a random strafe direction, then check to see if doing a strafe would be
        // reasonably valid
        let side = if (roll) != 0 { -1 } else { 1 };
        let npc_org = ctx.world.entity(npc_id).r.currentOrigin;
        // Inline VectorMA: end = origin + scalar * right
        for i in 0..3 {
            end[i] = npc_org[i] + SEEKER_STRAFE_DIS * side as f32 * right[i];
        }

        let npc_number = ctx.world.entity(npc_id).s.number;
        crate::trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut tr,
                &npc_org,
                core::ptr::null(),
                core::ptr::null(),
                &end,
                npc_number,
                MASK_SOLID,
            ),
        );

        // Close enough
        if tr.fraction > 0.9f32 {
            let mut vel = SEEKER_STRAFE_VEL;
            let mut upPush = SEEKER_UPWARD_PUSH;
            if unsafe { (*client).NPC_class } != CLASS_BOBAFETT {
                crate::g_utils::G_Sound(
                    ctx,
                    Some(npc_id),
                    CHAN_AUTO,
                    G_SoundIndex("sound/chars/seeker/misc/hiss"),
                );
            } else {
                vel *= 3.0f32;
                upPush *= 4.0f32;
            }
            // Inline VectorMA: velocity += vel * side * right
            unsafe {
                for i in 0..3 {
                    (*client).ps.velocity[i] += vel * side as f32 * right[i];
                }
                // Add a slight upward push
                (*client).ps.velocity[2] += upPush;
            }

            // FLAG (task #7): NPCInfo (gNPC_t) standTime — raw write.
            let stand =
                ctx.world.level.time + 1000 + (ctx.world.bg_state.rng.random() * 500.0f32) as c_int;
            unsafe {
                (*npc_info).standTime = stand;
            }
        }
    } else {
        // guaranteed non-null by the `if` branch above (enemy is_some && enemy.client non-null)
        let enemy_id = enemy.unwrap();

        // Do a strafe to try and keep on the side of their enemy.
        // FLAG (task #7): enemy pool/real `gclient_t` — raw deref for eyeAngles.
        let enemy_client = ctx.world.entity(enemy_id).client;
        let enemy_eye = unsafe { (*enemy_client).renderInfo.eyeAngles };
        crate::q_math::AngleVectors(enemy_eye, Some(&mut dir), Some(&mut right), None);

        let roll = ctx.world.bg_state.rng.rand() & 1;
        // Pick a random side
        let side = if (roll) != 0 { -1 } else { 1 };
        let mut stDis = SEEKER_STRAFE_DIS;
        if unsafe { (*client).NPC_class } == CLASS_BOBAFETT {
            stDis *= 2.0f32;
        }
        let enemy_org = ctx.world.entity(enemy_id).r.currentOrigin;
        // Inline VectorMA: end = enemy_origin + stDis * side * right
        for i in 0..3 {
            end[i] = enemy_org[i] + stDis * side as f32 * right[i];
        }

        // then add a very small bit of random in front of/behind the player action
        // VectorMA is the live `#if 1` MACRO (q_shared.h:1365) — the scale expr
        // `crandom()*25` is substituted per component: THREE crandom draws, each
        // product in f64, narrowed at the store. Never reason from `_VectorMA`'s
        // float-scale signature (dead `#else` branch, q_shared.h:1381).
        // Source: oracle/codemp/game/NPC_AI_Seeker.c:207
        for i in 0..3 {
            end[i] =
                (end[i] as f64 + ctx.world.bg_state.rng.crandom() * 25.0 * dir[i] as f64) as f32;
        }

        let npc_org = ctx.world.entity(npc_id).r.currentOrigin;
        let npc_number = ctx.world.entity(npc_id).s.number;
        crate::trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut tr,
                &npc_org,
                core::ptr::null(),
                core::ptr::null(),
                &end,
                npc_number,
                MASK_SOLID,
            ),
        );

        // Close enough
        if tr.fraction > 0.9f32 {
            let mut upPush: f32;

            // Inline VectorSubtract: dir = endpos - origin
            for i in 0..3 {
                dir[i] = tr.endpos[i] - npc_org[i];
            }
            dir[2] *= 0.25f32; // do less upward change
            let dis = crate::q_math::VectorNormalize(&mut dir);

            // Inline VectorMA: velocity += dis * dir
            unsafe {
                for i in 0..3 {
                    (*client).ps.velocity[i] += dis * dir[i];
                }
            }

            upPush = SEEKER_UPWARD_PUSH;
            if unsafe { (*client).NPC_class } != CLASS_BOBAFETT {
                crate::g_utils::G_Sound(
                    ctx,
                    Some(npc_id),
                    CHAN_AUTO,
                    G_SoundIndex("sound/chars/seeker/misc/hiss"),
                );
            } else {
                upPush *= 4.0f32;
            }

            // Add a slight upward push
            unsafe {
                (*client).ps.velocity[2] += upPush;
            }

            // FLAG (task #7): NPCInfo (gNPC_t) standTime — raw write.
            let stand =
                ctx.world.level.time + 2500 + (ctx.world.bg_state.rng.random() * 500.0f32) as c_int;
            unsafe {
                (*npc_info).standTime = stand;
            }
        }
    }
}

/// Raven `Seeker_Hunt`.
///
/// Source: `oracle/codemp/game/NPC_AI_Seeker.c:242-287`
pub fn Seeker_Hunt(ctx: &mut GameContext, visible: qboolean, advance: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);

    // If we're not supposed to stand still, pursue the player.
    // FLAG (task #7): NPCInfo (gNPC_t) standTime — raw read.
    if unsafe { (*npc_info).standTime } < ctx.world.level.time {
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
        // Move towards our goal.
        // FLAG (task #7): NPCInfo (gNPC_t) goalEntity/goalRadius — raw writes.
        let enemy = ctx.world.entity(npc_id).enemy;
        unsafe {
            (*npc_info).goalEntity = enemy;
            (*npc_info).goalRadius = 24;
        }

        // Get our direction from the navigator if we can't see our target
        let mut forward: vec3_t = [0.0f32; 3];
        let mut distance: f32 = 0.0f32;
        if crate::NPC_move::NPC_GetMoveDirection(ctx, &mut forward, &mut distance) == qfalse {
            return;
        }

        let speed = SEEKER_FORWARD_BASE_SPEED
            + SEEKER_FORWARD_MULTIPLIER * ctx.world.cvars.g_spskill.integer as f32;
        let client = ctx.world.entity(npc_id).client;
        unsafe {
            for i in 0..3 {
                (*client).ps.velocity[i] += speed * forward[i];
            }
        }
    } else {
        let mut forward: vec3_t = [0.0f32; 3];
        let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();
        let enemy_org = ctx.world.entity(enemy_id).r.currentOrigin;
        let npc_org = ctx.world.entity(npc_id).r.currentOrigin;
        forward[0] = enemy_org[0] - npc_org[0];
        forward[1] = enemy_org[1] - npc_org[1];
        forward[2] = enemy_org[2] - npc_org[2];
        let _distance = crate::q_math::VectorNormalize(&mut forward);

        let speed = SEEKER_FORWARD_BASE_SPEED
            + SEEKER_FORWARD_MULTIPLIER * ctx.world.cvars.g_spskill.integer as f32;
        let client = ctx.world.entity(npc_id).client;
        unsafe {
            for i in 0..3 {
                (*client).ps.velocity[i] += speed * forward[i];
            }
        }
    }
}

/// Raven `Seeker_Fire`.
///
/// Source: `oracle/codemp/game/NPC_AI_Seeker.c:290-317`
pub fn Seeker_Fire(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let mut dir: vec3_t = [0.0f32; 3];
    let mut enemy_org: vec3_t = [0.0f32; 3];
    let mut muzzle: vec3_t = [0.0f32; 3];

    let enemy_id = ctx.world.entity(npc_id).enemy;
    crate::NPC_utils::CalcEntitySpot(ctx, enemy_id, spot_t::SPOT_HEAD, &mut enemy_org);
    let npc_org = ctx.world.entity(npc_id).r.currentOrigin;
    // Inline VectorSubtract: dir = enemy_org - origin
    for i in 0..3 {
        dir[i] = enemy_org[i] - npc_org[i];
    }
    crate::q_math::VectorNormalize(&mut dir);

    // move a bit forward in the direction we shall shoot in so that the bolt doesn't poke out the other side of the seeker
    // Inline VectorMA: muzzle = origin + 15 * dir
    for i in 0..3 {
        muzzle[i] = npc_org[i] + 15.0f32 * dir[i];
    }

    let missile_id = CreateMissile(ctx, muzzle, dir, 1000.0f32, 10000, npc_id, false);

    crate::g_utils::G_PlayEffectID(
        G_EffectIndex("blaster/muzzle_flash"),
        npc_org,
        dir,
    );

    {
        let missile = ctx.world.entity_mut(missile_id);
        missile.classname = c"blaster".as_ptr().cast_mut();
        missile.s.weapon = WP_BLASTER as c_int;
        missile.damage = 5;
        missile.dflags = crate::level::damage_flags::DAMAGE_DEATH_KNOCKBACK;
        missile.methodOfDeath = MOD_BLASTER as c_int;
        missile.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    }
    let npc_owner = ctx.world.entity(npc_id).r.ownerNum;
    if npc_owner < ENTITYNUM_NONE {
        ctx.world.entity_mut(missile_id).r.ownerNum = npc_owner;
    }
}

/// Raven `Seeker_Ranged`.
///
/// Source: `oracle/codemp/game/NPC_AI_Seeker.c:320-347`
pub fn Seeker_Ranged(ctx: &mut GameContext, visible: qboolean, advance: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    // FLAG (task #7): NPC pool `gclient_t` — raw deref for NPC_class.
    let client = ctx.world.entity(npc_id).client;
    if unsafe { (*client).NPC_class } != CLASS_BOBAFETT {
        if ctx.world.entity(npc_id).count > 0 {
            if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"attackDelay".as_ptr()) != 0 {
                let delay = ctx.world.bg_state.rng.Q_irand(250, 2500);
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attackDelay".as_ptr(), delay);
                Seeker_Fire(ctx);
                ctx.world.entity_mut(npc_id).count -= 1;
            }
        } else {
            // out of ammo, so let it die...give it a push up so it can fall more and blow up on impact
            crate::g_combat::G_Damage(
                ctx,
                Some(npc_id),
                Some(npc_id),
                Some(npc_id),
                None,
                crate::q_math::vec3_origin,
                999,
                0,
                MOD_UNKNOWN as c_int,
            );
        }
    }

    // FLAG (task #7): NPCInfo (gNPC_t) scriptFlags — raw read.
    if (unsafe { (*npc_info).scriptFlags } & SCF_CHASE_ENEMIES) != 0 {
        Seeker_Hunt(ctx, visible, advance);
    }
}

/// Raven `Seeker_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Seeker.c:350-380`
pub fn Seeker_Attack(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    // Always keep a good height off the ground
    Seeker_MaintainHeight(ctx);

    // Rate our distance to the target, and our visibilty
    let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();
    let npc_org = ctx.world.entity(npc_id).r.currentOrigin;
    let enemy_org = ctx.world.entity(enemy_id).r.currentOrigin;
    let distance = crate::q_math::DistanceHorizontalSquared(npc_org, enemy_org);
    let enemy = ctx.world.entity(npc_id).enemy;
    let visible = crate::NPC_utils::NPC_ClearLOS4(ctx, enemy);
    let mut advance = if distance > MIN_DISTANCE_SQR as f32 {
        qtrue
    } else {
        qfalse
    };

    // FLAG (task #7): NPC pool `gclient_t` — raw deref for NPC_class.
    let client = ctx.world.entity(npc_id).client;
    if unsafe { (*client).NPC_class } == CLASS_BOBAFETT {
        advance = if distance > (200.0f32 * 200.0f32) {
            qtrue
        } else {
            qfalse
        };
    }

    // If we cannot see our target, move to see it
    if visible == qfalse {
        // FLAG (task #7): NPCInfo (gNPC_t) scriptFlags — raw read.
        if unsafe { (*npc_info).scriptFlags } & (SCF_CHASE_ENEMIES as c_int) != 0 {
            Seeker_Hunt(ctx, visible, advance);
            return;
        }
    }

    Seeker_Ranged(ctx, visible, advance);
}

/// Raven `Seeker_FindEnemy`.
///
/// Source: `oracle/codemp/game/NPC_AI_Seeker.c:383-436`
pub fn Seeker_FindEnemy(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let mut dis: f32;
    let mut bestDis: f32 = SEEKER_SEEK_RADIUS * SEEKER_SEEK_RADIUS + 1.0f32;
    let mut mins: vec3_t = [-SEEKER_SEEK_RADIUS; 3];
    let mut maxs: vec3_t = [SEEKER_SEEK_RADIUS; 3];
    let mut entityList: [c_int; mp_qshared::shared::MAX_GENTITIES] =
        [0; mp_qshared::shared::MAX_GENTITIES];
    let mut best: Option<EntityId> = None;

    let numFound = crate::trap::EntitiesInBox(
        ctx.engine,
        mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs::new(
            mins.as_ptr() as *const vec3_t,
            maxs.as_ptr() as *const vec3_t,
            entityList.as_mut_ptr(),
            mp_qshared::shared::MAX_GENTITIES as c_int,
        ),
    );

    for i in 0..numFound {
        // `entityList[i]` is a valid arena index returned by the trap.
        let ent_id = EntityId(entityList[i as usize] as u32);

        let (ent_number, ent_client, ent_health, ent_inuse) = {
            let e = ctx.world.entity(ent_id);
            (e.s.number, e.client, e.health, e.inuse)
        };
        let npc_number = ctx.world.entity(npc_id).s.number;
        if ent_number == npc_number || ent_client.is_null() || ent_health <= 0 || ent_inuse == 0 {
            continue;
        }

        // FLAG (task #7): both entities can be NPCs carrying pool `gclient_t`s;
        // `playerTeam` is read through each entity's client pointer, raw, exactly
        // as Raven does — never via a `level.clients` index.
        let npc_client = ctx.world.entity(npc_id).client;
        let ent_team = unsafe { (*ent_client).playerTeam };
        let npc_team = unsafe { (*npc_client).playerTeam };
        if ent_team == npc_team || ent_team == crate::teams::npcteam::NPCTEAM_NEUTRAL {
            // don't attack same team or bots
            continue;
        }

        // try to find the closest visible one
        if crate::NPC_utils::NPC_ClearLOS4(ctx, Some(ent_id)) == 0 {
            continue;
        }

        let npc_org = ctx.world.entity(npc_id).r.currentOrigin;
        let ent_org = ctx.world.entity(ent_id).r.currentOrigin;
        dis = crate::q_math::DistanceHorizontalSquared(npc_org, ent_org);

        if dis <= bestDis {
            bestDis = dis;
            best = Some(ent_id);
        }
    }

    if let Some(best_id) = best {
        // used to offset seekers around a circle so they don't occupy the same spot.  This is not a fool-proof method.
        let draw = ctx.world.bg_state.rng.random() * 6.3f32; // roughly 2pi
        ctx.world.entity_mut(npc_id).random = draw;

        ctx.world.entity_mut(npc_id).enemy = Some(best_id);
    }
}

/// Raven `Seeker_FollowOwner`.
///
/// Source: `oracle/codemp/game/NPC_AI_Seeker.c:439-520`
pub fn Seeker_FollowOwner(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    let mut dis: f32;
    let mut minDistSqr: f32;
    let mut pt: vec3_t = [0.0f32; 3];
    let mut dir: vec3_t = [0.0f32; 3];
    let mut owner_id: Option<EntityId> = Some(EntityId(ctx.world.entity(npc_id).s.owner as u32));

    Seeker_MaintainHeight(ctx);

    // FLAG (task #7): NPC pool `gclient_t` — raw deref for NPC_class / jetPackTime.
    let client = ctx.world.entity(npc_id).client;
    if unsafe { (*client).NPC_class } == CLASS_BOBAFETT {
        owner_id = ctx.world.entity(npc_id).enemy;
        if owner_id.is_none() {
            return;
        }
    }
    let owner_id = owner_id.unwrap();
    let owner_client = ctx.world.entity(owner_id).client;
    if owner_id == npc_id || owner_client.is_null() {
        return;
    }
    //rwwFIXMEFIXME: Care about all clients not just 0
    let npc_org = ctx.world.entity(npc_id).r.currentOrigin;
    let owner_org = ctx.world.entity(owner_id).r.currentOrigin;
    dis = crate::q_math::DistanceHorizontalSquared(npc_org, owner_org);

    minDistSqr = MIN_DISTANCE_SQR as f32;

    if unsafe { (*client).NPC_class } == CLASS_BOBAFETT {
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"flameTime".as_ptr()) != 0 {
            minDistSqr = 200.0f32 * 200.0f32;
        }
    }

    if dis < minDistSqr {
        // generally circle the player closely till we take an enemy..this is our target point
        // `cos`/`sin` are the double libm: the f32 argument is promoted, the
        // transcendental and its scaling evaluate in f64, and the sum with the
        // float origin narrows to f32 only on store.
        let time = ctx.world.level.time;
        let random = ctx.world.entity(npc_id).random;
        let owner_org = ctx.world.entity(owner_id).r.currentOrigin;
        if unsafe { (*client).NPC_class } == CLASS_BOBAFETT {
            pt[0] = (owner_org[0] as f64 + ((time as f32 * 0.001f32 + random) as f64).cos() * 250.0)
                as f32;
            pt[1] = (owner_org[1] as f64 + ((time as f32 * 0.001f32 + random) as f64).sin() * 250.0)
                as f32;
            if unsafe { (*client).jetPackTime } < time {
                pt[2] = ctx.world.entity(npc_id).r.currentOrigin[2] - 64.0f32;
            } else {
                pt[2] = owner_org[2] + 200.0f32;
            }
        } else {
            pt[0] = (owner_org[0] as f64 + ((time as f32 * 0.001f32 + random) as f64).cos() * 56.0)
                as f32;
            pt[1] = (owner_org[1] as f64 + ((time as f32 * 0.001f32 + random) as f64).sin() * 56.0)
                as f32;
            pt[2] = owner_org[2] + 40.0f32;
        }

        let npc_org = ctx.world.entity(npc_id).r.currentOrigin;
        // Inline VectorSubtract: dir = pt - origin
        for i in 0..3 {
            dir[i] = pt[i] - npc_org[i];
        }
        // Inline VectorMA: velocity += 0.8 * dir
        unsafe {
            for i in 0..3 {
                (*client).ps.velocity[i] += 0.8f32 * dir[i];
            }
        }
    } else {
        if unsafe { (*client).NPC_class } != CLASS_BOBAFETT {
            if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"seekerhiss".as_ptr()) != 0 {
                let delay = (1000.0f32 + ctx.world.bg_state.rng.random() * 1000.0f32) as c_int;
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"seekerhiss".as_ptr(), delay);
                crate::g_utils::G_Sound(
                    ctx,
                    Some(npc_id),
                    CHAN_AUTO,
                    G_SoundIndex("sound/chars/seeker/misc/hiss"),
                );
            }
        }

        // Hey come back!
        // FLAG (task #7): NPCInfo (gNPC_t) goalEntity/goalRadius — raw writes.
        unsafe {
            (*npc_info).goalEntity = Some(owner_id);
            (*npc_info).goalRadius = 32;
        }
        crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        ctx.world.entity_mut(npc_id).parent = Some(owner_id);
    }

    // FLAG (task #7): NPCInfo (gNPC_t) enemyCheckDebounceTime — raw read/write.
    if unsafe { (*npc_info).enemyCheckDebounceTime } < ctx.world.level.time {
        // check twice a second to find a new enemy
        Seeker_FindEnemy(ctx);
        let t = ctx.world.level.time + 500;
        unsafe {
            (*npc_info).enemyCheckDebounceTime = t;
        }
    }

    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
}

/// Raven `NPC_BSSeeker_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_Seeker.c:523-574`
pub fn NPC_BSSeeker_Default(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let npc_info = ctx.world.globals.NPCInfo;

    //N/A for MP.
    if ctx.world.entity(npc_id).r.ownerNum < ENTITYNUM_NONE {
        let owner_id = EntityId(0);
        let owner_health = ctx.world.entity(owner_id).health;
        // FLAG (task #7): owner client — read through the entity's client
        // pointer, dereffed raw exactly as Raven does.
        let owner_client = ctx.world.entity(owner_id).client;
        if owner_health <= 0
            || (!owner_client.is_null()
                && unsafe { (*owner_client).pers.connected }
                    == crate::client::client_connected::CON_DISCONNECTED)
        {
            //owner is dead or gone
            //remove me
            crate::g_combat::G_Damage(
                ctx,
                Some(npc_id),
                None,
                None,
                None,
                crate::q_math::vec3_origin,
                10000,
                crate::level::damage_flags::DAMAGE_NO_PROTECTION,
                MOD_TELEFRAG as c_int,
            );
            return;
        }
    }

    if ctx.world.entity(npc_id).random == 0.0f32 {
        // used to offset seekers around a circle so they don't occupy the same spot.  This is not a fool-proof method.
        let draw = ctx.world.bg_state.rng.random() * 6.3f32; // roughly 2pi
        ctx.world.entity_mut(npc_id).random = draw;
    }

    if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
        let enemy_health = ctx.world.entity(enemy_id).health;
        let enemy_inuse = ctx.world.entity(enemy_id).inuse;
        // Oracle tests `NPC->enemy->health` truthy (`!= 0`), not `> 0`.
        if enemy_health != 0 && enemy_inuse != 0 {
            // FLAG (task #7): NPC / enemy pool `gclient_t` — raw derefs for NPC_class.
            let client = ctx.world.entity(npc_id).client;
            let npc_class = unsafe { (*client).NPC_class };
            let enemy_number = ctx.world.entity(enemy_id).s.number;
            let enemy_client = ctx.world.entity(enemy_id).client;
            let enemy_is_seeker =
                !enemy_client.is_null() && unsafe { (*enemy_client).NPC_class } == CLASS_SEEKER;
            if npc_class != CLASS_BOBAFETT && (enemy_number == 0 || enemy_is_seeker) {
                //hacked to never take the player as an enemy, even if the player shoots at it
                ctx.world.entity_mut(npc_id).enemy = None;
            } else {
                Seeker_Attack(ctx);
                let client = ctx.world.entity(npc_id).client;
                if unsafe { (*client).NPC_class } == CLASS_BOBAFETT {
                    crate::NPC_AI_Jedi::Boba_FireDecide(ctx);
                }
                return;
            }
        }
    }

    // In all other cases, follow the player and look for enemies to take on
    Seeker_FollowOwner(ctx);
}
