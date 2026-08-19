//! This is a port of `oracle/codemp/game/NPC_AI_Howler.c`.
//!
//! Functions reach file-scope game state (`level`, `g_entities`, cvars) and engine traps
//! through the threaded `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::public::set_anim::{SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE};

// Raven `#define LSTATE_*`, file-scope local state for the Howler NPC.
// The state lives in `gNPC_t::localState`.
// Source: `oracle/codemp/game/NPC_AI_Howler.c:10-11`
pub const LSTATE_CLEAR: i32 = 0;
pub const LSTATE_WAITING: i32 = 1;

// These define the working combat range for these suckers
// Source: `oracle/codemp/game/NPC_AI_Howler.c:3-7`
const MIN_DISTANCE: c_int = 54;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;
const MAX_DISTANCE: c_int = 128;

// `SETANIM_BOTH` (`SETANIM_TORSO | SETANIM_LEGS`), `BOTH_PAIN1`, and `BOTH_ATTACK1` come from the prelude (`set_anim` / `anim_number`).
// No local copies live here, so the enum values stay authoritative.
// `SETANIM_FLAG_*` comes from `mp_bg::public::set_anim`.
// Source: `oracle/codemp/game/bg_public.h:500`, `anims.h`

/// Raven `NPC_Howler_Precache`.
///
/// This precaches sounds and effects for the Howler NPC.
/// The function is empty in Raven.
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:18-20`
pub fn NPC_Howler_Precache() {
}

/// Raven `Howler_Idle`.
///
/// This is the idle behavior for the Howler NPC.
/// The function is empty in Raven.
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:28-30`
pub fn Howler_Idle() {
}

/// Raven `Howler_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:38-71`
pub fn Howler_Patrol(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: NPCInfo (gNPC_t) has no safe accessor, so the deref stays raw.
    let npc_info = ctx.world.globals.NPCInfo;

    unsafe {
        (*npc_info).localState = LSTATE_CLEAR;
    }

    //If we have somewhere to go, then do that
    if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
        ctx.world.globals.ucmd.buttons &= !BUTTON_WALKING;
        crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
    } else {
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"patrolTime".as_ptr()) != 0 {
            let delay = (ctx.world.bg_state.rng.crandom() * 5000.0 + 5000.0) as c_int;
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"patrolTime".as_ptr(), delay);
        }
    }

    //rwwFIXMEFIXME: Care about all clients, not just client 0
    let mut dif: vec3_t = [0.0; 3];
    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    crate::q_math::_VectorSubtract(
        ctx.world.g_entities[0].r.currentOrigin,
        npc_origin,
        &mut dif,
    );

    if crate::q_math::VectorLengthSquared(dif) < 256.0 * 256.0 {
        crate::NPC_combat::G_SetEnemy(ctx, npc_id, EntityId::from_num(0));
    }

    if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue) == qfalse {
        Howler_Idle();
        return;
    }
}

/// Raven `Howler_Move`.
///
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:78-86`
pub fn Howler_Move(ctx: &mut GameContext, visible: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: NPCInfo (gNPC_t) has no safe accessor, so the derefs stay raw.
    let npc_info = ctx.world.globals.NPCInfo;

    unsafe {
        if (*npc_info).localState != LSTATE_WAITING {
            (*npc_info).goalEntity = ctx.world.entity(npc_id).enemy;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
            (*npc_info).goalRadius = MAX_DISTANCE;
        }
    }
}

/// Raven `Howler_TryDamage`.
///
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:89-109`
pub fn Howler_TryDamage(ctx: &mut GameContext, enemy: Option<EntityId>, damage: c_int) {
    if enemy.is_none() {
        return;
    }

    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let mut end: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut tr: trace_t = unsafe { std::mem::zeroed() };

    // FLAG: NPC pool `gclient_t` (`gClPtrs`, g_utils.c:430) is not a `level.clients` slot.
    // The pointer is read via the entity borrow and dereffed raw, matching Raven.
    let client = ctx.world.entity(npc_id).client;
    let viewangles = unsafe { (*client).ps.viewangles };
    crate::q_math::AngleVectors(viewangles, Some(&mut dir), None, None);

    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    crate::q_math::_VectorMA(npc_origin, MIN_DISTANCE as f32, dir, &mut end);

    // Should probably trace from the mouth, but, ah well.
    let npc_number = ctx.world.entity(npc_id).s.number;
    crate::trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &npc_origin as *const vec3_t,
            &vec3_origin as *const vec3_t,
            &vec3_origin as *const vec3_t,
            &end as *const vec3_t,
            npc_number,
            MASK_SHOT,
        ),
    );

    if tr.entityNum != ENTITYNUM_WORLD as c_short {
        crate::g_combat::G_Damage(
            ctx,
            EntityId::from_num(tr.entityNum as c_int),
            Some(npc_id),
            Some(npc_id),
            Some(&mut dir),
            tr.endpos,
            damage,
            DAMAGE_NO_KNOCKBACK,
            MOD_MELEE as c_int,
        );
    }
}

/// Raven `Howler_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:112-131`
pub fn Howler_Attack(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if crate::g_timer::TIMER_Exists(ctx, Some(npc_id), c"attacking".as_ptr()) == qfalse {
        let delay = (1700.0 + (ctx.world.bg_state.rng.random() as f32 * 200.0)) as c_int;
        // Going to do ATTACK1
        crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attacking".as_ptr(), delay);
        crate::npc_c::NPC_SetAnim(
            ctx,
            npc_id,
            SETANIM_BOTH,
            BOTH_ATTACK1 as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );

        crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg".as_ptr(), 200);
    }

    // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks
    if crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"attack_dmg".as_ptr(), qtrue) != 0 {
        let enemy = ctx.world.entity(npc_id).enemy;
        Howler_TryDamage(ctx, enemy, 5);
    }

    // Just using this to remove the attacking flag at the right time
    crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"attacking".as_ptr(), qtrue);
}

/// Raven `Howler_Combat`.
///
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:134-171`
pub fn Howler_Combat(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: NPCInfo (gNPC_t) has no safe accessor, so the derefs stay raw.
    let npc_info = ctx.world.globals.NPCInfo;

    let distance: f32;
    let advance: qboolean;

    // If we cannot see our target or we have somewhere to go, then do that
    let enemy = ctx.world.entity(npc_id).enemy;
    if crate::NPC_utils::NPC_ClearLOS4(ctx, enemy) == qfalse
        || !crate::NPC_goal::UpdateGoal(ctx).is_null()
    {
        unsafe {
            (*npc_info).combatMove = qtrue;
            (*npc_info).goalEntity = ctx.world.entity(npc_id).enemy;
            (*npc_info).goalRadius = MAX_DISTANCE; // just get us within combat range
        }

        crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        return;
    }

    // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
    crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);

    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    // Raven derefs `NPC->enemy` directly.
    // The caller, `NPC_BSHowler_Default`, only enters combat with a live enemy, so this unwraps here.
    // A null enemy would be a null deref in Raven too.
    let enemy_origin = ctx.world.entity(enemy.unwrap()).r.currentOrigin;
    distance = crate::q_math::DistanceHorizontalSquared(npc_origin, enemy_origin);
    advance = (distance > MIN_DISTANCE_SQR as f32) as qboolean;

    if (advance != 0 || unsafe { (*npc_info).localState } == LSTATE_WAITING)
        && crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"attacking".as_ptr()) != 0
    {
        // waiting monsters can't attack
        if crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"takingPain".as_ptr(), qtrue) != 0 {
            unsafe {
                (*npc_info).localState = LSTATE_CLEAR;
            }
        } else {
            Howler_Move(ctx, 1 as qboolean);
        }
    } else {
        Howler_Attack(ctx);
    }
}

/// Raven `NPC_Howler_Pain`.
///
/// This runs on damage of 10 or more.
/// The function sets the pain animation, sets the waiting state, and cancels the current attack.
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:178-194`
pub fn NPC_Howler_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    // `attacker` is unused in the body, matching Raven.
    if damage >= 10 {
        crate::g_timer::TIMER_Remove(ctx, Some(self_), c"attacking".as_ptr());
        crate::g_timer::TIMER_Set(ctx, Some(self_), c"takingPain".as_ptr(), 2900);

        // FLAG: NPCInfo (gNPC_t) has no safe accessor, so the deref stays raw.
        let npc = ctx.world.entity(self_).NPC;
        if !npc.is_null() {
            let last_path_angles = unsafe { (*npc).lastPathAngles };
            crate::q_math::_VectorCopy(last_path_angles, &mut ctx.world.entity_mut(self_).s.angles);
        }

        crate::npc_c::NPC_SetAnim(
            ctx,
            self_,
            SETANIM_BOTH,
            BOTH_PAIN1 as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );

        // FLAG: NPCInfo (gNPC_t) has no safe accessor, so the deref stays raw.
        let npc = ctx.world.entity(self_).NPC;
        if !npc.is_null() {
            unsafe {
                (*npc).localState = LSTATE_WAITING;
            }
        }
    }
}

/// Raven `NPC_BSHowler_Default`.
///
/// This is the default behavior state for the Howler NPC.
/// It dispatches based on whether the Howler has an enemy target or is in patrol or idle mode.
/// Source: `oracle/codemp/game/NPC_AI_Howler.c:202-218`
pub fn NPC_BSHowler_Default(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: NPCInfo (gNPC_t) has no safe accessor, so the deref stays raw.
    let npc_info = ctx.world.globals.NPCInfo;

    if ctx.world.entity(npc_id).enemy.is_some() {
        Howler_Combat(ctx);
    } else if (unsafe { (*npc_info).scriptFlags } & SCF_LOOK_FOR_ENEMIES) != 0 {
        Howler_Patrol(ctx);
    } else {
        Howler_Idle();
    }

    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
}
