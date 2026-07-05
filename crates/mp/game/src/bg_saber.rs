// PORT-COMPLETE: bg_saber.c 11/33
//! Port of `oracle/oracle/codemp/game/bg_saber.c`.
//!
//! Originally generated as a FAITHFUL signature skeleton by
//! `tools/closure-prototype/fnskel.py` from the fnsweep manifest; bodies have
//! since been filled in.
//!
//! Nearly every function in this file reads/writes the file-static Raven
//! `pmove_t *pm` working set (`pm->ps`, `pm->cmd`, `pm->animations`, …). Per
//! ruling 12/8a, that working set is threaded as `PmoveContext` — those
//! functions are ported as `impl PmoveContext<'_>` methods below rather than
//! free functions. `saberMoveData`/`transitionMove`/`saberMoveTransitionAngle`
//! (the move-data const tables) are ported and used directly. Functions that
//! read the Raven `g_entities` global reach the entity arena/world through
//! `PmoveContext`/`GameCallbacks` (ruling 16) instead of a bare global. The
//! pure, pointer/value-parameterized functions that need none of that stay as
//! free functions below.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

use mp_bg::public::saber_move_name::{
    LS_A_BL2TR, LS_A_BR2TL, LS_A_L2R, LS_A_R2L, LS_A_T2B, LS_A_TL2BR, LS_A_TR2BL, LS_H1_BL,
    LS_H1_BR, LS_H1_B_, LS_H1_T_, LS_H1_TL, LS_H1_TR, LS_NONE, LS_PARRY_LL, LS_PARRY_LR,
    LS_PARRY_UL, LS_PARRY_UP, LS_PARRY_UR, LS_READY, LS_V1_BL, LS_V1_BR, LS_V1_B_, LS_V1__L,
    LS_V1__R, LS_V1_T_, LS_V1_TL, LS_V1_TR, LS_B1_BL, LS_B1_BR, LS_D1_BL, LS_D1_BR,
};
use mp_bg::public::saber_quadrant::saberQuadrant_t;
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::public::saberlock::SABERLOCK_WIN;
use mp_bg::local::force_power_needed::forcePowerNeeded;
use mp_qshared::shared::force_powers::{
    FP_GRIP, FP_LEVITATION, FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE,
};
use mp_bg::public::saber_move_data_table::saberMoveData;
use mp_bg::public::transition_move_table::transitionMove;
use mp_bg::public::saber_move_transition_angle_table::saberMoveTransitionAngle;
use mp_bg::public::parry_debounce_table::bg_parryDebounce;
use crate::bg_channel::PmoveContext;
use crate::bg_panimate::{BG_InSaberLock, PM_SaberBounceForAttack};
use crate::bg_panimate::{
    BG_FlippingAnim, BG_InKataAnim, BG_InRoll, BG_InSaberLockOld, BG_InSaberStandAnim,
    BG_InSpecialJump, BG_KickMove, BG_KickingAnim, BG_SaberInAttack, BG_SaberInIdle,
    BG_SaberInKata, BG_SaberInSpecial, BG_SaberInSpecialAttack, BG_SaberInTransitionAny,
    BG_SpinningSaberAnim, BG_SuperBreakLoseAnim, BG_SuperBreakWinAnim, PM_InKnockDown,
    PM_JumpingAnim, PM_SaberInKnockaway, PM_SaberInParry, PM_SaberInReflect, PM_SaberInReturn,
    PM_SaberInStart, PM_SaberInTransition,
};
use crate::q_math::{
    DistanceSquared, Q_random, AngleVectors, VectorSet, _VectorMA, _VectorScale, _VectorSubtract,
    PITCH, ROLL, YAW,
};
use crate::bg_misc::{
    BG_AddPredictableEventToPlayerstate, BG_CanUseFPNow, BG_HasYsalamiri,
};
use crate::bg_pmove::{
    BG_InKnockDown, BG_InSlopeAnim, BG_KnockDownable, BG_SabersOff, PM_RunningAnim, PM_SwimmingAnim,
    PM_WalkingAnim,
};
use mp_bg::public::saber_move_name as ls;
use mp_bg::public::anim_number::animNumber_t as A;
// Raven `saber_styles_t` variants (`SS_*`) spelled bare in the ported bodies.
use mp_qshared::common::mp::qcommon::saber::saber_styles::saber_styles_t::*;

// Per-file `#define` consts (porting-rules convention: cite the Raven #define,
// keep them local since no shared home exists yet).
/// `BACK_STAB_DISTANCE`. Source: `oracle/oracle/codemp/game/bg_saber.c:1623`
pub const BACK_STAB_DISTANCE: f32 = 128.0;
/// `FLIPHACK_DISTANCE`. Source: `oracle/oracle/codemp/game/bg_saber.c:1802`
pub const FLIPHACK_DISTANCE: f32 = 200.0;
/// `SABER_ALT_ATTACK_POWER`. Source: `oracle/oracle/codemp/game/bg_saber.c:2112`
pub const SABER_ALT_ATTACK_POWER: c_int = 50;
/// `SABER_ALT_ATTACK_POWER_LR`. Source: `oracle/oracle/codemp/game/bg_saber.c:2113`
pub const SABER_ALT_ATTACK_POWER_LR: c_int = 10;
/// `SABER_ALT_ATTACK_POWER_FB`. Source: `oracle/oracle/codemp/game/bg_saber.c:2114`
pub const SABER_ALT_ATTACK_POWER_FB: c_int = 25;
/// `DIR_*`. Source: `oracle/oracle/codemp/game/bg_public.h:220-225`
pub const DIR_RIGHT: c_int = 0;
pub const DIR_LEFT: c_int = 1;
pub const DIR_FRONT: c_int = 2;
pub const DIR_BACK: c_int = 3;
// `SFL_NO_*` saber flags live canonically in `crate::saber::saber_flags`
// (reached via the prelude glob); the duplicate local defs were removed to
// resolve the SFL_* import ambiguity (E0659). Source: `oracle/oracle/codemp/game/q_shared.h:703-712`

/// `SFL2_TRANSITION_DAMAGE`. If set, the blade does damage in start, transition and return anims (like strong style does).
/// Source: `oracle/oracle/codemp/game/q_shared.h:723`
pub const SFL2_TRANSITION_DAMAGE: c_int = 1 << 8;

/// `SFL2_TRANSITION_DAMAGE2`. If set, the blade does damage in start, transition and return anims (like strong style does).
/// Source: `oracle/oracle/codemp/game/q_shared.h:733`
pub const SFL2_TRANSITION_DAMAGE2: c_int = 1 << 17;

/// `SFL2_NO_MANUAL_DEACTIVATE`. If set, the blades cannot manually be toggled on and off.
/// Source: `oracle/oracle/codemp/game/q_shared.h:722`
pub const SFL2_NO_MANUAL_DEACTIVATE: c_int = 1 << 7;

/// `SFL2_NO_MANUAL_DEACTIVATE2`. If set, the blades cannot manually be toggled on and off.
/// Source: `oracle/oracle/codemp/game/q_shared.h:732`
pub const SFL2_NO_MANUAL_DEACTIVATE2: c_int = 1 << 16;

// Remaining `SFL2_*` primary/secondary blade-style flags.
// Source: `oracle/oracle/codemp/game/q_shared.h:715-734`
/// `SFL2_NO_WALL_MARKS`. Stops the saber from drawing marks on the world.
pub const SFL2_NO_WALL_MARKS: c_int = 1 << 0;
/// `SFL2_NO_DLIGHT`. Stops the saber from drawing a dynamic light.
pub const SFL2_NO_DLIGHT: c_int = 1 << 1;
/// `SFL2_NO_BLADE`. Stops the saber from drawing a blade.
pub const SFL2_NO_BLADE: c_int = 1 << 2;
/// `SFL2_NO_CLASH_FLARE`. The saber will not do the big, white clash flare with other sabers.
pub const SFL2_NO_CLASH_FLARE: c_int = 1 << 3;
/// `SFL2_NO_DISMEMBERMENT`. The saber never does dismemberment.
pub const SFL2_NO_DISMEMBERMENT: c_int = 1 << 4;
/// `SFL2_NO_IDLE_EFFECT`. The saber will not do damage or any effects when it is idle.
pub const SFL2_NO_IDLE_EFFECT: c_int = 1 << 5;
/// `SFL2_ALWAYS_BLOCK`. The blades will always be blocking.
pub const SFL2_ALWAYS_BLOCK: c_int = 1 << 6;
/// `SFL2_NO_WALL_MARKS2`. Secondary blade: stops the saber from drawing marks on the world.
pub const SFL2_NO_WALL_MARKS2: c_int = 1 << 9;
/// `SFL2_NO_DLIGHT2`. Secondary blade: stops the saber from drawing a dynamic light.
pub const SFL2_NO_DLIGHT2: c_int = 1 << 10;
/// `SFL2_NO_BLADE2`. Secondary blade: stops the saber from drawing a blade.
pub const SFL2_NO_BLADE2: c_int = 1 << 11;
/// `SFL2_NO_CLASH_FLARE2`. Secondary blade: no clash flare.
pub const SFL2_NO_CLASH_FLARE2: c_int = 1 << 12;
/// `SFL2_NO_DISMEMBERMENT2`. Secondary blade: never does dismemberment.
pub const SFL2_NO_DISMEMBERMENT2: c_int = 1 << 13;
/// `SFL2_NO_IDLE_EFFECT2`. Secondary blade: no idle effect.
pub const SFL2_NO_IDLE_EFFECT2: c_int = 1 << 14;
/// `SFL2_ALWAYS_BLOCK2`. Secondary blade: always blocking.
pub const SFL2_ALWAYS_BLOCK2: c_int = 1 << 15;

// Vector helpers are the canonical `crate::q_math` forms reached via the
// prelude glob: `VectorSet`/`_VectorMA`/`_VectorScale`/`_VectorSubtract`
// (out-param) and `DistanceSquared`. Return-value call sites below are rewritten
// to the out-param shape.

// Raven `qboolean` is `c_int` (`qfalse == 0`, `qtrue == 1`); the lowercase
// `qtrue`/`qfalse` spellings are not exported here (see `w_saber.rs`), so the
// ported bodies below return the bare `1`/`0` those constants alias.

/// Raven `BG_ForcePowerDrain`.
///
/// Source: `oracle/oracle/codemp/game/bg_saber.c:27-100`
pub fn BG_ForcePowerDrain(ps: *mut playerState_t, forcePower: forcePowers_t, overrideAmt: c_int) {
    unsafe {
        // take away the power
        let mut drain = overrideAmt;

        if drain == 0 {
            drain = forcePowerNeeded[(*ps).fd.forcePowerLevel[forcePower as usize] as usize]
                [forcePower as usize];
        }
        if drain == 0 {
            return;
        }

        if forcePower == FP_LEVITATION {
            // special case
            let mut jumpDrain = 0;

            if (*ps).velocity[2] > 250.0 {
                jumpDrain = 20;
            } else if (*ps).velocity[2] > 200.0 {
                jumpDrain = 16;
            } else if (*ps).velocity[2] > 150.0 {
                jumpDrain = 12;
            } else if (*ps).velocity[2] > 100.0 {
                jumpDrain = 8;
            } else if (*ps).velocity[2] > 50.0 {
                jumpDrain = 6;
            } else if (*ps).velocity[2] > 0.0 {
                jumpDrain = 4;
            }

            if jumpDrain != 0 {
                if (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] != 0 {
                    // don't divide by 0!
                    jumpDrain /= (*ps).fd.forcePowerLevel[FP_LEVITATION as usize];
                }
            }

            (*ps).fd.forcePower -= jumpDrain;
            if (*ps).fd.forcePower < 0 {
                (*ps).fd.forcePower = 0;
            }

            return;
        }

        (*ps).fd.forcePower -= drain;
        if (*ps).fd.forcePower < 0 {
            (*ps).fd.forcePower = 0;
        }
    }
}

/// Raven `PM_AttackMoveForQuad`.
///
/// Raven: maps a saber quadrant to the "instant attack" saber move that
/// starts from that quadrant.
/// Source: `oracle/oracle/codemp/game/bg_saber.c:392-420`
pub fn PM_AttackMoveForQuad(
    quad: c_int,
) -> saberMoveName_t {
    if quad == saberQuadrant_t::Q_B as c_int || quad == saberQuadrant_t::Q_BR as c_int {
        return LS_A_BR2TL;
    }
    if quad == saberQuadrant_t::Q_R as c_int {
        return LS_A_R2L;
    }
    if quad == saberQuadrant_t::Q_TR as c_int {
        return LS_A_TR2BL;
    }
    if quad == saberQuadrant_t::Q_T as c_int {
        return LS_A_T2B;
    }
    if quad == saberQuadrant_t::Q_TL as c_int {
        return LS_A_TL2BR;
    }
    if quad == saberQuadrant_t::Q_L as c_int {
        return LS_A_L2R;
    }
    if quad == saberQuadrant_t::Q_BL as c_int {
        return LS_A_BL2TR;
    }
    LS_NONE
}

/// Raven `PM_SaberMoveQuadrantForMovement`.
///
/// Raven: picks the attack quadrant implied by the player's forward/right
/// movement command.
/// Source: `oracle/oracle/codemp/game/bg_saber.c:650-697`
pub fn PM_SaberMoveQuadrantForMovement(
    ucmd: *mut usercmd_t,
) -> c_int {
    unsafe {
        let rightmove = (*ucmd).rightmove;
        let forwardmove = (*ucmd).forwardmove;
        if rightmove > 0 {
            //moving right
            if forwardmove > 0 {
                //forward right = TL2BR slash
                saberQuadrant_t::Q_TL as c_int
            } else if forwardmove < 0 {
                //backward right = BL2TR uppercut
                saberQuadrant_t::Q_BL as c_int
            } else {
                //just right is a left slice
                saberQuadrant_t::Q_L as c_int
            }
        } else if rightmove < 0 {
            //moving left
            if forwardmove > 0 {
                //forward left = TR2BL slash
                saberQuadrant_t::Q_TR as c_int
            } else if forwardmove < 0 {
                //backward left = BR2TL uppercut
                saberQuadrant_t::Q_BR as c_int
            } else {
                //just left is a right slice
                saberQuadrant_t::Q_R as c_int
            }
        } else {
            //not moving left or right
            if forwardmove > 0 {
                //forward= T2B slash
                saberQuadrant_t::Q_T as c_int
            } else if forwardmove < 0 {
                //backward= T2B slash //or B2T uppercut?
                saberQuadrant_t::Q_T as c_int
            } else {
                //Not moving at all
                saberQuadrant_t::Q_R as c_int
            }
        }
    }
}

/// Raven `PM_SaberInBounce`.
///
/// Source: `oracle/oracle/codemp/game/bg_saber.c:700-711`
pub fn PM_SaberInBounce(
    r#move: c_int,
) -> qboolean {
    if r#move >= LS_B1_BR && r#move <= LS_B1_BL {
        return 1;
    }
    if r#move >= LS_D1_BR && r#move <= LS_D1_BL {
        return 1;
    }
    0
}

/// Raven `PM_SaberAttackChainAngle`.
///
/// Source: `oracle/oracle/codemp/game/bg_saber.c:783-790`
pub fn PM_SaberAttackChainAngle(move1: c_int, move2: c_int) -> c_int {
    if move1 == -1 || move2 == -1 {
        return -1;
    }
    saberMoveTransitionAngle[saberMoveData[move1 as usize].endQuad as usize]
        [saberMoveData[move2 as usize].startQuad as usize]
}

/// Raven `PM_SetAnimFrame`.
///
/// Raven: `torso`/`legs` params are unused in the compiled body (only the
/// saber-lock frame is ever set here).
/// Source: `oracle/oracle/codemp/game/bg_saber.c:886-889`
pub fn PM_SetAnimFrame(
    gent: *mut playerState_t,
    frame: c_int,
    torso: qboolean,
    legs: qboolean,
) {
    unsafe {
        (*gent).saberLockFrame = frame;
    }
}

/// Raven `BG_CheckIncrementLockAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_saber.c:1342-1398`
pub fn BG_CheckIncrementLockAnim(
    anim: c_int,
    winOrLose: c_int,
) -> qboolean {
    let mut increment: qboolean = 0; // qfalse //???
    // RULE: if you are the first style in the lock anim, you advance from LOSING position to WINNING position
    //       if you are the second style in the lock anim, you advance from WINNING position to LOSING position

    // increment to win:
    const INCREMENT_TO_WIN: &[animNumber_t] = &[
        animNumber_t::BOTH_LK_DL_DL_S_L_1, //lock if I'm using dual vs. dual and I initiated
        animNumber_t::BOTH_LK_DL_DL_S_L_2, //lock if I'm using dual vs. dual and other initiated
        animNumber_t::BOTH_LK_DL_DL_T_L_1, //lock if I'm using dual vs. dual and I initiated
        animNumber_t::BOTH_LK_DL_DL_T_L_2, //lock if I'm using dual vs. dual and other initiated
        animNumber_t::BOTH_LK_DL_S_S_L_1,  //lock if I'm using dual vs. a single
        animNumber_t::BOTH_LK_DL_S_T_L_1,  //lock if I'm using dual vs. a single
        animNumber_t::BOTH_LK_DL_ST_S_L_1, //lock if I'm using dual vs. a staff
        animNumber_t::BOTH_LK_DL_ST_T_L_1, //lock if I'm using dual vs. a staff
        animNumber_t::BOTH_LK_S_S_S_L_1,   //lock if I'm using single vs. a single and I initiated
        animNumber_t::BOTH_LK_S_S_T_L_2, //lock if I'm using single vs. a single and other initiated
        animNumber_t::BOTH_LK_ST_S_S_L_1, //lock if I'm using staff vs. a single
        animNumber_t::BOTH_LK_ST_S_T_L_1, //lock if I'm using staff vs. a single
        animNumber_t::BOTH_LK_ST_ST_T_L_1, //lock if I'm using staff vs. a staff and I initiated
        animNumber_t::BOTH_LK_ST_ST_T_L_2, //lock if I'm using staff vs. a staff and other initiated
    ];

    // decrement to win:
    const DECREMENT_TO_WIN: &[animNumber_t] = &[
        animNumber_t::BOTH_LK_S_DL_S_L_1, //lock if I'm using single vs. a dual
        animNumber_t::BOTH_LK_S_DL_T_L_1, //lock if I'm using single vs. a dual
        animNumber_t::BOTH_LK_S_S_S_L_2, //lock if I'm using single vs. a single and other intitiated
        animNumber_t::BOTH_LK_S_S_T_L_1, //lock if I'm using single vs. a single and I initiated
        animNumber_t::BOTH_LK_S_ST_S_L_1, //lock if I'm using single vs. a staff
        animNumber_t::BOTH_LK_S_ST_T_L_1, //lock if I'm using single vs. a staff
        animNumber_t::BOTH_LK_ST_DL_S_L_1, //lock if I'm using staff vs. dual
        animNumber_t::BOTH_LK_ST_DL_T_L_1, //lock if I'm using staff vs. dual
        animNumber_t::BOTH_LK_ST_ST_S_L_1, //lock if I'm using staff vs. a staff and I initiated
        animNumber_t::BOTH_LK_ST_ST_S_L_2, //lock if I'm using staff vs. a staff and other initiated
    ];

    if INCREMENT_TO_WIN.iter().any(|&a| a as c_int == anim) {
        increment = if winOrLose == SABERLOCK_WIN { 1 } else { 0 };
    } else if DECREMENT_TO_WIN.iter().any(|&a| a as c_int == anim) {
        increment = if winOrLose == SABERLOCK_WIN { 0 } else { 1 };
    }
    increment
}

/// Raven `PM_SaberInBrokenParry`.
///
/// Source: `oracle/oracle/codemp/game/bg_saber.c:1583-1594`
pub fn PM_SaberInBrokenParry(
    r#move: c_int,
) -> qboolean {
    if r#move >= LS_V1_BR && r#move <= LS_V1_B_ {
        return 1;
    }
    if r#move >= LS_H1_T_ && r#move <= LS_H1_BL {
        return 1;
    }
    0
}

/// Raven `PM_BrokenParryForParry`.
///
/// Source: `oracle/oracle/codemp/game/bg_saber.c:1597-1621`
pub fn PM_BrokenParryForParry(
    r#move: c_int,
) -> c_int {
    if r#move == LS_PARRY_UP {
        return LS_H1_T_;
    }
    if r#move == LS_PARRY_UR {
        return LS_H1_TR;
    }
    if r#move == LS_PARRY_UL {
        return LS_H1_TL;
    }
    if r#move == LS_PARRY_LR {
        return LS_H1_BL;
    }
    if r#move == LS_PARRY_LL {
        return LS_H1_BR;
    }
    if r#move == LS_READY {
        return LS_H1_B_;
    }
    LS_NONE
}

/// Raven `PM_CheckPullAttack`.
///
/// Raven: the entire body is `#if 0`-disabled ("disabling these for MP, they
/// aren't useful") — the compiled function is unconditionally `return LS_NONE;`.
/// Source: `oracle/oracle/codemp/game/bg_saber.c:2117-2226`
pub fn PM_CheckPullAttack() -> saberMoveName_t {
    LS_NONE
}

/// Raven `BG_MySaber`.
///
/// Returns a pointer to the requested saber for a client, or NULL if the client
/// doesn't have that saber equipped.
///
/// Raven: returns a pointer to the requested saberNum.
///
/// Source: `oracle/oracle/codemp/game/bg_saber.c:4100-4141`
pub fn BG_MySaber(clientNum: c_int, saberNum: c_int, bg: &BgState) -> *mut saberInfo_t {
    unsafe {
        // Per oracle C code (QAGAME branch):
        // gentity_t *ent = &g_entities[clientNum];
        // if ( ent->inuse && ent->client ) {
        //   if ( !ent->client->saber[saberNum].model || !ent->client->saber[saberNum].model[0] )
        //       return NULL;
        //   return &ent->client->saber[saberNum];
        // }
        // return NULL;

        // PORT-NOTE(entity-access-arena): BG_MySaber needs to access g_entities[clientNum].
        // Per ruling 14, entity access is normally through PM_BGEntForNum (PmoveContext method).
        // In this free-fn context, the entity array is accessed through an arena pattern.
        // Assuming the g_entities array is accessible via a module-level or prelude mechanism,
        // we dereference it as a contiguous array of gentity_t pointers and index by clientNum.

        // Use the gentity arena accessor pattern (entities stored as pointers in an array)
        // This accesses the global g_entities arena, casting appropriately
        let entities_arena_ptr: *mut gentity_t = *(core::ptr::from_ref(&g_entities) as *const *mut gentity_t);

        if entities_arena_ptr.is_null() || clientNum < 0 {
            return core::ptr::null_mut();
        }

        let ent: *mut gentity_t = entities_arena_ptr.add(clientNum as usize);

        // Check inuse and client existence
        if (*ent).inuse == 0 || (*ent).client.is_null() {
            return core::ptr::null_mut();
        }

        // Check if the saber has a model
        if (*((*ent).client as *mut gclient_t)).saber[saberNum as usize].model.is_null()
            || (*((*ent).client as *mut gclient_t)).saber[saberNum as usize].model[0] as c_int == 0
        {
            return core::ptr::null_mut();
        }

        // Return mutable pointer to the saber
        &mut (*((*ent).client as *mut gclient_t)).saber[saberNum as usize]
    }
}

// ============================================================================
// Pass-3 real bodies: the pmove working-set functions previously skeletoned
// as free `todo!()` fns are ported here as methods on `PmoveContext` —
// matching the `bg_pmove.rs` precedent (`PM_BGEntForNum`) where the resolved
// shape for a state-touching bg fn is a method on `PmoveContext` (ruling
// 12/8a: "pmove working set -> methods on PmoveContext, reach the working set
// via self.pm/self.pml/..."). The old free-fn stubs (and the escalation
// markers they carried) have been removed now that this shape resolves them.
// `unsafe` here is the confined pmove/entity-overlay deref (porting-rules
// §D11; ruling 14).
// ============================================================================
impl PmoveContext<'_> {
    /// Raven `PM_irand_timesync`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:10-25`
    pub fn PM_irand_timesync(&mut self, val1: c_int, val2: c_int) -> c_int {
        unsafe {
            let seed = &mut (*self.pm).cmd.serverTime;
            let mut i = ((val1 - 1) as f32 + Q_random(seed) * (val2 - val1) as f32 + 1.0) as c_int;
            if i < val1 {
                i = val1;
            }
            if i > val2 {
                i = val2;
            }
            i
        }
    }

    /// Raven `BG_EnoughForcePowerForMove`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:102-111`
    pub fn BG_EnoughForcePowerForMove(&mut self, cost: c_int) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            if (*ps).fd.forcePower < cost {
                PM_AddEvent(EV_NOAMMO as c_int);
                return 0;
            }
            1
        }
    }

    /// Raven `PM_SaberAnimTransitionAnim`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:424-581`
    pub fn PM_SaberAnimTransitionAnim(&mut self, curmove: c_int, newmove: c_int) -> c_int {
        let mut retmove = newmove;
        if curmove == LS_READY {
            if matches!(
                newmove,
                x if x == LS_A_TL2BR || x == LS_A_L2R || x == LS_A_BL2TR || x == LS_A_BR2TL
                    || x == LS_A_R2L || x == LS_A_TR2BL || x == LS_A_T2B
            ) {
                retmove = LS_S_TL2BR + (newmove - LS_A_TL2BR);
            }
        } else if newmove == LS_READY {
            if matches!(
                curmove,
                x if x == LS_A_TL2BR || x == LS_A_L2R || x == LS_A_BL2TR || x == LS_A_BR2TL
                    || x == LS_A_R2L || x == LS_A_TR2BL || x == LS_A_T2B
            ) {
                retmove = LS_R_TL2BR + (newmove - LS_A_TL2BR);
            }
        } else if matches!(
            newmove,
            x if x == LS_A_TL2BR || x == LS_A_L2R || x == LS_A_BL2TR || x == LS_A_BR2TL
                || x == LS_A_R2L || x == LS_A_TR2BL || x == LS_A_T2B
        ) {
            if newmove == curmove {
                if self.PM_SaberKataDone(curmove, newmove) != 0 {
                    retmove = LS_R_TL2BR + (newmove - LS_A_TL2BR);
                } else {
                    retmove = transitionMove[saberMoveData[curmove as usize].endQuad as usize]
                        [saberMoveData[newmove as usize].startQuad as usize];
                }
            } else if saberMoveData[curmove as usize].endQuad
                == saberMoveData[newmove as usize].startQuad
            {
                retmove = newmove;
            } else {
                // PORT-NOTE(saber-move-name-coverage): the full "transitioning
                // from an attack/return/bounce/parry/reflection/knockaway"
                // curmove set from the oracle switch; transcribed as an
                // inclusive numeric range check over the same LS_* identifiers
                // the oracle lists (attacks..knockaways are numerically
                // contiguous in bg_public.h's saberMoveName_t enum).
                if (curmove >= LS_A_TL2BR && curmove <= LS_A_T2B)
                    || (curmove >= LS_D1_BR && curmove <= LS_D1_BL)
                    || (curmove >= LS_R_TL2BR && curmove <= LS_R_T2B)
                    || (curmove >= LS_PARRY_UP && curmove <= LS_PARRY_LL)
                    || (curmove >= LS_REFLECT_UP && curmove <= LS_REFLECT_LL)
                    || (curmove >= LS_K1_T_ && curmove <= LS_K1_BL)
                    || (curmove >= LS_V1_BR && curmove <= LS_V1_B_)
                    || (curmove >= LS_H1_T_ && curmove <= LS_H1_BL)
                {
                    retmove = transitionMove[saberMoveData[curmove as usize].endQuad as usize]
                        [saberMoveData[newmove as usize].startQuad as usize];
                }
            }
        }

        if retmove == LS_NONE {
            newmove
        } else {
            retmove
        }
    }

    /// Raven `PM_CheckStabDown`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:584-648`
    pub fn PM_CheckStabDown(&mut self) -> saberMoveName_t {
        unsafe {
            let ps = (*self.pm).ps;
            let saber1 = self.BG_MySaber((*ps).clientNum, 0);
            let saber2 = self.BG_MySaber((*ps).clientNum, 1);
            if !saber1.is_null() && ((*saber1).saberFlags & SFL_NO_STABDOWN) != 0 {
                return LS_NONE;
            }
            if !saber2.is_null() && ((*saber2).saberFlags & SFL_NO_STABDOWN) != 0 {
                return LS_NONE;
            }

            if (*ps).groundEntityNum == ENTITYNUM_NONE as c_int {
                return LS_NONE;
            }
            if (*ps).clientNum < MAX_CLIENTS as c_int {
                (*ps).velocity[2] = 0.0;
                (*self.pm).cmd.upmove = 0;
            }

            let mut facingAngles: vec3_t = [0.0; 3];
            VectorSet(&mut facingAngles, 0.0, (*ps).viewangles[YAW as usize], 0.0);
            let mut faceFwd = [0.0f32; 3];
            AngleVectors(facingAngles, Some(&mut faceFwd), None, None);

            let mut fwd: vec3_t = [0.0; 3];
            _VectorMA((*ps).origin, 164.0, faceFwd, &mut fwd);

            let mut tr: trace_t = core::mem::zeroed();
            let mut trmins: vec3_t = [0.0; 3];
            VectorSet(&mut trmins, -15.0, -15.0, -15.0);
            let mut trmaxs: vec3_t = [0.0; 3];
            VectorSet(&mut trmaxs, 15.0, 15.0, 15.0);
            self.traps.trace(
                &mut tr,
                &(*ps).origin,
                &trmins,
                &trmaxs,
                &fwd,
                (*ps).clientNum,
                MASK_PLAYERSOLID as c_int,
            );

            let mut ent: *mut bgEntity_t = core::ptr::null_mut();
            if tr.entityNum < ENTITYNUM_WORLD as c_int {
                ent = self.PM_BGEntForNum(tr.entityNum);
            }

            if !ent.is_null()
                && ((*ent).s.eType == ET_PLAYER as c_int || (*ent).s.eType == ET_NPC as c_int)
                && BG_InKnockDown((*ent).s.legsAnim) != 0
            {
                if (*ps).fd.saberAnimLevel == SS_DUAL as c_int {
                    return LS_STABDOWN_DUAL;
                } else if (*ps).fd.saberAnimLevel == SS_STAFF as c_int {
                    return LS_STABDOWN_STAFF;
                } else {
                    return LS_STABDOWN;
                }
            }
            LS_NONE
        }
    }

    /// Raven `PM_SaberKataDone`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:792-884`
    pub fn PM_SaberKataDone(&mut self, curmove: c_int, newmove: c_int) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            if (*ps).m_iVehicleNum != 0 {
                if (*ps).saberAttackChainCount > 0 {
                    return 1;
                }
            }

            if (*ps).fd.saberAnimLevel == SS_DESANN as c_int
                || (*ps).fd.saberAnimLevel == SS_TAVION as c_int
            {
                return 0;
            }

            if (*ps).fd.saberAnimLevel == SS_STAFF as c_int {
                return 0;
            } else if (*ps).fd.saberAnimLevel == SS_DUAL as c_int {
                return 0;
            } else if (*ps).fd.saberAnimLevel == FORCE_LEVEL_3 as c_int {
                if curmove == LS_NONE || newmove == LS_NONE {
                    if (*ps).fd.saberAnimLevel >= FORCE_LEVEL_3 as c_int
                        && (*ps).saberAttackChainCount > self.PM_irand_timesync(0, 1)
                    {
                        return 1;
                    }
                } else if (*ps).saberAttackChainCount > self.PM_irand_timesync(2, 3) {
                    return 1;
                } else if (*ps).saberAttackChainCount > 0 {
                    let chainAngle = PM_SaberAttackChainAngle(curmove, newmove);
                    if chainAngle < 135 || chainAngle > 215 {
                        return 1;
                    } else if chainAngle == 180 {
                        if (*ps).saberAttackChainCount > 1 {
                            return 1;
                        }
                    } else if (*ps).saberAttackChainCount > 2 {
                        return 1;
                    }
                }
            } else {
                if newmove == LS_A_TL2BR
                    || newmove == LS_A_L2R
                    || newmove == LS_A_BL2TR
                    || newmove == LS_A_BR2TL
                    || newmove == LS_A_R2L
                    || newmove == LS_A_TR2BL
                {
                    let chainTolerance = if (*ps).fd.saberAnimLevel == FORCE_LEVEL_1 as c_int {
                        5
                    } else {
                        3
                    };

                    if (*ps).saberAttackChainCount >= chainTolerance
                        && self.PM_irand_timesync(1, (*ps).saberAttackChainCount) > chainTolerance
                    {
                        return 1;
                    }
                }
                if (*ps).fd.saberAnimLevel == FORCE_LEVEL_2 as c_int
                    && (*ps).saberAttackChainCount > self.PM_irand_timesync(2, 5)
                {
                    return 1;
                }
            }
            0
        }
    }

    /// Raven `PM_SaberLockWinAnim`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:891-983`
    pub fn PM_SaberLockWinAnim(&mut self, victory: qboolean, superBreak: qboolean) -> c_int {
        unsafe {
            let ps = (*self.pm).ps;
            let mut winAnim: c_int = -1;
            match (*ps).torsoAnim {
                x if x == BOTH_BF2LOCK as c_int => {
                    if superBreak != 0 {
                        winAnim = BOTH_LK_S_S_T_SB_1_W as c_int;
                    } else if victory == 0 {
                        winAnim = BOTH_BF1BREAK as c_int;
                    } else {
                        (*ps).saberMove = LS_A_T2B as c_short;
                        winAnim = BOTH_A3_T__B_ as c_int;
                    }
                }
                x if x == BOTH_BF1LOCK as c_int => {
                    if superBreak != 0 {
                        winAnim = BOTH_LK_S_S_T_SB_1_W as c_int;
                    } else if victory == 0 {
                        winAnim = BOTH_KNOCKDOWN4 as c_int;
                    } else {
                        (*ps).saberMove = LS_K1_T_ as c_short;
                        winAnim = BOTH_K1_S1_T_ as c_int;
                    }
                }
                x if x == BOTH_CWCIRCLELOCK as c_int => {
                    if superBreak != 0 {
                        winAnim = BOTH_LK_S_S_S_SB_1_W as c_int;
                    } else if victory == 0 {
                        (*ps).saberMove = LS_V1_BL as c_short;
                        (*ps).saberBlocked = BLOCKED_PARRY_BROKEN as c_int;
                        winAnim = BOTH_V1_BL_S1 as c_int;
                    } else {
                        winAnim = BOTH_CWCIRCLEBREAK as c_int;
                    }
                }
                x if x == BOTH_CCWCIRCLELOCK as c_int => {
                    if superBreak != 0 {
                        winAnim = BOTH_LK_S_S_S_SB_1_W as c_int;
                    } else if victory == 0 {
                        (*ps).saberMove = LS_V1_BR as c_short;
                        (*ps).saberBlocked = BLOCKED_PARRY_BROKEN as c_int;
                        winAnim = BOTH_V1_BR_S1 as c_int;
                    } else {
                        winAnim = BOTH_CCWCIRCLEBREAK as c_int;
                    }
                }
                _ => {}
            }
            if winAnim != -1 {
                self.PM_SetAnim(
                    SETANIM_BOTH as c_int,
                    winAnim,
                    (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
                    0,
                );
                (*ps).weaponTime = (*ps).torsoTimer;
                (*ps).saberBlocked = BLOCKED_NONE as c_int;
                (*ps).weaponstate = WEAPON_FIRING;
            }
            winAnim
        }
    }

    /// Raven `PM_SaberLockLoseAnim`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1000-1126`
    pub fn PM_SaberLockLoseAnim(
        &mut self,
        genemy: *mut playerState_t,
        victory: qboolean,
        superBreak: qboolean,
    ) -> c_int {
        unsafe {
            let mut loseAnim: c_int = -1;
            match (*genemy).torsoAnim {
                x if x == BOTH_BF2LOCK as c_int => {
                    if superBreak != 0 {
                        loseAnim = BOTH_LK_S_S_T_SB_1_L as c_int;
                    } else if victory == 0 {
                        loseAnim = BOTH_BF1BREAK as c_int;
                    } else if victory == 0 {
                        (*genemy).saberMove = LS_K1_T_ as c_short;
                        loseAnim = BOTH_K1_S1_T_ as c_int;
                    } else {
                        loseAnim = BOTH_BF1BREAK as c_int;
                    }
                }
                x if x == BOTH_BF1LOCK as c_int => {
                    if superBreak != 0 {
                        loseAnim = BOTH_LK_S_S_T_SB_1_L as c_int;
                    } else if victory == 0 {
                        loseAnim = BOTH_KNOCKDOWN4 as c_int;
                    } else if victory == 0 {
                        (*genemy).saberMove = LS_A_T2B as c_short;
                        loseAnim = BOTH_A3_T__B_ as c_int;
                    } else {
                        loseAnim = BOTH_KNOCKDOWN4 as c_int;
                    }
                }
                x if x == BOTH_CWCIRCLELOCK as c_int => {
                    if superBreak != 0 {
                        loseAnim = BOTH_LK_S_S_S_SB_1_L as c_int;
                    } else if victory == 0 {
                        (*genemy).saberMove = LS_V1_BL as c_short;
                        (*genemy).saberBlocked = BLOCKED_PARRY_BROKEN as c_int;
                        loseAnim = BOTH_V1_BL_S1 as c_int;
                    } else if victory == 0 {
                        loseAnim = BOTH_CCWCIRCLEBREAK as c_int;
                    } else {
                        (*genemy).saberMove = LS_V1_BL as c_short;
                        (*genemy).saberBlocked = BLOCKED_PARRY_BROKEN as c_int;
                        loseAnim = BOTH_V1_BL_S1 as c_int;
                    }
                }
                x if x == BOTH_CCWCIRCLELOCK as c_int => {
                    if superBreak != 0 {
                        loseAnim = BOTH_LK_S_S_S_SB_1_L as c_int;
                    } else if victory == 0 {
                        (*genemy).saberMove = LS_V1_BR as c_short;
                        (*genemy).saberBlocked = BLOCKED_PARRY_BROKEN as c_int;
                        loseAnim = BOTH_V1_BR_S1 as c_int;
                    } else if victory == 0 {
                        loseAnim = BOTH_CWCIRCLEBREAK as c_int;
                    } else {
                        (*genemy).saberMove = LS_V1_BR as c_short;
                        (*genemy).saberBlocked = BLOCKED_PARRY_BROKEN as c_int;
                        loseAnim = BOTH_V1_BR_S1 as c_int;
                    }
                }
                _ => {}
            }
            if loseAnim != -1 {
                // QAGAME branch (server-side): apply on the enemy directly via
                // the GameCallbacks upcall (ruling 16), matching the C
                // `NPC_SetAnim(&g_entities[genemy->clientNum], ...)` call.
                self.callbacks.npc_set_anim(
                    (*genemy).clientNum,
                    SETANIM_BOTH as c_int,
                    loseAnim,
                    (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
                );
                (*genemy).weaponTime = (*genemy).torsoTimer;
                (*genemy).saberBlocked = BLOCKED_NONE as c_int;
                (*genemy).weaponstate = WEAPON_READY;
            }
            loseAnim
        }
    }

    /// Raven `PM_SaberLockResultAnim`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1128-1232`
    pub fn PM_SaberLockResultAnim(
        &mut self,
        duelist: *mut playerState_t,
        superBreak: qboolean,
        won: qboolean,
    ) -> c_int {
        unsafe {
            let mut baseAnim = (*duelist).torsoAnim;
            match baseAnim {
                x if x == BOTH_LK_S_S_S_L_2 as c_int => baseAnim = BOTH_LK_S_S_S_L_1 as c_int,
                x if x == BOTH_LK_S_S_T_L_2 as c_int => baseAnim = BOTH_LK_S_S_T_L_1 as c_int,
                x if x == BOTH_LK_DL_DL_S_L_2 as c_int => baseAnim = BOTH_LK_DL_DL_S_L_1 as c_int,
                x if x == BOTH_LK_DL_DL_T_L_2 as c_int => baseAnim = BOTH_LK_DL_DL_T_L_1 as c_int,
                x if x == BOTH_LK_ST_ST_S_L_2 as c_int => baseAnim = BOTH_LK_ST_ST_S_L_1 as c_int,
                x if x == BOTH_LK_ST_ST_T_L_2 as c_int => baseAnim = BOTH_LK_ST_ST_T_L_1 as c_int,
                _ => {}
            }
            // what kind of break? (Raven's `if (!superBreak) {} else if (superBreak)
            // {} else { return -1; }` — the `else` arm is dead C code since the two
            // conditions are exhaustive; transcribed faithfully.)
            if superBreak == 0 {
                baseAnim -= 2;
            } else {
                baseAnim += 1;
            }
            if won != 0 {
                baseAnim += 1;
            }

            let ps = (*self.pm).ps;
            if (*duelist).clientNum == (*ps).clientNum {
                self.PM_SetAnim(
                    SETANIM_BOTH as c_int,
                    baseAnim,
                    (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
                    0,
                );
            } else {
                self.callbacks.npc_set_anim(
                    (*duelist).clientNum,
                    SETANIM_BOTH as c_int,
                    baseAnim,
                    (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
                );
            }

            if superBreak != 0 && won == 0 {
                // QAGAME: always true server-side.
                (*duelist).saberMove = LS_NONE as c_short;
                (*duelist).torsoTimer += 250;
            }

            // QAGAME: always true server-side.
            (*duelist).weaponTime = (*duelist).torsoTimer;
            (*duelist).saberBlocked = BLOCKED_NONE as c_int;

            baseAnim
        }
    }

    /// Raven `PM_SaberLockBreak`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1234-1340`
    pub fn PM_SaberLockBreak(&mut self, genemy: *mut playerState_t, victory: qboolean, strength: c_int) {
        unsafe {
            let ps = (*self.pm).ps;
            let noKnockdown: qboolean = 0;
            let mut singleVsSingle = true;
            let superBreak: qboolean =
                if strength + (*ps).saberLockHits > self.bg.rng.Q_irand(2, 4) { 1 } else { 0 };

            let winAnim = self.PM_SaberLockWinAnim(victory, superBreak);
            if winAnim != -1 {
                self.PM_SaberLockLoseAnim(genemy, victory, superBreak);
            } else {
                singleVsSingle = false;
                self.PM_SaberLockResultAnim(ps, superBreak, 1);
                (*ps).weaponstate = WEAPON_FIRING;
                self.PM_SaberLockResultAnim(genemy, superBreak, 0);
                (*genemy).weaponstate = WEAPON_READY;
            }
            let _ = singleVsSingle;

            if victory != 0 {
                if (*ps).saberLockHits != 0 && superBreak == 0 {
                    let strength = 8;
                    let mut oppDir: vec3_t = [0.0; 3];
                    _VectorSubtract((*genemy).origin, (*ps).origin, &mut oppDir);
                    let _ = crate::q_math::VectorNormalize(&mut oppDir);

                    if noKnockdown != 0 {
                        // (dead per the oracle's own `noKnockdown` never set nonzero;
                        // transcribed faithfully.)
                    }
                    if noKnockdown == 0 && BG_KnockDownable(genemy) != 0 {
                        (*genemy).forceHandExtend = HANDEXTEND_KNOCKDOWN as c_int;
                        (*genemy).forceHandExtendTime = (*self.pm).cmd.serverTime + 1100;
                        (*genemy).forceDodgeAnim = 0;
                        (*genemy).otherKiller = (*ps).clientNum;
                        (*genemy).otherKillerTime = (*self.pm).cmd.serverTime + 5000;
                        (*genemy).otherKillerDebounceTime = (*self.pm).cmd.serverTime + 100;
                        (*genemy).velocity[0] = oppDir[0] * (strength * 40) as f32;
                        (*genemy).velocity[1] = oppDir[1] * (strength * 40) as f32;
                        (*genemy).velocity[2] = 100.0;
                    }

                    (*self.pm).checkDuelLoss = (*genemy).clientNum + 1;
                    (*ps).saberEventFlags |= SEF_LOCK_WON as c_int;
                }
            } else {
                let strength = 4;
                let mut oppDir: vec3_t = [0.0; 3];
                _VectorSubtract((*genemy).origin, (*ps).origin, &mut oppDir);
                let _ = crate::q_math::VectorNormalize(&mut oppDir);
                (*genemy).velocity[0] = oppDir[0] * (strength * 40) as f32;
                (*genemy).velocity[1] = oppDir[1] * (strength * 40) as f32;
                (*genemy).velocity[2] = 150.0;

                let mut oppDir2: vec3_t = [0.0; 3];
                _VectorSubtract((*ps).origin, (*genemy).origin, &mut oppDir2);
                let _ = crate::q_math::VectorNormalize(&mut oppDir2);
                (*ps).velocity[0] = oppDir2[0] * (strength * 40) as f32;
                (*ps).velocity[1] = oppDir2[1] * (strength * 40) as f32;
                (*ps).velocity[2] = 150.0;

                (*genemy).forceHandExtend = HANDEXTEND_WEAPONREADY as c_int;
            }

            (*ps).weaponTime = 0;
            (*genemy).weaponTime = 0;
            (*ps).saberLockTime = 0;
            (*genemy).saberLockTime = 0;
            (*ps).saberLockFrame = 0;
            (*genemy).saberLockFrame = 0;
            (*ps).saberLockEnemy = 0;
            (*genemy).saberLockEnemy = 0;

            (*ps).forceHandExtend = HANDEXTEND_WEAPONREADY as c_int;

            PM_AddEvent(EV_JUMP as c_int);
            if victory == 0 {
                BG_AddPredictableEventToPlayerstate(EV_JUMP as c_int, 0, genemy);
            } else if self.PM_irand_timesync(0, 1) != 0 {
                let parm = self.PM_irand_timesync(0, 75);
                BG_AddPredictableEventToPlayerstate(EV_JUMP as c_int, parm, genemy);
            }
        }
    }

    /// Raven `PM_SaberLocked`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1401-1581`
    pub fn PM_SaberLocked(&mut self) {
        unsafe {
            let ps = (*self.pm).ps;
            let eGenemy = self.PM_BGEntForNum((*ps).saberLockEnemy);
            if eGenemy.is_null() {
                return;
            }
            let genemy = (*eGenemy).playerState;
            if genemy.is_null() {
                return;
            }

            if (*ps).saberLockFrame != 0
                && (*genemy).saberLockFrame != 0
                && BG_InSaberLock((*ps).torsoAnim) != 0
                && BG_InSaberLock((*genemy).torsoAnim) != 0
            {
                (*ps).torsoTimer = 0;
                (*ps).weaponTime = 0;
                (*genemy).torsoTimer = 0;
                (*genemy).weaponTime = 0;

                let dist = DistanceSquared((*ps).origin, (*genemy).origin);
                if dist < 64.0 || dist > 6400.0 {
                    self.PM_SaberLockBreak(genemy, 0, 0);
                    return;
                }

                if (*ps).saberLockAdvance != 0 {
                    (*ps).saberLockAdvance = 0;

                    let anim = (*self.pm).animations.add((*ps).torsoAnim as usize);
                    let currentFrame = (*ps).saberLockFrame as f32;
                    let strength = (*ps).fd.forcePowerLevel[FP_SABER_OFFENSE as usize] + 1;

                    let mut remaining: c_int;
                    let curFrame;
                    if BG_InSaberLockOld((*ps).torsoAnim) != 0 {
                        if (*ps).torsoAnim == BOTH_CCWCIRCLELOCK as c_int
                            || (*ps).torsoAnim == BOTH_BF2LOCK as c_int
                        {
                            curFrame = currentFrame.floor() as c_int - strength;
                            if curFrame <= (*anim).firstFrame {
                                self.PM_SaberLockBreak(genemy, 1, strength);
                                return;
                            }
                            PM_SetAnimFrame(ps, curFrame, 1, 1);
                            remaining = curFrame - (*anim).firstFrame;
                        } else {
                            curFrame = currentFrame.ceil() as c_int + strength;
                            if curFrame >= (*anim).firstFrame + (*anim).numFrames {
                                self.PM_SaberLockBreak(genemy, 1, strength);
                                return;
                            }
                            PM_SetAnimFrame(ps, curFrame, 1, 1);
                            remaining = (*anim).firstFrame + (*anim).numFrames - curFrame;
                        }
                    } else if BG_CheckIncrementLockAnim((*ps).torsoAnim, SABERLOCK_WIN as c_int) != 0
                    {
                        curFrame = currentFrame.ceil() as c_int + strength;
                        if curFrame >= (*anim).firstFrame + (*anim).numFrames {
                            self.PM_SaberLockBreak(genemy, 1, strength);
                            return;
                        }
                        PM_SetAnimFrame(ps, curFrame, 1, 1);
                        remaining = (*anim).firstFrame + (*anim).numFrames - curFrame;
                    } else {
                        curFrame = currentFrame.floor() as c_int - strength;
                        if curFrame <= (*anim).firstFrame {
                            self.PM_SaberLockBreak(genemy, 1, strength);
                            return;
                        }
                        PM_SetAnimFrame(ps, curFrame, 1, 1);
                        remaining = curFrame - (*anim).firstFrame;
                    }

                    if self.PM_irand_timesync(0, 2) == 0 {
                        PM_AddEvent(EV_JUMP as c_int);
                    }

                    let anim2 = (*self.pm).animations.add((*genemy).torsoAnim as usize);
                    if BG_InSaberLockOld((*genemy).torsoAnim) != 0 {
                        if (*genemy).torsoAnim == BOTH_CWCIRCLELOCK as c_int
                            || (*genemy).torsoAnim == BOTH_BF1LOCK as c_int
                        {
                            if self.PM_irand_timesync(0, 2) == 0 {
                                BG_AddPredictableEventToPlayerstate(
                                    EV_PAIN as c_int,
                                    (80.0f32 / 100.0 * 100.0).floor() as c_int,
                                    genemy,
                                );
                            }
                            PM_SetAnimFrame(genemy, (*anim2).firstFrame + remaining, 1, 1);
                        } else {
                            PM_SetAnimFrame(
                                genemy,
                                (*anim2).firstFrame + (*anim2).numFrames - remaining,
                                1,
                                1,
                            );
                        }
                    } else if BG_CheckIncrementLockAnim((*genemy).torsoAnim, SABERLOCK_LOSE as c_int)
                        != 0
                    {
                        if self.PM_irand_timesync(0, 2) == 0 {
                            BG_AddPredictableEventToPlayerstate(
                                EV_PAIN as c_int,
                                (80.0f32 / 100.0 * 100.0).floor() as c_int,
                                genemy,
                            );
                        }
                        PM_SetAnimFrame(
                            genemy,
                            (*anim2).firstFrame + (*anim2).numFrames - remaining,
                            1,
                            1,
                        );
                    } else {
                        PM_SetAnimFrame(genemy, (*anim2).firstFrame + remaining, 1, 1);
                    }
                }
            } else {
                self.PM_SaberLockBreak(genemy, 0, 0);
            }
        }
    }

    /// Raven `PM_CanBackstab`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1625-1655`
    pub fn PM_CanBackstab(&mut self) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            let mut flatAng = (*ps).viewangles;
            flatAng[PITCH as usize] = 0.0;
            let mut fwd = [0.0f32; 3];
            AngleVectors(flatAng, Some(&mut fwd), None, None);

            let back = [
                (*ps).origin[0] - fwd[0] * BACK_STAB_DISTANCE,
                (*ps).origin[1] - fwd[1] * BACK_STAB_DISTANCE,
                (*ps).origin[2] - fwd[2] * BACK_STAB_DISTANCE,
            ];

            let mut tr: trace_t = core::mem::zeroed();
            let mut trmins: vec3_t = [0.0; 3];
            VectorSet(&mut trmins, -15.0, -15.0, -8.0);
            let mut trmaxs: vec3_t = [0.0; 3];
            VectorSet(&mut trmaxs, 15.0, 15.0, 8.0);
            self.traps.trace(
                &mut tr,
                &(*ps).origin,
                &trmins,
                &trmaxs,
                &back,
                (*ps).clientNum,
                MASK_PLAYERSOLID as c_int,
            );

            if tr.fraction != 1.0 && tr.entityNum >= 0 && tr.entityNum < ENTITYNUM_NONE as c_int {
                let bgEnt = self.PM_BGEntForNum(tr.entityNum);
                if !bgEnt.is_null()
                    && ((*bgEnt).s.eType == ET_PLAYER as c_int || (*bgEnt).s.eType == ET_NPC as c_int)
                {
                    return 1;
                }
            }
            0
        }
    }

    /// Raven `PM_SaberFlipOverAttackMove`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1657-1753`
    pub fn PM_SaberFlipOverAttackMove(&mut self) -> saberMoveName_t {
        unsafe {
            let ps = (*self.pm).ps;
            let saber1 = self.BG_MySaber((*ps).clientNum, 0);
            let saber2 = self.BG_MySaber((*ps).clientNum, 1);
            if !saber1.is_null() && (*saber1).jumpAtkFwdMove != LS_INVALID {
                if (*saber1).jumpAtkFwdMove != LS_NONE {
                    return (*saber1).jumpAtkFwdMove;
                }
            }
            if !saber2.is_null() && (*saber2).jumpAtkFwdMove != LS_INVALID {
                if (*saber2).jumpAtkFwdMove != LS_NONE {
                    return (*saber2).jumpAtkFwdMove;
                }
            }
            if !saber1.is_null() && (*saber1).jumpAtkFwdMove == LS_NONE {
                return LS_A_T2B;
            }
            if !saber2.is_null() && (*saber2).jumpAtkFwdMove == LS_NONE {
                return LS_A_T2B;
            }

            let mut fwdAngles = (*ps).viewangles;
            fwdAngles[PITCH as usize] = 0.0;
            fwdAngles[ROLL as usize] = 0.0;
            let mut jumpFwd = [0.0f32; 3];
            AngleVectors(fwdAngles, Some(&mut jumpFwd), None, None);
            _VectorScale(jumpFwd, 150.0, &mut (*ps).velocity);
            (*ps).velocity[2] = 400.0;

            PM_SetForceJumpZStart((*ps).origin[2]);
            PM_AddEvent(EV_JUMP as c_int);
            (*ps).fd.forceJumpSound = 1;
            (*self.pm).cmd.upmove = 0;

            LS_A_FLIP_SLASH
        }
    }

    /// Raven `PM_SaberBackflipAttackMove`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1755-1791`
    pub fn PM_SaberBackflipAttackMove(&mut self) -> c_int {
        unsafe {
            let ps = (*self.pm).ps;
            let saber1 = self.BG_MySaber((*ps).clientNum, 0);
            let saber2 = self.BG_MySaber((*ps).clientNum, 1);
            if !saber1.is_null() && (*saber1).jumpAtkBackMove != LS_INVALID {
                if (*saber1).jumpAtkBackMove != LS_NONE {
                    return (*saber1).jumpAtkBackMove;
                }
            }
            if !saber2.is_null() && (*saber2).jumpAtkBackMove != LS_INVALID {
                if (*saber2).jumpAtkBackMove != LS_NONE {
                    return (*saber2).jumpAtkBackMove;
                }
            }
            if !saber1.is_null() && (*saber1).jumpAtkBackMove == LS_NONE {
                return LS_A_T2B;
            }
            if !saber2.is_null() && (*saber2).jumpAtkBackMove == LS_NONE {
                return LS_A_T2B;
            }
            (*self.pm).cmd.upmove = 127;
            (*ps).velocity[2] = 500.0;
            LS_A_BACKFLIP_ATK
        }
    }

    /// Raven `PM_SaberDualJumpAttackMove`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1793-1800`
    pub fn PM_SaberDualJumpAttackMove(&mut self) -> c_int {
        unsafe {
            (*self.pm).cmd.upmove = 0;
        }
        LS_JUMPATTACK_DUAL
    }

    /// Raven `PM_SomeoneInFront`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1804-1833`
    pub fn PM_SomeoneInFront(&mut self, tr: *mut trace_t) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            let mut flatAng = (*ps).viewangles;
            flatAng[PITCH as usize] = 0.0;
            let mut fwd = [0.0f32; 3];
            AngleVectors(flatAng, Some(&mut fwd), None, None);

            let back = [
                (*ps).origin[0] + fwd[0] * FLIPHACK_DISTANCE,
                (*ps).origin[1] + fwd[1] * FLIPHACK_DISTANCE,
                (*ps).origin[2] + fwd[2] * FLIPHACK_DISTANCE,
            ];

            let mut trmins: vec3_t = [0.0; 3];
            VectorSet(&mut trmins, -15.0, -15.0, -8.0);
            let mut trmaxs: vec3_t = [0.0; 3];
            VectorSet(&mut trmaxs, 15.0, 15.0, 8.0);
            self.traps.trace(
                tr,
                &(*ps).origin,
                &trmins,
                &trmaxs,
                &back,
                (*ps).clientNum,
                MASK_PLAYERSOLID as c_int,
            );

            if (*tr).fraction != 1.0 && (*tr).entityNum >= 0 && (*tr).entityNum < ENTITYNUM_NONE as c_int
            {
                let bgEnt = self.PM_BGEntForNum((*tr).entityNum);
                if !bgEnt.is_null()
                    && ((*bgEnt).s.eType == ET_PLAYER as c_int || (*bgEnt).s.eType == ET_NPC as c_int)
                {
                    return 1;
                }
            }
            0
        }
    }

    /// Raven `PM_SaberLungeAttackMove`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1835-1889`
    pub fn PM_SaberLungeAttackMove(&mut self, noSpecials: qboolean) -> saberMoveName_t {
        unsafe {
            let ps = (*self.pm).ps;
            let saber1 = self.BG_MySaber((*ps).clientNum, 0);
            let saber2 = self.BG_MySaber((*ps).clientNum, 1);
            if !saber1.is_null() && (*saber1).lungeAtkMove != LS_INVALID {
                if (*saber1).lungeAtkMove != LS_NONE {
                    return (*saber1).lungeAtkMove;
                }
            }
            if !saber2.is_null() && (*saber2).lungeAtkMove != LS_INVALID {
                if (*saber2).lungeAtkMove != LS_NONE {
                    return (*saber2).lungeAtkMove;
                }
            }
            if !saber1.is_null() && (*saber1).lungeAtkMove == LS_NONE {
                return LS_A_T2B;
            }
            if !saber2.is_null() && (*saber2).lungeAtkMove == LS_NONE {
                return LS_A_T2B;
            }

            if (*ps).fd.saberAnimLevel == SS_FAST as c_int {
                let mut fwdAngles = (*ps).viewangles;
                fwdAngles[PITCH as usize] = 0.0;
                fwdAngles[ROLL as usize] = 0.0;
                let mut jumpFwd = [0.0f32; 3];
                AngleVectors(fwdAngles, Some(&mut jumpFwd), None, None);
                _VectorScale(jumpFwd, 150.0, &mut (*ps).velocity);
                PM_AddEvent(EV_JUMP as c_int);
                return LS_A_LUNGE;
            } else if noSpecials == 0 && (*ps).fd.saberAnimLevel == SS_STAFF as c_int {
                return LS_SPINATTACK;
            } else if noSpecials == 0 {
                return LS_SPINATTACK_DUAL;
            }
            LS_A_T2B
        }
    }

    /// Raven `PM_SaberJumpAttackMove2`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1891-1945`
    pub fn PM_SaberJumpAttackMove2(&mut self) -> saberMoveName_t {
        unsafe {
            let ps = (*self.pm).ps;
            let saber1 = self.BG_MySaber((*ps).clientNum, 0);
            let saber2 = self.BG_MySaber((*ps).clientNum, 1);
            if !saber1.is_null() && (*saber1).jumpAtkFwdMove != LS_INVALID {
                if (*saber1).jumpAtkFwdMove != LS_NONE {
                    return (*saber1).jumpAtkFwdMove;
                }
            }
            if !saber2.is_null() && (*saber2).jumpAtkFwdMove != LS_INVALID {
                if (*saber2).jumpAtkFwdMove != LS_NONE {
                    return (*saber2).jumpAtkFwdMove;
                }
            }
            if !saber1.is_null() && (*saber1).jumpAtkFwdMove == LS_NONE {
                return LS_A_T2B;
            }
            if !saber2.is_null() && (*saber2).jumpAtkFwdMove == LS_NONE {
                return LS_A_T2B;
            }

            if (*ps).fd.saberAnimLevel == SS_DUAL as c_int {
                self.PM_SaberDualJumpAttackMove()
            } else {
                LS_JUMPATTACK_STAFF_RIGHT
            }
        }
    }

    /// Raven `PM_SaberJumpAttackMove`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1947-1993`
    pub fn PM_SaberJumpAttackMove(&mut self) -> saberMoveName_t {
        unsafe {
            let ps = (*self.pm).ps;
            let saber1 = self.BG_MySaber((*ps).clientNum, 0);
            let saber2 = self.BG_MySaber((*ps).clientNum, 1);
            if !saber1.is_null() && (*saber1).jumpAtkFwdMove != LS_INVALID {
                if (*saber1).jumpAtkFwdMove != LS_NONE {
                    return (*saber1).jumpAtkFwdMove;
                }
            }
            if !saber2.is_null() && (*saber2).jumpAtkFwdMove != LS_INVALID {
                if (*saber2).jumpAtkFwdMove != LS_NONE {
                    return (*saber2).jumpAtkFwdMove;
                }
            }
            if !saber1.is_null() && (*saber1).jumpAtkFwdMove == LS_NONE {
                return LS_A_T2B;
            }
            if !saber2.is_null() && (*saber2).jumpAtkFwdMove == LS_NONE {
                return LS_A_T2B;
            }

            let mut fwdAngles = (*ps).viewangles;
            fwdAngles[PITCH as usize] = 0.0;
            fwdAngles[ROLL as usize] = 0.0;
            let mut jumpFwd = [0.0f32; 3];
            AngleVectors(fwdAngles, Some(&mut jumpFwd), None, None);
            _VectorScale(jumpFwd, 300.0, &mut (*ps).velocity);
            (*ps).velocity[2] = 280.0;
            PM_SetForceJumpZStart((*ps).origin[2]);

            PM_AddEvent(EV_JUMP as c_int);
            (*ps).fd.forceJumpSound = 1;
            (*self.pm).cmd.upmove = 0;

            LS_A_JUMP_T__B_
        }
    }

    /// Raven `PM_GroundDistance`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:1995-2009`
    pub fn PM_GroundDistance(&mut self) -> f32 {
        unsafe {
            let ps = (*self.pm).ps;
            let mut down = (*ps).origin;
            down[2] -= 4096.0;

            let mut tr: trace_t = core::mem::zeroed();
            self.traps.trace(
                &mut tr,
                &(*ps).origin,
                &(*self.pm).mins,
                &(*self.pm).maxs,
                &down,
                (*ps).clientNum,
                MASK_SOLID as c_int,
            );

            let mut d: vec3_t = [0.0; 3];
            _VectorSubtract((*ps).origin, tr.endpos, &mut d);
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        }
    }

    /// Raven `PM_WalkableGroundDistance`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2011-2030`
    pub fn PM_WalkableGroundDistance(&mut self) -> f32 {
        unsafe {
            let ps = (*self.pm).ps;
            let mut down = (*ps).origin;
            down[2] -= 4096.0;

            let mut tr: trace_t = core::mem::zeroed();
            self.traps.trace(
                &mut tr,
                &(*ps).origin,
                &(*self.pm).mins,
                &(*self.pm).maxs,
                &down,
                (*ps).clientNum,
                MASK_SOLID as c_int,
            );

            if tr.plane.normal[2] < MIN_WALK_NORMAL {
                return 4096.0;
            }

            let mut d: vec3_t = [0.0; 3];
            _VectorSubtract((*ps).origin, tr.endpos, &mut d);
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        }
    }

    /// Raven `PM_CanDoDualDoubleAttacks`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2033-2056`
    pub fn PM_CanDoDualDoubleAttacks(&mut self) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            if (*ps).weapon == WP_SABER as c_int {
                let saber = self.BG_MySaber((*ps).clientNum, 0);
                if !saber.is_null() && ((*saber).saberFlags & SFL_NO_MIRROR_ATTACKS) != 0 {
                    return 0;
                }
                let saber = self.BG_MySaber((*ps).clientNum, 1);
                if !saber.is_null() && ((*saber).saberFlags & SFL_NO_MIRROR_ATTACKS) != 0 {
                    return 0;
                }
            }
            if BG_SaberInSpecialAttack((*ps).torsoAnim) != 0
                || BG_SaberInSpecialAttack((*ps).legsAnim) != 0
            {
                return 0;
            }
            1
        }
    }

    /// Raven `PM_CheckEnemyPresence`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2058-2110`
    pub fn PM_CheckEnemyPresence(&mut self, dir: c_int, radius: f32) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            let tSize = 12.0f32;
            let mut tMins: vec3_t = [0.0; 3];
            VectorSet(&mut tMins, -tSize, -tSize, -tSize);
            let mut tMaxs: vec3_t = [0.0; 3];
            VectorSet(&mut tMaxs, tSize, tSize, tSize);

            let mut angles = (*ps).viewangles;
            angles[PITCH as usize] = 0.0;

            let mut checkDir = [0.0f32; 3];
            match dir {
                x if x == DIR_RIGHT => {
                    let mut right = [0.0f32; 3];
                    AngleVectors(angles, None, Some(&mut right), None);
                    checkDir = right;
                }
                x if x == DIR_LEFT => {
                    let mut right = [0.0f32; 3];
                    AngleVectors(angles, None, Some(&mut right), None);
                    _VectorScale(right, -1.0, &mut checkDir);
                }
                x if x == DIR_FRONT => {
                    let mut fwd = [0.0f32; 3];
                    AngleVectors(angles, Some(&mut fwd), None, None);
                    checkDir = fwd;
                }
                x if x == DIR_BACK => {
                    let mut fwd = [0.0f32; 3];
                    AngleVectors(angles, Some(&mut fwd), None, None);
                    _VectorScale(fwd, -1.0, &mut checkDir);
                }
                _ => {}
            }

            let mut tTo: vec3_t = [0.0; 3];
            _VectorMA((*ps).origin, radius, checkDir, &mut tTo);
            let mut tr: trace_t = core::mem::zeroed();
            self.traps.trace(
                &mut tr,
                &(*ps).origin,
                &tMins,
                &tMaxs,
                &tTo,
                (*ps).clientNum,
                MASK_PLAYERSOLID as c_int,
            );

            if tr.fraction != 1.0 && tr.entityNum < ENTITYNUM_WORLD as c_int {
                let bgEnt = self.PM_BGEntForNum(tr.entityNum);
                if !bgEnt.is_null()
                    && ((*bgEnt).s.eType == ET_PLAYER as c_int || (*bgEnt).s.eType == ET_NPC as c_int)
                {
                    return 1;
                }
            }
            0
        }
    }

    /// Raven `PM_InSecondaryStyle`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2228-2239`
    pub fn PM_InSecondaryStyle(&mut self) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            if (*ps).fd.saberAnimLevelBase == SS_STAFF as c_int
                || (*ps).fd.saberAnimLevelBase == SS_DUAL as c_int
            {
                if (*ps).fd.saberAnimLevel != (*ps).fd.saberAnimLevelBase {
                    return 1;
                }
            }
            0
        }
    }

    /// Raven `PM_SaberAttackForMovement`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2241-2618`
    pub fn PM_SaberAttackForMovement(&mut self, curmove: saberMoveName_t) -> saberMoveName_t {
        unsafe {
            let mut newmove = LS_NONE;
            let noSpecials = self.PM_InSecondaryStyle();
            let mut allowCartwheels = true;
            let mut overrideJumpRightAttackMove = LS_INVALID;
            let mut overrideJumpLeftAttackMove = LS_INVALID;

            let ps = (*self.pm).ps;
            if (*ps).weapon == WP_SABER as c_int {
                let saber1 = self.BG_MySaber((*ps).clientNum, 0);
                let saber2 = self.BG_MySaber((*ps).clientNum, 1);

                if !saber1.is_null() && (*saber1).jumpAtkRightMove != LS_INVALID {
                    if (*saber1).jumpAtkRightMove != LS_NONE {
                        overrideJumpRightAttackMove = (*saber1).jumpAtkRightMove;
                    } else if !saber2.is_null() && (*saber2).jumpAtkRightMove > LS_NONE {
                        overrideJumpRightAttackMove = (*saber2).jumpAtkRightMove;
                    } else {
                        overrideJumpRightAttackMove = LS_NONE;
                    }
                } else if !saber2.is_null() && (*saber2).jumpAtkRightMove != LS_INVALID {
                    overrideJumpRightAttackMove = (*saber2).jumpAtkRightMove;
                }

                if !saber1.is_null() && (*saber1).jumpAtkLeftMove != LS_INVALID {
                    if (*saber1).jumpAtkLeftMove != LS_NONE {
                        overrideJumpLeftAttackMove = (*saber1).jumpAtkLeftMove;
                    } else if !saber2.is_null() && (*saber2).jumpAtkLeftMove > LS_NONE {
                        overrideJumpLeftAttackMove = (*saber2).jumpAtkLeftMove;
                    } else {
                        overrideJumpLeftAttackMove = LS_NONE;
                    }
                } else if !saber2.is_null() && (*saber2).jumpAtkLeftMove != LS_INVALID {
                    // PORT-NOTE(faithful-oracle-bug): the oracle reads `saber1`
                    // here (bg_saber.c:2297), not `saber2`, in this branch — a
                    // likely upstream copy-paste bug. Preserved faithfully.
                    overrideJumpLeftAttackMove = (*saber1).jumpAtkLeftMove;
                }

                if !saber1.is_null() && ((*saber1).saberFlags & SFL_NO_CARTWHEELS) != 0 {
                    allowCartwheels = false;
                }
                if !saber2.is_null() && ((*saber2).saberFlags & SFL_NO_CARTWHEELS) != 0 {
                    allowCartwheels = false;
                }
            }

            let cmd = &mut (*self.pm).cmd;
            if cmd.rightmove > 0 {
                if noSpecials == 0
                    && overrideJumpRightAttackMove != LS_NONE
                    && (*ps).velocity[2] > 20.0
                    && (cmd.buttons & BUTTON_ATTACK as c_int) != 0
                    && self.PM_GroundDistance() < 70.0
                    && (cmd.upmove > 0 || ((*ps).pm_flags & PMF_JUMP_HELD as c_int) != 0)
                    && self.BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_LR) != 0
                {
                    BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_LR);
                    if overrideJumpRightAttackMove != LS_INVALID {
                        return overrideJumpRightAttackMove;
                    } else {
                        let mut fwdAngles: vec3_t = [0.0; 3];
                        VectorSet(&mut fwdAngles, 0.0, (*ps).viewangles[YAW as usize], 0.0);
                        let mut right = [0.0f32; 3];
                        AngleVectors(fwdAngles, None, Some(&mut right), None);
                        (*ps).velocity[0] = 0.0;
                        (*ps).velocity[1] = 0.0;
                        _VectorMA((*ps).velocity, 190.0, right, &mut (*ps).velocity);
                        if (*ps).fd.saberAnimLevel == SS_STAFF as c_int {
                            newmove = LS_BUTTERFLY_RIGHT;
                            (*ps).velocity[2] = 350.0;
                        } else if allowCartwheels {
                            PM_AddEvent(EV_JUMP as c_int);
                            (*ps).velocity[2] = 300.0;
                            newmove = LS_JUMPATTACK_ARIAL_RIGHT;
                        }
                    }
                } else if cmd.forwardmove > 0 {
                    newmove = LS_A_TL2BR;
                } else if cmd.forwardmove < 0 {
                    newmove = LS_A_BL2TR;
                } else {
                    newmove = LS_A_L2R;
                }
            } else if cmd.rightmove < 0 {
                if noSpecials == 0
                    && overrideJumpLeftAttackMove != LS_NONE
                    && (*ps).velocity[2] > 20.0
                    && (cmd.buttons & BUTTON_ATTACK as c_int) != 0
                    && self.PM_GroundDistance() < 70.0
                    && (cmd.upmove > 0 || ((*ps).pm_flags & PMF_JUMP_HELD as c_int) != 0)
                    && self.BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_LR) != 0
                {
                    BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_LR);
                    if overrideJumpLeftAttackMove != LS_INVALID {
                        return overrideJumpLeftAttackMove;
                    } else {
                        let mut fwdAngles: vec3_t = [0.0; 3];
                        VectorSet(&mut fwdAngles, 0.0, (*ps).viewangles[YAW as usize], 0.0);
                        let mut right = [0.0f32; 3];
                        AngleVectors(fwdAngles, None, Some(&mut right), None);
                        (*ps).velocity[0] = 0.0;
                        (*ps).velocity[1] = 0.0;
                        _VectorMA((*ps).velocity, -190.0, right, &mut (*ps).velocity);
                        if (*ps).fd.saberAnimLevel == SS_STAFF as c_int {
                            newmove = LS_BUTTERFLY_LEFT;
                            (*ps).velocity[2] = 250.0;
                        } else if allowCartwheels {
                            PM_AddEvent(EV_JUMP as c_int);
                            (*ps).velocity[2] = 350.0;
                            newmove = LS_JUMPATTACK_ARIAL_LEFT;
                        }
                    }
                } else if cmd.forwardmove > 0 {
                    newmove = LS_A_TR2BL;
                } else if cmd.forwardmove < 0 {
                    newmove = LS_A_BR2TL;
                } else {
                    newmove = LS_A_R2L;
                }
            } else if cmd.forwardmove > 0 {
                if noSpecials == 0
                    && ((*ps).fd.saberAnimLevel == SS_DUAL as c_int
                        || (*ps).fd.saberAnimLevel == SS_STAFF as c_int)
                    && (*ps).fd.forceRageRecoveryTime < cmd.serverTime
                    && ((*ps).groundEntityNum != ENTITYNUM_NONE as c_int
                        || self.PM_GroundDistance() <= 40.0)
                    && (*ps).velocity[2] >= 0.0
                    && (cmd.upmove > 0 || ((*ps).pm_flags & PMF_JUMP_HELD as c_int) != 0)
                    && BG_SaberInTransitionAny((*ps).saberMove as c_int) == 0
                    && BG_SaberInAttack((*ps).saberMove as c_int) == 0
                    && (*ps).weaponTime <= 0
                    && (*ps).forceHandExtend == HANDEXTEND_NONE as c_int
                    && (cmd.buttons & BUTTON_ATTACK as c_int) != 0
                    && self.BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != 0
                {
                    newmove = self.PM_SaberJumpAttackMove2();
                    if newmove != LS_A_T2B && newmove != LS_NONE {
                        BG_ForcePowerDrain((*self.pm).ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                    }
                } else if noSpecials == 0
                    && (*ps).fd.saberAnimLevel == SS_MEDIUM as c_int
                    && (*ps).velocity[2] > 100.0
                    && self.PM_GroundDistance() < 32.0
                    && BG_InSpecialJump((*ps).legsAnim) == 0
                    && BG_SaberInSpecialAttack((*ps).torsoAnim) == 0
                    && self.BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != 0
                {
                    newmove = self.PM_SaberFlipOverAttackMove();
                    if newmove != LS_A_T2B && newmove != LS_NONE {
                        BG_ForcePowerDrain((*self.pm).ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                    }
                } else if noSpecials == 0
                    && (*ps).fd.saberAnimLevel == SS_STRONG as c_int
                    && (*ps).velocity[2] > 100.0
                    && self.PM_GroundDistance() < 32.0
                    && BG_InSpecialJump((*ps).legsAnim) == 0
                    && BG_SaberInSpecialAttack((*ps).torsoAnim) == 0
                    && self.BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != 0
                {
                    newmove = self.PM_SaberJumpAttackMove();
                    if newmove != LS_A_T2B && newmove != LS_NONE {
                        BG_ForcePowerDrain((*self.pm).ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                    }
                } else if ((*ps).fd.saberAnimLevel == SS_FAST as c_int
                    || (*ps).fd.saberAnimLevel == SS_DUAL as c_int
                    || (*ps).fd.saberAnimLevel == SS_STAFF as c_int)
                    && (*ps).groundEntityNum != ENTITYNUM_NONE as c_int
                    && ((*ps).pm_flags & PMF_DUCKED as c_int) != 0
                    && (*ps).weaponTime <= 0
                    && BG_SaberInSpecialAttack((*ps).torsoAnim) == 0
                    && self.BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != 0
                {
                    newmove = self.PM_SaberLungeAttackMove(noSpecials);
                    if newmove != LS_A_T2B && newmove != LS_NONE {
                        BG_ForcePowerDrain((*self.pm).ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                    }
                } else if noSpecials == 0 {
                    let stabDownMove = self.PM_CheckStabDown();
                    if stabDownMove != LS_NONE
                        && self.BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != 0
                    {
                        newmove = stabDownMove;
                        BG_ForcePowerDrain((*self.pm).ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                    } else {
                        newmove = LS_A_T2B;
                    }
                }
            } else if cmd.forwardmove < 0 {
                if noSpecials == 0
                    && (*ps).fd.saberAnimLevel == SS_STAFF as c_int
                    && (*ps).fd.forceRageRecoveryTime < cmd.serverTime
                    && (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_1 as c_int
                    && ((*ps).groundEntityNum != ENTITYNUM_NONE as c_int
                        || self.PM_GroundDistance() <= 40.0)
                    && (*ps).velocity[2] >= 0.0
                    && (cmd.upmove > 0 || ((*ps).pm_flags & PMF_JUMP_HELD as c_int) != 0)
                    && BG_SaberInTransitionAny((*ps).saberMove as c_int) == 0
                    && BG_SaberInAttack((*ps).saberMove as c_int) == 0
                    && (*ps).weaponTime <= 0
                    && (*ps).forceHandExtend == HANDEXTEND_NONE as c_int
                    && (cmd.buttons & BUTTON_ATTACK as c_int) != 0
                {
                    newmove = self.PM_SaberBackflipAttackMove();
                } else if self.PM_CanBackstab() != 0 && BG_SaberInSpecialAttack((*ps).torsoAnim) == 0 {
                    if (*ps).fd.saberAnimLevel >= FORCE_LEVEL_2 as c_int
                        && (*ps).fd.saberAnimLevel != SS_STAFF as c_int
                    {
                        if ((*ps).pm_flags & PMF_DUCKED as c_int) != 0 || cmd.upmove < 0 {
                            newmove = LS_A_BACK_CR;
                        } else {
                            newmove = LS_A_BACK;
                        }
                    } else {
                        newmove = LS_A_BACKSTAB;
                    }
                } else {
                    newmove = LS_A_T2B;
                }
            } else if PM_SaberInBounce(curmove) != 0 {
                newmove = saberMoveData[curmove as usize].chain_attack;
                if self.PM_SaberKataDone(curmove, newmove) != 0 {
                    newmove = saberMoveData[curmove as usize].chain_idle;
                } else {
                    newmove = saberMoveData[curmove as usize].chain_attack;
                }
            } else if curmove == LS_READY {
                newmove = LS_A_T2B;
            }

            let ps = (*self.pm).ps;
            let cmd = &mut (*self.pm).cmd;
            if (*ps).fd.saberAnimLevel == SS_DUAL as c_int {
                if (newmove == LS_A_R2L
                    || newmove == LS_S_R2L
                    || newmove == LS_A_L2R
                    || newmove == LS_S_L2R)
                    && self.PM_CanDoDualDoubleAttacks() != 0
                    && self.PM_CheckEnemyPresence(DIR_RIGHT, 100.0) != 0
                    && self.PM_CheckEnemyPresence(DIR_LEFT, 100.0) != 0
                {
                    newmove = LS_DUAL_LR;
                    (*self.pm).cmd.rightmove = 0;
                } else if (newmove == LS_A_T2B
                    || newmove == LS_S_T2B
                    || newmove == LS_A_BACK
                    || newmove == LS_A_BACK_CR)
                    && self.PM_CanDoDualDoubleAttacks() != 0
                    && self.PM_CheckEnemyPresence(DIR_FRONT, 100.0) != 0
                    && self.PM_CheckEnemyPresence(DIR_BACK, 100.0) != 0
                {
                    newmove = LS_DUAL_FB;
                    (*self.pm).cmd.forwardmove = 0;
                }
            }
            let _ = cmd;

            newmove
        }
    }

    /// Raven `PM_KickMoveForConditions`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2620-2691`
    pub fn PM_KickMoveForConditions(&mut self) -> c_int {
        unsafe {
            let mut kickMove: c_int = -1;
            let cmd = &mut (*self.pm).cmd;
            if cmd.rightmove != 0 {
                kickMove = if cmd.rightmove > 0 { LS_KICK_R } else { LS_KICK_L };
                cmd.rightmove = 0;
            } else if cmd.forwardmove != 0 {
                kickMove = if cmd.forwardmove > 0 { LS_KICK_F } else { LS_KICK_B };
                cmd.forwardmove = 0;
            }
            // The "fancy kicks" `else` branch is `if (0)`-disabled in the
            // oracle — dead code, faithfully dropped.
            kickMove
        }
    }

    /// Raven `PM_SaberMoveOkayForKata`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2696-2707`
    pub fn PM_SaberMoveOkayForKata(&mut self) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            if (*ps).saberMove as c_int == LS_READY || PM_SaberInStart((*ps).saberMove as c_int) != 0 {
                1
            } else {
                0
            }
        }
    }

    /// Raven `PM_CanDoKata`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2709-2748`
    pub fn PM_CanDoKata(&mut self) -> qboolean {
        unsafe {
            if self.PM_InSecondaryStyle() != 0 {
                return 0;
            }
            let ps = (*self.pm).ps;
            let cmd = &(*self.pm).cmd;
            if (*ps).saberInFlight == 0
                && self.PM_SaberMoveOkayForKata() != 0
                && BG_SaberInKata((*ps).saberMove as c_int) == 0
                && BG_InKataAnim((*ps).legsAnim) == 0
                && BG_InKataAnim((*ps).torsoAnim) == 0
                && (*ps).groundEntityNum != ENTITYNUM_NONE as c_int
                && (cmd.buttons & BUTTON_ATTACK as c_int) != 0
                && (cmd.buttons & BUTTON_ALT_ATTACK as c_int) != 0
                && cmd.forwardmove == 0
                && cmd.rightmove == 0
                && cmd.upmove <= 0
                && self.BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER) != 0
            {
                let saber = self.BG_MySaber((*ps).clientNum, 0);
                if !saber.is_null() && (*saber).kataMove == LS_NONE {
                    return 0;
                }
                let saber = self.BG_MySaber((*ps).clientNum, 1);
                if !saber.is_null() && (*saber).kataMove == LS_NONE {
                    return 0;
                }
                return 1;
            }
            0
        }
    }

    /// Raven `PM_CheckAltKickAttack`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2750-2775`
    pub fn PM_CheckAltKickAttack(&mut self) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            if (*ps).weapon == WP_SABER as c_int {
                let saber = self.BG_MySaber((*ps).clientNum, 0);
                if !saber.is_null() && ((*saber).saberFlags & SFL_NO_KICKS) != 0 {
                    return 0;
                }
                let saber = self.BG_MySaber((*ps).clientNum, 1);
                if !saber.is_null() && ((*saber).saberFlags & SFL_NO_KICKS) != 0 {
                    return 0;
                }
            }
            let cmd = &(*self.pm).cmd;
            if (cmd.buttons & BUTTON_ALT_ATTACK as c_int) != 0
                && (BG_FlippingAnim((*ps).legsAnim) == 0 || (*ps).legsTimer <= 250)
                && (*ps).fd.saberAnimLevel == SS_STAFF as c_int
                && (*ps).saberHolstered == 0
            {
                return 1;
            }
            0
        }
    }

    /// Raven `PM_SaberPowerCheck`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2785-2800`
    pub fn PM_SaberPowerCheck(&mut self) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            let need = forcePowerNeeded
                [(*ps).fd.forcePowerLevel[FP_SABERTHROW as usize] as usize][FP_SABERTHROW as usize];
            if (*ps).saberInFlight != 0 {
                if (*ps).fd.forcePower > need {
                    return 1;
                }
            } else {
                return self.BG_EnoughForcePowerForMove(need);
            }
            0
        }
    }

    /// Raven `PM_CanDoRollStab`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2802-2820`
    pub fn PM_CanDoRollStab(&mut self) -> qboolean {
        unsafe {
            let ps = (*self.pm).ps;
            if (*ps).weapon == WP_SABER as c_int {
                let saber = self.BG_MySaber((*ps).clientNum, 0);
                if !saber.is_null() && ((*saber).saberFlags & SFL_NO_ROLL_STAB) != 0 {
                    return 0;
                }
                let saber = self.BG_MySaber((*ps).clientNum, 1);
                if !saber.is_null() && ((*saber).saberFlags & SFL_NO_ROLL_STAB) != 0 {
                    return 0;
                }
            }
            1
        }
    }

    /// Raven `PM_WeaponLightsaber`.
    ///
    /// Raven: the main per-frame saber weapon state machine (965 LOC).
    /// PORT-NOTE(scope): transcribed as faithfully as the packet's resolved
    /// call surface allows; several referenced helpers (`PM_GetSaberStance`,
    /// `PM_BeginWeaponChange`, `PM_FinishWeaponChange`, `PM_SaberBounceForAttack`,
    /// `BG_HasYsalamiri`, `BG_CanUseFPNow`, weaponData) are called exactly as
    /// their resolved (possibly still-parked) free-fn signatures — not this
    /// packet's job to fix. The single `goto weapChecks;` is transcribed as a
    /// `checkOnlyWeap` early-return split (both goto sites jump to the same
    /// point the fallthrough path reaches next).
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:2836-3800`
    pub fn PM_WeaponLightsaber(&mut self) {
        unsafe {
            let ps = (*self.pm).ps;

            if PM_InKnockDown(ps) != 0 || BG_InRoll(ps, (*ps).legsAnim) != 0 {
                if (*ps).weaponTime > 0 {
                    (*ps).weaponTime -= self.pml.msec;
                    if (*ps).weaponTime <= 0 {
                        (*ps).weaponTime = 0;
                    }
                }
                if (*ps).legsAnim == BOTH_ROLL_F as c_int && (*ps).legsTimer <= 250 {
                    if ((*self.pm).cmd.buttons & BUTTON_ATTACK as c_int) != 0
                        && self.BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != 0
                        && (*ps).saberInFlight == 0
                        && self.PM_CanDoRollStab() != 0
                    {
                        if (*ps).saberHolstered == 2 {
                            (*ps).saberHolstered = 0;
                            PM_AddEvent(EV_SABER_UNHOLSTER as c_int);
                        }
                        self.PM_SetSaberMove(LS_ROLL_STAB as c_short);
                        BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                    }
                }
                return;
            }

            if (*ps).saberLockTime > (*self.pm).cmd.serverTime {
                (*ps).saberMove = LS_NONE as c_short;
                self.PM_SaberLocked();
                return;
            } else if (*ps).saberLockFrame != 0 {
                if (*ps).saberLockEnemy < ENTITYNUM_NONE as c_int && (*ps).saberLockEnemy >= 0 {
                    let bgEnt = self.PM_BGEntForNum((*ps).saberLockEnemy);
                    if !bgEnt.is_null() {
                        let en = (*bgEnt).playerState;
                        if !en.is_null() {
                            self.PM_SaberLockBreak(en, 0, 0);
                            return;
                        }
                    }
                }
                if (*ps).saberLockFrame != 0 {
                    (*ps).torsoTimer = 0;
                    self.PM_SetAnim(SETANIM_TORSO as c_int, BOTH_STAND1 as c_int, SETANIM_FLAG_OVERRIDE as c_int, 100);
                    (*ps).saberLockFrame = 0;
                }
            }

            if BG_KickingAnim((*ps).legsAnim) != 0 || BG_KickingAnim((*ps).torsoAnim) != 0 {
                if (*ps).legsTimer > 0 {
                    return;
                }
                (*ps).saberMove = LS_READY as c_short;
                (*ps).weaponTime = 0;
            }

            if BG_SuperBreakLoseAnim((*ps).torsoAnim) != 0 || BG_SuperBreakWinAnim((*ps).torsoAnim) != 0 {
                if (*ps).torsoTimer > 0 {
                    return;
                }
            }

            let mut checkOnlyWeap = false;

            if BG_SabersOff(ps) != 0 {
                if (*ps).saberMove as c_int != LS_READY {
                    self.PM_SetSaberMove(LS_READY as c_short);
                }
                if (*ps).legsAnim != (*ps).torsoAnim
                    && BG_InSlopeAnim((*ps).legsAnim) == 0
                    && (*ps).torsoTimer <= 0
                {
                    self.PM_SetAnim(SETANIM_TORSO as c_int, (*ps).legsAnim, SETANIM_FLAG_OVERRIDE as c_int, 100);
                } else if BG_InSlopeAnim((*ps).legsAnim) != 0 && (*ps).torsoTimer <= 0 {
                    let stance = self.PM_GetSaberStance();
                    self.PM_SetAnim(SETANIM_TORSO as c_int, stance, SETANIM_FLAG_OVERRIDE as c_int, 100);
                }

                if (*ps).weaponTime < 1
                    && (((*self.pm).cmd.buttons & BUTTON_ALT_ATTACK as c_int) != 0
                        || ((*self.pm).cmd.buttons & BUTTON_ATTACK as c_int) != 0)
                {
                    if (*ps).duelTime < (*self.pm).cmd.serverTime {
                        if (*ps).m_iVehicleNum == 0 {
                            (*ps).saberHolstered = 0;
                            PM_AddEvent(EV_SABER_UNHOLSTER as c_int);
                        } else {
                            (*self.pm).cmd.buttons &= !(BUTTON_ALT_ATTACK as c_int);
                            (*self.pm).cmd.buttons &= !(BUTTON_ATTACK as c_int);
                        }
                    }
                }

                if (*ps).weaponTime > 0 {
                    (*ps).weaponTime -= self.pml.msec;
                }

                checkOnlyWeap = true;
                return self.PM_WeaponLightsaber_weapChecks(checkOnlyWeap);
            }

            if (*ps).saberEntityNum == 0 && (*ps).saberInFlight != 0 {
                if (*ps).fd.saberAnimLevel == SS_DUAL as c_int {
                    if (*ps).saberHolstered > 1 {
                        (*ps).saberHolstered = 1;
                    }
                } else {
                    (*self.pm).cmd.buttons &= !(BUTTON_ATTACK as c_int);
                }
                (*self.pm).cmd.buttons &= !(BUTTON_ALT_ATTACK as c_int);
            }

            if ((*self.pm).cmd.buttons & BUTTON_ALT_ATTACK as c_int) != 0 {
                if (*ps).fd.saberAnimLevel == SS_STAFF as c_int {
                    if (*ps).weaponTime > 0
                        && PM_SaberInReturn((*ps).saberMove as c_int) != 0
                        && (*ps).saberBlocked == BLOCKED_NONE as c_int
                        && ((*self.pm).cmd.buttons & BUTTON_ATTACK as c_int) == 0
                    {
                        if ((*self.pm).cmd.forwardmove != 0 || (*self.pm).cmd.rightmove != 0)
                            && self.PM_CheckAltKickAttack() != 0
                        {
                            let kickMove = self.PM_KickMoveForConditions();
                            if kickMove != -1 {
                                (*ps).weaponTime = 0;
                                self.PM_SetSaberMove(kickMove as c_short);
                                return;
                            }
                        }
                    }
                } else if (*ps).weaponTime < 1
                    && (*ps).saberCanThrow != 0
                    && BG_HasYsalamiri((*self.pm).gametype, ps) == 0
                    && BG_CanUseFPNow((*self.pm).gametype, ps, (*self.pm).cmd.serverTime, FP_SABERTHROW) != 0
                    && (*ps).fd.forcePowerLevel[FP_SABERTHROW as usize] > 0
                    && self.PM_SaberPowerCheck() != 0
                {
                    let mut sabMins: vec3_t = [0.0; 3];
                    VectorSet(&mut sabMins, SABERMINS_X, SABERMINS_Y, SABERMINS_Z);
                    let mut sabMaxs: vec3_t = [0.0; 3];
                    VectorSet(&mut sabMaxs, SABERMAXS_X, SABERMAXS_Y, SABERMAXS_Z);
                    let mut fwd = [0.0f32; 3];
                    AngleVectors((*ps).viewangles, Some(&mut fwd), None, None);
                    let mut minFwd: vec3_t = [0.0; 3];
                    _VectorMA((*ps).origin, SABER_MIN_THROW_DIST, fwd, &mut minFwd);

                    let mut sabTr: trace_t = core::mem::zeroed();
                    self.traps.trace(
                        &mut sabTr,
                        &(*ps).origin,
                        &sabMins,
                        &sabMaxs,
                        &minFwd,
                        (*ps).clientNum,
                        MASK_PLAYERSOLID as c_int,
                    );

                    if sabTr.allsolid != 0 || sabTr.startsolid != 0 || sabTr.fraction < 1.0 {
                        // not enough room to throw
                    } else {
                        if (*ps).saberInFlight == 0 {
                            (*ps).fd.forcePower -= forcePowerNeeded
                                [(*ps).fd.forcePowerLevel[FP_SABERTHROW as usize] as usize]
                                [FP_SABERTHROW as usize];
                        }
                        (*ps).saberInFlight = 1;
                    }
                }
            }

            if (*ps).saberInFlight != 0 && (*ps).saberEntityNum != 0 {
                if (*ps).fd.saberAnimLevel != SS_DUAL as c_int
                    || (*ps).saberHolstered != 0
                    || (((*self.pm).cmd.buttons & BUTTON_ATTACK as c_int) == 0
                        && ((*ps).torsoAnim == BOTH_SABERDUAL_STANCE as c_int
                            || (*ps).torsoAnim == BOTH_SABERPULL as c_int
                            || (*ps).torsoAnim == BOTH_STAND1 as c_int
                            || PM_RunningAnim((*ps).torsoAnim) != 0
                            || PM_WalkingAnim((*ps).torsoAnim) != 0
                            || PM_JumpingAnim((*ps).torsoAnim) != 0
                            || PM_SwimmingAnim((*ps).torsoAnim) != 0))
                {
                    self.PM_SetAnim(
                        SETANIM_TORSO as c_int,
                        BOTH_SABERPULL as c_int,
                        (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
                        100,
                    );
                    (*ps).torsoTimer = 1;
                    return;
                }
            }

            if (*ps).stats[STAT_HEALTH as usize] <= 0 {
                return;
            }

            if (*ps).weaponTime > 0 {
                let pullmove = PM_CheckPullAttack();
                if pullmove != LS_NONE {
                    (*ps).weaponTime = 0;
                    (*ps).torsoTimer = 0;
                    (*ps).legsTimer = 0;
                    (*ps).forceHandExtend = HANDEXTEND_NONE as c_int;
                    (*ps).weaponstate = WEAPON_READY;
                    self.PM_SetSaberMove(pullmove as c_short);
                    return;
                }
                (*ps).weaponTime -= self.pml.msec;
            } else {
                (*ps).weaponstate = WEAPON_READY;
            }

            if (*ps).saberBlocked != 0 {
                if (*ps).saberBlocked >= BLOCKED_UPPER_RIGHT as c_int
                    && (*ps).saberBlocked < BLOCKED_UPPER_RIGHT_PROJ as c_int
                {
                    (*ps).weaponTime = bg_parryDebounce[(*ps).fd.forcePowerLevel[FP_SABER_DEFENSE as usize] as usize] + 200;
                }
                match (*ps).saberBlocked {
                    x if x == BLOCKED_BOUNCE_MOVE as c_int => {
                        (*ps).torsoTimer = 0;
                        self.PM_SetSaberMove((*ps).saberMove);
                        (*ps).weaponTime = (*ps).torsoTimer;
                        (*ps).saberBlocked = 0;
                    }
                    x if x == BLOCKED_PARRY_BROKEN as c_int => {
                        let nextMove = if PM_SaberInBrokenParry((*ps).saberMove as c_int) != 0 {
                            (*ps).saberMove as c_int
                        } else {
                            PM_BrokenParryForParry((*ps).saberMove as c_int)
                        };
                        if nextMove != LS_NONE {
                            self.PM_SetSaberMove(nextMove as c_short);
                            (*ps).weaponTime = (*ps).torsoTimer;
                        }
                    }
                    x if x == BLOCKED_ATK_BOUNCE as c_int => {
                        if (*ps).saberMove as c_int >= LS_T1_BR__R {
                            (*ps).saberBlocked = BLOCKED_NONE as c_int;
                        } else {
                            let bounceMove;
                            if PM_SaberInBounce((*ps).saberMove as c_int) != 0
                                || BG_SaberInAttack((*ps).saberMove as c_int) == 0
                            {
                                if ((*self.pm).cmd.buttons & BUTTON_ATTACK as c_int) != 0 {
                                    let mut newQuad =
                                        PM_SaberMoveQuadrantForMovement(&mut (*self.pm).cmd);
                                    while newQuad
                                        == saberMoveData[(*ps).saberMove as usize].startQuad
                                    {
                                        newQuad = self.PM_irand_timesync(Q_BR as c_int, Q_BL as c_int);
                                    }
                                    bounceMove = transitionMove
                                        [saberMoveData[(*ps).saberMove as usize].startQuad as usize]
                                        [newQuad as usize];
                                } else if saberMoveData[(*ps).saberMove as usize].startQuad
                                    == Q_T as c_int
                                {
                                    bounceMove = LS_R_BL2TR;
                                } else if saberMoveData[(*ps).saberMove as usize].startQuad
                                    < Q_T as c_int
                                {
                                    bounceMove = LS_R_TL2BR
                                        + saberMoveData[(*ps).saberMove as usize].startQuad
                                        - Q_BR as c_int;
                                } else {
                                    bounceMove = LS_R_BR2TL
                                        + saberMoveData[(*ps).saberMove as usize].startQuad
                                        - Q_TL as c_int;
                                }
                            } else {
                                bounceMove =
                                    PM_SaberBounceForAttack((*ps).saberMove as c_int);
                            }
                            self.PM_SetSaberMove(bounceMove as c_short);
                            (*ps).weaponTime = (*ps).torsoTimer;
                        }
                    }
                    x if x == BLOCKED_UPPER_RIGHT as c_int => self.PM_SetSaberMove(LS_PARRY_UR as c_short),
                    x if x == BLOCKED_UPPER_RIGHT_PROJ as c_int => self.PM_SetSaberMove(LS_REFLECT_UR as c_short),
                    x if x == BLOCKED_UPPER_LEFT as c_int => self.PM_SetSaberMove(LS_PARRY_UL as c_short),
                    x if x == BLOCKED_UPPER_LEFT_PROJ as c_int => self.PM_SetSaberMove(LS_REFLECT_UL as c_short),
                    x if x == BLOCKED_LOWER_RIGHT as c_int => self.PM_SetSaberMove(LS_PARRY_LR as c_short),
                    x if x == BLOCKED_LOWER_RIGHT_PROJ as c_int => self.PM_SetSaberMove(LS_REFLECT_LR as c_short),
                    x if x == BLOCKED_LOWER_LEFT as c_int => self.PM_SetSaberMove(LS_PARRY_LL as c_short),
                    x if x == BLOCKED_LOWER_LEFT_PROJ as c_int => self.PM_SetSaberMove(LS_REFLECT_LL as c_short),
                    x if x == BLOCKED_TOP as c_int => self.PM_SetSaberMove(LS_PARRY_UP as c_short),
                    x if x == BLOCKED_TOP_PROJ as c_int => self.PM_SetSaberMove(LS_REFLECT_UP as c_short),
                    _ => {
                        (*ps).saberBlocked = BLOCKED_NONE as c_int;
                    }
                }
                if (*ps).saberBlocked >= BLOCKED_UPPER_RIGHT as c_int
                    && (*ps).saberBlocked < BLOCKED_UPPER_RIGHT_PROJ as c_int
                    && (*ps).torsoTimer < (*ps).weaponTime
                {
                    (*ps).torsoTimer = (*ps).weaponTime;
                }

                (*ps).saberBlocked = 0;
                (*ps).weaponstate = WEAPON_READY;
                return;
            }

            self.PM_WeaponLightsaber_weapChecks(checkOnlyWeap)
        }
    }

    /// The `weapChecks:` tail of `PM_WeaponLightsaber` (bg_saber.c:3327-3800).
    /// Both `goto weapChecks;` sites and the natural fallthrough reach this same
    /// point; factored out because Rust has no `goto` (porting-rules §C10).
    fn PM_WeaponLightsaber_weapChecks(&mut self, checkOnlyWeap: bool) {
        unsafe {
            let ps = (*self.pm).ps;

            if (*ps).saberEntityNum != 0 {
                if (*ps).weaponTime <= 0 && (*ps).torsoTimer <= 0 {
                    if (*ps).weapon != (*self.pm).cmd.weapon {
                        PM_BeginWeaponChange((*self.pm).cmd.weapon);
                    }
                }
            }

            if self.PM_CanDoKata() != 0 {
                let mut overrideMove = LS_INVALID;
                let saber1 = self.BG_MySaber((*ps).clientNum, 0);
                let saber2 = self.BG_MySaber((*ps).clientNum, 1);
                if !saber1.is_null() && (*saber1).kataMove != LS_INVALID {
                    if (*saber1).kataMove != LS_NONE {
                        overrideMove = (*saber1).kataMove;
                    }
                }
                if overrideMove == LS_INVALID {
                    if !saber2.is_null() && (*saber2).kataMove != LS_INVALID {
                        if (*saber2).kataMove != LS_NONE {
                            overrideMove = (*saber2).kataMove;
                        }
                    }
                }
                if overrideMove == LS_INVALID {
                    if !saber2.is_null() && (*saber2).kataMove == LS_NONE {
                        overrideMove = LS_NONE;
                    }
                }
                if overrideMove == LS_INVALID {
                    match (*ps).fd.saberAnimLevel {
                        x if x == SS_FAST as c_int || x == SS_TAVION as c_int => {
                            self.PM_SetSaberMove(LS_A1_SPECIAL as c_short);
                        }
                        x if x == SS_MEDIUM as c_int => self.PM_SetSaberMove(LS_A2_SPECIAL as c_short),
                        x if x == SS_STRONG as c_int || x == SS_DESANN as c_int => {
                            self.PM_SetSaberMove(LS_A3_SPECIAL as c_short);
                        }
                        x if x == SS_DUAL as c_int => {
                            self.PM_SetSaberMove(LS_DUAL_SPIN_PROTECT as c_short);
                        }
                        x if x == SS_STAFF as c_int => {
                            self.PM_SetSaberMove(LS_STAFF_SOULCAL as c_short);
                        }
                        _ => {}
                    }
                    (*ps).weaponstate = WEAPON_FIRING;
                    BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER);
                } else if overrideMove != LS_NONE {
                    self.PM_SetSaberMove(overrideMove as c_short);
                    (*ps).weaponstate = WEAPON_FIRING;
                    BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER);
                }
                if overrideMove != LS_NONE {
                    return;
                }
            }

            if (*ps).weaponTime > 0 {
                return;
            }

            if (*ps).weaponstate == WEAPON_DROPPING {
                PM_FinishWeaponChange();
                return;
            }

            if (*ps).weaponstate == WEAPON_RAISING {
                (*ps).weaponstate = WEAPON_IDLE;
                if (*ps).legsAnim == BOTH_WALK1 as c_int {
                    self.PM_SetAnim(SETANIM_TORSO as c_int, BOTH_WALK1 as c_int, SETANIM_FLAG_NORMAL as c_int, 100);
                } else if (*ps).legsAnim == BOTH_RUN1 as c_int {
                    self.PM_SetAnim(SETANIM_TORSO as c_int, BOTH_RUN1 as c_int, SETANIM_FLAG_NORMAL as c_int, 100);
                } else if (*ps).legsAnim == BOTH_RUN2 as c_int {
                    self.PM_SetAnim(SETANIM_TORSO as c_int, BOTH_RUN2 as c_int, SETANIM_FLAG_NORMAL as c_int, 100);
                } else if (*ps).legsAnim == BOTH_RUN_STAFF as c_int {
                    self.PM_SetAnim(SETANIM_TORSO as c_int, BOTH_RUN_STAFF as c_int, SETANIM_FLAG_NORMAL as c_int, 100);
                } else if (*ps).legsAnim == BOTH_RUN_DUAL as c_int {
                    self.PM_SetAnim(SETANIM_TORSO as c_int, BOTH_RUN_DUAL as c_int, SETANIM_FLAG_NORMAL as c_int, 100);
                } else if (*ps).legsAnim == BOTH_WALK2 as c_int {
                    self.PM_SetAnim(SETANIM_TORSO as c_int, BOTH_WALK2 as c_int, SETANIM_FLAG_NORMAL as c_int, 100);
                } else if (*ps).legsAnim == BOTH_WALK_STAFF as c_int {
                    self.PM_SetAnim(SETANIM_TORSO as c_int, BOTH_WALK_STAFF as c_int, SETANIM_FLAG_NORMAL as c_int, 100);
                } else if (*ps).legsAnim == BOTH_WALK_DUAL as c_int {
                    self.PM_SetAnim(SETANIM_TORSO as c_int, BOTH_WALK_DUAL as c_int, SETANIM_FLAG_NORMAL as c_int, 100);
                } else {
                    let stance = self.PM_GetSaberStance();
                    self.PM_SetAnim(SETANIM_TORSO as c_int, stance, SETANIM_FLAG_NORMAL as c_int, 100);
                }

                if (*ps).weaponstate == WEAPON_RAISING {
                    return;
                }
            }

            if checkOnlyWeap {
                return;
            }

            if (*ps).fd.saberAnimLevel == SS_STAFF as c_int
                && ((*self.pm).cmd.buttons & BUTTON_ALT_ATTACK as c_int) != 0
            {
                let mut kickMove: c_int = -1;
                if BG_KickingAnim((*ps).torsoAnim) == 0
                    && BG_KickingAnim((*ps).legsAnim) == 0
                    && BG_InRoll(ps, (*ps).legsAnim) == 0
                    && (*ps).saberMove as c_int == LS_READY
                    && ((*ps).pm_flags & PMF_DUCKED as c_int) == 0
                    && (*self.pm).cmd.upmove >= 0
                {
                    kickMove = self.PM_KickMoveForConditions();
                }

                if kickMove != -1 {
                    if (*ps).groundEntityNum == ENTITYNUM_NONE as c_int {
                        let gDist = self.PM_GroundDistance();
                        if (BG_FlippingAnim((*ps).legsAnim) == 0 || (*ps).legsTimer <= 0)
                            && gDist > 64.0
                            && gDist > (-(*ps).velocity[2]) - 64.0
                        {
                            kickMove = match kickMove {
                                x if x == LS_KICK_F => LS_KICK_F_AIR,
                                x if x == LS_KICK_B => LS_KICK_B_AIR,
                                x if x == LS_KICK_R => LS_KICK_R_AIR,
                                x if x == LS_KICK_L => LS_KICK_L_AIR,
                                _ => -1,
                            };
                        } else if gDist > 128.0 || (*ps).velocity[2] >= 0.0 {
                            kickMove = -1;
                        }
                    }

                    if kickMove != -1 {
                        self.PM_SetSaberMove(kickMove as c_short);
                        return;
                    }
                }
            }

            (*self.pm).cmd.buttons &= !(BUTTON_ALT_ATTACK as c_int);

            let mut anim: c_int = -1;
            let mut newmove = LS_NONE;
            let mut curmove;

            if (*ps).saberMove as c_int > LS_NONE && ((*ps).saberMove as c_int) < LS_MOVE_MAX {
                curmove = (*ps).saberMove as c_int;
            } else {
                curmove = LS_READY;
            }

            if curmove == LS_A_JUMP_T__B_ || (*ps).torsoAnim == BOTH_FORCELEAP2_T__B_ as c_int {
                newmove = LS_R_T2B;
            } else if ((*self.pm).cmd.buttons & (BUTTON_ATTACK as c_int | BUTTON_ALT_ATTACK as c_int))
                == 0
            {
                (*ps).weaponTime = 0;

                if (*ps).weaponTime > 0 {
                    (*ps).weaponstate = WEAPON_FIRING;
                } else if (*ps).weaponstate != WEAPON_READY {
                    (*ps).weaponstate = WEAPON_IDLE;
                }

                if curmove >= LS_S_TL2BR && curmove <= LS_S_T2B {
                    newmove = LS_A_TL2BR + (curmove - LS_S_TL2BR);
                } else if curmove >= LS_A_TL2BR && curmove <= LS_A_T2B {
                    newmove = LS_R_TL2BR + (curmove - LS_A_TL2BR);
                } else if PM_SaberInTransition(curmove) != 0 {
                    newmove = saberMoveData[curmove as usize].chain_attack;
                } else if PM_SaberInBounce(curmove) != 0 {
                    newmove = saberMoveData[curmove as usize].chain_idle;
                } else {
                    self.PM_SetSaberMove(LS_READY as c_short);
                    return;
                }
            }

            if (*ps).weaponTime > 0 {
                (*ps).weaponstate = WEAPON_FIRING;
                return;
            }

            if (*ps).torsoAnim == BOTH_FORCELONGLEAP_ATTACK as c_int
                || (*ps).torsoAnim == BOTH_FORCELONGLEAP_LAND as c_int
            {
                return;
            } else if (*ps).torsoAnim == BOTH_FORCELONGLEAP_START as c_int {
                if (*ps).torsoTimer >= 200 {
                    self.PM_SetSaberMove(LS_LEAP_ATTACK as c_short);
                }
                return;
            }

            if curmove >= LS_PARRY_UP && curmove <= LS_REFLECT_LL {
                match saberMoveData[curmove as usize].endQuad {
                    x if x == Q_T as c_int => newmove = LS_A_T2B,
                    x if x == Q_TR as c_int => newmove = LS_A_TR2BL,
                    x if x == Q_TL as c_int => newmove = LS_A_TL2BR,
                    x if x == Q_BR as c_int => newmove = LS_A_BR2TL,
                    x if x == Q_BL as c_int => newmove = LS_A_BL2TR,
                    _ => {}
                }
            }

            if newmove != LS_NONE {
                anim = saberMoveData[newmove as usize].animToUse;
            }

            let mut both = false;

            if anim == -1 {
                if PM_SaberInTransition(curmove) != 0 {
                    newmove = saberMoveData[curmove as usize].chain_attack;
                } else if curmove >= LS_S_TL2BR && curmove <= LS_S_T2B {
                    newmove = LS_A_TL2BR + (curmove - LS_S_TL2BR);
                } else if PM_SaberInBrokenParry(curmove) != 0 {
                    newmove = LS_READY;
                } else {
                    newmove = self.PM_SaberAttackForMovement(curmove);
                    if (PM_SaberInBounce(curmove) != 0 || PM_SaberInBrokenParry(curmove) != 0)
                        && saberMoveData[newmove as usize].startQuad
                            == saberMoveData[curmove as usize].endQuad
                    {
                        newmove = saberMoveData[curmove as usize].chain_attack;
                    }
                    if self.PM_SaberKataDone(curmove, newmove) != 0 {
                        newmove = saberMoveData[curmove as usize].chain_idle;
                    }
                }
                if newmove != LS_NONE {
                    newmove = self.PM_SaberAnimTransitionAnim(curmove, newmove);
                    anim = saberMoveData[newmove as usize].animToUse;
                }
            }

            if anim == -1 {
                newmove = saberMoveData[curmove as usize].chain_attack;
                anim = saberMoveData[newmove as usize].animToUse;

                if (*self.pm).cmd.forwardmove == 0
                    && (*self.pm).cmd.rightmove == 0
                    && (*self.pm).cmd.upmove >= 0
                    && (*ps).groundEntityNum != ENTITYNUM_NONE as c_int
                {
                    both = true;
                }
            }

            if anim == -1 {
                anim = match (*ps).legsAnim {
                    x if x == BOTH_WALK1 as c_int
                        || x == BOTH_WALK2 as c_int
                        || x == BOTH_WALK_STAFF as c_int
                        || x == BOTH_WALK_DUAL as c_int
                        || x == BOTH_WALKBACK1 as c_int
                        || x == BOTH_WALKBACK2 as c_int
                        || x == BOTH_WALKBACK_STAFF as c_int
                        || x == BOTH_WALKBACK_DUAL as c_int
                        || x == BOTH_RUN1 as c_int
                        || x == BOTH_RUN2 as c_int
                        || x == BOTH_RUN_STAFF as c_int
                        || x == BOTH_RUN_DUAL as c_int
                        || x == BOTH_RUNBACK1 as c_int
                        || x == BOTH_RUNBACK2 as c_int
                        || x == BOTH_RUNBACK_STAFF as c_int =>
                    {
                        (*ps).legsAnim
                    }
                    _ => self.PM_GetSaberStance(),
                };
                newmove = LS_READY;
            }

            self.PM_SetSaberMove(newmove as c_short);

            if both && (*ps).torsoAnim == anim {
                self.PM_SetAnim(
                    SETANIM_LEGS as c_int,
                    anim,
                    (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
                    100,
                );
            }

            (*ps).weaponTime = (*ps).torsoTimer;

            (*ps).weaponstate = WEAPON_FIRING;
            let amount = weaponData[(*ps).weapon as usize].energyPerShot;
            let _ = amount;
            let mut addTime = (*ps).weaponTime;
            (*ps).saberAttackSequence = (*ps).torsoAnim;
            if addTime == 0 {
                addTime = weaponData[(*ps).weapon as usize].fireTime;
            }
            (*ps).weaponTime = addTime;
        }
    }

    /// Raven `PM_SetSaberMove`.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:3802-4098`
    pub fn PM_SetSaberMove(&mut self, newMove: c_short) {
        unsafe {
            let ps = (*self.pm).ps;
            let newMove = newMove as c_int;
            let mut setflags = saberMoveData[newMove as usize].animSetFlags as c_int;
            let mut anim = saberMoveData[newMove as usize].animToUse;
            let mut parts = SETANIM_TORSO as c_int;

            if newMove == LS_READY || newMove == LS_A_FLIP_STAB || newMove == LS_A_FLIP_SLASH {
                (*ps).saberAttackChainCount = 0;
            } else if BG_SaberInAttack(newMove) != 0 {
                (*ps).saberAttackChainCount += 1;
            }
            if (*ps).saberAttackChainCount > 16 {
                (*ps).saberAttackChainCount = 16;
            }

            if newMove == LS_DRAW {
                let saber1 = self.BG_MySaber((*ps).clientNum, 0);
                let saber2 = self.BG_MySaber((*ps).clientNum, 1);
                if !saber1.is_null() && (*saber1).drawAnim != -1 {
                    anim = (*saber1).drawAnim;
                } else if !saber2.is_null() && (*saber2).drawAnim != -1 {
                    anim = (*saber2).drawAnim;
                } else if (*ps).fd.saberAnimLevel == SS_STAFF as c_int {
                    anim = BOTH_S1_S7 as c_int;
                } else if (*ps).fd.saberAnimLevel == SS_DUAL as c_int {
                    anim = BOTH_S1_S6 as c_int;
                }
            } else if newMove == LS_PUTAWAY {
                let saber1 = self.BG_MySaber((*ps).clientNum, 0);
                let saber2 = self.BG_MySaber((*ps).clientNum, 1);
                if !saber1.is_null() && (*saber1).putawayAnim != -1 {
                    anim = (*saber1).putawayAnim;
                } else if !saber2.is_null() && (*saber2).putawayAnim != -1 {
                    anim = (*saber2).putawayAnim;
                } else if (*ps).fd.saberAnimLevel == SS_STAFF as c_int {
                    anim = BOTH_S7_S1 as c_int;
                } else if (*ps).fd.saberAnimLevel == SS_DUAL as c_int {
                    anim = BOTH_S6_S1 as c_int;
                }
            } else if (*ps).fd.saberAnimLevel == SS_STAFF as c_int
                && newMove >= LS_S_TL2BR
                && newMove < LS_REFLECT_LL
            {
                if newMove >= LS_V1_BR && newMove <= LS_REFLECT_LL {
                    anim = BOTH_P7_S7_T_ as c_int + (anim - BOTH_P1_S1_T_ as c_int);
                } else {
                    anim += ((*ps).fd.saberAnimLevel - FORCE_LEVEL_1 as c_int) * SABER_ANIM_GROUP_SIZE;
                }
            } else if (*ps).fd.saberAnimLevel == SS_DUAL as c_int
                && newMove >= LS_S_TL2BR
                && newMove < LS_REFLECT_LL
            {
                if newMove >= LS_V1_BR && newMove <= LS_REFLECT_LL {
                    anim = BOTH_P6_S6_T_ as c_int + (anim - BOTH_P1_S1_T_ as c_int);
                } else {
                    anim += ((*ps).fd.saberAnimLevel - FORCE_LEVEL_1 as c_int) * SABER_ANIM_GROUP_SIZE;
                }
            } else if (*ps).fd.saberAnimLevel > FORCE_LEVEL_1 as c_int
                && BG_SaberInIdle(newMove) == 0
                && PM_SaberInParry(newMove) == 0
                && PM_SaberInKnockaway(newMove) == 0
                && PM_SaberInBrokenParry(newMove) == 0
                && PM_SaberInReflect(newMove) == 0
                && BG_SaberInSpecial(newMove) == 0
            {
                anim += ((*ps).fd.saberAnimLevel - FORCE_LEVEL_1 as c_int) * SABER_ANIM_GROUP_SIZE;
            }

            if saberMoveData[(*ps).saberMove as usize].animToUse == anim && newMove > LS_PUTAWAY {
                setflags |= SETANIM_FLAG_RESTART as c_int;
            }

            if (*ps).m_iVehicleNum == 0 {
                if BG_SaberInSpecial(newMove) != 0 {
                    setflags |= SETANIM_FLAG_OVERRIDE as c_int;
                }
            }
            if BG_InSaberStandAnim(anim) != 0 || anim == BOTH_STAND1 as c_int {
                anim = (*ps).legsAnim;

                if (anim >= BOTH_STAND1 as c_int && anim <= BOTH_STAND4TOATTACK2 as c_int)
                    || (anim >= TORSO_DROPWEAP1 as c_int && anim <= TORSO_WEAPONIDLE10 as c_int)
                {
                    anim = self.PM_GetSaberStance();
                }
                if ((*ps).pm_flags & PMF_DUCKED as c_int) != 0 {
                    anim = self.PM_GetSaberStance();
                }
                if anim == BOTH_WALKBACK1 as c_int || anim == BOTH_WALKBACK2 as c_int || anim == BOTH_WALK1 as c_int {
                    // PORT-NOTE(faithful-empty-branch): the oracle's `if`
                    // (bg_saber.c:3950-3953) has an empty body here — transcribed
                    // faithfully (no-op), not a missing case.
                }
                if BG_InSlopeAnim(anim) != 0 {
                    anim = self.PM_GetSaberStance();
                }
                parts = SETANIM_TORSO as c_int;
            }

            if (*ps).m_iVehicleNum == 0 {
                if newMove == LS_JUMPATTACK_ARIAL_RIGHT || newMove == LS_JUMPATTACK_ARIAL_LEFT {
                    parts = SETANIM_LEGS as c_int;
                } else if newMove == LS_A_LUNGE
                    || newMove == LS_A_JUMP_T__B_
                    || newMove == LS_A_BACKSTAB
                    || newMove == LS_A_BACK
                    || newMove == LS_A_BACK_CR
                    || newMove == LS_ROLL_STAB
                    || newMove == LS_A_FLIP_STAB
                    || newMove == LS_A_FLIP_SLASH
                    || newMove == LS_JUMPATTACK_DUAL
                    || newMove == LS_JUMPATTACK_ARIAL_LEFT
                    || newMove == LS_JUMPATTACK_ARIAL_RIGHT
                    || newMove == LS_JUMPATTACK_CART_LEFT
                    || newMove == LS_JUMPATTACK_CART_RIGHT
                    || newMove == LS_JUMPATTACK_STAFF_LEFT
                    || newMove == LS_JUMPATTACK_STAFF_RIGHT
                    || newMove == LS_A_BACKFLIP_ATK
                    || newMove == LS_STABDOWN
                    || newMove == LS_STABDOWN_STAFF
                    || newMove == LS_STABDOWN_DUAL
                    || newMove == LS_DUAL_SPIN_PROTECT
                    || newMove == LS_STAFF_SOULCAL
                    || newMove == LS_A1_SPECIAL
                    || newMove == LS_A2_SPECIAL
                    || newMove == LS_A3_SPECIAL
                    || newMove == LS_UPSIDE_DOWN_ATTACK
                    || newMove == LS_PULL_ATTACK_STAB
                    || newMove == LS_PULL_ATTACK_SWING
                    || BG_KickMove(newMove) != 0
                {
                    parts = SETANIM_BOTH as c_int;
                } else if BG_SpinningSaberAnim(anim) != 0 {
                    parts = SETANIM_BOTH as c_int;
                } else if (*self.pm).cmd.forwardmove == 0
                    && (*self.pm).cmd.rightmove == 0
                    && (*self.pm).cmd.upmove == 0
                {
                    if BG_FlippingAnim((*ps).legsAnim) == 0
                        && BG_InRoll(ps, (*ps).legsAnim) == 0
                        && PM_InKnockDown(ps) == 0
                        && PM_JumpingAnim((*ps).legsAnim) == 0
                        && BG_InSpecialJump((*ps).legsAnim) == 0
                        && anim != self.PM_GetSaberStance()
                        && (*ps).groundEntityNum != ENTITYNUM_NONE as c_int
                        && ((*ps).pm_flags & PMF_DUCKED as c_int) == 0
                    {
                        parts = SETANIM_BOTH as c_int;
                    } else if ((*ps).pm_flags & PMF_DUCKED as c_int) == 0
                        && (newMove == LS_SPINATTACK_DUAL || newMove == LS_SPINATTACK)
                    {
                        parts = SETANIM_BOTH as c_int;
                    }
                }

                self.PM_SetAnim(parts, anim, setflags, saberMoveData[newMove as usize].blendTime);
                if parts != SETANIM_LEGS as c_int
                    && ((*ps).legsAnim == BOTH_ARIAL_LEFT as c_int
                        || (*ps).legsAnim == BOTH_ARIAL_RIGHT as c_int)
                    && (*ps).legsTimer > (*ps).torsoTimer
                {
                    (*ps).legsTimer = (*ps).torsoTimer;
                }
            }

            if (*ps).torsoAnim == anim {
                if BG_SaberInAttack(newMove) != 0 || BG_SaberInSpecialAttack(anim) != 0 {
                    if (*ps).saberMove as c_int != newMove {
                        if newMove != LS_KICK_F
                            && newMove != LS_KICK_B
                            && newMove != LS_KICK_R
                            && newMove != LS_KICK_L
                            && newMove != LS_KICK_F_AIR
                            && newMove != LS_KICK_B_AIR
                            && newMove != LS_KICK_R_AIR
                            && newMove != LS_KICK_L_AIR
                        {
                            PM_AddEvent(EV_SABER_ATTACK as c_int);
                        }

                        if (*ps).brokenLimbs != 0 {
                            let mut iFactor: c_int = -1;
                            if ((*ps).brokenLimbs & (1 << BROKENLIMB_RARM as c_int)) != 0 {
                                iFactor = 5;
                            } else if ((*ps).brokenLimbs & (1 << BROKENLIMB_LARM as c_int)) != 0 {
                                iFactor = 10;
                            }
                            if iFactor != -1 && self.PM_irand_timesync(0, iFactor) == 0 {
                                let parm = self.PM_irand_timesync(1, 100);
                                BG_AddPredictableEventToPlayerstate(EV_PAIN as c_int, parm, ps);
                            }
                        }
                    }
                }

                if BG_SaberInSpecial(newMove) != 0 && (*ps).weaponTime < (*ps).torsoTimer {
                    (*ps).weaponTime = (*ps).torsoTimer;
                }

                (*ps).saberMove = newMove as c_short;
                (*ps).saberBlocking = saberMoveData[newMove as usize].blocking;
                (*ps).torsoAnim = anim;

                if (*ps).weaponTime <= 0 {
                    (*ps).saberBlocked = BLOCKED_NONE as c_int;
                }
            }
        }
    }

    /// Raven `BG_MySaber`.
    ///
    /// PORT-NOTE(client-saber-field): the oracle reads
    /// `g_entities[clientNum].client->saber[saberNum]`. `gclient_t.saber` is
    /// not yet present on the ported struct — referenced here per ruling 14's
    /// overlay idiom; a fixer must land the field.
    /// Source: `oracle/oracle/codemp/game/bg_saber.c:4100-4141`
    pub fn BG_MySaber(&mut self, clientNum: c_int, saberNum: c_int) -> *mut saberInfo_t {
        unsafe {
            let ent = self.PM_BGEntForNum(clientNum) as *mut gentity_t;
            if ent.is_null() {
                return core::ptr::null_mut();
            }
            if (*ent).inuse != 0 && !(*ent).client.is_null() {
                let saber = &mut (*((*ent).client as *mut gclient_t)).saber[saberNum as usize] as *mut saberInfo_t;
                if (*saber).model[0] == 0 {
                    return core::ptr::null_mut();
                }
                return saber;
            }
            core::ptr::null_mut()
        }
    }
}
