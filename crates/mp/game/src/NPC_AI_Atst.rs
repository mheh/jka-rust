// PORT-COMPLETE: NPC_AI_Atst.c 9/9
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_Atst.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::bg_misc::BG_FindItemForWeapon;
use crate::g_items::RegisterItem;
use crate::g_utils::{G_EffectIndex, G_SoundIndex, G_SoundOnEnt};
use crate::prelude::*;
use crate::NPC_AI_Default::NPC_BSIdle;
use crate::NPC_reactions::NPC_Pain;

/// Min melee attack range.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:3`
const MIN_MELEE_RANGE: c_int = 640;

/// Min melee range squared.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:4`
pub const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;

/// Min distance.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:6`
pub const MIN_DISTANCE: c_int = 128;

/// Min distance squared.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:7`
pub const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

/// Surface render status flag: turn off.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:9`
pub const TURN_OFF: c_int = 0x00000100;

/// Left arm health.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:11`
pub const LEFT_ARM_HEALTH: c_int = 40;

/// Right arm health.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:12`
pub const RIGHT_ARM_HEALTH: c_int = 40;

/// Raven `NPC_ATST_Precache`.
///
/// Precache weapon and effect resources.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:20-34`
pub fn NPC_ATST_Precache(ctx: &mut GameContext) {
    // SAFETY: G_SoundIndex, G_EffectIndex, RegisterItem accessed through game context.
    G_SoundIndex(b"sound/chars/atst/atst_damaged1\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/atst/atst_damaged2\0".as_ptr() as *const c_char);

    RegisterItem(ctx, BG_FindItemForWeapon(WP_BOWCASTER));
    RegisterItem(ctx, BG_FindItemForWeapon(WP_ROCKET_LAUNCHER));

    G_EffectIndex(b"env/med_explode2\0".as_ptr() as *const c_char);
    G_EffectIndex(b"blaster/smoke_bolton\0".as_ptr() as *const c_char);
    G_EffectIndex(b"explosions/droidexplosion1\0".as_ptr() as *const c_char);
}

/// Raven `G_ATSTCheckPain`.
///
/// Called by NPC's and player in an ATST. Plays a damage sound.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:66-113`
pub fn G_ATSTCheckPain(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    damage: c_int,
) {
    if ctx.world.bg_state.rng.rand() & 1 != 0 {
        G_SoundOnEnt(
            ctx,
            self_,
            CHAN_LESS_ATTEN,
            b"sound/chars/atst/atst_damaged1\0".as_ptr() as *const c_char,
        );
    } else {
        G_SoundOnEnt(
            ctx,
            self_,
            CHAN_LESS_ATTEN,
            b"sound/chars/atst/atst_damaged2\0".as_ptr() as *const c_char,
        );
    }
}

/// Raven `NPC_ATST_Pain`.
///
/// Called when ATST takes damage. Plays pain sound and calls NPC pain handler.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:119-123`
pub fn NPC_ATST_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    G_ATSTCheckPain(ctx, self_, attacker, damage);
    NPC_Pain(ctx, self_, attacker, damage);
}

/// Raven `ATST_Hunt`.
///
/// Hunt down the enemy. Set goal to enemy and move toward it.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:130-142`
pub fn ATST_Hunt(ctx: &mut GameContext, visible: qboolean, advance: qboolean) {
    // PORT-NOTE(ai-context): NPC via the entity accessor; NPCInfo (gNPC_t) has no
    // safe pool accessor yet, so its deref stays unsafe (NPC_AI_Default precedent).
    let npc = ctx.world.globals.NPC;
    // SAFETY: NPCInfo points into the module-owned gNPC pool (no aliasing accessor).
    let npc_info = unsafe { &mut *ctx.world.globals.NPCInfo };

    if npc_info.goalEntity.is_none() {
        // hunt
        let npc_id = ctx.entity_id_of(npc).unwrap();
        npc_info.goalEntity = ctx.entity(npc_id).enemy;
    }

    npc_info.combatMove = qtrue;

    NPC_MoveToGoal(ctx, qtrue);
}

/// Raven `ATST_Ranged`.
///
/// Perform a ranged attack. Check attack delay, fire weapons, chase if needed.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:149-170`
pub fn ATST_Ranged(
    ctx: &mut GameContext,
    visible: qboolean,
    advance: qboolean,
    altAttack: qboolean,
) {
    // PORT-NOTE(ai-context): NPC/ucmd via ctx.world; NPCInfo (gNPC_t) deref stays unsafe.
    let npc = ctx.world.globals.NPC;

    if TIMER_Done(
        ctx,
        ctx.entity_id_of(npc),
        b"atkDelay\0".as_ptr() as *const c_char,
    ) != qfalse
        && visible != qfalse
    {
        let npc_id = ctx.entity_id_of(npc);
        let delay = ctx.world.bg_state.rng.Q_irand(500, 3000);
        // Attack?
        TIMER_Set(ctx, npc_id, b"atkDelay\0".as_ptr() as *const c_char, delay);

        if altAttack != qfalse {
            ctx.world.globals.ucmd.buttons |= BUTTON_ATTACK | BUTTON_ALT_ATTACK;
        } else {
            ctx.world.globals.ucmd.buttons |= BUTTON_ATTACK;
        }
    }

    // SAFETY: NPCInfo (gNPC_t) has no safe pool accessor; deref stays unsafe.
    if (unsafe { &*ctx.world.globals.NPCInfo }.scriptFlags & SCF_CHASE_ENEMIES) != 0 {
        ATST_Hunt(ctx, visible, advance);
    }
}

/// Raven `ATST_Attack`.
///
/// Main attack decision logic. Check if enemy still valid, determine distance,
/// check visibility, and decide weapon type based on distance.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:177-264`
pub fn ATST_Attack(ctx: &mut GameContext) {
    // PORT-NOTE(ai-context): NPC via the entity accessor; NPCInfo (gNPC_t) deref stays unsafe.
    let npc = ctx.world.globals.NPC;

    let mut alt_attack: qboolean = qfalse;
    let mut blaster_test: c_int;
    let mut charger_test: c_int;
    let mut weapon: c_int;
    let distance: c_int;
    let dist_rate: distance_e;
    let visible: qboolean;
    let advance: qboolean;

    if NPC_CheckEnemyExt(ctx, qfalse) == qfalse {
        let npc_id = ctx.entity_id_of(npc).unwrap();
        ctx.entity_mut(npc_id).enemy = None;
        return;
    }

    NPC_FaceEnemy(ctx, qtrue);

    // Rate our distance to the target, and our visibility
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let enemy_id = ctx.entity(npc_id).enemy.unwrap();
    let npc_origin = ctx.entity(npc_id).r.currentOrigin;
    let enemy_origin = ctx.entity(enemy_id).r.currentOrigin;
    distance = DistanceHorizontalSquared(npc_origin, enemy_origin) as c_int;
    dist_rate = if distance > MIN_MELEE_RANGE_SQR {
        DIST_LONG
    } else {
        DIST_MELEE
    };
    visible = NPC_ClearLOS4(ctx, Some(enemy_id));
    advance = if distance > MIN_DISTANCE_SQR {
        qtrue
    } else {
        qfalse
    };

    // If we cannot see our target, move to see it
    if visible == qfalse {
        // SAFETY: NPCInfo (gNPC_t) has no safe pool accessor; deref stays unsafe.
        if (unsafe { &*ctx.world.globals.NPCInfo }.scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            ATST_Hunt(ctx, visible, advance);
            return;
        }
    }

    // Decide what type of attack to do
    match dist_rate {
        DIST_MELEE => {
            // NPC_ChangeWeapon( WP_ATST_MAIN );
        }

        DIST_LONG => {
            // NPC_ChangeWeapon( WP_ATST_SIDE );
            // rwwFIXMEFIXME: make atst weaps work.

            // See if the side weapons are there
            blaster_test = trap::G2API_GetSurfaceRenderStatus(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                    ctx.entity(npc_id).ghoul2,
                    0,
                    cstr("head_light_blaster_cann"),
                ),
            );
            charger_test = trap::G2API_GetSurfaceRenderStatus(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                    ctx.entity(npc_id).ghoul2,
                    0,
                    cstr("head_concussion_charger"),
                ),
            );

            // It has both side weapons
            if blaster_test != -1
                && (blaster_test & TURN_OFF) == 0
                && charger_test != -1
                && (charger_test & TURN_OFF) == 0
            {
                weapon = ctx.world.bg_state.rng.Q_irand(0, 1); // 0 is blaster, 1 is charger (ALT SIDE)

                if weapon != 0 {
                    // Fire charger
                    alt_attack = qtrue;
                } else {
                    alt_attack = qfalse;
                }
            } else if blaster_test != -1 && (blaster_test & TURN_OFF) == 0 {
                // Blaster is on
                alt_attack = qfalse;
            } else if charger_test != -1 && (charger_test & TURN_OFF) == 0 {
                // Charger is on
                alt_attack = qtrue;
            } else {
                NPC_ChangeWeapon(WP_NONE);
            }
        }

        _ => {}
    }

    NPC_FaceEnemy(ctx, qtrue);

    ATST_Ranged(ctx, visible, advance, alt_attack);
}

/// Raven `ATST_Patrol`.
///
/// Patrol the area. Check for stealth players, update goal if no enemy,
/// and move toward goal.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:271-290`
pub fn ATST_Patrol(ctx: &mut GameContext) {
    // PORT-NOTE(ai-context): NPC/ucmd via ctx.world.
    let npc = ctx.world.globals.NPC;

    if NPC_CheckPlayerTeamStealth(ctx) != qfalse {
        NPC_UpdateAngles(ctx, qtrue, qtrue);
        return;
    }

    // If we have somewhere to go, then do that
    let npc_id = ctx.entity_id_of(npc).unwrap();
    if ctx.entity(npc_id).enemy.is_none() {
        if UpdateGoal(ctx) != core::ptr::null_mut() {
            ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(ctx, qtrue);
            NPC_UpdateAngles(ctx, qtrue, qtrue);
        }
    }
}

/// Raven `ATST_Idle`.
///
/// ATST in idle state. Play idle behavior and set stand animation.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:297-303`
pub fn ATST_Idle(ctx: &mut GameContext) {
    // PORT-NOTE(ai-context): NPC accessed via ctx.world.globals
    let npc = ctx.world.globals.NPC;

    NPC_BSIdle(ctx);

    NPC_SetAnim(
        ctx,
        ctx.entity_id_of(npc).unwrap(),
        SETANIM_BOTH,
        BOTH_STAND1 as c_int,
        SETANIM_FLAG_NORMAL,
    );
}

/// Raven `NPC_BSATST_Default`.
///
/// Main behavior state machine for ATST. Choose between attack, patrol,
/// and idle based on NPC state.
/// Source: `oracle/codemp/game/NPC_AI_Atst.c:310-328`
pub fn NPC_BSATST_Default(ctx: &mut GameContext) {
    // PORT-NOTE(ai-context): NPC via the entity accessor; NPCInfo (gNPC_t) deref stays unsafe.
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // SAFETY: NPCInfo points into the module-owned gNPC pool (no aliasing accessor).
    let npc_info = unsafe { &mut *ctx.world.globals.NPCInfo };

    if ctx.entity(npc_id).enemy.is_some() {
        if (npc_info.scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            npc_info.goalEntity = ctx.entity(npc_id).enemy;
        }
        ATST_Attack(ctx);
    } else if (npc_info.scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
        ATST_Patrol(ctx);
    } else {
        ATST_Idle(ctx);
    }
}
