// PORT-COMPLETE: w_saber.c
//! FAITHFUL port of `oracle/codemp/game/w_saber.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5) instead of raw `gentity_t*`; ctx-free leaf
//! helpers borrow `&gentity_t`/`&mut gentity_t`.
//!
//! Safe-state migration **Stage 2b** (body sweep): every world reach is a
//! checked `ctx.world.…` field access — the transitional `ctx.world`
//! raw-deref regime (F1) is gone (one irreducible `&mut vec3_t` out-param site
//! into `G_Damage` excepted, marked in-code). The per-body entity/client
//! pointers stay raw by design: this file is gclient-saturated (`(*ent).client
//! as *mut gclient_t` chains), and gclient dissolution is out of scope, so the
//! `// STAGE-1:` entity re-derives and their `unsafe` blocks legitimately hold
//! genuine raw ops. Behavior is byte-identical, referee-verified.
//! `BG_MySaber`'s `ents` is the `g_entities` array base (pointer arithmetic),
//! so it stays raw by design.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
// Shadows the prelude's `crate::q_shared::Q_stricmp` glob export (pointer version);
// genuine pointer-vs-pointer survivors are re-qualified `crate::q_shared::Q_stricmp`.
use native_string::Q_stricmp;

// Constants used by the ported (non-parked) bodies below. Raven `#define`/enum
// constants live in the already-ported qshared/bg type modules; the prelude
// re-exports the owning *types* but not these value consts, so name them here.
use mp_bg::public::saber_move_name::{
    LS_A_BACK, LS_A_BACKSTAB, LS_A_BACK_CR, LS_K1_BL, LS_K1_BR, LS_K1_TL, LS_K1_TR, LS_K1_T_,
    LS_NONE, LS_PARRY_LL, LS_PARRY_LR, LS_PARRY_UL, LS_PARRY_UP, LS_PARRY_UR, LS_REFLECT_LL,
    LS_REFLECT_LR, LS_REFLECT_UL, LS_REFLECT_UP, LS_REFLECT_UR,
};
use mp_bg::public::weaponstate::weaponstate_t;
use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::common::mp::qcommon::saber::saber_info::MAX_SABERS;
use mp_qshared::common::mp::qcommon::saber::saber_type::saberType_t;
use mp_qshared::probe;
use mp_qshared::shared::saber_blocked_type::saberBlockedType_t;
use mp_qshared::shared::CHAN_WEAPON;

// --- pass-2 body-fill callee/const imports (jampgame shard 1/2) ---
// SPINE (fork rulings 1/4/8 + `engine-seam.md`): stateful fns thread
// `GameContext` (`.world: &mut GameWorld`, `.engine`); `level` →
// `ctx.world.level`, cvars → `ctx.world.cvars`, `g_entities[i]` →
// `ctx.world.g_entities[i]`, traps → `trap::X(ctx.engine, …)`. Cross-file
// callees use their resolved raw-pointer signatures verbatim.
//
// NOTE (integration-deferred, mirroring `w_force.rs`): a few Raven constants
// this shard needs are not yet surfaced by the prelude nor an owning enum
// (`DAMAGE_NO_KNOCKBACK`, `FL_NO_KNOCKBACK`). Per porting-rules the port
// preserves the Raven spelling; their exact enum-qualification / module path is
// resolved at integration (the mega-pass tree is not compiled per porter).
use crate::bg_channel::{GameBgTraps, GameCallbacksImpl};
use crate::client::render_info::renderInfo_t;
use crate::g_combat::{G_Damage, G_Knockdown};
use crate::g_mover::G_EntIsBreakable;
use crate::g_object::G_RunObject;
use crate::g_utils::{G_FreeEntity, G_InitGentity, G_Spawn};
use crate::g_utils::{G_Sound, G_SoundIndex, G_Throw};
use crate::q_math::{vec3_origin, VectorLength, VectorNormalize, PITCH};
use crate::trap;
use crate::NPC_utils::G_GetBoltPosition;
use mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs;
use mp_abi::game::syscalls::G_G2TRACE::GG2TraceArgs;
use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::bg_pmove::{BG_InKnockDown, BG_KnockDownable};
use mp_bg::bg_saberLoad::WP_SaberBladeUseSecondBladeStyle;
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::public::duel_team::duelTeam_t::DUELTEAM_LONE;
use mp_bg::public::gametype::{GT_DUEL, GT_JEDIMASTER, GT_POWERDUEL};
use mp_bg::public::saberlock::{
    SABERLOCK_LOCK, SABERLOCK_LOSE, SABERLOCK_SUPERBREAK, SABERLOCK_TOP, SABERLOCK_WIN,
};
use mp_qshared::common::mp::qcommon::usercmd_button::{
    BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_FORCEGRIP, BUTTON_FORCEPOWER, BUTTON_FORCE_DRAIN,
    BUTTON_FORCE_LIGHTNING, BUTTON_GESTURE,
};
use mp_qshared::shared::RAND_MAX;
// --- pass-2 shard-2 body-fill callee imports (resolved owning files per packet) ---
use crate::ai_wpnav::G_TestLine;
use crate::g_client::{G_UpdateClientAnims, SetClientViewAngle};
use crate::g_team::OnSameTeam;
use crate::g_timer::TIMER_Set;
use crate::g_utils::{G_EntitySound, G_SetAnim, G_SetOrigin, G_TempEntity};
use crate::q_math::{
    vectoangles, AngleDelta, AngleNormalize180, AngleVectors, AnglesToAxis, Distance,
    DistanceSquared, LerpAngle, VectorCompare, VectorNormalize2,
};
use crate::saber::saber_flags::{
    SFL_NOT_DISARMABLE, SFL_NOT_THROWABLE, SFL_RETURN_DAMAGE, SFL_SINGLE_BLADE_THROWABLE,
};
use crate::tri_coll_test::tri_tri_intersect;
use crate::w_force::{ForceThrow, WP_ForcePowerUsable};
use crate::NPC_AI_Jedi::{Jedi_Ambush, Jedi_SaberBlockGo, Jedi_WaitingAmbush};
use crate::NPC_senses::InFront;
use mp_bg::bg_misc::BG_EvaluateTrajectory;
use mp_bg::bg_misc::BG_GiveMeVectorFromMatrix;
use mp_bg::bg_misc::{BG_CanUseFPNow, BG_HasYsalamiri};
use mp_bg::bg_panimate::{
    BG_BrokenParryForParry, BG_InSpecialJump, BG_KnockawayForParry, BG_SaberInAttackPure,
    BG_SaberInReturn, BG_SaberInSpecialAttack, BG_SpinningSaberAnim, BG_StabDownAnim,
    PM_InKnockDown, PM_SaberInDeflect, PM_SaberInParry, PM_SaberInReflect,
};
use mp_bg::bg_panimate::{BG_InExtraDefenseSaberMove, BG_SuperBreakLoseAnim};
use mp_bg::bg_panimate::{
    BG_InGrappleMove, BG_InSaberLock, BG_KickingAnim, BG_SaberInAttack, BG_SaberInKata,
    BG_SaberInSpecial, BG_SaberInTransitionAny, BG_SaberStartTransAnim, BG_SuperBreakWinAnim,
    PM_InSaberAnim, PM_SaberInTransition,
};
use mp_bg::bg_pmove::BG_SabersOff;
use mp_bg::bg_saber::PM_SaberInBounce;
use mp_bg::bg_saber::PM_SaberInBrokenParry;
use mp_bg::bg_saberLoad::WP_SaberBladeDoTransitionDamage;
use mp_bg::public::set_anim::{SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE};

// --- pass-3 shard-2 body-fill imports (resolved owning modules per packet) ---
use crate::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, VectorClear,
    VectorSet,
};
use crate::saber::saber_face_t::saberFace_t;
use crate::saber::w_saber_consts::{
    SABER_REFLECT_MISSILE_CONE, SEF_BLOCKED, SEF_DEFLECTED, SEF_HITENEMY, SEF_HITOBJECT,
    SEF_HITWALL, SEF_PARRIED,
};
// --- pass-3 shard-1 (angles/lock/deflection/collide) body-fill imports ---
use crate::npc::g_npc_t::gNPC_t;
use crate::npc::look_mode::lookMode_t;
use crate::saber::saber_flags::SFL_NOT_LOCKABLE;
use mp_qshared::common::mp::qcommon::collision_record::{G2Trace_t, MAX_G2_COLLISIONS};
use mp_qshared::shared::saber_block_type::saberBlockType_t;
use crate::q_shared;

// w_saber.c file-local `#define`s used by this shard (matching the file's
// existing file-local const scoping, e.g. `SABER_NONATTACK_DAMAGE` above).
/// Source: `oracle/codemp/game/w_saber.c:1091`
const SABER_HITDAMAGE: c_int = 35;
/// Source: `oracle/codemp/game/w_saber.c:2850`
const SABER_EXTRAPOLATE_DIST: f32 = 16.0;
/// Source: `oracle/codemp/game/w_saber.c:5284`
const MAX_SABER_SWING_INC: f32 = 0.33;
/// Source: `oracle/codemp/game/w_saber.c:639`
const LOOK_DEFAULT_SPEED: f32 = 0.15;
// `saberBlockedType_t` variants used by this shard's bodies (stored as `c_int`
// in playerState), surfaced like the file's existing `BLOCKED_*` consts.
const BLOCKED_BOUNCE_MOVE: c_int = saberBlockedType_t::BLOCKED_BOUNCE_MOVE as c_int;
const BLOCKED_PARRY_BROKEN: c_int = saberBlockedType_t::BLOCKED_PARRY_BROKEN as c_int;
const BLOCKED_ATK_BOUNCE: c_int = saberBlockedType_t::BLOCKED_ATK_BOUNCE as c_int;

// Raven `#define`s local to `w_saber.c` itself (not `w_saber.h`), so they are
// not part of the `saber::w_saber_consts` header-const family; named here at
// their one call site, matching the oracle's file-local scoping.
/// Source: `oracle/codemp/game/w_saber.c:7`
const SABER_BOX_SIZE: f32 = 16.0;
/// Source: `oracle/codemp/game/w_saber.c:284`
const PROPER_THROWN_VALUE: c_int = 999;
/// Source: `oracle/codemp/game/w_saber.c:2235`
const SABER_NONATTACK_DAMAGE: c_int = 1;
/// Source: `oracle/codemp/game/w_saber.c:3503`
const MAX_SABER_VICTIMS: c_int = 16;
/// Source: `oracle/codemp/game/w_saber.c:5885`
const MIN_SABER_SLICE_DISTANCE: f32 = 50.0;
/// Source: `oracle/codemp/game/w_saber.c:5887`
const MIN_SABER_SLICE_RETURN_DISTANCE: f32 = 30.0;
/// Source: `oracle/codemp/game/w_saber.c:5889`
const SABER_THROWN_HIT_DAMAGE: c_int = 30;
/// Source: `oracle/codemp/game/w_saber.c:6337`
const MAX_LEAVE_TIME: c_int = 20000;
/// Source: `oracle/codemp/game/w_saber.c:6510`
// 3 seconds for now. This will leave you nice and open if you lose your saber.
const SABER_RETRIEVE_DELAY: c_int = 3000;
/// Source: `oracle/codemp/game/w_saber.c:7115`
const SABER_MAX_THROW_DISTANCE: f32 = 700.0;

// `saberBlockedType_t`/`weaponstate_t` are `#[repr(i32)]` enums, but the
// playerState `saberBlocked`/`weaponstate` fields are stored as `c_int` — so
// surface the variants used by the ported switch bodies as `c_int` consts.
const BLOCKED_NONE: c_int = saberBlockedType_t::BLOCKED_NONE as c_int;
const BLOCKED_UPPER_RIGHT: c_int = saberBlockedType_t::BLOCKED_UPPER_RIGHT as c_int;
const BLOCKED_UPPER_LEFT: c_int = saberBlockedType_t::BLOCKED_UPPER_LEFT as c_int;
const BLOCKED_LOWER_RIGHT: c_int = saberBlockedType_t::BLOCKED_LOWER_RIGHT as c_int;
const BLOCKED_LOWER_LEFT: c_int = saberBlockedType_t::BLOCKED_LOWER_LEFT as c_int;
const BLOCKED_TOP: c_int = saberBlockedType_t::BLOCKED_TOP as c_int;
const BLOCKED_UPPER_RIGHT_PROJ: c_int = saberBlockedType_t::BLOCKED_UPPER_RIGHT_PROJ as c_int;
const BLOCKED_UPPER_LEFT_PROJ: c_int = saberBlockedType_t::BLOCKED_UPPER_LEFT_PROJ as c_int;
const BLOCKED_LOWER_RIGHT_PROJ: c_int = saberBlockedType_t::BLOCKED_LOWER_RIGHT_PROJ as c_int;
const BLOCKED_LOWER_LEFT_PROJ: c_int = saberBlockedType_t::BLOCKED_LOWER_LEFT_PROJ as c_int;
const BLOCKED_TOP_PROJ: c_int = saberBlockedType_t::BLOCKED_TOP_PROJ as c_int;
const WEAPON_FIRING: c_int = weaponstate_t::WEAPON_FIRING as c_int;
const WEAPON_RAISING: c_int = weaponstate_t::WEAPON_RAISING as c_int;
const WEAPON_DROPPING: c_int = weaponstate_t::WEAPON_DROPPING as c_int;

// Raven `qboolean` is `c_int` (`qfalse == 0`, `qtrue == 1`); the lowercase
// `qtrue`/`qfalse` spellings are not exported here, so the ported bodies below
// return the bare `1`/`0` those constants alias.

/// Raven `RandFloat`.
///
/// Source: `oracle/codemp/game/w_saber.c:39-43`
pub fn RandFloat(ctx: &mut GameContext, min: f32, max: f32) -> f32 {
    // Raven (linux path): `((rand() * (max - min)) / (float)RAND_MAX) + min`.
    // The LCG lives on `bg_state.rng`.
    ((ctx.world.bg_state.rng.rand() as f32 * (max - min)) / RAND_MAX as f32) + min
}

/// Raven `G_DebugBoxLines`.
///
/// `DEBUG_SABER_BOX` is `#define`d unconditionally at `g_local.h:82`, so this
/// compiles into every oracle game TU; every call site is further gated at
/// runtime behind the `g_saberDebugBox` `CVAR_CHEAT` cvar (default `"0"`),
/// which is why normal play — including the combat corpus — never draws
/// these lines.
///
/// Source: `oracle/codemp/game/w_saber.c:46-78`
pub fn G_DebugBoxLines(ctx: &mut GameContext, mins: vec3_t, maxs: vec3_t, duration: c_int) {
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];

    let x = maxs[0] - mins[0];
    let y = maxs[1] - mins[1];

    // top of box
    start = maxs;
    end = maxs;
    start[0] -= x;
    G_TestLine(ctx, start, end, 0x00000ff, duration);
    end[0] = start[0];
    end[1] -= y;
    G_TestLine(ctx, start, end, 0x00000ff, duration);
    start[1] = end[1];
    start[0] += x;
    G_TestLine(ctx, start, end, 0x00000ff, duration);
    G_TestLine(ctx, start, maxs, 0x00000ff, duration);
    // bottom of box
    start = mins;
    end = mins;
    start[0] += x;
    G_TestLine(ctx, start, end, 0x00000ff, duration);
    end[0] = start[0];
    end[1] += y;
    G_TestLine(ctx, start, end, 0x00000ff, duration);
    start[1] = end[1];
    start[0] -= x;
    G_TestLine(ctx, start, end, 0x00000ff, duration);
    G_TestLine(ctx, start, mins, 0x00000ff, duration);
}

/// Raven `G_CanBeEnemy`.
///
/// Source: `oracle/codemp/game/w_saber.c:82-115`
pub fn G_CanBeEnemy(ctx: &mut GameContext, self_: EntityId, enemy: EntityId) -> bool {
    let (sc, ec, self_number, enemy_number) = {
        let se = ctx.world.entity(self_);
        let ee = ctx.world.entity(enemy);
        if se.inuse == 0 || ee.inuse == 0 || se.client.is_null() || ee.client.is_null() {
            return false;
        }
        (se.client, ee.client, se.s.number, ee.s.number)
    };

    // FLAG: pool clients (NPC-capable combatants); deref raw per recipe 2b.
    unsafe {
        if (*sc).ps.duelInProgress != 0 && (*sc).ps.duelIndex != enemy_number {
            // dueling but not with this person
            return false;
        }

        if (*ec).ps.duelInProgress != 0 && (*ec).ps.duelIndex != self_number {
            // other guy dueling but not with me
            return false;
        }
    }

    if ctx.world.cvars.g_gametype.integer < mp_bg::public::gametype::GT_TEAM {
        // ok, sure
        return true;
    }

    if ctx.world.cvars.g_friendlyFire.integer != 0 {
        // if ff on then can inflict damage normally on teammates
        return true;
    }

    if OnSameTeam(ctx, Some(self_), Some(enemy)) != 0 {
        // ff not on, don't hurt teammates
        return false;
    }

    true
}

/// Raven `G_SaberAttackPower`.
///
/// Source: `oracle/codemp/game/w_saber.c:120-217`
pub fn G_SaberAttackPower(ctx: &mut GameContext, ent: Option<EntityId>, attacking: bool) -> c_int {
    // Raven asserts a live entity here; every caller passes one.
    let ent = ent.unwrap();
    // FLAG: pool client (NPC-capable saber wielder); deref raw per recipe 2b.
    let client = ctx.world.entity(ent).client;
    unsafe {
        debug_assert!(!client.is_null());

        let mut baseLevel: c_int = (*client).ps.fd.saberAnimLevel;

        // Give "medium" strength for the two special stances.
        if baseLevel == saber_styles_t::SS_DUAL as c_int {
            baseLevel = 2;
        } else if baseLevel == saber_styles_t::SS_STAFF as c_int {
            baseLevel = 2;
        }

        if attacking {
            // The attacker gets a boost to help penetrate defense; general boost
            // up so the individual levels make a bigger difference.
            baseLevel *= 2;
            baseLevel += 1;

            // Get the "speed" of the swing, roughly, and add more power based on it.
            if (*client).lastSaberStorageTime >= (ctx.world.level.time - 50)
                && (*client).olderIsValid != 0
            {
                // Different "tolerance" per stance, else fast would have more
                // advantage than it should (its anims are much faster).
                let toleranceAmt: c_int = match (*client).ps.fd.saberAnimLevel {
                    x if x == saber_styles_t::SS_STRONG as c_int => 8,
                    x if x == saber_styles_t::SS_MEDIUM as c_int => 16,
                    x if x == saber_styles_t::SS_FAST as c_int => 24,
                    _ => 16, // dual, staff, etc.
                };

                let a = (*client).lastSaberBase_Always;
                let b = (*client).olderSaberBase;
                let vSub: vec3_t = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                let mut swingDist = VectorLength(vSub) as c_int;

                while swingDist > 0 {
                    baseLevel += 1;
                    swingDist -= toleranceAmt;
                }
            }

            if ctx.world.cvars.g_saberDebugPrint.integer > 1 {
                let s = format!(
                    "Client {}: ATT STR: {}\n",
                    ctx.world.entity(ent).s.number,
                    baseLevel
                );
                Com_Printf(&s);
            }
        }

        if ((*client).ps.brokenLimbs & (1 << (BROKENLIMB_RARM as c_int))) != 0
            || ((*client).ps.brokenLimbs & (1 << (BROKENLIMB_LARM as c_int))) != 0
        {
            // We're very weak when one of our arms is broken.
            baseLevel = (baseLevel as f64 * 0.3) as c_int;
        }

        // Cap at reasonable values now.
        if baseLevel < 1 {
            baseLevel = 1;
        } else if baseLevel > 16 {
            baseLevel = 16;
        }

        if ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
            && (*client).sess.duelTeam == DUELTEAM_LONE as c_int
        {
            // Get more power then.
            return baseLevel * 2;
        } else if attacking && ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            // In siege, saber battles should be quicker and biased to the attacker.
            return baseLevel * 3;
        }

        baseLevel
    }
}

/// Raven `WP_DeactivateSaber`.
///
/// Source: `oracle/codemp/game/w_saber.c:219-248`
pub fn WP_DeactivateSaber(ctx: &mut GameContext, self_: Option<EntityId>, clearLength: qboolean) {
    let _ = clearLength; // oracle's SetSaberLength(0) path is commented out.
    let self_ = match self_ {
        Some(e) => e,
        None => return,
    };
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let client = ctx.world.entity(self_).client;
    if client.is_null() {
        return;
    }
    unsafe {
        // keep my saber off!
        if (*client).ps.saberHolstered == 0 {
            (*client).ps.saberHolstered = 2;
            // Doesn't matter ATM (SetSaberLength commented out in oracle).
            if (*client).saber[0].soundOff != 0 {
                G_Sound(
                    ctx,
                    Some(self_),
                    CHAN_WEAPON as c_int,
                    (*client).saber[0].soundOff,
                );
            }

            if (*client).saber[1].soundOff != 0 && (*client).saber[1].model[0] != 0 {
                G_Sound(
                    ctx,
                    Some(self_),
                    CHAN_WEAPON as c_int,
                    (*client).saber[1].soundOff,
                );
            }
        }
    }
}

/// Raven `WP_ActivateSaber`.
///
/// Source: `oracle/codemp/game/w_saber.c:250-282`
pub fn WP_ActivateSaber(ctx: &mut GameContext, self_: Option<EntityId>) {
    let self_ = match self_ {
        Some(e) => e,
        None => return,
    };
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let client = ctx.world.entity(self_).client;
    if client.is_null() {
        return;
    }
    let level_time = ctx.world.level.time;
    let has_npc = !ctx.world.entity(self_).NPC.is_null();
    unsafe {
        if has_npc
            && (*client).ps.forceHandExtend == HANDEXTEND_JEDITAUNT as c_int
            && ((*client).ps.forceHandExtendTime - level_time) > 200
        {
            // if we're an NPC and in the middle of a taunt then stop it
            (*client).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
            (*client).ps.forceHandExtendTime = 0;
        } else if (*client).ps.fd.forceGripCripple != 0 {
            // can't activate saber while being gripped
            return;
        }

        if (*client).ps.saberHolstered != 0 {
            (*client).ps.saberHolstered = 0;
            if (*client).saber[0].soundOn != 0 {
                G_Sound(
                    ctx,
                    Some(self_),
                    CHAN_WEAPON as c_int,
                    (*client).saber[0].soundOn,
                );
            }

            if (*client).saber[1].soundOn != 0 {
                G_Sound(
                    ctx,
                    Some(self_),
                    CHAN_WEAPON as c_int,
                    (*client).saber[1].soundOn,
                );
            }
        }
    }
}

/// Raven `SaberUpdateSelf`.
///
/// Source: `oracle/codemp/game/w_saber.c:286-358`
pub fn SaberUpdateSelf(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    unsafe {
        let level_time = ctx.world.level.time;
        let owner_num = (*ent).r.ownerNum;

        if owner_num == ENTITYNUM_NONE {
            (*ent).think = Some(EntThink::G_FreeEntity).into();
            (*ent).nextthink = level_time;
            return;
        }

        let owner = &mut ctx.world.g_entities[owner_num as usize] as *mut gentity_t;

        if (*owner).inuse == 0 || (*owner).client.is_null() {
            (*ent).think = Some(EntThink::G_FreeEntity).into();
            (*ent).nextthink = level_time;
            return;
        }

        let oc = (*owner).client;

        if (*oc).ps.saberInFlight != 0 && (*owner).health > 0 {
            // let The Master take care of us now (we'll get treated like a missile until we return)
            (*ent).nextthink = level_time;
            (*ent).genericValue5 = PROPER_THROWN_VALUE;
            return;
        }

        (*ent).genericValue5 = 0;

        if (*oc).ps.weapon != WP_SABER
            || ((*oc).ps.pm_flags & PMF_FOLLOW) != 0
            || (*oc).sess.sessionTeam == TEAM_SPECTATOR
            || (*oc).tempSpectate >= level_time
            || (*owner).health < 1
            || BG_SabersOff(&mut (*oc).ps as *mut playerState_t) != 0
            || ((*oc).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] == 0
                && (*owner).s.eType != ET_NPC as c_int)
        {
            // owner is not using saber, spectating, dead, saber holstered, or has no attack level
            (*ent).r.contents = 0;
            (*ent).clipmask = 0;
        } else {
            // Standard contents (saber is active)
            if ctx.world.cvars.g_saberDebugBox.integer == 1
                || ctx.world.cvars.g_saberDebugBox.integer == 4
            {
                let mut dbgMins: vec3_t = [0.0; 3];
                let mut dbgMaxs: vec3_t = [0.0; 3];

                _VectorAdd((*ent).r.currentOrigin, (*ent).r.mins, &mut dbgMins);
                _VectorAdd((*ent).r.currentOrigin, (*ent).r.maxs, &mut dbgMaxs);

                let dbg_duration =
                    ((10.0f32 / ctx.world.cvars.g_svfps.integer as f32) * 100.0) as c_int;
                G_DebugBoxLines(ctx, dbgMins, dbgMaxs, dbg_duration);
            }
            if (*ent).r.contents != CONTENTS_LIGHTSABER {
                if (level_time - (*oc).lastSaberStorageTime) <= 200 {
                    // Only go back to solid once we're sure our owner has updated recently
                    (*ent).r.contents = CONTENTS_LIGHTSABER;
                    (*ent).clipmask = MASK_PLAYERSOLID | CONTENTS_LIGHTSABER;
                }
            } else {
                (*ent).r.contents = CONTENTS_LIGHTSABER;
                (*ent).clipmask = MASK_PLAYERSOLID | CONTENTS_LIGHTSABER;
            }
        }

        trap::LinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(ent.cast()),
        );

        (*ent).nextthink = level_time;
    }
}

/// Raven `SaberGotHit`.
///
/// Source: `oracle/codemp/game/w_saber.c:360-370`
pub fn SaberGotHit(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    let _ = (other, trace);
    // `own = &g_entities[self->r.ownerNum]` — an array slot, never null; the
    // oracle's `!own` guard is vacuous in Rust, only the client check bites.
    let owner_num = ctx.world.entity(self_).r.ownerNum;
    // FLAG: pool client (NPC-capable); read the pointer via the safe borrow.
    let own_client = ctx.world.entity(EntityId(owner_num as u32)).client;
    if own_client.is_null() {
        return;
    }
    // Raven no-op: projectiles are now handled in their own functions.
}

/// Raven `SetSaberBoxSize`.
///
/// Source: `oracle/codemp/game/w_saber.c:376-564`
pub fn SetSaberBoxSize(ctx: &mut GameContext, saberent: Option<EntityId>) {
    let level_time = ctx.world.level.time;
    let mut saberOrg: vec3_t = [0.0; 3];
    let mut saberTip: vec3_t = [0.0; 3];
    let mut j = 0i32;
    let mut k = 0i32;
    let mut dualSabers = qfalse;
    let mut alwaysBlock = [[qfalse; MAX_BLADES as usize]; MAX_SABERS as usize];
    let mut forceBlock = qfalse;

    // Raven asserts a live saber entity; every caller passes one.
    let saberent = saberent.unwrap();
    debug_assert!(ctx.world.entity(saberent).inuse != 0);

    let on = ctx.world.entity(saberent).r.ownerNum;
    let mut owner_id: Option<EntityId> = None;
    if on < (MAX_CLIENTS) as i32 && on >= 0 {
        owner_id = Some(EntityId(on as u32));
    } else if on >= 0
        && on < ENTITYNUM_WORLD
        && ctx.world.g_entities[on as usize].s.eType == ET_NPC as c_int
    {
        owner_id = Some(EntityId(on as u32));
    }

    let owner_id = match owner_id {
        Some(id) if ctx.world.entity(id).inuse != 0 && !ctx.world.entity(id).client.is_null() => id,
        _ => {
            debug_assert!(false, "Saber with no owner?");
            return;
        }
    };

    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let oc = ctx.world.entity(owner_id).client;

    unsafe {
        if (*oc).saber[1].model[0] != 0 {
            dualSabers = qtrue;
        }

        if PM_SaberInBrokenParry((*oc).ps.saberMove) != 0
            || BG_SuperBreakLoseAnim((*oc).ps.torsoAnim) != 0
        {
            // let swings go right through when we're in this state
            for i in 0..MAX_SABERS as usize {
                if i > 0 && dualSabers == 0 {
                    // not using a second saber, set it to not blocking
                    for jj in 0..MAX_BLADES as usize {
                        alwaysBlock[i][jj] = qfalse;
                    }
                } else {
                    if ((*oc).saber[i].saberFlags2 & SFL2_ALWAYS_BLOCK) != 0 {
                        for jj in 0..(*oc).saber[i].numBlades as usize {
                            alwaysBlock[i][jj] = qtrue;
                            forceBlock = qtrue;
                        }
                    }
                    if (*oc).saber[i].bladeStyle2Start > 0 {
                        for jj in (*oc).saber[i].bladeStyle2Start as usize
                            ..(*oc).saber[i].numBlades as usize
                        {
                            if ((*oc).saber[i].saberFlags2 & SFL2_ALWAYS_BLOCK2) != 0 {
                                alwaysBlock[i][jj] = qtrue;
                                forceBlock = qtrue;
                            } else {
                                alwaysBlock[i][jj] = qfalse;
                            }
                        }
                    }
                }
            }
            if forceBlock == 0 {
                // no sabers/blades to FORCE on, so turn off blocking altogether
                let e = ctx.world.entity_mut(saberent);
                e.r.mins = [0.0; 3];
                e.r.maxs = [0.0; 3];
                if ctx.world.cvars.g_saberDebugPrint.integer > 1 {
                    let s = format!(
                        "Client {} in broken parry, saber box 0\n",
                        ctx.world.entity(owner_id).s.number
                    );
                    Com_Printf(&s);
                }
                return;
            }
        }

        // Raven reads saber[j].blade[k] with j left over from the broken-parry+forceBlock
        // loops (terminal MAX_BLADES/numBlades) — an OOB read; the loops here use fresh
        // i/jj, so j and k stay 0 and we read saber[0].blade[0] as the defined behavior (§19).
        if (level_time - (*oc).lastSaberStorageTime) > 200
            || (level_time - (*oc).saber[j as usize].blade[k as usize].storageTime) > 100
        {
            // it's been too long since a reliable point storage, use defaults and leave.
            let e = ctx.world.entity_mut(saberent);
            e.r.mins = [-SABER_BOX_SIZE, -SABER_BOX_SIZE, -SABER_BOX_SIZE];
            e.r.maxs = [SABER_BOX_SIZE, SABER_BOX_SIZE, SABER_BOX_SIZE];
            return;
        }

        if dualSabers != 0 || (*oc).saber[0].numBlades > 1 {
            // dual sabers or multi-blade saber
            if (*oc).ps.saberHolstered > 1 {
                // entirely off - no blocking at all
                let e = ctx.world.entity_mut(saberent);
                e.r.mins = [0.0; 3];
                e.r.maxs = [0.0; 3];
                return;
            }
        } else {
            // single saber
            if (*oc).ps.saberHolstered != 0 {
                // off - no blocking at all
                let e = ctx.world.entity_mut(saberent);
                e.r.mins = [0.0; 3];
                e.r.maxs = [0.0; 3];
                return;
            }
        }

        // Start at the saber origin, then go through all the blades and push out the
        // extents, then set the box relative to the origin.
        {
            let e = ctx.world.entity_mut(saberent);
            e.r.mins = e.r.currentOrigin;
            e.r.maxs = e.r.currentOrigin;
        }

        for i in 0..3usize {
            j = 0;
            while j < (MAX_SABERS) as i32 {
                if (*oc).saber[j as usize].model[0] == 0 {
                    break;
                }
                if dualSabers != 0 && (*oc).ps.saberHolstered == 1 && j == 1 {
                    // this mother is holstered, get outta here.
                    j += 1;
                    continue;
                }
                k = 0;
                while k < (*oc).saber[j as usize].numBlades {
                    if k > 0
                        && dualSabers == 0
                        && (*oc).saber[j as usize].numBlades > 1
                        && (*oc).ps.saberHolstered == 1
                    {
                        // all blades after the first one are off
                        break;
                    }
                    if forceBlock != 0 && alwaysBlock[j as usize][k as usize] == 0 {
                        // this blade shouldn't be blocking
                        k += 1;
                        continue;
                    }
                    let (muzzlePoint, lengthMax, muzzleDir) = {
                        let blade = &(*oc).saber[j as usize].blade[k as usize];
                        (blade.muzzlePoint, blade.lengthMax, blade.muzzleDir)
                    };
                    saberOrg = muzzlePoint;
                    _VectorMA(muzzlePoint, lengthMax, muzzleDir, &mut saberTip);

                    if saberOrg[i] < ctx.world.entity(saberent).r.mins[i] {
                        ctx.world.entity_mut(saberent).r.mins[i] = saberOrg[i];
                    }
                    if saberTip[i] < ctx.world.entity(saberent).r.mins[i] {
                        ctx.world.entity_mut(saberent).r.mins[i] = saberTip[i];
                    }

                    if saberOrg[i] > ctx.world.entity(saberent).r.maxs[i] {
                        ctx.world.entity_mut(saberent).r.maxs[i] = saberOrg[i];
                    }
                    if saberTip[i] > ctx.world.entity(saberent).r.maxs[i] {
                        ctx.world.entity_mut(saberent).r.maxs[i] = saberTip[i];
                    }
                    k += 1;
                }
                j += 1;
            }
        }

        let e = ctx.world.entity_mut(saberent);
        let mins = e.r.mins;
        let maxs = e.r.maxs;
        let origin = e.r.currentOrigin;
        _VectorSubtract(mins, origin, &mut e.r.mins);
        _VectorSubtract(maxs, origin, &mut e.r.maxs);
    }
}

/// Raven `WP_SaberInitBladeData`.
///
/// Source: `oracle/codemp/game/w_saber.c:566-637`
pub fn WP_SaberInitBladeData(ctx: &mut GameContext, ent: EntityId) {
    let level_time = ctx.world.level.time;
    let mut saberent: Option<EntityId> = None;
    let mut i = 0;

    let ent_number = ctx.world.entity(ent).s.number;

    while i < ctx.world.level.num_entities {
        // make sure there are no other saber entities floating around that think
        // they belong to this client.
        let check_id = EntityId(i as u32);
        let ce = ctx.world.entity(check_id);
        let matches = ce.inuse != 0
            && ce.neverFree != 0
            && ce.r.ownerNum == ent_number
            && !ce.classname_str().is_empty()
            && Q_stricmp(&ce.classname_str(), "lightsaber") == 0;

        if matches {
            if saberent.is_some() {
                // already have one
                let ce = ctx.world.entity_mut(check_id);
                ce.neverFree = qfalse;
                ce.think = Some(EntThink::G_FreeEntity).into();
                ce.nextthink = level_time;
            } else {
                // take it as my own; free but don't issue a kg2.
                ctx.world.entity_mut(check_id).s.modelGhoul2 = 0;
                G_FreeEntity(ctx, Some(check_id));

                // now init it manually and reuse this ent slot.
                G_InitGentity(ctx, check_id);
                saberent = Some(check_id);
            }
        }

        i += 1;
    }

    let saberent = match saberent {
        Some(id) => id,
        None => {
            // ok, make one then
            let sp = G_Spawn(ctx);
            sp
        }
    };

    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let ec = ctx.world.entity(ent).client;
    let saber_number = ctx.world.entity(saberent).s.number;
    unsafe {
        (*ec).saberStoredIndex = saber_number;
        (*ec).ps.saberEntityNum = saber_number;
    }

    ctx.ent_set(saberent, PrefixSet::ClassnameStatic(c"lightsaber"));
    {
        let se = ctx.world.entity_mut(saberent);
        se.neverFree = qtrue; // the saber being removed would be terrible.
        se.r.svFlags = SVF_USE_CURRENT_ORIGIN;
        se.r.ownerNum = ent_number;
        se.clipmask = MASK_PLAYERSOLID | CONTENTS_LIGHTSABER;
        se.r.contents = CONTENTS_LIGHTSABER;
    }

    SetSaberBoxSize(ctx, Some(saberent));

    {
        let se = ctx.world.entity_mut(saberent);
        se.mass = 10.0;
        se.s.eFlags |= EF_NODRAW;
        se.r.svFlags |= SVF_NOCLIENT;
        se.s.modelGhoul2 = 1;
        se.touch = Some(EntTouch::SaberGotHit).into();
        se.think = Some(EntThink::SaberUpdateSelf).into();
        se.genericValue5 = 0;
        se.nextthink = level_time + 50;
    }

    ctx.world.globals.saberSpinSound =
        G_SoundIndex(ctx, "sound/weapons/saber/saberspin.wav");
}

/// Raven `G_CheckLookTarget`.
///
/// Source: `oracle/codemp/game/w_saber.c:642-724`
pub fn G_CheckLookTarget(
    ctx: &mut GameContext,
    ent: EntityId,
    lookAngles: &mut vec3_t,
    lookingSpeed: *mut f32,
) -> bool {
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let sc = ctx.world.entity(ent).client;

    // an NPC bolted to a vehicle should just look around randomly
    let is_bolted_npc = {
        let e = ctx.world.entity(ent);
        e.s.eType == ET_NPC as c_int
            && e.s.m_iVehicleNum != 0
            && e.s.NPC_class != CLASS_VEHICLE as c_int
    };
    if is_bolted_npc {
        let look_around = cstr("lookAround");
        if TIMER_Done(ctx, Some(ent), look_around.as_ptr()) != 0 {
            // FLAG: gNPC_t (NPCInfo) has no accessor; deref stays raw (recipe 2c).
            let npc = ctx.world.entity(ent).NPC;
            let yaw = ctx.world.bg_state.rng.flrand(0.0, 360.0);
            unsafe {
                (*npc).shootAngles[YAW as usize] = yaw;
            }
            let look_time = ctx.world.bg_state.rng.Q_irand(500, 3000);
            TIMER_Set(ctx, Some(ent), look_around.as_ptr(), look_time);
        }
        // FLAG: gNPC_t (NPCInfo); deref raw.
        let npc = ctx.world.entity(ent).NPC;
        let shoot_yaw = unsafe { (*npc).shootAngles[YAW as usize] };
        VectorSet(lookAngles, 0.0, shoot_yaw, 0.0);
        return true;
    }

    // Now calc head angle to lookTarget, if any
    unsafe {
        let look_target = (*sc).renderInfo.lookTarget;
        if look_target >= 0 && look_target < ENTITYNUM_WORLD {
            let mut lookDir: vec3_t = [0.0; 3];
            let mut lookOrg: vec3_t = [0.0; 3];
            let mut eyeOrg: vec3_t = [0.0; 3];

            if (*sc).renderInfo.lookMode == lookMode_t::LM_ENT {
                // `lookCent = &g_entities[lookTarget]` — an array slot, never null.
                let look_cent = EntityId(look_target as u32);
                // `enemy` is `Option<EntityId>`; identity-compare by id.
                if ctx.world.entity(ent).enemy != Some(look_cent) {
                    // We turn heads faster than headbob speed, but not as fast
                    // as if watching an enemy
                    *lookingSpeed = LOOK_DEFAULT_SPEED;
                }

                // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
                let lcc = ctx.world.entity(look_cent).client;
                if !lcc.is_null() {
                    lookOrg = (*lcc).renderInfo.eyePoint;
                } else if ctx.world.entity(look_cent).inuse != 0
                    && !VectorCompare(ctx.world.entity(look_cent).r.currentOrigin, vec3_origin)
                {
                    lookOrg = ctx.world.entity(look_cent).r.currentOrigin;
                } else {
                    // at origin of world
                    return false;
                }
            } else if (*sc).renderInfo.lookMode == lookMode_t::LM_INTEREST
                && look_target > -1
                && look_target < MAX_INTEREST_POINTS as c_int
            {
                lookOrg = ctx.world.level.interestPoints[look_target as usize].origin;
            } else {
                return false;
            }

            eyeOrg = (*sc).renderInfo.eyePoint;

            _VectorSubtract(lookOrg, eyeOrg, &mut lookDir);

            vectoangles(lookDir, lookAngles);

            for i in 0..3usize {
                lookAngles[i] = AngleNormalize180(lookAngles[i]);
                (*sc).renderInfo.eyeAngles[i] = AngleNormalize180((*sc).renderInfo.eyeAngles[i]);
            }
            let la = *lookAngles;
            AnglesSubtract(la, (*sc).renderInfo.eyeAngles, lookAngles);
            return true;
        }

        false
    }
}

/// Raven `G_G2NPCAngles`.
///
/// Source: `oracle/codemp/game/w_saber.c:732-879`
pub fn G_G2NPCAngles(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    legs: *mut vec3_t,
    angles: &mut vec3_t,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), ent) };
    unsafe {
        let cranium_bone = cstr("cranium");
        let thoracic_bone = cstr("thoracic"); // only used by atst so doesn't need a case
        let mut looking = false;
        let mut viewAngles: vec3_t = [0.0; 3];
        let mut lookAngles: vec3_t = [0.0; 3];

        if (*ent).client.is_null() {
            return;
        }
        let sc = (*ent).client;

        if (*sc).NPC_class == CLASS_PROBE
            || (*sc).NPC_class == CLASS_R2D2
            || (*sc).NPC_class == CLASS_R5D2
            || (*sc).NPC_class == CLASS_ATST
        {
            // Raven leaves this local uninitialized (the CG_ATSTLegsYaw call is
            // commented out); zero-init is the chosen defined behavior (§19).
            let mut trailingLegsAngles: vec3_t = [0.0; 3];

            if (*ent).s.eType == ET_NPC as c_int
                && (*ent).s.m_iVehicleNum != 0
                && (*ent).s.NPC_class != CLASS_VEHICLE as c_int
            {
                // an NPC bolted to a vehicle should use the full angles
                *angles = (*ent).r.currentAngles;
            } else {
                *angles = (*sc).ps.viewangles;
                angles[PITCH as usize] = 0.0;
            }

            viewAngles = (*sc).ps.viewangles;
            viewAngles[PITCH as usize] = (viewAngles[PITCH as usize] as f64 * 0.5) as f32;
            lookAngles = viewAngles;

            lookAngles[1] = 0.0;

            if (*sc).NPC_class == CLASS_ATST {
                // body pitch
                NPC_SetBoneAngles(
                    ctx,
                    ctx.entity_id_of(ent).unwrap(),
                    thoracic_bone.as_ptr() as *mut c_char,
                    lookAngles,
                );
            }

            lookAngles = viewAngles;

            if !ent.is_null() && !(*ent).client.is_null() && (*sc).NPC_class == CLASS_ATST {
                // CG_ATSTLegsYaw( cent, trailingLegsAngles );
                AnglesToAxis(trailingLegsAngles, legs);
            } else {
                // FIXME: this needs to properly set the legs.yawing field (client-side
                // block dropped: not reachable from the game module).
            }

            {
                // look at lookTarget!
                let mut lookingSpeed = 0.3f32;
                looking = G_CheckLookTarget(
                    ctx,
                    ctx.entity_id_of(ent).unwrap(),
                    &mut lookAngles,
                    &mut lookingSpeed,
                );
                lookAngles[PITCH as usize] = 0.0;
                lookAngles[ROLL as usize] = 0.0; // droids can't pitch or roll their heads
                if looking {
                    // keep doing this lerp behavior for a full second after stopped looking
                    (*sc).renderInfo.lookingDebounceTime = ctx.world.level.time + 1000;
                }
            }
            if (*sc).renderInfo.lookingDebounceTime > ctx.world.level.time {
                // adjust for current body orientation
                let mut oldLookAngles: vec3_t = [0.0; 3];

                lookAngles[YAW as usize] -= 0.0;

                // normalize
                lookAngles[YAW as usize] = AngleNormalize180(lookAngles[YAW as usize]);

                // slowly lerp to this new value; remember last headAngles
                oldLookAngles = (*sc).renderInfo.lastHeadAngles;
                if !VectorCompare(oldLookAngles, lookAngles) {
                    lookAngles[YAW as usize] = oldLookAngles[YAW as usize]
                        + (lookAngles[YAW as usize] - oldLookAngles[YAW as usize]) * 0.4f32;
                }
                // Remember current lookAngles next time
                (*sc).renderInfo.lastHeadAngles = lookAngles;
            } else {
                // Remember current lookAngles next time
                (*sc).renderInfo.lastHeadAngles = lookAngles;
            }
            if (*sc).NPC_class == CLASS_ATST {
                lookAngles = (*sc).ps.viewangles;
                lookAngles[0] = 0.0;
                lookAngles[2] = 0.0;
                lookAngles[YAW as usize] -= trailingLegsAngles[YAW as usize];
            } else {
                lookAngles[PITCH as usize] = 0.0;
                lookAngles[ROLL as usize] = 0.0;
                lookAngles[YAW as usize] -= (*sc).ps.viewangles[YAW as usize];
            }

            NPC_SetBoneAngles(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                cranium_bone.as_ptr() as *mut c_char,
                lookAngles,
            );
        }
        let _ = looking;
    }
}

/// Raven `G_G2PlayerAngles`.
///
/// Source: `oracle/codemp/game/w_saber.c:881-1034`
pub fn G_G2PlayerAngles(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    legs: *mut vec3_t,
    legsAngles: &mut vec3_t,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), ent) };
    unsafe {
        let sc = (*ent).client;
        let mut tPitching: qboolean = qfalse;
        let mut tYawing: qboolean = qfalse;
        let mut lYawing: qboolean = qfalse;
        let mut tYawAngle: f32 = (*sc).ps.viewangles[YAW as usize];
        let mut tPitchAngle: f32 = 0.0;
        let mut lYawAngle: f32 = (*sc).ps.viewangles[YAW as usize];

        let ciLegs = (*sc).ps.legsAnim;
        let ciTorso = (*sc).ps.torsoAnim;

        let mut turAngles: vec3_t = [0.0; 3];
        let mut lerpOrg: vec3_t = [0.0; 3];
        let mut lerpAng: vec3_t = [0.0; 3];

        if (*ent).s.eType == ET_NPC as c_int && !(*ent).client.is_null() {
            // sort of hacky, but it saves a pretty big load off the server
            let mut i: c_int = 0;

            // If no real clients are in the same PVS then don't do any of this stuff
            while i < MAX_CLIENTS as c_int {
                let clEnt = &mut ctx.world.g_entities[i as usize] as *mut gentity_t;

                if !clEnt.is_null()
                    && (*clEnt).inuse != 0
                    && !(*clEnt).client.is_null()
                    && trap::InPVS(
                        ctx.engine,
                        GInPvsArgs::new(
                            &(*((*clEnt).client)).ps.origin as *const vec3_t,
                            &(*sc).ps.origin as *const vec3_t,
                        ),
                    ) != 0
                {
                    // this client can see him
                    break;
                }

                i += 1;
            }

            if i == MAX_CLIENTS as c_int {
                // no one can see him, just return
                return;
            }
        }

        lerpOrg = (*sc).ps.origin;
        lerpAng = (*sc).ps.viewangles;

        if (*ent).localAnimIndex <= 1 {
            // don't do these things on non-humanoids
            let mut lookAngles: vec3_t = [0.0; 3];
            let mut emplaced: *mut entityState_t = core::ptr::null_mut();

            if (*sc).ps.hasLookTarget != 0 {
                _VectorSubtract(
                    ctx.world.g_entities[(*sc).ps.lookTarget as usize]
                        .r
                        .currentOrigin,
                    (*sc).ps.origin,
                    &mut lookAngles,
                );
                let la = lookAngles;
                vectoangles(la, &mut lookAngles);
                (*sc).lookTime = ctx.world.level.time + 1000;
            } else {
                lookAngles = (*sc).ps.origin;
            }
            lookAngles[PITCH as usize] = 0.0;
            // Referee probe: caller-side look-target angles + hasLookTarget/lookTime.
            probe!(
                "LOOK_TGT",
                "t={} en={} hlt={} lkt={} li={:08x},{:08x},{:08x}",
                ctx.world.level.time,
                (*ent).s.number,
                (*sc).ps.hasLookTarget,
                (*sc).lookTime,
                lookAngles[0].to_bits(),
                lookAngles[1].to_bits(),
                lookAngles[2].to_bits(),
            );

            if (*sc).ps.emplacedIndex != 0 {
                emplaced = &mut ctx.world.g_entities[(*sc).ps.emplacedIndex as usize].s
                    as *mut entityState_t;
            }

            mp_bg::bg_pmove::BG_G2PlayerAngles(
                (*ent).ghoul2,
                (*sc).renderInfo.motionBolt,
                &mut (*ent).s as *mut entityState_t,
                ctx.world.level.time,
                lerpOrg,
                lerpAng,
                legs,
                legsAngles,
                &mut tYawing as *mut qboolean,
                &mut tPitching as *mut qboolean,
                &mut lYawing as *mut qboolean,
                &mut tYawAngle as *mut f32,
                &mut tPitchAngle as *mut f32,
                &mut lYawAngle as *mut f32,
                FRAMETIME,
                &mut turAngles,
                (*ent).modelScale,
                ciLegs,
                ciTorso,
                &mut (*sc).corrTime as *mut c_int,
                lookAngles,
                &mut (*sc).lastHeadAngles,
                (*sc).lookTime,
                emplaced,
                core::ptr::null_mut(),
                &GameBgTraps::new(ctx.engine),
            );

            if (*sc).ps.heldByClient != 0 && (*sc).ps.heldByClient <= MAX_CLIENTS as c_int {
                // then put our arm in this client's hand (index+1 because index 0 is valid)
                let heldByIndex = (*sc).ps.heldByClient - 1;
                let other = &mut ctx.world.g_entities[heldByIndex as usize] as *mut gentity_t;
                let mut lHandBolt: c_int = 0;

                if !other.is_null()
                    && (*other).inuse != 0
                    && !(*other).client.is_null()
                    && !(*other).ghoul2.is_null()
                {
                    lHandBolt = trap::G2API_AddBolt(ctx.engine, (*other).ghoul2, 0, "*l_hand");
                } else {
                    // they left the game, perhaps?
                    (*sc).ps.heldByClient = 0;
                    return;
                }

                if lHandBolt != 0 {
                    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
                    let mut boltOrg: vec3_t = [0.0; 3];
                    let mut tAngles: vec3_t = [0.0; 3];

                    let oc = (*other).client;
                    tAngles = (*oc).ps.viewangles;
                    tAngles[PITCH as usize] = 0.0;
                    tAngles[ROLL as usize] = 0.0;

                    trap::G2API_GetBoltMatrix(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                            (*other).ghoul2,
                            0,
                            lHandBolt,
                            &mut boltMatrix as *mut mdxaBone_t,
                            &tAngles as *const vec3_t,
                            &(*oc).ps.origin as *const vec3_t,
                            ctx.world.level.time,
                            core::ptr::null_mut(),
                            &(*other).modelScale as *const vec3_t,
                        ),
                    );
                    boltOrg[0] = boltMatrix.matrix[0][3];
                    boltOrg[1] = boltMatrix.matrix[1][3];
                    boltOrg[2] = boltMatrix.matrix[2][3];

                    mp_bg::bg_pmove::BG_IK_MoveArm(
                        (*ent).ghoul2,
                        lHandBolt,
                        ctx.world.level.time,
                        &mut (*ent).s as *mut entityState_t,
                        (*sc).ps.torsoAnim,
                        boltOrg,
                        &mut (*sc).ikStatus as *mut qboolean,
                        (*sc).ps.origin,
                        (*sc).ps.viewangles,
                        (*ent).modelScale,
                        500,
                        qfalse,
                        &ctx.world.bg_state,
                        &GameBgTraps::new(ctx.engine),
                    );
                }
            } else if (*sc).ikStatus != 0 {
                // make sure we aren't IKing if we don't have anyone to hold onto us.
                let mut lHandBolt: c_int = 0;

                if !ent.is_null()
                    && (*ent).inuse != 0
                    && !(*ent).client.is_null()
                    && !(*ent).ghoul2.is_null()
                {
                    lHandBolt = trap::G2API_AddBolt(ctx.engine, (*ent).ghoul2, 0, "*l_hand");
                } else {
                    // failsafe
                    (*sc).ikStatus = qfalse;
                }

                if lHandBolt != 0 {
                    mp_bg::bg_pmove::BG_IK_MoveArm(
                        (*ent).ghoul2,
                        lHandBolt,
                        ctx.world.level.time,
                        &mut (*ent).s as *mut entityState_t,
                        (*sc).ps.torsoAnim,
                        vec3_origin,
                        &mut (*sc).ikStatus as *mut qboolean,
                        (*sc).ps.origin,
                        (*sc).ps.viewangles,
                        (*ent).modelScale,
                        500,
                        qtrue,
                        &ctx.world.bg_state,
                        &GameBgTraps::new(ctx.engine),
                    );
                }
            }
        } else if !(*ent).m_pVehicle.is_null()
            && (*((*((*ent).m_pVehicle)).m_pVehicleInfo
                as *mut mp_bg::vehicles::vehicle_info_t::vehicleInfo_t))
                .r#type
                == VH_WALKER
        {
            let mut lookAngles: vec3_t = [0.0; 3];

            *legsAngles = (*sc).ps.viewangles;
            legsAngles[PITCH as usize] = 0.0;
            AnglesToAxis(*legsAngles, legs);

            lookAngles = (*sc).ps.viewangles;
            lookAngles[YAW as usize] = 0.0;
            lookAngles[ROLL as usize] = 0.0;

            mp_bg::bg_pmove::BG_G2ATSTAngles(
                (*ent).ghoul2,
                ctx.world.level.time,
                lookAngles,
                &GameBgTraps::new(ctx.engine),
            );
        } else if !(*ent).NPC.is_null() {
            // an NPC not using a humanoid skeleton, do special angle stuff.
            if (*ent).s.eType == ET_NPC as c_int
                && (*ent).s.NPC_class == CLASS_VEHICLE as c_int
                && !(*ent).m_pVehicle.is_null()
                && (*((*((*ent).m_pVehicle)).m_pVehicleInfo
                    as *mut mp_bg::vehicles::vehicle_info_t::vehicleInfo_t))
                    .r#type
                    == VH_FIGHTER
            {
                // fighters take pitch and roll into account for the axial angles
                *legsAngles = (*sc).ps.viewangles;
                AnglesToAxis(*legsAngles, legs);
            } else {
                G_G2NPCAngles(ctx, ctx.entity_id_of(ent), legs, legsAngles);
            }
        }
    }
}

/// Raven `SaberAttacking`.
///
/// Source: `oracle/codemp/game/w_saber.c:1036-1073`
pub fn SaberAttacking(self_: &gentity_t) -> bool {
    let client = self_.client;
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let (saberMove, weaponstate, saberBlocked) = unsafe {
        let ps = &(*client).ps;
        (ps.saberMove, ps.weaponstate, ps.saberBlocked)
    };
    if mp_bg::bg_panimate::PM_SaberInParry(saberMove) != 0 {
        return false;
    }
    if mp_bg::bg_saber::PM_SaberInBrokenParry(saberMove) != 0 {
        return false;
    }
    if mp_bg::bg_panimate::PM_SaberInDeflect(saberMove) != 0 {
        return false;
    }
    if mp_bg::bg_saber::PM_SaberInBounce(saberMove) != 0 {
        return false;
    }
    if mp_bg::bg_panimate::PM_SaberInKnockaway(saberMove) != 0 {
        return false;
    }

    if mp_bg::bg_panimate::BG_SaberInAttack(saberMove) != 0
        && weaponstate == WEAPON_FIRING as c_int
        && saberBlocked == BLOCKED_NONE as c_int
    {
        // if we're firing and not blocking, then we're attacking.
        return true;
    }

    if mp_bg::bg_panimate::BG_SaberInSpecial(saberMove) != 0 {
        return true;
    }

    false
}

/// Raven `G_SaberLockAnim`.
///
/// Source: `oracle/codemp/game/w_saber.c:1094-1211`
pub fn G_SaberLockAnim(
    attackerSaberStyle: c_int,
    defenderSaberStyle: c_int,
    topOrSide: c_int,
    lockOrBreakOrSuperBreak: c_int,
    winOrLose: c_int,
) -> c_int {
    // `BOTH_LK_*` are `animNumber_t` enum variants; `baseAnim` is arithmetic
    // `int` in Raven, so each seed is taken as its `c_int` discriminant.
    let ss_fast = saber_styles_t::SS_FAST as c_int;
    let ss_tavion = saber_styles_t::SS_TAVION as c_int;
    let ss_dual = saber_styles_t::SS_DUAL as c_int;
    let ss_staff = saber_styles_t::SS_STAFF as c_int;

    let mut baseAnim: c_int = -1;
    if lockOrBreakOrSuperBreak == SABERLOCK_LOCK {
        // special case: if we're using the same style and locking
        if attackerSaberStyle == defenderSaberStyle
            || (attackerSaberStyle >= ss_fast
                && attackerSaberStyle <= ss_tavion
                && defenderSaberStyle >= ss_fast
                && defenderSaberStyle <= ss_tavion)
        {
            // using same style
            if winOrLose == SABERLOCK_LOSE {
                // you want the defender's stance...
                if defenderSaberStyle == ss_dual {
                    baseAnim = if topOrSide == SABERLOCK_TOP {
                        animNumber_t::BOTH_LK_DL_DL_T_L_2 as c_int
                    } else {
                        animNumber_t::BOTH_LK_DL_DL_S_L_2 as c_int
                    };
                } else if defenderSaberStyle == ss_staff {
                    baseAnim = if topOrSide == SABERLOCK_TOP {
                        animNumber_t::BOTH_LK_ST_ST_T_L_2 as c_int
                    } else {
                        animNumber_t::BOTH_LK_ST_ST_S_L_2 as c_int
                    };
                } else {
                    baseAnim = if topOrSide == SABERLOCK_TOP {
                        animNumber_t::BOTH_LK_S_S_T_L_2 as c_int
                    } else {
                        animNumber_t::BOTH_LK_S_S_S_L_2 as c_int
                    };
                }
            }
        }
    }
    if baseAnim == -1 {
        if attackerSaberStyle == ss_dual {
            baseAnim = if defenderSaberStyle == ss_dual {
                animNumber_t::BOTH_LK_DL_DL_S_B_1_L as c_int
            } else if defenderSaberStyle == ss_staff {
                animNumber_t::BOTH_LK_DL_ST_S_B_1_L as c_int
            } else {
                animNumber_t::BOTH_LK_DL_S_S_B_1_L as c_int // single
            };
        } else if attackerSaberStyle == ss_staff {
            baseAnim = if defenderSaberStyle == ss_dual {
                animNumber_t::BOTH_LK_ST_DL_S_B_1_L as c_int
            } else if defenderSaberStyle == ss_staff {
                animNumber_t::BOTH_LK_ST_ST_S_B_1_L as c_int
            } else {
                animNumber_t::BOTH_LK_ST_S_S_B_1_L as c_int // single
            };
        } else {
            // single
            baseAnim = if defenderSaberStyle == ss_dual {
                animNumber_t::BOTH_LK_S_DL_S_B_1_L as c_int
            } else if defenderSaberStyle == ss_staff {
                animNumber_t::BOTH_LK_S_ST_S_B_1_L as c_int
            } else {
                animNumber_t::BOTH_LK_S_S_S_B_1_L as c_int // single
            };
        }
        // side lock or top lock?
        if topOrSide == SABERLOCK_TOP {
            baseAnim += 5;
        }
        // lock, break or superbreak?
        if lockOrBreakOrSuperBreak == SABERLOCK_LOCK {
            baseAnim += 2;
        } else {
            // a break or superbreak
            if lockOrBreakOrSuperBreak == SABERLOCK_SUPERBREAK {
                baseAnim += 3;
            }
            // winner or loser?
            if winOrLose == SABERLOCK_WIN {
                baseAnim += 1;
            }
        }
    }
    baseAnim
}

/// Raven `WP_SabersCheckLock2`.
///
/// Source: `oracle/codemp/game/w_saber.c:1218-1460`
pub fn WP_SabersCheckLock2(
    ctx: &mut GameContext,
    attacker: EntityId,
    defender: EntityId,
    mut lockMode: sabersLockMode_t,
) -> bool {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let attacker: *mut gentity_t = ctx.entity_mut(attacker);
    let defender: *mut gentity_t = ctx.entity_mut(defender);
    unsafe {
        let ac = (*attacker).client;
        let dc = (*defender).client;

        let mut attAnim: c_int = 0;
        let mut defAnim: c_int = 0;
        let mut attStart: f32 = 0.5f32;
        let mut defStart: f32 = 0.5f32;
        let mut idealDist: f32 = 48.0f32;
        let mut attAngles: vec3_t = [0.0; 3];
        let mut defAngles: vec3_t = [0.0; 3];
        let mut defDir: vec3_t = [0.0; 3];
        let mut newOrg: vec3_t = [0.0; 3];
        let mut attDir: vec3_t = [0.0; 3];
        let mut diff: f32 = 0.0;
        let mut trace: trace_t = core::mem::zeroed();

        let ss_fast = saber_styles_t::SS_FAST as c_int;
        let ss_tavion = saber_styles_t::SS_TAVION as c_int;

        // MATCH ANIMS
        if lockMode == LOCK_RANDOM {
            lockMode = core::mem::transmute::<c_int, sabersLockMode_t>(
                ctx.world
                    .bg_state
                    .rng
                    .Q_irand(LOCK_FIRST as c_int, (LOCK_RANDOM as c_int) - 1),
            );
        }
        if (*ac).ps.fd.saberAnimLevel >= ss_fast
            && (*ac).ps.fd.saberAnimLevel <= ss_tavion
            && (*dc).ps.fd.saberAnimLevel >= ss_fast
            && (*dc).ps.fd.saberAnimLevel <= ss_tavion
        {
            // 2 single sabers?  Just do it the old way...
            if lockMode == LOCK_TOP {
                attAnim = BOTH_BF2LOCK as c_int;
                defAnim = BOTH_BF1LOCK as c_int;
                attStart = 0.5f32;
                defStart = 0.5f32;
                idealDist = LOCK_IDEAL_DIST_TOP;
            } else if lockMode == LOCK_DIAG_TR {
                attAnim = BOTH_CCWCIRCLELOCK as c_int;
                defAnim = BOTH_CWCIRCLELOCK as c_int;
                attStart = 0.5f32;
                defStart = 0.5f32;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            } else if lockMode == LOCK_DIAG_TL {
                attAnim = BOTH_CWCIRCLELOCK as c_int;
                defAnim = BOTH_CCWCIRCLELOCK as c_int;
                attStart = 0.5f32;
                defStart = 0.5f32;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            } else if lockMode == LOCK_DIAG_BR {
                attAnim = BOTH_CWCIRCLELOCK as c_int;
                defAnim = BOTH_CCWCIRCLELOCK as c_int;
                attStart = 0.85f32;
                defStart = 0.85f32;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            } else if lockMode == LOCK_DIAG_BL {
                attAnim = BOTH_CCWCIRCLELOCK as c_int;
                defAnim = BOTH_CWCIRCLELOCK as c_int;
                attStart = 0.85f32;
                defStart = 0.85f32;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            } else if lockMode == LOCK_R {
                attAnim = BOTH_CCWCIRCLELOCK as c_int;
                defAnim = BOTH_CWCIRCLELOCK as c_int;
                attStart = 0.75f32;
                defStart = 0.75f32;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            } else if lockMode == LOCK_L {
                attAnim = BOTH_CWCIRCLELOCK as c_int;
                defAnim = BOTH_CCWCIRCLELOCK as c_int;
                attStart = 0.75f32;
                defStart = 0.75f32;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            } else {
                return false;
            }
        } else {
            // use the new system — all new saberlocks are 46.08 apart
            idealDist = LOCK_IDEAL_DIST_JKA;
            if lockMode == LOCK_TOP {
                // top lock
                attAnim = G_SaberLockAnim(
                    (*ac).ps.fd.saberAnimLevel,
                    (*dc).ps.fd.saberAnimLevel,
                    SABERLOCK_TOP,
                    SABERLOCK_LOCK,
                    SABERLOCK_WIN,
                );
                defAnim = G_SaberLockAnim(
                    (*dc).ps.fd.saberAnimLevel,
                    (*ac).ps.fd.saberAnimLevel,
                    SABERLOCK_TOP,
                    SABERLOCK_LOCK,
                    SABERLOCK_LOSE,
                );
                attStart = 0.5f32;
                defStart = 0.5f32;
            } else {
                // side lock
                if lockMode == LOCK_DIAG_TR {
                    attAnim = G_SaberLockAnim(
                        (*ac).ps.fd.saberAnimLevel,
                        (*dc).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    defAnim = G_SaberLockAnim(
                        (*dc).ps.fd.saberAnimLevel,
                        (*ac).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    attStart = 0.5f32;
                    defStart = 0.5f32;
                } else if lockMode == LOCK_DIAG_TL {
                    attAnim = G_SaberLockAnim(
                        (*ac).ps.fd.saberAnimLevel,
                        (*dc).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    defAnim = G_SaberLockAnim(
                        (*dc).ps.fd.saberAnimLevel,
                        (*ac).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    attStart = 0.5f32;
                    defStart = 0.5f32;
                } else if lockMode == LOCK_DIAG_BR {
                    attAnim = G_SaberLockAnim(
                        (*ac).ps.fd.saberAnimLevel,
                        (*dc).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    defAnim = G_SaberLockAnim(
                        (*dc).ps.fd.saberAnimLevel,
                        (*ac).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    if mp_bg::bg_saber::BG_CheckIncrementLockAnim(attAnim, SABERLOCK_WIN) != 0 {
                        attStart = 0.85f32;
                    } else {
                        attStart = 0.15f32;
                    }
                    if mp_bg::bg_saber::BG_CheckIncrementLockAnim(defAnim, SABERLOCK_LOSE) != 0 {
                        defStart = 0.85f32;
                    } else {
                        defStart = 0.15f32;
                    }
                } else if lockMode == LOCK_DIAG_BL {
                    attAnim = G_SaberLockAnim(
                        (*ac).ps.fd.saberAnimLevel,
                        (*dc).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    defAnim = G_SaberLockAnim(
                        (*dc).ps.fd.saberAnimLevel,
                        (*ac).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    if mp_bg::bg_saber::BG_CheckIncrementLockAnim(attAnim, SABERLOCK_WIN) != 0 {
                        attStart = 0.85f32;
                    } else {
                        attStart = 0.15f32;
                    }
                    if mp_bg::bg_saber::BG_CheckIncrementLockAnim(defAnim, SABERLOCK_LOSE) != 0 {
                        defStart = 0.85f32;
                    } else {
                        defStart = 0.15f32;
                    }
                } else if lockMode == LOCK_R {
                    attAnim = G_SaberLockAnim(
                        (*ac).ps.fd.saberAnimLevel,
                        (*dc).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    defAnim = G_SaberLockAnim(
                        (*dc).ps.fd.saberAnimLevel,
                        (*ac).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    if mp_bg::bg_saber::BG_CheckIncrementLockAnim(attAnim, SABERLOCK_WIN) != 0 {
                        attStart = 0.75f32;
                    } else {
                        attStart = 0.25f32;
                    }
                    if mp_bg::bg_saber::BG_CheckIncrementLockAnim(defAnim, SABERLOCK_LOSE) != 0 {
                        defStart = 0.75f32;
                    } else {
                        defStart = 0.25f32;
                    }
                } else if lockMode == LOCK_L {
                    attAnim = G_SaberLockAnim(
                        (*ac).ps.fd.saberAnimLevel,
                        (*dc).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    defAnim = G_SaberLockAnim(
                        (*dc).ps.fd.saberAnimLevel,
                        (*ac).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    // attacker starts with advantage
                    if mp_bg::bg_saber::BG_CheckIncrementLockAnim(attAnim, SABERLOCK_WIN) != 0 {
                        attStart = 0.75f32;
                    } else {
                        attStart = 0.25f32;
                    }
                    if mp_bg::bg_saber::BG_CheckIncrementLockAnim(defAnim, SABERLOCK_LOSE) != 0 {
                        defStart = 0.75f32;
                    } else {
                        defStart = 0.25f32;
                    }
                } else {
                    return false;
                }
            }
        }

        G_SetAnim(
            ctx,
            ctx.entity_id_of(attacker).unwrap(),
            core::ptr::null_mut(),
            SETANIM_BOTH,
            attAnim,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            0,
        );
        {
            let anim = &*(&ctx.world.bg_state.bgAllAnims)[(*attacker).localAnimIndex as usize]
                .anims
                .add(attAnim as usize);
            (*ac).ps.saberLockFrame =
                anim.firstFrame as c_int + (anim.numFrames as f32 * attStart) as c_int;
        }

        G_SetAnim(
            ctx,
            ctx.entity_id_of(defender).unwrap(),
            core::ptr::null_mut(),
            SETANIM_BOTH,
            defAnim,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            0,
        );
        {
            let anim = &*(&ctx.world.bg_state.bgAllAnims)[(*defender).localAnimIndex as usize]
                .anims
                .add(defAnim as usize);
            (*dc).ps.saberLockFrame =
                anim.firstFrame as c_int + (anim.numFrames as f32 * defStart) as c_int;
        }

        (*ac).ps.saberLockHits = 0;
        (*dc).ps.saberLockHits = 0;

        (*ac).ps.saberLockAdvance = qfalse;
        (*dc).ps.saberLockAdvance = qfalse;

        VectorClear(&mut (*ac).ps.velocity);
        VectorClear(&mut (*dc).ps.velocity);
        (*dc).ps.saberLockTime = ctx.world.level.time + 10000;
        (*ac).ps.saberLockTime = (*dc).ps.saberLockTime;
        (*ac).ps.saberLockEnemy = (*defender).s.number;
        (*dc).ps.saberLockEnemy = (*attacker).s.number;
        // delay 1 to 3 seconds before pushing
        (*dc).ps.weaponTime = ctx.world.bg_state.rng.Q_irand(1000, 3000);
        (*ac).ps.weaponTime = (*dc).ps.weaponTime;

        _VectorSubtract(
            (*defender).r.currentOrigin,
            (*attacker).r.currentOrigin,
            &mut defDir,
        );
        attAngles = (*ac).ps.viewangles;
        attAngles[YAW as usize] = mp_bg::bg_misc::vectoyaw(defDir);
        SetClientViewAngle(&mut *attacker, attAngles);
        defAngles[PITCH as usize] = attAngles[PITCH as usize] * -1.0;
        defAngles[YAW as usize] = AngleNormalize180(attAngles[YAW as usize] + 180.0);
        defAngles[ROLL as usize] = 0.0;
        SetClientViewAngle(&mut *defender, defAngles);

        // MATCH POSITIONS — diff is the total error in dist
        diff = VectorNormalize(&mut defDir) - idealDist;
        // try to move attacker half the diff towards the defender
        _VectorMA(
            (*attacker).r.currentOrigin,
            diff * 0.5f32,
            defDir,
            &mut newOrg,
        );

        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &(*attacker).r.currentOrigin as *const vec3_t,
                &(*attacker).r.mins as *const vec3_t,
                &(*attacker).r.maxs as *const vec3_t,
                &newOrg as *const vec3_t,
                (*attacker).s.number,
                (*attacker).clipmask,
            ),
        );
        if trace.startsolid == 0 && trace.allsolid == 0 {
            G_SetOrigin(&mut *(attacker), trace.endpos);
            if !(*attacker).client.is_null() {
                (*ac).ps.origin = trace.endpos;
            }
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(attacker.cast()));
        }
        // now get the defender's dist and do it for him too
        _VectorSubtract(
            (*attacker).r.currentOrigin,
            (*defender).r.currentOrigin,
            &mut attDir,
        );
        diff = VectorNormalize(&mut attDir) - idealDist;
        // try to move defender all of the remaining diff towards the attacker
        _VectorMA((*defender).r.currentOrigin, diff, attDir, &mut newOrg);
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &(*defender).r.currentOrigin as *const vec3_t,
                &(*defender).r.mins as *const vec3_t,
                &(*defender).r.maxs as *const vec3_t,
                &newOrg as *const vec3_t,
                (*defender).s.number,
                (*defender).clipmask,
            ),
        );
        if trace.startsolid == 0 && trace.allsolid == 0 {
            if !(*defender).client.is_null() {
                (*dc).ps.origin = trace.endpos;
            }
            G_SetOrigin(&mut *(defender), trace.endpos);
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(defender.cast()));
        }

        // DONE!
        true
    }
}

/// Raven `WP_SabersCheckLock`.
///
/// Source: `oracle/codemp/game/w_saber.c:1462-1889`
pub fn WP_SabersCheckLock(ctx: &mut GameContext, ent1: EntityId, ent2: EntityId) -> bool {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let ent1: *mut gentity_t = ctx.entity_mut(ent1);
    let ent2: *mut gentity_t = ctx.entity_mut(ent2);
    unsafe {
        let mut ent1BlockingPlayer: qboolean = qfalse;
        let mut ent2BlockingPlayer: qboolean = qfalse;

        if ctx.world.cvars.g_debugSaberLocks.integer != 0 {
            WP_SabersCheckLock2(
                ctx,
                ctx.entity_id_of(ent1).unwrap(),
                ctx.entity_id_of(ent2).unwrap(),
                LOCK_RANDOM,
            );
            return true;
        }
        // for now.. it's not fair to the lone duelist (no dual saber lock anims).
        if ctx.world.cvars.g_gametype.integer == GT_POWERDUEL {
            return false;
        }

        if ctx.world.cvars.g_saberLocking.integer == 0 {
            return false;
        }

        if (*ent1).client.is_null() || (*ent2).client.is_null() {
            return false;
        }
        let c1 = (*ent1).client;
        let c2 = (*ent2).client;

        if (*ent1).s.eType == ET_NPC as c_int || (*ent2).s.eType == ET_NPC as c_int {
            // never let an NPC lock with someone on the same playerTeam
            if (*c1).playerTeam == (*c2).playerTeam {
                return false;
            }
        }

        if (*c1).ps.saberEntityNum == 0
            || (*c2).ps.saberEntityNum == 0
            || (*c1).ps.saberInFlight != 0
            || (*c2).ps.saberInFlight != 0
        {
            // can't get in lock if one has had the saber knocked out of his hand
            return false;
        }

        if (*ent1).s.eType != ET_NPC as c_int && (*ent2).s.eType != ET_NPC as c_int {
            // can always get into locks with NPCs
            if (*c1).ps.duelInProgress == 0
                || (*c2).ps.duelInProgress == 0
                || (*c1).ps.duelIndex != (*ent2).s.number
                || (*c2).ps.duelIndex != (*ent1).s.number
            {
                // only allow saber locking if two players are dueling with each other
                if ctx.world.cvars.g_gametype.integer != GT_DUEL
                    && ctx.world.cvars.g_gametype.integer != GT_POWERDUEL
                {
                    return false;
                }
            }
        }

        if ((*ent1).r.currentOrigin[2] - (*ent2).r.currentOrigin[2]).abs() > 16.0 {
            return false;
        }
        if (*c1).ps.groundEntityNum == ENTITYNUM_NONE || (*c2).ps.groundEntityNum == ENTITYNUM_NONE
        {
            return false;
        }
        let dist = DistanceSquared((*ent1).r.currentOrigin, (*ent2).r.currentOrigin);
        if dist < 64.0 || dist > 6400.0 {
            // between 8 and 80 from each other
            return false;
        }

        if mp_bg::bg_panimate::BG_InSpecialJump((*c1).ps.legsAnim) != 0 {
            return false;
        }
        if mp_bg::bg_panimate::BG_InSpecialJump((*c2).ps.legsAnim) != 0 {
            return false;
        }

        if mp_bg::bg_panimate::BG_InRoll(&mut (*c1).ps as *mut playerState_t, (*c1).ps.legsAnim)
            != 0
        {
            return false;
        }
        if mp_bg::bg_panimate::BG_InRoll(&mut (*c2).ps as *mut playerState_t, (*c2).ps.legsAnim)
            != 0
        {
            return false;
        }

        if (*c1).ps.forceHandExtend != HANDEXTEND_NONE as c_int
            || (*c2).ps.forceHandExtend != HANDEXTEND_NONE as c_int
        {
            return false;
        }

        if ((*c1).ps.pm_flags & PMF_DUCKED) != 0 || ((*c2).ps.pm_flags & PMF_DUCKED) != 0 {
            return false;
        }

        if ((*c1).saber[0].saberFlags & SFL_NOT_LOCKABLE) != 0
            || ((*c2).saber[0].saberFlags & SFL_NOT_LOCKABLE) != 0
        {
            return false;
        }
        // (Raven's `saber[1].model &&` array-address test is vacuously true.)
        if (*c1).saber[1].model[0] != 0
            && (*c1).ps.saberHolstered == 0
            && ((*c1).saber[1].saberFlags & SFL_NOT_LOCKABLE) != 0
        {
            return false;
        }
        if (*c2).saber[1].model[0] != 0
            && (*c2).ps.saberHolstered == 0
            && ((*c2).saber[1].saberFlags & SFL_NOT_LOCKABLE) != 0
        {
            return false;
        }

        if InFront(
            (*c1).ps.origin,
            (*c2).ps.origin,
            (*c2).ps.viewangles,
            0.4f32,
        ) == 0
        {
            return false;
        }
        if InFront(
            (*c2).ps.origin,
            (*c1).ps.origin,
            (*c1).ps.viewangles,
            0.4f32,
        ) == 0
        {
            return false;
        }

        let ta1 = (*c1).ps.torsoAnim;
        let ta2 = (*c2).ps.torsoAnim;

        // T to B lock
        if ta1 == BOTH_A1_T__B_ as c_int
            || ta1 == BOTH_A2_T__B_ as c_int
            || ta1 == BOTH_A3_T__B_ as c_int
            || ta1 == BOTH_A4_T__B_ as c_int
            || ta1 == BOTH_A5_T__B_ as c_int
            || ta1 == BOTH_A6_T__B_ as c_int
            || ta1 == BOTH_A7_T__B_ as c_int
        {
            // ent1 is attacking top-down
            return WP_SabersCheckLock2(
                ctx,
                ctx.entity_id_of(ent1).unwrap(),
                ctx.entity_id_of(ent2).unwrap(),
                LOCK_TOP,
            );
        }

        if ta2 == BOTH_A1_T__B_ as c_int
            || ta2 == BOTH_A2_T__B_ as c_int
            || ta2 == BOTH_A3_T__B_ as c_int
            || ta2 == BOTH_A4_T__B_ as c_int
            || ta2 == BOTH_A5_T__B_ as c_int
            || ta2 == BOTH_A6_T__B_ as c_int
            || ta2 == BOTH_A7_T__B_ as c_int
        {
            // ent2 is attacking top-down
            return WP_SabersCheckLock2(
                ctx,
                ctx.entity_id_of(ent2).unwrap(),
                ctx.entity_id_of(ent1).unwrap(),
                LOCK_TOP,
            );
        }

        if (*ent1).s.number == 0
            && (*c1).ps.saberBlocking == saberBlockType_t::BLK_WIDE as c_int
            && (*c1).ps.weaponTime <= 0
        {
            ent1BlockingPlayer = qtrue;
        }
        if (*ent2).s.number == 0
            && (*c2).ps.saberBlocking == saberBlockType_t::BLK_WIDE as c_int
            && (*c2).ps.weaponTime <= 0
        {
            ent2BlockingPlayer = qtrue;
        }

        // TR to BL lock
        if ta1 == BOTH_A1_TR_BL as c_int
            || ta1 == BOTH_A2_TR_BL as c_int
            || ta1 == BOTH_A3_TR_BL as c_int
            || ta1 == BOTH_A4_TR_BL as c_int
            || ta1 == BOTH_A5_TR_BL as c_int
            || ta1 == BOTH_A6_TR_BL as c_int
            || ta1 == BOTH_A7_TR_BL as c_int
        {
            // ent1 is attacking diagonally
            if ent2BlockingPlayer != 0 {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent1).unwrap(),
                    ctx.entity_id_of(ent2).unwrap(),
                    LOCK_DIAG_TR,
                );
            }
            if ta2 == BOTH_A1_TR_BL as c_int
                || ta2 == BOTH_A2_TR_BL as c_int
                || ta2 == BOTH_A3_TR_BL as c_int
                || ta2 == BOTH_A4_TR_BL as c_int
                || ta2 == BOTH_A5_TR_BL as c_int
                || ta2 == BOTH_A6_TR_BL as c_int
                || ta2 == BOTH_A7_TR_BL as c_int
                || ta2 == BOTH_P1_S1_TL as c_int
            {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent1).unwrap(),
                    ctx.entity_id_of(ent2).unwrap(),
                    LOCK_DIAG_TR,
                );
            }
            if ta2 == BOTH_A1_BR_TL as c_int
                || ta2 == BOTH_A2_BR_TL as c_int
                || ta2 == BOTH_A3_BR_TL as c_int
                || ta2 == BOTH_A4_BR_TL as c_int
                || ta2 == BOTH_A5_BR_TL as c_int
                || ta2 == BOTH_A6_BR_TL as c_int
                || ta2 == BOTH_A7_BR_TL as c_int
                || ta2 == BOTH_P1_S1_BL as c_int
            {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent1).unwrap(),
                    ctx.entity_id_of(ent2).unwrap(),
                    LOCK_DIAG_BL,
                );
            }
            return false;
        }

        if ta2 == BOTH_A1_TR_BL as c_int
            || ta2 == BOTH_A2_TR_BL as c_int
            || ta2 == BOTH_A3_TR_BL as c_int
            || ta2 == BOTH_A4_TR_BL as c_int
            || ta2 == BOTH_A5_TR_BL as c_int
            || ta2 == BOTH_A6_TR_BL as c_int
            || ta2 == BOTH_A7_TR_BL as c_int
        {
            // ent2 is attacking diagonally
            if ent1BlockingPlayer != 0 {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent2).unwrap(),
                    ctx.entity_id_of(ent1).unwrap(),
                    LOCK_DIAG_TR,
                );
            }
            if ta1 == BOTH_A1_TR_BL as c_int
                || ta1 == BOTH_A2_TR_BL as c_int
                || ta1 == BOTH_A3_TR_BL as c_int
                || ta1 == BOTH_A4_TR_BL as c_int
                || ta1 == BOTH_A5_TR_BL as c_int
                || ta1 == BOTH_A6_TR_BL as c_int
                || ta1 == BOTH_A7_TR_BL as c_int
                || ta1 == BOTH_P1_S1_TL as c_int
            {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent2).unwrap(),
                    ctx.entity_id_of(ent1).unwrap(),
                    LOCK_DIAG_TR,
                );
            }
            if ta1 == BOTH_A1_BR_TL as c_int
                || ta1 == BOTH_A2_BR_TL as c_int
                || ta1 == BOTH_A3_BR_TL as c_int
                || ta1 == BOTH_A4_BR_TL as c_int
                || ta1 == BOTH_A5_BR_TL as c_int
                || ta1 == BOTH_A6_BR_TL as c_int
                || ta1 == BOTH_A7_BR_TL as c_int
                || ta1 == BOTH_P1_S1_BL as c_int
            {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent2).unwrap(),
                    ctx.entity_id_of(ent1).unwrap(),
                    LOCK_DIAG_BL,
                );
            }
            return false;
        }

        // TL to BR lock
        if ta1 == BOTH_A1_TL_BR as c_int
            || ta1 == BOTH_A2_TL_BR as c_int
            || ta1 == BOTH_A3_TL_BR as c_int
            || ta1 == BOTH_A4_TL_BR as c_int
            || ta1 == BOTH_A5_TL_BR as c_int
            || ta1 == BOTH_A6_TL_BR as c_int
            || ta1 == BOTH_A7_TL_BR as c_int
        {
            // ent1 is attacking diagonally
            if ent2BlockingPlayer != 0 {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent1).unwrap(),
                    ctx.entity_id_of(ent2).unwrap(),
                    LOCK_DIAG_TL,
                );
            }
            if ta2 == BOTH_A1_TL_BR as c_int
                || ta2 == BOTH_A2_TL_BR as c_int
                || ta2 == BOTH_A3_TL_BR as c_int
                || ta2 == BOTH_A4_TL_BR as c_int
                || ta2 == BOTH_A5_TL_BR as c_int
                || ta2 == BOTH_A6_TL_BR as c_int
                || ta2 == BOTH_A7_TL_BR as c_int
                || ta2 == BOTH_P1_S1_TR as c_int
            {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent1).unwrap(),
                    ctx.entity_id_of(ent2).unwrap(),
                    LOCK_DIAG_TL,
                );
            }
            if ta2 == BOTH_A1_BL_TR as c_int
                || ta2 == BOTH_A2_BL_TR as c_int
                || ta2 == BOTH_A3_BL_TR as c_int
                || ta2 == BOTH_A4_BL_TR as c_int
                || ta2 == BOTH_A5_BL_TR as c_int
                || ta2 == BOTH_A6_BL_TR as c_int
                || ta2 == BOTH_A7_BL_TR as c_int
                || ta2 == BOTH_P1_S1_BR as c_int
            {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent1).unwrap(),
                    ctx.entity_id_of(ent2).unwrap(),
                    LOCK_DIAG_BR,
                );
            }
            return false;
        }

        if ta2 == BOTH_A1_TL_BR as c_int
            || ta2 == BOTH_A2_TL_BR as c_int
            || ta2 == BOTH_A3_TL_BR as c_int
            || ta2 == BOTH_A4_TL_BR as c_int
            || ta2 == BOTH_A5_TL_BR as c_int
            || ta2 == BOTH_A6_TL_BR as c_int
            || ta2 == BOTH_A7_TL_BR as c_int
        {
            // ent2 is attacking diagonally
            if ent1BlockingPlayer != 0 {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent2).unwrap(),
                    ctx.entity_id_of(ent1).unwrap(),
                    LOCK_DIAG_TL,
                );
            }
            if ta1 == BOTH_A1_TL_BR as c_int
                || ta1 == BOTH_A2_TL_BR as c_int
                || ta1 == BOTH_A3_TL_BR as c_int
                || ta1 == BOTH_A4_TL_BR as c_int
                || ta1 == BOTH_A5_TL_BR as c_int
                || ta1 == BOTH_A6_TL_BR as c_int
                || ta1 == BOTH_A7_TL_BR as c_int
                || ta1 == BOTH_P1_S1_TR as c_int
            {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent2).unwrap(),
                    ctx.entity_id_of(ent1).unwrap(),
                    LOCK_DIAG_TL,
                );
            }
            if ta1 == BOTH_A1_BL_TR as c_int
                || ta1 == BOTH_A2_BL_TR as c_int
                || ta1 == BOTH_A3_BL_TR as c_int
                || ta1 == BOTH_A4_BL_TR as c_int
                || ta1 == BOTH_A5_BL_TR as c_int
                || ta1 == BOTH_A6_BL_TR as c_int
                || ta1 == BOTH_A7_BL_TR as c_int
                || ta1 == BOTH_P1_S1_BR as c_int
            {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent2).unwrap(),
                    ctx.entity_id_of(ent1).unwrap(),
                    LOCK_DIAG_BR,
                );
            }
            return false;
        }
        // L to R lock
        if ta1 == BOTH_A1__L__R as c_int
            || ta1 == BOTH_A2__L__R as c_int
            || ta1 == BOTH_A3__L__R as c_int
            || ta1 == BOTH_A4__L__R as c_int
            || ta1 == BOTH_A5__L__R as c_int
            || ta1 == BOTH_A6__L__R as c_int
            || ta1 == BOTH_A7__L__R as c_int
        {
            // ent1 is attacking l to r
            if ent2BlockingPlayer != 0 {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent1).unwrap(),
                    ctx.entity_id_of(ent2).unwrap(),
                    LOCK_L,
                );
            }
            if ta2 == BOTH_A1_TL_BR as c_int
                || ta2 == BOTH_A2_TL_BR as c_int
                || ta2 == BOTH_A3_TL_BR as c_int
                || ta2 == BOTH_A4_TL_BR as c_int
                || ta2 == BOTH_A5_TL_BR as c_int
                || ta2 == BOTH_A6_TL_BR as c_int
                || ta2 == BOTH_A7_TL_BR as c_int
                || ta2 == BOTH_P1_S1_TR as c_int
                || ta2 == BOTH_P1_S1_BL as c_int
            {
                // ent2 is attacking or blocking on the r
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent1).unwrap(),
                    ctx.entity_id_of(ent2).unwrap(),
                    LOCK_L,
                );
            }
            return false;
        }
        if ta2 == BOTH_A1__L__R as c_int
            || ta2 == BOTH_A2__L__R as c_int
            || ta2 == BOTH_A3__L__R as c_int
            || ta2 == BOTH_A4__L__R as c_int
            || ta2 == BOTH_A5__L__R as c_int
            || ta2 == BOTH_A6__L__R as c_int
            || ta2 == BOTH_A7__L__R as c_int
        {
            // ent2 is attacking l to r
            if ent1BlockingPlayer != 0 {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent2).unwrap(),
                    ctx.entity_id_of(ent1).unwrap(),
                    LOCK_L,
                );
            }
            if ta1 == BOTH_A1_TL_BR as c_int
                || ta1 == BOTH_A2_TL_BR as c_int
                || ta1 == BOTH_A3_TL_BR as c_int
                || ta1 == BOTH_A4_TL_BR as c_int
                || ta1 == BOTH_A5_TL_BR as c_int
                || ta1 == BOTH_A6_TL_BR as c_int
                || ta1 == BOTH_A7_TL_BR as c_int
                || ta1 == BOTH_P1_S1_TR as c_int
                || ta1 == BOTH_P1_S1_BL as c_int
            {
                // ent1 is attacking or blocking on the r
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent2).unwrap(),
                    ctx.entity_id_of(ent1).unwrap(),
                    LOCK_L,
                );
            }
            return false;
        }
        // R to L lock
        if ta1 == BOTH_A1__R__L as c_int
            || ta1 == BOTH_A2__R__L as c_int
            || ta1 == BOTH_A3__R__L as c_int
            || ta1 == BOTH_A4__R__L as c_int
            || ta1 == BOTH_A5__R__L as c_int
            || ta1 == BOTH_A6__R__L as c_int
            || ta1 == BOTH_A7__R__L as c_int
        {
            // ent1 is attacking r to l
            if ent2BlockingPlayer != 0 {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent1).unwrap(),
                    ctx.entity_id_of(ent2).unwrap(),
                    LOCK_R,
                );
            }
            if ta2 == BOTH_A1_TR_BL as c_int
                || ta2 == BOTH_A2_TR_BL as c_int
                || ta2 == BOTH_A3_TR_BL as c_int
                || ta2 == BOTH_A4_TR_BL as c_int
                || ta2 == BOTH_A5_TR_BL as c_int
                || ta2 == BOTH_A6_TR_BL as c_int
                || ta2 == BOTH_A7_TR_BL as c_int
                || ta2 == BOTH_P1_S1_TL as c_int
                || ta2 == BOTH_P1_S1_BR as c_int
            {
                // ent2 is attacking or blocking on the l
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent1).unwrap(),
                    ctx.entity_id_of(ent2).unwrap(),
                    LOCK_R,
                );
            }
            return false;
        }
        if ta2 == BOTH_A1__R__L as c_int
            || ta2 == BOTH_A2__R__L as c_int
            || ta2 == BOTH_A3__R__L as c_int
            || ta2 == BOTH_A4__R__L as c_int
            || ta2 == BOTH_A5__R__L as c_int
            || ta2 == BOTH_A6__R__L as c_int
            || ta2 == BOTH_A7__R__L as c_int
        {
            // ent2 is attacking r to l
            if ent1BlockingPlayer != 0 {
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent2).unwrap(),
                    ctx.entity_id_of(ent1).unwrap(),
                    LOCK_R,
                );
            }
            if ta1 == BOTH_A1_TR_BL as c_int
                || ta1 == BOTH_A2_TR_BL as c_int
                || ta1 == BOTH_A3_TR_BL as c_int
                || ta1 == BOTH_A4_TR_BL as c_int
                || ta1 == BOTH_A5_TR_BL as c_int
                || ta1 == BOTH_A6_TR_BL as c_int
                || ta1 == BOTH_A7_TR_BL as c_int
                || ta1 == BOTH_P1_S1_TL as c_int
                || ta1 == BOTH_P1_S1_BR as c_int
            {
                // ent1 is attacking or blocking on the l
                return WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(ent2).unwrap(),
                    ctx.entity_id_of(ent1).unwrap(),
                    LOCK_R,
                );
            }
            return false;
        }
        if ctx.world.bg_state.rng.Q_irand(0, 10) == 0 {
            return WP_SabersCheckLock2(
                ctx,
                ctx.entity_id_of(ent1).unwrap(),
                ctx.entity_id_of(ent2).unwrap(),
                LOCK_RANDOM,
            );
        }
        false
    }
}

/// Raven `G_GetParryForBlock`.
///
/// Source: `oracle/codemp/game/w_saber.c:1891-1930`
pub fn G_GetParryForBlock(block: c_int) -> c_int {
    match block {
        BLOCKED_UPPER_RIGHT => LS_PARRY_UR,
        BLOCKED_UPPER_RIGHT_PROJ => LS_REFLECT_UR,
        BLOCKED_UPPER_LEFT => LS_PARRY_UL,
        BLOCKED_UPPER_LEFT_PROJ => LS_REFLECT_UL,
        BLOCKED_LOWER_RIGHT => LS_PARRY_LR,
        BLOCKED_LOWER_RIGHT_PROJ => LS_REFLECT_LR,
        BLOCKED_LOWER_LEFT => LS_PARRY_LL,
        BLOCKED_LOWER_LEFT_PROJ => LS_REFLECT_LL,
        BLOCKED_TOP => LS_PARRY_UP,
        BLOCKED_TOP_PROJ => LS_REFLECT_UP,
        _ => LS_NONE,
    }
}

/// Raven `WP_GetSaberDeflectionAngle`.
///
/// Source: `oracle/codemp/game/w_saber.c:1938-2208`
pub fn WP_GetSaberDeflectionAngle(
    ctx: &mut GameContext,
    attacker: Option<EntityId>,
    defender: Option<EntityId>,
    saberHitFraction: f32,
) -> bool {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let attacker: *mut gentity_t =
        unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), attacker) };
    let defender: *mut gentity_t =
        unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), defender) };
    unsafe {
        let animBasedDeflection: qboolean = qtrue;
        let _ = saberHitFraction;

        if attacker.is_null() || (*attacker).client.is_null() || (*attacker).ghoul2.is_null() {
            return false;
        }
        if defender.is_null() || (*defender).client.is_null() || (*defender).ghoul2.is_null() {
            return false;
        }
        let ac = (*attacker).client;
        let dc = (*defender).client;

        if (ctx.world.level.time - (*ac).lastSaberStorageTime) > 500 {
            // last update too long ago; something prevents his saber from updating
            return false;
        }
        if (ctx.world.level.time - (*dc).lastSaberStorageTime) > 500 {
            return false;
        }

        let attSaberLevel = G_SaberAttackPower(
            ctx,
            ctx.entity_id_of(attacker),
            SaberAttacking(&*(attacker)),
        );
        let defSaberLevel = G_SaberAttackPower(
            ctx,
            ctx.entity_id_of(defender),
            SaberAttacking(&*(defender)),
        );

        if animBasedDeflection != 0 {
            // base it off the anim
            let attQuadStart =
                ctx.world.bg_state.saberMoveData[(*ac).ps.saberMove as usize].startQuad;
            let attQuadEnd = ctx.world.bg_state.saberMoveData[(*ac).ps.saberMove as usize].endQuad;
            let mut defQuad = ctx.world.bg_state.saberMoveData[(*dc).ps.saberMove as usize].endQuad;
            let mut quadDiff = ((defQuad - attQuadStart) as f32).abs() as c_int;

            if (*dc).ps.saberMove == LS_READY {
                return false;
            }

            // reverse the left/right of the defQuad (mirrored when facing each other)
            if defQuad == Q_BR as c_int {
                defQuad = Q_BL as c_int;
            } else if defQuad == Q_R as c_int {
                defQuad = Q_L as c_int;
            } else if defQuad == Q_TR as c_int {
                defQuad = Q_TL as c_int;
            } else if defQuad == Q_TL as c_int {
                defQuad = Q_TR as c_int;
            } else if defQuad == Q_L as c_int {
                defQuad = Q_R as c_int;
            } else if defQuad == Q_BL as c_int {
                defQuad = Q_BR as c_int;
            }

            if quadDiff > 4 {
                // wrap so diff is never greater than 180 (4 * 45)
                quadDiff = 4 - (quadDiff - 4);
            }
            // have the quads, find a good anim to use
            if (quadDiff == 0 || (quadDiff == 1 && ctx.world.bg_state.rng.Q_irand(0, 1) != 0))
                && (defSaberLevel == attSaberLevel
                    || ctx
                        .world
                        .bg_state
                        .rng
                        .Q_irand(0, defSaberLevel - attSaberLevel)
                        >= 0)
            {
                // bounce straight back
                let attMove = (*ac).ps.saberMove;
                (*ac).ps.saberMove =
                    mp_bg::bg_panimate::PM_SaberBounceForAttack((*ac).ps.saberMove);
                if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                    let s = format!(
                        "attack {} vs. parry {} bounced to {}\n",
                        cstr_to_str(
                            animTable[ctx.world.bg_state.saberMoveData[attMove as usize].animToUse
                                as usize]
                                .name
                        ),
                        cstr_to_str(
                            animTable[ctx.world.bg_state.saberMoveData[(*dc).ps.saberMove as usize]
                                .animToUse as usize]
                                .name
                        ),
                        cstr_to_str(
                            animTable[ctx.world.bg_state.saberMoveData[(*ac).ps.saberMove as usize]
                                .animToUse as usize]
                                .name
                        ),
                    );
                    Com_Printf(&s);
                }
                (*ac).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                return false;
            } else {
                // attack hit at an angle; figure out what angle it bounces off att
                quadDiff = defQuad - attQuadEnd;
                // add half the diff between the defense and attack end to attack end
                if quadDiff > 4 {
                    quadDiff = 4 - (quadDiff - 4);
                } else if quadDiff < -4 {
                    quadDiff = -4 + (quadDiff + 4);
                }
                let mut newQuad = attQuadEnd + ((quadDiff as f32) / 2.0f32).ceil() as c_int;
                if newQuad < Q_BR as c_int {
                    // less than zero wraps around
                    newQuad = Q_B as c_int + newQuad;
                }
                if newQuad == attQuadStart {
                    // never come off at the same angle as an uninterrupted attack
                    if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                        newQuad -= 1;
                    } else {
                        newQuad += 1;
                    }
                    if newQuad < Q_BR as c_int {
                        newQuad = Q_B as c_int;
                    } else if newQuad > Q_B as c_int {
                        newQuad = Q_BR as c_int;
                    }
                }
                if newQuad == defQuad {
                    // bounce straight back
                    let attMove = (*ac).ps.saberMove;
                    (*ac).ps.saberMove =
                        mp_bg::bg_panimate::PM_SaberBounceForAttack((*ac).ps.saberMove);
                    if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                        let s = format!(
                            "attack {} vs. parry {} bounced to {}\n",
                            cstr_to_str(
                                animTable[ctx.world.bg_state.saberMoveData[attMove as usize]
                                    .animToUse as usize]
                                    .name
                            ),
                            cstr_to_str(
                                animTable[ctx.world.bg_state.saberMoveData
                                    [(*dc).ps.saberMove as usize]
                                    .animToUse as usize]
                                    .name
                            ),
                            cstr_to_str(
                                animTable[ctx.world.bg_state.saberMoveData
                                    [(*ac).ps.saberMove as usize]
                                    .animToUse as usize]
                                    .name
                            ),
                        );
                        Com_Printf(&s);
                    }
                    (*ac).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                    return false;
                } else {
                    // else, pick a deflection
                    let attMove = (*ac).ps.saberMove;
                    (*ac).ps.saberMove = mp_bg::bg_panimate::PM_SaberDeflectionForQuad(newQuad);
                    if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                        let s = format!(
                            "attack {} vs. parry {} deflected to {}\n",
                            cstr_to_str(
                                animTable[ctx.world.bg_state.saberMoveData[attMove as usize]
                                    .animToUse as usize]
                                    .name
                            ),
                            cstr_to_str(
                                animTable[ctx.world.bg_state.saberMoveData
                                    [(*dc).ps.saberMove as usize]
                                    .animToUse as usize]
                                    .name
                            ),
                            cstr_to_str(
                                animTable[ctx.world.bg_state.saberMoveData
                                    [(*ac).ps.saberMove as usize]
                                    .animToUse as usize]
                                    .name
                            ),
                        );
                        Com_Printf(&s);
                    }
                    (*ac).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
                    return true;
                }
            }
        } else {
            // old math-based method (probably broken)
            let mut att_HitDir: vec3_t = [0.0; 3];
            let mut def_BladeDir: vec3_t = [0.0; 3];
            let mut temp: vec3_t = [0.0; 3];
            let hitDot: f32;

            temp = (*ac).lastSaberBase_Always;

            AngleVectors((*ac).lastSaberDir_Always, Some(&mut att_HitDir), None, None);

            AngleVectors(
                (*dc).lastSaberDir_Always,
                Some(&mut def_BladeDir),
                None,
                None,
            );

            // now compare
            hitDot = _DotProduct(att_HitDir, def_BladeDir);
            if hitDot < 0.25f32 && hitDot > -0.25f32 {
                // hit pretty much perpendicular, pop straight back
                (*ac).ps.saberMove =
                    mp_bg::bg_panimate::PM_SaberBounceForAttack((*ac).ps.saberMove);
                (*ac).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                return false;
            } else {
                // a deflection
                let mut att_Right: vec3_t = [0.0; 3];
                let mut att_Up: vec3_t = [0.0; 3];
                let mut att_DeflectionDir: vec3_t = [0.0; 3];
                let swingRDot: f32;
                let swingUDot: f32;

                // get the direction of the deflection
                _VectorScale(def_BladeDir, hitDot, &mut att_DeflectionDir);
                // get our bounce straight back direction
                _VectorScale(att_HitDir, -1.0f32, &mut temp);
                // add the bounce back and deflection
                let dd = att_DeflectionDir;
                _VectorAdd(dd, temp, &mut att_DeflectionDir);
                // normalize to determine which direction our saber should bounce toward
                VectorNormalize(&mut att_DeflectionDir);

                // need the deflection dir relative to the attacker's facing
                VectorSet(&mut temp, 0.0, (*ac).ps.viewangles[YAW as usize], 0.0); // presumes no pitch!
                AngleVectors(temp, None, Some(&mut att_Right), Some(&mut att_Up));
                swingRDot = _DotProduct(att_Right, att_DeflectionDir);
                swingUDot = _DotProduct(att_Up, att_DeflectionDir);

                if swingRDot > 0.25f32 {
                    // deflect to right
                    if swingUDot > 0.25f32 {
                        (*ac).ps.saberMove = LS_D1_TR;
                    } else if swingUDot < -0.25f32 {
                        (*ac).ps.saberMove = LS_D1_BR;
                    } else {
                        (*ac).ps.saberMove = LS_D1__R;
                    }
                } else if swingRDot < -0.25f32 {
                    // deflect to left
                    if swingUDot > 0.25f32 {
                        (*ac).ps.saberMove = LS_D1_TL;
                    } else if swingUDot < -0.25f32 {
                        (*ac).ps.saberMove = LS_D1_BL;
                    } else {
                        (*ac).ps.saberMove = LS_D1__L;
                    }
                } else {
                    // deflect in middle
                    if swingUDot > 0.25f32 {
                        (*ac).ps.saberMove = LS_D1_T_;
                    } else if swingUDot < -0.25f32 {
                        (*ac).ps.saberMove = LS_D1_B_;
                    } else {
                        // no straight back in my face, so use top
                        if swingRDot > 0.0 {
                            (*ac).ps.saberMove = LS_D1_TR;
                        } else if swingRDot < 0.0 {
                            (*ac).ps.saberMove = LS_D1_TL;
                        } else {
                            (*ac).ps.saberMove = LS_D1_T_;
                        }
                    }
                }

                (*ac).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
                return true;
            }
        }
    }
}

/// Raven `G_KnockawayForParry`.
///
/// Source: `oracle/codemp/game/w_saber.c:2210-2233`
pub fn G_KnockawayForParry(r#move: c_int) -> c_int {
    // FIXME(Raven): need actual anims for this; need to know which side of the
    // saber was hit — presume it gets knocked away from the center.
    // Oracle's `case LS_PARRY_UR:` falls through into `default:` (both return
    // LS_K1_TR), so the `_` arm covers LS_PARRY_UR and every unmatched value.
    match r#move {
        LS_PARRY_UP => LS_K1_T_, // push up
        LS_PARRY_UL => LS_K1_TL, // push up and to left
        LS_PARRY_LR => LS_K1_BR, // push down and to left
        LS_PARRY_LL => LS_K1_BL, // push down and to right
        _ => LS_K1_TR,           // LS_PARRY_UR / default: push up, slightly right
    }
}

/// Raven `G_GetAttackDamage`.
///
/// Source: `oracle/codemp/game/w_saber.c:2238-2287`
pub fn G_GetAttackDamage(
    ctx: &mut GameContext,
    self_: EntityId,
    minDmg: c_int,
    maxDmg: c_int,
    multPoint: f32,
) -> c_int {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        let sc = (*self_).client;
        let mut totalDamage = maxDmg;
        let anim = &*(&ctx.world.bg_state.bgAllAnims)[(*self_).localAnimIndex as usize]
            .anims
            .add((*sc).ps.torsoAnim as usize);
        let mut attackAnimLength = anim.numFrames as f32 * ((anim.frameLerp as f32).abs());
        let mut animSpeedFactor = 1.0f32;

        // Be sure to scale by the proper anim speed
        BG_SaberStartTransAnim(
            (*self_).s.number,
            (*sc).ps.fd.saberAnimLevel,
            (*sc).ps.weapon,
            (*sc).ps.torsoAnim,
            &mut animSpeedFactor,
            (*sc).ps.brokenLimbs,
            // STAGE-2b: irreducible — the ruling-21 `GameCallbacksImpl` seam
            // adapter holds a raw `*mut GameWorld`; `my_saber` reaches the game
            // arena by client number (replaces the old `g_entities` base arg).
            &mut GameCallbacksImpl {
                world: ctx.world_raw(),
                engine: ctx.engine,
            },
        );
        let speedDif = (attackAnimLength - (attackAnimLength * animSpeedFactor)) as c_int;
        attackAnimLength += speedDif as f32;
        let mut peakPoint = attackAnimLength;
        peakPoint -= attackAnimLength * multPoint;

        // we treat torsoTimer as the point in the animation
        let currentPoint = (*sc).ps.torsoTimer as f32;

        let _peakDif = if peakPoint > currentPoint {
            (peakPoint - currentPoint) as c_int
        } else {
            (currentPoint - peakPoint) as c_int
        };

        let mut damageFactor = currentPoint / peakPoint;
        if damageFactor > 1.0 {
            damageFactor = 2.0 - damageFactor;
        }

        totalDamage = (totalDamage as f32 * damageFactor) as c_int;
        if totalDamage < minDmg {
            totalDamage = minDmg;
        }
        if totalDamage > maxDmg {
            totalDamage = maxDmg;
        }

        totalDamage
    }
}

/// Raven `G_GetAnimPoint`.
///
/// Source: `oracle/codemp/game/w_saber.c:2290-2310`
pub fn G_GetAnimPoint(ctx: &mut GameContext, self_: EntityId) -> f32 {
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let sc = ctx.world.entity(self_).client;
    let local_anim_index = ctx.world.entity(self_).localAnimIndex;
    let s_number = ctx.world.entity(self_).s.number;
    unsafe {
        let anim = &*(&ctx.world.bg_state.bgAllAnims)[local_anim_index as usize]
            .anims
            .add((*sc).ps.torsoAnim as usize);
        let mut attackAnimLength = anim.numFrames as f32 * ((anim.frameLerp as f32).abs());
        let mut animSpeedFactor = 1.0f32;

        BG_SaberStartTransAnim(
            s_number,
            (*sc).ps.fd.saberAnimLevel,
            (*sc).ps.weapon,
            (*sc).ps.torsoAnim,
            &mut animSpeedFactor,
            (*sc).ps.brokenLimbs,
            // STAGE-2b: irreducible — the ruling-21 `GameCallbacksImpl` seam
            // adapter holds a raw `*mut GameWorld`; `my_saber` reaches the game
            // arena by client number (replaces the old `g_entities` base arg).
            &mut GameCallbacksImpl {
                world: ctx.world_raw(),
                engine: ctx.engine,
            },
        );
        let speedDif = (attackAnimLength - (attackAnimLength * animSpeedFactor)) as c_int;
        attackAnimLength += speedDif as f32;

        let currentPoint = (*sc).ps.torsoTimer as f32;

        currentPoint / attackAnimLength
    }
}

/// Raven `G_ClientIdleInWorld`.
///
/// Source: `oracle/codemp/game/w_saber.c:2312-2334`
pub fn G_ClientIdleInWorld(ent: &gentity_t) -> bool {
    if ent.s.eType == ET_NPC as c_int {
        return false;
    }
    let client = ent.client;
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let (upmove, forwardmove, rightmove, buttons) = unsafe {
        let cmd = &(*client).pers.cmd;
        (cmd.upmove, cmd.forwardmove, cmd.rightmove, cmd.buttons)
    };
    if upmove == 0
        && forwardmove == 0
        && rightmove == 0
        && (buttons & BUTTON_GESTURE) == 0
        && (buttons & BUTTON_FORCEGRIP) == 0
        && (buttons & BUTTON_ALT_ATTACK) == 0
        && (buttons & BUTTON_FORCEPOWER) == 0
        && (buttons & BUTTON_FORCE_LIGHTNING) == 0
        && (buttons & BUTTON_FORCE_DRAIN) == 0
        && (buttons & BUTTON_ATTACK) == 0
    {
        return true;
    }
    false
}

/// Raven `G_G2TraceCollide`.
///
/// Source: `oracle/codemp/game/w_saber.c:2336-2426`
// Referee probe: one saber-trace call (start/end, trace bounds, fraction, hit ent).
fn g6t(
    ctx: &GameContext,
    tag: &str,
    a: &vec3_t,
    b: &vec3_t,
    mn: &vec3_t,
    mx: &vec3_t,
    fr: f32,
    en: i32,
) {
    let t = ctx.world.level.time;
    probe!(
        "SAB_TRACE",
        "{} t={} a={:08x},{:08x},{:08x} b={:08x},{:08x},{:08x} mn={:08x} mx={:08x} fr={:08x} en={}",
        tag,
        t,
        a[0].to_bits(),
        a[1].to_bits(),
        a[2].to_bits(),
        b[0].to_bits(),
        b[1].to_bits(),
        b[2].to_bits(),
        mn[0].to_bits(),
        mx[0].to_bits(),
        fr.to_bits(),
        en
    );
}

pub fn G_G2TraceCollide(
    ctx: &mut GameContext,
    tr: *mut trace_t,
    lastValidStart: vec3_t,
    lastValidEnd: vec3_t,
    traceMins: vec3_t,
    traceMaxs: vec3_t,
) -> bool {
    // Hit the ent with the normal trace, try the collision trace.
    unsafe {
        let mut G2Trace: G2Trace_t = core::mem::zeroed();
        let mut angles: vec3_t = [0.0; 3];
        let mut tN: c_int = 0;
        let mut fRadius: f32 = 0.0;

        if ctx.world.cvars.d_saberGhoul2Collision.integer == 0 {
            return false;
        }

        if ctx.world.g_entities[(*tr).entityNum as usize].inuse == 0 {
            // don't do perpoly on corpses.
            return false;
        }

        if traceMins[0] != 0.0
            || traceMins[1] != 0.0
            || traceMins[2] != 0.0
            || traceMaxs[0] != 0.0
            || traceMaxs[1] != 0.0
            || traceMaxs[2] != 0.0
        {
            fRadius = (traceMaxs[0] - traceMins[0]) / 2.0f32;
        }

        // memset(&G2Trace,0,sizeof(G2Trace)) — covered by zeroed() above.
        while tN < MAX_G2_COLLISIONS as c_int {
            G2Trace[tN as usize].mEntityNum = -1;
            tN += 1;
        }
        let g2Hit = &mut ctx.world.g_entities[(*tr).entityNum as usize] as *mut gentity_t;

        if !g2Hit.is_null() && (*g2Hit).inuse != 0 && !(*g2Hit).ghoul2.is_null() {
            let mut g2HitOrigin: vec3_t = [0.0; 3];

            angles[ROLL as usize] = 0.0;
            angles[PITCH as usize] = 0.0;

            if !(*g2Hit).client.is_null() {
                let gc = (*g2Hit).client;
                g2HitOrigin = (*gc).ps.origin;
                angles[YAW as usize] = (*gc).ps.viewangles[YAW as usize];
            } else {
                g2HitOrigin = (*g2Hit).r.currentOrigin;
                angles[YAW as usize] = (*g2Hit).r.currentAngles[YAW as usize];
            }

            if ctx.world.cvars.g_optvehtrace.integer != 0
                && (*g2Hit).s.eType == ET_NPC as c_int
                && (*g2Hit).s.NPC_class == CLASS_VEHICLE as c_int
                && !(*g2Hit).m_pVehicle.is_null()
            {
                trap::G2API_CollisionDetectCache(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2_COLLISIONDETECTCACHE::GG2CollisiondetectcacheArgs::new(
                        G2Trace.as_mut_ptr(),
                        (*g2Hit).ghoul2,
                        angles.as_ptr(),
                        g2HitOrigin.as_ptr(),
                        ctx.world.level.time,
                        (*g2Hit).s.number,
                        lastValidStart.as_ptr() as *mut f32,
                        lastValidEnd.as_ptr() as *mut f32,
                        (*g2Hit).modelScale.as_ptr() as *mut f32,
                        0,
                        ctx.world.cvars.g_g2TraceLod.integer,
                        fRadius,
                    ),
                );
            } else {
                trap::G2API_CollisionDetect(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2_COLLISIONDETECT::GG2CollisiondetectArgs::new(
                        G2Trace.as_mut_ptr(),
                        (*g2Hit).ghoul2,
                        angles.as_ptr(),
                        g2HitOrigin.as_ptr(),
                        ctx.world.level.time,
                        (*g2Hit).s.number,
                        lastValidStart.as_ptr() as *mut f32,
                        lastValidEnd.as_ptr() as *mut f32,
                        (*g2Hit).modelScale.as_ptr() as *mut f32,
                        0,
                        ctx.world.cvars.g_g2TraceLod.integer,
                        fRadius,
                    ),
                );
            }

            // Referee probe: saber ghoul2 collide check — trace ent vs G2 hit ent.
            probe!(
                "SAB_CD",
                "t={} en={} m={}",
                ctx.world.level.time,
                (*tr).entityNum,
                G2Trace[0].mEntityNum
            );
            if G2Trace[0].mEntityNum != (*g2Hit).s.number {
                (*tr).fraction = 1.0f32;
                (*tr).entityNum = ENTITYNUM_NONE as c_short;
                (*tr).startsolid = 0;
                (*tr).allsolid = 0;
                return false;
            } else {
                // The ghoul2 trace result matches; copy the collision position back.
                (*tr).endpos = G2Trace[0].mCollisionPosition;
                (*tr).plane.normal = G2Trace[0].mCollisionNormal;

                if !(*g2Hit).client.is_null() {
                    let gc = (*g2Hit).client;
                    (*gc).g2LastSurfaceHit = G2Trace[0].mSurfaceIndex;
                    (*gc).g2LastSurfaceTime = ctx.world.level.time;
                }
                return true;
            }
        }

        false
    }
}

/// Raven `G_SaberInBackAttack`.
///
/// Source: `oracle/codemp/game/w_saber.c:2428-2439`
pub fn G_SaberInBackAttack(r#move: c_int) -> bool {
    match r#move {
        LS_A_BACK | LS_A_BACK_CR | LS_A_BACKSTAB => true,
        _ => false,
    }
}

/// Raven `G_BuildSaberFaces`.
///
/// Source: `oracle/codemp/game/w_saber.c:2454-2570`
pub fn G_BuildSaberFaces(
    ctx: &mut GameContext,
    base: vec3_t,
    tip: vec3_t,
    radius: f32,
    fwd: vec3_t,
    right: vec3_t,
    fNum: *mut c_int,
    fList: *mut *mut saberFace_t,
) {
    unsafe {
        // Raven's function-local `static saberFace_t faces[12]` is returned by
        // pointer; `GameWorld.scratch` owns it (§B3), same persistent-buffer behavior.
        let faces = &mut ctx.world.scratch.faces;
        let mut i: usize = 0;
        let mut invFwd: vec3_t = [0.0; 3];
        let mut invRight: vec3_t = [0.0; 3];

        invFwd = fwd;
        VectorInverse(&mut invFwd);
        invRight = right;
        VectorInverse(&mut invRight);

        while i < 8 {
            // yeah, this part is kind of a hack, but eh
            let (d1, d2): (vec3_t, vec3_t) = if i < 2 {
                // "left" surface
                (fwd, invRight)
            } else if i < 4 {
                // "right" surface
                (fwd, right)
            } else if i < 6 {
                // "front" surface
                (right, fwd)
            } else {
                // "back" surface
                (right, invFwd)
            };

            // first triangle for this surface
            _VectorMA(base, radius / 2.0f32, d1, &mut faces[i].v1);
            let v = faces[i].v1;
            _VectorMA(v, radius / 2.0f32, d2, &mut faces[i].v1);

            _VectorMA(tip, radius / 2.0f32, d1, &mut faces[i].v2);
            let v = faces[i].v2;
            _VectorMA(v, radius / 2.0f32, d2, &mut faces[i].v2);

            _VectorMA(tip, -radius / 2.0f32, d1, &mut faces[i].v3);
            let v = faces[i].v3;
            _VectorMA(v, radius / 2.0f32, d2, &mut faces[i].v3);

            i += 1;

            // second triangle for this surface
            _VectorMA(tip, -radius / 2.0f32, d1, &mut faces[i].v1);
            let v = faces[i].v1;
            _VectorMA(v, radius / 2.0f32, d2, &mut faces[i].v1);

            _VectorMA(base, radius / 2.0f32, d1, &mut faces[i].v2);
            let v = faces[i].v2;
            _VectorMA(v, radius / 2.0f32, d2, &mut faces[i].v2);

            _VectorMA(base, -radius / 2.0f32, d1, &mut faces[i].v3);
            let v = faces[i].v3;
            _VectorMA(v, radius / 2.0f32, d2, &mut faces[i].v3);

            i += 1;
        }

        // top surface — face 1
        _VectorMA(tip, radius / 2.0f32, fwd, &mut faces[i].v1);
        let v = faces[i].v1;
        _VectorMA(v, -radius / 2.0f32, right, &mut faces[i].v1);

        _VectorMA(tip, radius / 2.0f32, fwd, &mut faces[i].v2);
        let v = faces[i].v2;
        _VectorMA(v, radius / 2.0f32, right, &mut faces[i].v2);

        _VectorMA(tip, -radius / 2.0f32, fwd, &mut faces[i].v3);
        let v = faces[i].v3;
        _VectorMA(v, -radius / 2.0f32, right, &mut faces[i].v3);

        i += 1;

        // face 2
        _VectorMA(tip, radius / 2.0f32, fwd, &mut faces[i].v1);
        let v = faces[i].v1;
        _VectorMA(v, radius / 2.0f32, right, &mut faces[i].v1);

        _VectorMA(tip, -radius / 2.0f32, fwd, &mut faces[i].v2);
        let v = faces[i].v2;
        _VectorMA(v, -radius / 2.0f32, right, &mut faces[i].v2);

        _VectorMA(tip, -radius / 2.0f32, fwd, &mut faces[i].v3);
        let v = faces[i].v3;
        _VectorMA(v, radius / 2.0f32, right, &mut faces[i].v3);

        i += 1;

        // bottom surface — face 1
        _VectorMA(base, radius / 2.0f32, fwd, &mut faces[i].v1);
        let v = faces[i].v1;
        _VectorMA(v, -radius / 2.0f32, right, &mut faces[i].v1);

        _VectorMA(base, radius / 2.0f32, fwd, &mut faces[i].v2);
        let v = faces[i].v2;
        _VectorMA(v, radius / 2.0f32, right, &mut faces[i].v2);

        _VectorMA(base, -radius / 2.0f32, fwd, &mut faces[i].v3);
        let v = faces[i].v3;
        _VectorMA(v, -radius / 2.0f32, right, &mut faces[i].v3);

        i += 1;

        // face 2
        _VectorMA(base, radius / 2.0f32, fwd, &mut faces[i].v1);
        let v = faces[i].v1;
        _VectorMA(v, radius / 2.0f32, right, &mut faces[i].v1);

        _VectorMA(base, -radius / 2.0f32, fwd, &mut faces[i].v2);
        let v = faces[i].v2;
        _VectorMA(v, -radius / 2.0f32, right, &mut faces[i].v2);

        _VectorMA(base, -radius / 2.0f32, fwd, &mut faces[i].v3);
        let v = faces[i].v3;
        _VectorMA(v, radius / 2.0f32, right, &mut faces[i].v3);

        i += 1;

        // yeah.. always going to be 12 I suppose.
        *fNum = i as c_int;
        *fList = faces.as_mut_ptr();
    }
}

/// Raven `G_SabCol_CalcPlaneEq`.
///
/// Source: `oracle/codemp/game/w_saber.c:2573-2579`
pub fn G_SabCol_CalcPlaneEq(x: vec3_t, y: vec3_t, z: vec3_t, planeEq: *mut f32) {
    unsafe {
        *planeEq.add(0) = x[1] * (y[2] - z[2]) + y[1] * (z[2] - x[2]) + z[1] * (x[2] - y[2]);
        *planeEq.add(1) = x[2] * (y[0] - z[0]) + y[2] * (z[0] - x[0]) + z[2] * (x[0] - y[0]);
        *planeEq.add(2) = x[0] * (y[1] - z[1]) + y[0] * (z[1] - x[1]) + z[0] * (x[1] - y[1]);
        *planeEq.add(3) = -(x[0] * (y[1] * z[2] - z[1] * y[2])
            + y[0] * (z[1] * x[2] - x[1] * z[2])
            + z[0] * (x[1] * y[2] - y[1] * x[2]));
    }
}

/// Raven `G_SabCol_PointRelativeToPlane`.
///
/// Source: `oracle/codemp/game/w_saber.c:2582-2596`
pub fn G_SabCol_PointRelativeToPlane(pos: vec3_t, side: *mut f32, planeEq: *mut f32) -> c_int {
    unsafe {
        *side = *planeEq.add(0) * pos[0]
            + *planeEq.add(1) * pos[1]
            + *planeEq.add(2) * pos[2]
            + *planeEq.add(3);

        if *side > 0.0f32 {
            1
        } else if *side < 0.0f32 {
            -1
        } else {
            0
        }
    }
}

/// Raven `G_SaberFaceCollisionCheck`.
///
/// Source: `oracle/codemp/game/w_saber.c:2599-2697`
pub fn G_SaberFaceCollisionCheck(
    ctx: &mut GameContext,
    fNum: c_int,
    fList: *mut saberFace_t,
    atkStart: vec3_t,
    atkEnd: vec3_t,
    atkMins: &mut vec3_t,
    atkMaxs: &mut vec3_t,
    impactPoint: &mut vec3_t,
) -> bool {
    let _ = ctx;
    unsafe {
        let mut planeEq: [f32; 4] = [0.0; 4];
        let mut side: f32 = 0.0;
        let mut side2: f32 = 0.0;
        let mut dist: f32;
        let mut dir: vec3_t = [0.0; 3];
        let mut point: vec3_t = [0.0; 3];
        let mut i: c_int = 0;

        if VectorCompare(*atkMins, vec3_origin) && VectorCompare(*atkMaxs, vec3_origin) {
            VectorSet(atkMins, -1.0f32, -1.0f32, -1.0f32);
            VectorSet(atkMaxs, 1.0f32, 1.0f32, 1.0f32);
        }

        _VectorSubtract(atkEnd, atkStart, &mut dir);

        let mut fl = fList;
        while i < fNum {
            G_SabCol_CalcPlaneEq((*fl).v1, (*fl).v2, (*fl).v3, planeEq.as_mut_ptr());

            if G_SabCol_PointRelativeToPlane(atkStart, &mut side, planeEq.as_mut_ptr())
                != G_SabCol_PointRelativeToPlane(atkEnd, &mut side2, planeEq.as_mut_ptr())
            {
                //start/end points intersect with the plane
                let mut extruded: vec3_t = [0.0; 3];
                let mut minPoint: vec3_t = [0.0; 3];
                let mut maxPoint: vec3_t = [0.0; 3];
                let mut planeNormal: vec3_t = [0.0; 3];
                let mut facing: c_int;

                planeNormal = [planeEq[0], planeEq[1], planeEq[2]];
                side2 = planeNormal[0] * dir[0] + planeNormal[1] * dir[1] + planeNormal[2] * dir[2];

                dist = side / side2;
                _VectorMA(atkStart, -dist, dir, &mut point);

                _VectorAdd(point, *atkMins, &mut minPoint);
                _VectorAdd(point, *atkMaxs, &mut maxPoint);

                //point is now the point at which we intersect on the plane.
                //see if that point is within the edges of the face.
                _VectorMA((*fl).v1, -2.0f32, planeNormal, &mut extruded);
                G_SabCol_CalcPlaneEq((*fl).v1, (*fl).v2, extruded, planeEq.as_mut_ptr());
                facing = G_SabCol_PointRelativeToPlane(point, &mut side, planeEq.as_mut_ptr());

                if facing < 0 {
                    //not intersecting.. let's try with the mins/maxs and see if they interesect on the edge plane
                    facing =
                        G_SabCol_PointRelativeToPlane(minPoint, &mut side, planeEq.as_mut_ptr());
                    if facing < 0 {
                        facing = G_SabCol_PointRelativeToPlane(
                            maxPoint,
                            &mut side,
                            planeEq.as_mut_ptr(),
                        );
                    }
                }

                if facing >= 0 {
                    //first edge is facing...
                    _VectorMA((*fl).v2, -2.0f32, planeNormal, &mut extruded);
                    G_SabCol_CalcPlaneEq((*fl).v2, (*fl).v3, extruded, planeEq.as_mut_ptr());
                    facing = G_SabCol_PointRelativeToPlane(point, &mut side, planeEq.as_mut_ptr());

                    if facing < 0 {
                        facing = G_SabCol_PointRelativeToPlane(
                            minPoint,
                            &mut side,
                            planeEq.as_mut_ptr(),
                        );
                        if facing < 0 {
                            facing = G_SabCol_PointRelativeToPlane(
                                maxPoint,
                                &mut side,
                                planeEq.as_mut_ptr(),
                            );
                        }
                    }

                    if facing >= 0 {
                        //second edge is facing...
                        _VectorMA((*fl).v3, -2.0f32, planeNormal, &mut extruded);
                        G_SabCol_CalcPlaneEq((*fl).v3, (*fl).v1, extruded, planeEq.as_mut_ptr());
                        facing =
                            G_SabCol_PointRelativeToPlane(point, &mut side, planeEq.as_mut_ptr());

                        if facing < 0 {
                            facing = G_SabCol_PointRelativeToPlane(
                                minPoint,
                                &mut side,
                                planeEq.as_mut_ptr(),
                            );
                            if facing < 0 {
                                facing = G_SabCol_PointRelativeToPlane(
                                    maxPoint,
                                    &mut side,
                                    planeEq.as_mut_ptr(),
                                );
                            }
                        }

                        if facing >= 0 {
                            //third edge is facing.. success
                            *impactPoint = point;
                            return true;
                        }
                    }
                }
            }

            i += 1;
            fl = fl.add(1);
        }

        //did not hit anything
        false
    }
}

/// Raven `G_SaberCollide`.
///
/// Source: `oracle/codemp/game/w_saber.c:2700-2777`
pub fn G_SaberCollide(
    ctx: &mut GameContext,
    atk: EntityId,
    def: EntityId,
    atkStart: vec3_t,
    atkEnd: vec3_t,
    mut atkMins: vec3_t,
    mut atkMaxs: vec3_t,
    mut impactPoint: vec3_t,
) -> bool {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let atk: *mut gentity_t = ctx.entity_mut(atk);
    let def: *mut gentity_t = ctx.entity_mut(def);
    unsafe {
        let mut i: c_int;
        let mut j: c_int;

        if ctx.world.cvars.g_saberBladeFaces.integer == 0 {
            //detailed check not enabled
            return true;
        }

        if (*atk).inuse == 0
            || (*atk).client.is_null()
            || (*def).inuse == 0
            || (*def).client.is_null()
        {
            //must have 2 clients and a valid saber entity
            return false;
        }

        let dc = (*def).client;

        i = 0;
        while i < MAX_SABERS as c_int {
            j = 0;
            if (*dc).saber[i as usize].model[0] != 0 {
                //valid saber on the defender
                let mut v: vec3_t = [0.0; 3];
                let mut fwd: vec3_t = [0.0; 3];
                let mut right: vec3_t = [0.0; 3];
                let mut base: vec3_t = [0.0; 3];
                let mut tip: vec3_t = [0.0; 3];
                let mut fNum: c_int = 0;
                let mut fList: *mut saberFace_t = core::ptr::null_mut();

                //go through each blade on the defender's sabers
                while j < (*dc).saber[i as usize].numBlades {
                    let blade = &mut (*dc).saber[i as usize].blade[j as usize] as *mut bladeInfo_t;

                    if (ctx.world.level.time - (*blade).storageTime) < 200 {
                        //recently updated
                        //first get base and tip of blade
                        base = (*blade).muzzlePoint;
                        _VectorMA(base, (*blade).lengthMax, (*blade).muzzleDir, &mut tip);

                        //Now get relative angles between the points
                        _VectorSubtract(tip, base, &mut v);
                        let vv = v;
                        vectoangles(vv, &mut v);
                        AngleVectors(v, None, Some(&mut right), Some(&mut fwd));

                        //now build collision faces for this blade
                        G_BuildSaberFaces(
                            ctx,
                            base,
                            tip,
                            (*blade).radius * 3.0f32,
                            fwd,
                            right,
                            &mut fNum,
                            &mut fList,
                        );
                        if fNum > 0 {
                            if G_SaberFaceCollisionCheck(
                                ctx,
                                fNum,
                                fList,
                                atkStart,
                                atkEnd,
                                &mut atkMins,
                                &mut atkMaxs,
                                &mut impactPoint,
                            ) {
                                //collided
                                return true;
                            }
                        }
                    }
                    j += 1;
                }
            }
            i += 1;
        }

        false
    }
}

/// Raven `WP_SaberBladeLength`.
///
/// Source: `oracle/codemp/game/w_saber.c:2779-2791`
pub fn WP_SaberBladeLength(saber: *mut saberInfo_t) -> f32 {
    // return largest length
    unsafe {
        let mut len = 0.0f32;
        for i in 0..(*saber).numBlades {
            if (*saber).blade[i as usize].lengthMax > len {
                len = (*saber).blade[i as usize].lengthMax;
            }
        }
        len
    }
}

/// Raven `WP_SaberLength`.
///
/// Source: `oracle/codemp/game/w_saber.c:2793-2813`
pub fn WP_SaberLength(ent: &gentity_t) -> f32 {
    // return largest length
    let client = ent.client;
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    if client.is_null() {
        return 0.0f32;
    }
    unsafe {
        let mut best_len = 0.0f32;
        for i in 0..MAX_SABERS as usize {
            let len = WP_SaberBladeLength(&mut (*client).saber[i]);
            if len > best_len {
                best_len = len;
            }
        }
        best_len
    }
}

/// Raven `WPDEBUG_SaberColor`.
///
/// Source: `oracle/codemp/game/w_saber.c:2814-2840`
pub fn WPDEBUG_SaberColor(saberColor: saber_colors_t) -> c_int {
    match saberColor {
        SABER_RED => 0x000000ff,
        SABER_ORANGE => 0x000088ff,
        SABER_YELLOW => 0x0000ffff,
        SABER_GREEN => 0x0000ff00,
        SABER_BLUE => 0x00ff0000,
        SABER_PURPLE => 0x00ff00ff,
        _ => 0x00ffffff, // white
    }
}

/// Raven `WP_SabersIntersect`.
///
/// Source: `oracle/codemp/game/w_saber.c:2851-2985`
pub fn WP_SabersIntersect(
    ctx: &mut GameContext,
    ent1: Option<EntityId>,
    ent1SaberNum: c_int,
    ent1BladeNum: c_int,
    ent2: Option<EntityId>,
    checkDir: qboolean,
) -> bool {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let ent1: *mut gentity_t = unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), ent1) };
    let ent2: *mut gentity_t = unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), ent2) };
    let _ = ctx;
    unsafe {
        let mut saberBase1: vec3_t = [0.0; 3];
        let mut saberTip1: vec3_t = [0.0; 3];
        let mut saberBaseNext1: vec3_t = [0.0; 3];
        let mut saberTipNext1: vec3_t = [0.0; 3];
        let mut saberBase2: vec3_t = [0.0; 3];
        let mut saberTip2: vec3_t = [0.0; 3];
        let mut saberBaseNext2: vec3_t = [0.0; 3];
        let mut saberTipNext2: vec3_t = [0.0; 3];
        let mut ent2SaberNum: c_int = 0;
        let mut ent2BladeNum: c_int = 0;
        let mut dir: vec3_t = [0.0; 3];

        if ent1.is_null() || ent2.is_null() {
            return false;
        }
        if (*ent1).client.is_null() || (*ent2).client.is_null() {
            return false;
        }
        let ec1 = (*ent1).client;
        let ec2 = (*ent2).client;
        if BG_SabersOff(&mut (*ec1).ps as *mut playerState_t) != 0
            || BG_SabersOff(&mut (*ec2).ps as *mut playerState_t) != 0
        {
            return false;
        }

        ent2SaberNum = 0;
        while ent2SaberNum < MAX_SABERS as c_int {
            if (*ec2).saber[ent2SaberNum as usize].r#type != saberType_t::SABER_NONE {
                ent2BladeNum = 0;
                while ent2BladeNum < (*ec2).saber[ent2SaberNum as usize].numBlades {
                    if (*ec2).saber[ent2SaberNum as usize].blade[ent2BladeNum as usize].lengthMax
                        > 0.0
                    {
                        //valid saber and this blade is on
                        //if ( ent1->client->saberInFlight )
                        {
                            let b1 =
                                &(*ec1).saber[ent1SaberNum as usize].blade[ent1BladeNum as usize];
                            saberBase1 = b1.muzzlePointOld;
                            saberBaseNext1 = b1.muzzlePoint;

                            _VectorSubtract(b1.muzzlePoint, b1.muzzlePointOld, &mut dir);
                            VectorNormalize(&mut dir);
                            let tmp = saberBaseNext1;
                            _VectorMA(tmp, SABER_EXTRAPOLATE_DIST, dir, &mut saberBaseNext1);

                            _VectorMA(
                                saberBase1,
                                b1.lengthMax + SABER_EXTRAPOLATE_DIST,
                                b1.muzzleDirOld,
                                &mut saberTip1,
                            );
                            _VectorMA(
                                saberBaseNext1,
                                b1.lengthMax + SABER_EXTRAPOLATE_DIST,
                                b1.muzzleDir,
                                &mut saberTipNext1,
                            );

                            _VectorSubtract(saberTipNext1, saberTip1, &mut dir);
                            VectorNormalize(&mut dir);
                            let tmp = saberTipNext1;
                            _VectorMA(tmp, SABER_EXTRAPOLATE_DIST, dir, &mut saberTipNext1);
                        }

                        //if ( ent2->client->saberInFlight )
                        {
                            let b2 =
                                &(*ec2).saber[ent2SaberNum as usize].blade[ent2BladeNum as usize];
                            saberBase2 = b2.muzzlePointOld;
                            saberBaseNext2 = b2.muzzlePoint;

                            _VectorSubtract(b2.muzzlePoint, b2.muzzlePointOld, &mut dir);
                            VectorNormalize(&mut dir);
                            let tmp = saberBaseNext2;
                            _VectorMA(tmp, SABER_EXTRAPOLATE_DIST, dir, &mut saberBaseNext2);

                            _VectorMA(
                                saberBase2,
                                b2.lengthMax + SABER_EXTRAPOLATE_DIST,
                                b2.muzzleDirOld,
                                &mut saberTip2,
                            );
                            _VectorMA(
                                saberBaseNext2,
                                b2.lengthMax + SABER_EXTRAPOLATE_DIST,
                                b2.muzzleDir,
                                &mut saberTipNext2,
                            );

                            _VectorSubtract(saberTipNext2, saberTip2, &mut dir);
                            VectorNormalize(&mut dir);
                            let tmp = saberTipNext2;
                            _VectorMA(tmp, SABER_EXTRAPOLATE_DIST, dir, &mut saberTipNext2);
                        }

                        if checkDir != 0 {
                            //check the direction of the two swings to make sure the sabers are swinging towards each other
                            let mut saberDir1: vec3_t = [0.0; 3];
                            let mut saberDir2: vec3_t = [0.0; 3];
                            let mut dot: f32 = 0.0;

                            _VectorSubtract(saberTipNext1, saberTip1, &mut saberDir1);
                            _VectorSubtract(saberTipNext2, saberTip2, &mut saberDir2);
                            VectorNormalize(&mut saberDir1);
                            VectorNormalize(&mut saberDir2);
                            if _DotProduct(saberDir1, saberDir2) > 0.6f32 {
                                //sabers moving in same dir, probably didn't actually hit
                                ent2BladeNum += 1;
                                continue;
                            }
                            //now check orientation of sabers, make sure they're not parallel or close to it
                            dot = _DotProduct(
                                (*ec1).saber[ent1SaberNum as usize].blade[ent1BladeNum as usize]
                                    .muzzleDir,
                                (*ec2).saber[ent2SaberNum as usize].blade[ent2BladeNum as usize]
                                    .muzzleDir,
                            );
                            if dot > 0.9f32 || dot < -0.9f32 {
                                //too parallel to really block effectively?
                                ent2BladeNum += 1;
                                continue;
                            }
                        }

                        if ctx.world.cvars.g_saberDebugBox.integer == 2
                            || ctx.world.cvars.g_saberDebugBox.integer == 4
                        {
                            G_TestLine(
                                ctx,
                                saberBase1,
                                saberTip1,
                                (*ec1).saber[ent1SaberNum as usize].blade[ent1BladeNum as usize]
                                    .color,
                                500,
                            );
                            G_TestLine(
                                ctx,
                                saberTip1,
                                saberTipNext1,
                                (*ec1).saber[ent1SaberNum as usize].blade[ent1BladeNum as usize]
                                    .color,
                                500,
                            );
                            G_TestLine(
                                ctx,
                                saberTipNext1,
                                saberBase1,
                                (*ec1).saber[ent1SaberNum as usize].blade[ent1BladeNum as usize]
                                    .color,
                                500,
                            );

                            G_TestLine(
                                ctx,
                                saberBase2,
                                saberTip2,
                                (*ec2).saber[ent2SaberNum as usize].blade[ent2BladeNum as usize]
                                    .color,
                                500,
                            );
                            G_TestLine(
                                ctx,
                                saberTip2,
                                saberTipNext2,
                                (*ec2).saber[ent2SaberNum as usize].blade[ent2BladeNum as usize]
                                    .color,
                                500,
                            );
                            G_TestLine(
                                ctx,
                                saberTipNext2,
                                saberBase2,
                                (*ec2).saber[ent2SaberNum as usize].blade[ent2BladeNum as usize]
                                    .color,
                                500,
                            );
                        }

                        if tri_tri_intersect(
                            saberBase1,
                            saberTip1,
                            saberBaseNext1,
                            saberBase2,
                            saberTip2,
                            saberBaseNext2,
                        ) != 0
                        {
                            return true;
                        }
                        if tri_tri_intersect(
                            saberBase1,
                            saberTip1,
                            saberBaseNext1,
                            saberBase2,
                            saberTip2,
                            saberTipNext2,
                        ) != 0
                        {
                            return true;
                        }
                        if tri_tri_intersect(
                            saberBase1,
                            saberTip1,
                            saberTipNext1,
                            saberBase2,
                            saberTip2,
                            saberBaseNext2,
                        ) != 0
                        {
                            return true;
                        }
                        if tri_tri_intersect(
                            saberBase1,
                            saberTip1,
                            saberTipNext1,
                            saberBase2,
                            saberTip2,
                            saberTipNext2,
                        ) != 0
                        {
                            return true;
                        }
                    }
                    ent2BladeNum += 1;
                }
            }
            ent2SaberNum += 1;
        }
        false
    }
}

/// Raven `G_PowerLevelForSaberAnim`.
///
/// Source: `oracle/codemp/game/w_saber.c:2987-3501`
pub fn G_PowerLevelForSaberAnim(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    saberNum: c_int,
    mySaberHit: qboolean,
) -> c_int {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), ent) };
    unsafe {
        if ent.is_null() || (*ent).client.is_null() || saberNum >= MAX_SABERS as c_int {
            return FORCE_LEVEL_0;
        }
        let ec = (*ent).client;
        let anim: c_int = (*ec).ps.torsoAnim;
        let animTimer: c_int = (*ec).ps.torsoTimer;
        let animTimeElapsed: c_int =
            mp_bg::bg_panimate::BG_AnimLength(&ctx.world.bg_state, (*ent).localAnimIndex, anim)
                - animTimer;
        let saber = &(*ec).saber[saberNum as usize];

        if anim >= BOTH_A1_T__B_ as c_int && anim <= BOTH_D1_B____ as c_int {
            //FIXME: these two need their own style
            if saber.r#type == saberType_t::SABER_LANCE {
                return FORCE_LEVEL_4;
            } else if saber.r#type == saberType_t::SABER_TRIDENT {
                return FORCE_LEVEL_3;
            }
            return FORCE_LEVEL_1;
        }
        if anim >= BOTH_A2_T__B_ as c_int && anim <= BOTH_D2_B____ as c_int {
            return FORCE_LEVEL_2;
        }
        if anim >= BOTH_A3_T__B_ as c_int && anim <= BOTH_D3_B____ as c_int {
            return FORCE_LEVEL_3;
        }
        if anim >= BOTH_A4_T__B_ as c_int && anim <= BOTH_D4_B____ as c_int {
            //desann
            return FORCE_LEVEL_4;
        }
        if anim >= BOTH_A5_T__B_ as c_int && anim <= BOTH_D5_B____ as c_int {
            //tavion
            return FORCE_LEVEL_2;
        }
        if anim >= BOTH_A6_T__B_ as c_int && anim <= BOTH_D6_B____ as c_int {
            //dual
            return FORCE_LEVEL_2;
        }
        if anim >= BOTH_A7_T__B_ as c_int && anim <= BOTH_D7_B____ as c_int {
            //staff
            return FORCE_LEVEL_2;
        }
        if anim >= BOTH_P1_S1_T_ as c_int && anim <= BOTH_H1_S1_BR as c_int {
            //parries, knockaways and broken parries
            return FORCE_LEVEL_1; //FIXME: saberAnimLevel?
        }
        match anim {
            a if a == BOTH_A2_STABBACK1 as c_int => {
                if mySaberHit != 0 {
                    //someone else hit my saber, not asking for damage level, but defense strength
                    return FORCE_LEVEL_1;
                }
                if animTimer < 450 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 400 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_ATTACK_BACK as c_int => {
                if animTimer < 500 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_CROUCHATTACKBACK1 as c_int => {
                if animTimer < 800 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_BUTTERFLY_LEFT as c_int
                || a == BOTH_BUTTERFLY_RIGHT as c_int
                || a == BOTH_BUTTERFLY_FL1 as c_int
                || a == BOTH_BUTTERFLY_FR1 as c_int =>
            {
                //FIXME: break up?
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_FJSS_TR_BL as c_int || a == BOTH_FJSS_TL_BR as c_int => {
                //FIXME: break up?
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_K1_S1_T_ as c_int
                || a == BOTH_K1_S1_TR as c_int
                || a == BOTH_K1_S1_TL as c_int
                || a == BOTH_K1_S1_BL as c_int
                || a == BOTH_K1_S1_B_ as c_int
                || a == BOTH_K1_S1_BR as c_int =>
            {
                //FIXME: break up?
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_LUNGE2_B__T_ as c_int => {
                if mySaberHit != 0 {
                    return FORCE_LEVEL_1;
                }
                if animTimer < 400 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 150 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_FORCELEAP2_T__B_ as c_int => {
                if animTimer < 400 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 550 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_VS_ATR_S as c_int
                || a == BOTH_VS_ATL_S as c_int
                || a == BOTH_VT_ATR_S as c_int
                || a == BOTH_VT_ATL_S as c_int =>
            {
                return FORCE_LEVEL_3; //???
            }
            a if a == BOTH_JUMPFLIPSLASHDOWN1 as c_int => {
                if animTimer <= 1000 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 600 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_JUMPFLIPSTABDOWN as c_int => {
                if animTimer <= 1300 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed <= 300 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_JUMPATTACK6 as c_int => {
                if (animTimer >= 1450 && animTimeElapsed >= 400)
                    || (animTimer >= 400 && animTimeElapsed >= 1100)
                {
                    //pretty much sideways
                    return FORCE_LEVEL_3;
                }
                return FORCE_LEVEL_0;
            }
            a if a == BOTH_JUMPATTACK7 as c_int => {
                if animTimer <= 1200 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 200 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_SPINATTACK6 as c_int => {
                if animTimeElapsed <= 200 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_2; //FORCE_LEVEL_3;
            }
            a if a == BOTH_SPINATTACK7 as c_int => {
                if animTimer <= 500 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 500 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_2; //FORCE_LEVEL_3;
            }
            a if a == BOTH_FORCELONGLEAP_ATTACK as c_int => {
                if animTimeElapsed <= 200 {
                    //1st four frames of anim
                    return FORCE_LEVEL_3;
                }
            }
            a if a == BOTH_STABDOWN as c_int => {
                if animTimer <= 900 {
                    return FORCE_LEVEL_3;
                }
            }
            a if a == BOTH_STABDOWN_STAFF as c_int => {
                if animTimer <= 850 {
                    return FORCE_LEVEL_3;
                }
            }
            a if a == BOTH_STABDOWN_DUAL as c_int => {
                if animTimer <= 900 {
                    return FORCE_LEVEL_3;
                }
            }
            a if a == BOTH_A6_SABERPROTECT as c_int => {
                if animTimer < 650 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 250 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_A7_SOULCAL as c_int => {
                if animTimer < 650 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 600 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_A1_SPECIAL as c_int => {
                if animTimer < 600 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 200 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_A2_SPECIAL as c_int => {
                if animTimer < 300 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 200 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_A3_SPECIAL as c_int => {
                if animTimer < 700 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 200 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_FLIP_ATTACK7 as c_int => {
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_PULL_IMPALE_STAB as c_int => {
                if mySaberHit != 0 {
                    return FORCE_LEVEL_1;
                }
                if animTimer < 1000 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_PULL_IMPALE_SWING as c_int => {
                if animTimer < 500 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 650 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_ALORA_SPIN_SLASH as c_int => {
                if animTimer < 900 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 250 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_A6_FB as c_int => {
                if mySaberHit != 0 {
                    return FORCE_LEVEL_1;
                }
                if animTimer < 250 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 250 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_A6_LR as c_int => {
                if mySaberHit != 0 {
                    return FORCE_LEVEL_1;
                }
                if animTimer < 250 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 250 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_3;
            }
            a if a == BOTH_A7_HILT as c_int => {
                return FORCE_LEVEL_0;
            }
            //===SABERLOCK SUPERBREAKS START===
            a if a == BOTH_LK_S_DL_T_SB_1_W as c_int => {
                if animTimer < 700 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_5;
            }
            a if a == BOTH_LK_S_ST_S_SB_1_W as c_int => {
                if animTimer < 300 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_5;
            }
            a if a == BOTH_LK_S_DL_S_SB_1_W as c_int || a == BOTH_LK_S_S_S_SB_1_W as c_int => {
                if animTimer < 700 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 400 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_5;
            }
            a if a == BOTH_LK_S_ST_T_SB_1_W as c_int || a == BOTH_LK_S_S_T_SB_1_W as c_int => {
                if animTimer < 150 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 400 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_5;
            }
            a if a == BOTH_LK_DL_DL_T_SB_1_W as c_int => {
                return FORCE_LEVEL_5;
            }
            a if a == BOTH_LK_DL_DL_S_SB_1_W as c_int || a == BOTH_LK_DL_ST_S_SB_1_W as c_int => {
                if animTimeElapsed < 1000 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_5;
            }
            a if a == BOTH_LK_DL_ST_T_SB_1_W as c_int => {
                if animTimer < 950 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 650 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_5;
            }
            a if a == BOTH_LK_DL_S_S_SB_1_W as c_int => {
                if saberNum != 0 {
                    //only right hand saber does damage in this suberbreak
                    return FORCE_LEVEL_0;
                }
                if animTimer < 900 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 450 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_5;
            }
            a if a == BOTH_LK_DL_S_T_SB_1_W as c_int => {
                if saberNum != 0 {
                    return FORCE_LEVEL_0;
                }
                if animTimer < 250 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 150 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_5;
            }
            a if a == BOTH_LK_ST_DL_S_SB_1_W as c_int => {
                return FORCE_LEVEL_5;
            }
            a if a == BOTH_LK_ST_DL_T_SB_1_W as c_int => {
                //special suberbreak - doesn't kill, just kicks them backwards
                return FORCE_LEVEL_0;
            }
            a if a == BOTH_LK_ST_ST_S_SB_1_W as c_int || a == BOTH_LK_ST_S_S_SB_1_W as c_int => {
                if animTimer < 800 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 350 {
                    return FORCE_LEVEL_0;
                }
                return FORCE_LEVEL_5;
            }
            a if a == BOTH_LK_ST_ST_T_SB_1_W as c_int || a == BOTH_LK_ST_S_T_SB_1_W as c_int => {
                return FORCE_LEVEL_5;
            }
            //===SABERLOCK SUPERBREAKS END===
            a if a == BOTH_HANG_ATTACK as c_int => {
                //FIME: break up
                if animTimer < 1000 {
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 250 {
                    return FORCE_LEVEL_0;
                } else {
                    //sweet spot
                    return FORCE_LEVEL_5;
                }
            }
            a if a == BOTH_ROLL_STAB as c_int => {
                if mySaberHit != 0 {
                    return FORCE_LEVEL_1;
                }
                if animTimeElapsed > 400 {
                    return FORCE_LEVEL_0;
                } else {
                    return FORCE_LEVEL_3;
                }
            }
            _ => {}
        }
        FORCE_LEVEL_0
    }
}

/// Raven `WP_SaberClearDamage`.
///
/// Source: `oracle/codemp/game/w_saber.c:3512-3526`
pub fn WP_SaberClearDamage(ctx: &mut GameContext) {
    let g = &mut ctx.world.globals;
    for ven in 0..MAX_SABER_VICTIMS as usize {
        g.victimEntityNum[ven] = ENTITYNUM_NONE;
        g.victimHitEffectDone[ven] = qfalse;
        g.totalDmg[ven] = 0.0;
        g.dmgDir[ven] = [0.0; 3];
        g.dmgSpot[ven] = [0.0; 3];
        g.dismemberDmg[ven] = qfalse;
        g.saberKnockbackFlags[ven] = 0;
    }
    g.numVictims = 0;
}

/// Raven `WP_SaberDamageAdd`.
///
/// Source: `oracle/codemp/game/w_saber.c:3528-3575`
pub fn WP_SaberDamageAdd(
    ctx: &mut GameContext,
    trVictimEntityNum: c_int,
    trDmgDir: vec3_t,
    trDmgSpot: vec3_t,
    trDmg: c_int,
    doDismemberment: qboolean,
    knockBackFlags: c_int,
) {
    if trVictimEntityNum < 0 || trVictimEntityNum >= ENTITYNUM_WORLD {
        return;
    }

    if trDmg != 0 {
        // did some damage to something
        let g = &mut ctx.world.globals;
        let mut curVictim = 0;
        let mut i = 0;

        while i < g.numVictims {
            if g.victimEntityNum[i as usize] == trVictimEntityNum {
                // already hit this guy before
                curVictim = i;
                break;
            }
            i += 1;
        }
        if i == g.numVictims {
            // haven't hit this guy before
            if g.numVictims + 1 >= MAX_SABER_VICTIMS {
                // can't add another victim at this time
                return;
            }
            // add a new victim to the list
            curVictim = g.numVictims;
            g.victimEntityNum[g.numVictims as usize] = trVictimEntityNum;
            g.numVictims += 1;
        }

        let cv = curVictim as usize;
        g.totalDmg[cv] += trDmg as f32;
        if VectorCompare(g.dmgDir[cv], vec3_origin) {
            g.dmgDir[cv] = trDmgDir;
        }
        if VectorCompare(g.dmgSpot[cv], vec3_origin) {
            g.dmgSpot[cv] = trDmgSpot;
        }
        if doDismemberment != 0 {
            g.dismemberDmg[cv] = qtrue;
        }
        g.saberKnockbackFlags[cv] |= knockBackFlags;
    }
}

/// Raven `WP_SaberApplyDamage`.
///
/// Source: `oracle/codemp/game/w_saber.c:3577-3605`
pub fn WP_SaberApplyDamage(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.globals.numVictims == 0 {
        return;
    }
    let mut i = 0;
    while i < ctx.world.globals.numVictims {
        let iu = i as usize;
        let mut dflags = 0;

        let victim_id = EntityId(ctx.world.globals.victimEntityNum[iu] as u32);

        // nmckenzie: SABER_DAMAGE_WALLS
        if ctx.world.entity(victim_id).client.is_null() {
            ctx.world.globals.totalDmg[iu] *= ctx.world.cvars.g_saberWallDamageScale.value;
        }

        if ctx.world.globals.dismemberDmg[iu] == 0 {
            // don't do dismemberment!
            dflags |= DAMAGE_NO_DISMEMBER;
        }
        dflags |= ctx.world.globals.saberKnockbackFlags[iu];

        let dmg_spot = ctx.world.globals.dmgSpot[iu];
        let total_dmg = ctx.world.globals.totalDmg[iu] as c_int;
        // STAGE-2b: irreducible — G_Damage's `dir` is a `&mut vec3_t` out-param,
        // so a checked `&mut ctx.world.globals.dmgDir[iu]` would alias `ctx`
        // across the call; derive it through the raw world so it holds no borrow.
        let dmg_dir = Some(unsafe { &mut (*ctx.world_raw()).globals.dmgDir[iu] });
        G_Damage(
            ctx,
            Some(victim_id),
            Some(self_),
            Some(self_),
            dmg_dir,
            dmg_spot,
            total_dmg,
            dflags,
            MOD_SABER as c_int,
        );
        i += 1;
    }
}

/// Raven `WP_SaberDoHit`.
///
/// Source: `oracle/codemp/game/w_saber.c:3608-3697`
pub fn WP_SaberDoHit(ctx: &mut GameContext, self_: EntityId, saberNum: c_int, bladeNum: c_int) {
    if ctx.world.globals.numVictims == 0 {
        return;
    }
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let sc = ctx.world.entity(self_).client;
    unsafe {
        let mut i = 0;
        while i < ctx.world.globals.numVictims {
            let iu = i as usize;
            i += 1;

            let mut isDroid = qfalse;

            if ctx.world.globals.victimHitEffectDone[iu] != 0 {
                continue;
            }

            ctx.world.globals.victimHitEffectDone[iu] = qtrue;

            let victim_id = EntityId(ctx.world.globals.victimEntityNum[iu] as u32);

            let dmg_spot = ctx.world.globals.dmgSpot[iu];
            if !ctx.world.entity(victim_id).client.is_null() {
                // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
                let vc = ctx.world.entity(victim_id).client;
                let npc_class = (*vc).NPC_class;

                if npc_class == CLASS_SEEKER
                    || npc_class == CLASS_PROBE
                    || npc_class == CLASS_MOUSE
                    || npc_class == CLASS_REMOTE
                    || npc_class == CLASS_GONK
                    || npc_class == CLASS_R2D2
                    || npc_class == CLASS_R5D2
                    || npc_class == CLASS_PROTOCOL
                    || npc_class == CLASS_MARK1
                    || npc_class == CLASS_MARK2
                    || npc_class == CLASS_INTERROGATOR
                    || npc_class == CLASS_ATST
                    || npc_class == CLASS_SENTRY
                {
                    // don't make "blood" sparks for droids.
                    isDroid = qtrue;
                }
            }

            let te_eid = G_TempEntity(ctx, dmg_spot, EV_SABER_HIT as c_int);
            // Raven's `if (te)` guard is vacuous — G_TempEntity never returns NULL.
            let victim_num = ctx.world.globals.victimEntityNum[iu];
            let self_num = ctx.world.entity(self_).s.number;
            let dmg_dir = ctx.world.globals.dmgDir[iu];
            let victim_client_null = ctx.world.entity(victim_id).client.is_null();
            let victim_etype = ctx.world.entity(victim_id).s.eType;
            let total_dmg = ctx.world.globals.totalDmg[iu];
            let te = ctx.entity_mut(te_eid);
            te.s.otherEntityNum = victim_num;
            te.s.otherEntityNum2 = self_num;
            te.s.weapon = saberNum;
            te.s.legsAnim = bladeNum;

            te.s.origin = dmg_spot;
            _VectorScale(dmg_dir, -1.0, &mut te.s.angles);

            if te.s.angles[0] == 0.0 && te.s.angles[1] == 0.0 && te.s.angles[2] == 0.0 {
                // don't let it play with no direction
                te.s.angles[1] = 1.0;
            }

            if isDroid == 0
                && (!victim_client_null
                    || victim_etype == ET_NPC as c_int
                    || victim_etype == ET_BODY as c_int)
            {
                if total_dmg < 5.0 {
                    te.s.eventParm = 3;
                } else if total_dmg < 20.0 {
                    te.s.eventParm = 2;
                } else {
                    te.s.eventParm = 1;
                }
            } else {
                let saber = &mut (*sc).saber[saberNum as usize] as *mut saberInfo_t;
                if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) == 0
                    && ((*saber).saberFlags2 & SFL2_NO_CLASH_FLARE) != 0
                {
                    // don't do clash flare
                } else if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) != 0
                    && ((*saber).saberFlags2 & SFL2_NO_CLASH_FLARE2) != 0
                {
                    // don't do clash flare
                } else {
                    if total_dmg > SABER_NONATTACK_DAMAGE as f32 {
                        let teS_id = G_TempEntity(ctx, dmg_spot, EV_SABER_CLASHFLARE as c_int);
                        ctx.entity_mut(teS_id).s.origin = dmg_spot;
                    }
                    ctx.entity_mut(te_eid).s.eventParm = 0;
                }
            }
        }
    }
}

/// Raven `WP_SaberRadiusDamage`.
///
/// Source: `oracle/codemp/game/w_saber.c:3701-3792`
pub fn WP_SaberRadiusDamage(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    point: vec3_t,
    radius: f32,
    damage: c_int,
    knockBack: f32,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), ent) };
    unsafe {
        if ent.is_null() || (*ent).client.is_null() {
            return;
        }
        if radius <= 0.0 || (damage <= 0 && knockBack <= 0.0) {
            return;
        }

        // Setup the bbox to search in.
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        for i in 0..3 {
            mins[i] = point[i] - radius;
            maxs[i] = point[i] + radius;
        }

        let mut radiusEnts: [c_int; 128] = [0; 128];
        let numEnts = trap::EntitiesInBox(
            ctx.engine,
            GEntitiesInBoxArgs::new(
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                radiusEnts.as_mut_ptr(),
                128,
            ),
        );

        for i in 0..numEnts {
            let radiusEnt =
                &mut ctx.world.g_entities[radiusEnts[i as usize] as usize] as *mut gentity_t;
            if (*radiusEnt).inuse == 0 {
                continue;
            }
            if radiusEnt == ent {
                // Skip myself.
                continue;
            }

            if (*radiusEnt).client.is_null() {
                // must be a client
                if G_EntIsBreakable(ctx, (*radiusEnt).s.number) != 0 {
                    // damage breakables within range, but not as much
                    let mut zeroDir = vec3_origin;
                    G_Damage(
                        ctx,
                        ctx.entity_id_of(radiusEnt),
                        ctx.entity_id_of(ent),
                        ctx.entity_id_of(ent),
                        Some(&mut zeroDir),
                        (*radiusEnt).r.currentOrigin,
                        10,
                        0,
                        MOD_MELEE as c_int,
                    );
                }
                continue;
            }

            let rClient = (*radiusEnt).client;
            if ((*rClient).ps.eFlags2 & EF2_HELD_BY_MONSTER) != 0 {
                // can't be one being held
                continue;
            }

            let ro = (*radiusEnt).r.currentOrigin;
            let mut entDir: vec3_t = [ro[0] - point[0], ro[1] - point[1], ro[2] - point[2]];
            let dist = VectorNormalize(&mut entDir);
            if dist <= radius {
                // in range
                if damage > 0 {
                    // do damage
                    let points = (damage as f32 * dist / radius).ceil() as c_int;
                    let mut zeroDir = vec3_origin;
                    G_Damage(
                        ctx,
                        ctx.entity_id_of(radiusEnt),
                        ctx.entity_id_of(ent),
                        ctx.entity_id_of(ent),
                        Some(&mut zeroDir),
                        (*radiusEnt).r.currentOrigin,
                        points,
                        DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE as c_int,
                    );
                }
                if knockBack > 0.0 {
                    // do knockback
                    if !(*radiusEnt).client.is_null()
                        && (*rClient).NPC_class != CLASS_RANCOR
                        && (*rClient).NPC_class != CLASS_ATST
                        && ((*radiusEnt).flags & FL_NO_KNOCKBACK) == 0
                    {
                        // don't throw them back
                        let knockbackStr = knockBack * dist / radius;
                        entDir[2] += 0.1;
                        VectorNormalize(&mut entDir);
                        G_Throw(
                            ctx,
                            ctx.entity_id_of(radiusEnt).unwrap(),
                            entDir,
                            knockbackStr,
                        );
                        if (*radiusEnt).health > 0 {
                            // still alive
                            if knockbackStr > 50.0 {
                                // close enough / knockback high enough to knock down
                                if dist < (radius * 0.5)
                                    || (*rClient).ps.groundEntityNum != ENTITYNUM_NONE
                                {
                                    G_Knockdown(ctx, ctx.entity_id_of(radiusEnt));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Raven `WP_SaberDoClash`.
///
/// Source: `oracle/codemp/game/w_saber.c:3798-3810`
pub fn WP_SaberDoClash(ctx: &mut GameContext, self_: EntityId, saberNum: c_int, bladeNum: c_int) {
    if ctx.world.globals.saberDoClashEffect != 0 {
        let clash_pos = ctx.world.globals.saberClashPos;
        let te = G_TempEntity(ctx, clash_pos, EV_SABER_BLOCK as c_int);
        // G_TempEntity always returns a live temp entity (Raven derefs it unchecked).
        let te_id = te;
        let self_number = ctx.world.entity(self_).s.number;
        let origin = ctx.world.globals.saberClashPos;
        let angles = ctx.world.globals.saberClashNorm;
        let event_parm = ctx.world.globals.saberClashEventParm;
        let te_e = ctx.world.entity_mut(te_id);
        te_e.s.origin = origin;
        te_e.s.angles = angles;
        te_e.s.eventParm = event_parm;
        te_e.s.otherEntityNum2 = self_number;
        te_e.s.weapon = saberNum;
        te_e.s.legsAnim = bladeNum;
    }
}

/// Raven `WP_SaberBounceSound`.
///
/// Source: `oracle/codemp/game/w_saber.c:3812-3844`
pub fn WP_SaberBounceSound(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    saberNum: c_int,
    bladeNum: c_int,
) {
    let mut index = 1;
    let ent = match ent {
        Some(e) => e,
        None => return,
    };
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let client = ctx.world.entity(ent).client;
    if client.is_null() {
        return;
    }
    index = ctx.world.bg_state.rng.Q_irand(1, 9);
    unsafe {
        let saber = &mut (*client).saber[saberNum as usize] as *mut saberInfo_t;

        if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) == 0 && (*saber).bounceSound[0] != 0 {
            let snd = (*saber).bounceSound[ctx.world.bg_state.rng.Q_irand(0, 2) as usize];
            G_Sound(ctx, Some(ent), CHAN_AUTO as c_int, snd);
        } else if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) != 0
            && (*saber).bounce2Sound[0] != 0
        {
            let snd = (*saber).bounce2Sound[ctx.world.bg_state.rng.Q_irand(0, 2) as usize];
            G_Sound(ctx, Some(ent), CHAN_AUTO as c_int, snd);
        } else if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) == 0
            && (*saber).blockSound[0] != 0
        {
            let snd = (*saber).blockSound[ctx.world.bg_state.rng.Q_irand(0, 2) as usize];
            G_Sound(ctx, Some(ent), CHAN_AUTO as c_int, snd);
        } else if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) != 0
            && (*saber).block2Sound[0] != 0
        {
            let snd = (*saber).block2Sound[ctx.world.bg_state.rng.Q_irand(0, 2) as usize];
            G_Sound(ctx, Some(ent), CHAN_AUTO as c_int, snd);
        } else {
            // `va("sound/weapons/saber/saberblock%d.wav", index)` — rendered as an
            // owned NUL-terminated string (appendix ruling: va -> owned String).
            let path = format!("sound/weapons/saber/saberblock{}.wav", index);
            let sound = G_SoundIndex(ctx, &path);
            G_Sound(
                ctx,
                Some(ent),
                CHAN_AUTO as c_int,
                sound,
            );
        }
    }
}

/// Raven `CheckSaberDamage`.
///
/// Source: `oracle/codemp/game/w_saber.c:3857-5273`
pub fn CheckSaberDamage(
    ctx: &mut GameContext,
    self_: EntityId,
    rSaberNum: c_int,
    rBladeNum: c_int,
    mut saberStart: vec3_t,
    // C `vec3_t saberEnd` decays to float*: the interpolate retrace loop's
    // writes propagate to the caller's array (reused for trail.tip and the
    // mid-point gate). By-value here dropped that write-back — the lockstep
    // frame-1806 saber-collision divergence (2026-07-14).
    saberEnd: &mut vec3_t,
    doInterpolate: qboolean,
    mut trMask: c_int,
    extrapolate: qboolean,
) -> bool {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        let base = ctx.world.g_entities.as_mut_ptr();
        let sc = (*self_).client;
        let mut tr: trace_t = core::mem::zeroed();
        let mut dir: vec3_t = [0.0; 3];
        let mut saberTrMins: vec3_t = [0.0; 3];
        let mut saberTrMaxs: vec3_t = [0.0; 3];
        let mut lastValidStart: vec3_t = [0.0; 3];
        let mut lastValidEnd: vec3_t = [0.0; 3];
        let mut selfSaberLevel: c_int;
        let mut otherSaberLevel: c_int = 0;
        let mut dmg: c_int = 0;
        let mut attackStr: c_int = 0;
        let mut saberBoxSize: f32 = ctx.world.cvars.d_saberBoxTraceSize.value;
        let mut idleDamage: qboolean = qfalse;
        let mut didHit = false;
        let mut sabersClashed: qboolean = qfalse;
        let mut unblockable: qboolean = qfalse;
        let mut didDefense: qboolean = qfalse;
        let mut didOffense: qboolean = qfalse;
        let mut saberTraceDone: qboolean = qfalse;
        let mut otherUnblockable: qboolean = qfalse;
        let mut tryDeflectAgain: qboolean = qfalse;
        let mut otherOwner: *mut gentity_t = core::ptr::null_mut();

        if BG_SabersOff(&mut (*sc).ps as *mut playerState_t) != 0 {
            return false;
        }

        selfSaberLevel =
            G_SaberAttackPower(ctx, ctx.entity_id_of(self_), SaberAttacking(&*(self_)));

        //Add the standard radius into the box size
        saberBoxSize += (*sc).saber[rSaberNum as usize].blade[rBladeNum as usize].radius * 0.5f32;

        if (*sc).ps.weaponTime <= 0 {
            //if not doing any attacks or anything, just use point traces.
            VectorClear(&mut saberTrMins);
            VectorClear(&mut saberTrMaxs);
        } else if ctx.world.cvars.d_saberGhoul2Collision.integer != 0 {
            if ctx.world.cvars.d_saberSPStyleDamage.integer != 0 {
                //SP-size saber damage traces
                VectorSet(&mut saberTrMins, -2.0, -2.0, -2.0);
                VectorSet(&mut saberTrMaxs, 2.0, 2.0, 2.0);
            } else {
                VectorSet(
                    &mut saberTrMins,
                    -saberBoxSize * 3.0,
                    -saberBoxSize * 3.0,
                    -saberBoxSize * 3.0,
                );
                VectorSet(
                    &mut saberTrMaxs,
                    saberBoxSize * 3.0,
                    saberBoxSize * 3.0,
                    saberBoxSize * 3.0,
                );
            }
        } else if (*sc).ps.fd.saberAnimLevel < FORCE_LEVEL_2 {
            //box trace for fast, because it doesn't get updated so often
            VectorSet(
                &mut saberTrMins,
                -saberBoxSize,
                -saberBoxSize,
                -saberBoxSize,
            );
            VectorSet(&mut saberTrMaxs, saberBoxSize, saberBoxSize, saberBoxSize);
        } else if ctx.world.cvars.d_saberAlwaysBoxTrace.integer != 0 {
            VectorSet(
                &mut saberTrMins,
                -saberBoxSize,
                -saberBoxSize,
                -saberBoxSize,
            );
            VectorSet(&mut saberTrMaxs, saberBoxSize, saberBoxSize, saberBoxSize);
        } else {
            //just trace the minimum blade radius
            saberBoxSize =
                (*sc).saber[rSaberNum as usize].blade[rBladeNum as usize].radius * 0.4f32;

            VectorSet(
                &mut saberTrMins,
                -saberBoxSize,
                -saberBoxSize,
                -saberBoxSize,
            );
            VectorSet(&mut saberTrMaxs, saberBoxSize, saberBoxSize, saberBoxSize);
        }

        while saberTraceDone == 0 {
            if doInterpolate != 0 && ctx.world.cvars.d_saberSPStyleDamage.integer == 0 {
                //This didn't quite work out like I hoped. But it's better than nothing. Sort of.
                let mut oldSaberStart: vec3_t = [0.0; 3];
                let mut oldSaberEnd: vec3_t = [0.0; 3];
                let mut saberDif: vec3_t = [0.0; 3];
                let mut oldSaberDif: vec3_t = [0.0; 3];
                let mut traceTests: c_int = 0;
                let mut trDif: f32 = 8.0;

                if (ctx.world.level.time
                    - (*sc).saber[rSaberNum as usize].blade[rBladeNum as usize]
                        .trail
                        .lastTime)
                    > 100
                {
                    //no valid last pos, use current
                    oldSaberStart = saberStart;
                    oldSaberEnd = *saberEnd;
                } else {
                    //trace from last pos
                    oldSaberStart = (*sc).saber[rSaberNum as usize].blade[rBladeNum as usize]
                        .trail
                        .base;
                    oldSaberEnd = (*sc).saber[rSaberNum as usize].blade[rBladeNum as usize]
                        .trail
                        .tip;
                }

                _VectorSubtract(saberStart, *saberEnd, &mut saberDif);
                _VectorSubtract(oldSaberStart, oldSaberEnd, &mut oldSaberDif);

                VectorNormalize(&mut saberDif);
                VectorNormalize(&mut oldSaberDif);

                saberEnd[0] = saberStart[0] - (saberDif[0] * trDif);
                saberEnd[1] = saberStart[1] - (saberDif[1] * trDif);
                saberEnd[2] = saberStart[2] - (saberDif[2] * trDif);

                oldSaberEnd[0] = oldSaberStart[0] - (oldSaberDif[0] * trDif);
                oldSaberEnd[1] = oldSaberStart[1] - (oldSaberDif[1] * trDif);
                oldSaberEnd[2] = oldSaberStart[2] - (oldSaberDif[2] * trDif);

                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        saberEnd as *const vec3_t,
                        &saberTrMins as *const vec3_t,
                        &saberTrMaxs as *const vec3_t,
                        &saberStart as *const vec3_t,
                        (*self_).s.number,
                        trMask,
                    ),
                );

                g6t(
                    ctx,
                    "i1",
                    &*saberEnd,
                    &saberStart,
                    &saberTrMins,
                    &saberTrMaxs,
                    tr.fraction,
                    tr.entityNum as i32,
                ); // TEMP G6
                lastValidStart = *saberEnd;
                lastValidEnd = saberStart;
                if (tr.entityNum as c_int) < MAX_CLIENTS as c_int {
                    G_G2TraceCollide(
                        ctx,
                        &mut tr,
                        lastValidStart,
                        lastValidEnd,
                        saberTrMins,
                        saberTrMaxs,
                    );
                } else if (tr.entityNum as c_int) < ENTITYNUM_WORLD {
                    let trHit = &mut ctx.world.g_entities[tr.entityNum as usize] as *mut gentity_t;

                    if (*trHit).inuse != 0 && !(*trHit).ghoul2.is_null() {
                        //hit a non-client entity with a g2 instance
                        G_G2TraceCollide(
                            ctx,
                            &mut tr,
                            lastValidStart,
                            lastValidEnd,
                            saberTrMins,
                            saberTrMaxs,
                        );
                    }
                }

                trDif += 1.0;

                while tr.fraction == 1.0
                    && traceTests < 4
                    && (tr.entityNum as c_int) >= ENTITYNUM_NONE
                {
                    if (ctx.world.level.time
                        - (*sc).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .trail
                            .lastTime)
                        > 100
                    {
                        //no valid last pos, use current
                        oldSaberStart = saberStart;
                        oldSaberEnd = *saberEnd;
                    } else {
                        //trace from last pos
                        oldSaberStart = (*sc).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .trail
                            .base;
                        oldSaberEnd = (*sc).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .trail
                            .tip;
                    }

                    _VectorSubtract(saberStart, *saberEnd, &mut saberDif);
                    _VectorSubtract(oldSaberStart, oldSaberEnd, &mut oldSaberDif);

                    VectorNormalize(&mut saberDif);
                    VectorNormalize(&mut oldSaberDif);

                    saberEnd[0] = saberStart[0] - (saberDif[0] * trDif);
                    saberEnd[1] = saberStart[1] - (saberDif[1] * trDif);
                    saberEnd[2] = saberStart[2] - (saberDif[2] * trDif);

                    oldSaberEnd[0] = oldSaberStart[0] - (oldSaberDif[0] * trDif);
                    oldSaberEnd[1] = oldSaberStart[1] - (oldSaberDif[1] * trDif);
                    oldSaberEnd[2] = oldSaberStart[2] - (oldSaberDif[2] * trDif);

                    trap::Trace(
                        ctx.engine,
                        GTraceArgs::new(
                            &mut tr as *mut trace_t,
                            saberEnd as *const vec3_t,
                            &saberTrMins as *const vec3_t,
                            &saberTrMaxs as *const vec3_t,
                            &saberStart as *const vec3_t,
                            (*self_).s.number,
                            trMask,
                        ),
                    );

                    g6t(
                        ctx,
                        "i2",
                        &*saberEnd,
                        &saberStart,
                        &saberTrMins,
                        &saberTrMaxs,
                        tr.fraction,
                        tr.entityNum as i32,
                    ); // TEMP G6
                    lastValidStart = *saberEnd;
                    lastValidEnd = saberStart;
                    if (tr.entityNum as c_int) < MAX_CLIENTS as c_int {
                        G_G2TraceCollide(
                            ctx,
                            &mut tr,
                            lastValidStart,
                            lastValidEnd,
                            saberTrMins,
                            saberTrMaxs,
                        );
                    } else if (tr.entityNum as c_int) < ENTITYNUM_WORLD {
                        let trHit =
                            &mut ctx.world.g_entities[tr.entityNum as usize] as *mut gentity_t;

                        if (*trHit).inuse != 0 && !(*trHit).ghoul2.is_null() {
                            //hit a non-client entity with a g2 instance
                            G_G2TraceCollide(
                                ctx,
                                &mut tr,
                                lastValidStart,
                                lastValidEnd,
                                saberTrMins,
                                saberTrMaxs,
                            );
                        }
                    }

                    traceTests += 1;
                    trDif += 8.0;
                }
            } else {
                let mut saberEndExtrapolated: vec3_t = [0.0; 3];
                if extrapolate != 0 {
                    //extrapolate 16
                    let mut diff: vec3_t = [0.0; 3];
                    _VectorSubtract(*saberEnd, saberStart, &mut diff);
                    VectorNormalize(&mut diff);
                    _VectorMA(
                        saberStart,
                        SABER_EXTRAPOLATE_DIST,
                        diff,
                        &mut saberEndExtrapolated,
                    );
                } else {
                    saberEndExtrapolated = *saberEnd;
                }
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &saberStart as *const vec3_t,
                        &saberTrMins as *const vec3_t,
                        &saberTrMaxs as *const vec3_t,
                        &saberEndExtrapolated as *const vec3_t,
                        (*self_).s.number,
                        trMask,
                    ),
                );

                g6t(
                    ctx,
                    "ex",
                    &saberStart,
                    &saberEndExtrapolated,
                    &saberTrMins,
                    &saberTrMaxs,
                    tr.fraction,
                    tr.entityNum as i32,
                ); // TEMP G6
                lastValidStart = saberStart;
                lastValidEnd = saberEndExtrapolated;
                if (tr.entityNum as c_int) < MAX_CLIENTS as c_int {
                    G_G2TraceCollide(
                        ctx,
                        &mut tr,
                        lastValidStart,
                        lastValidEnd,
                        saberTrMins,
                        saberTrMaxs,
                    );
                } else if (tr.entityNum as c_int) < ENTITYNUM_WORLD {
                    let trHit = &mut ctx.world.g_entities[tr.entityNum as usize] as *mut gentity_t;

                    if (*trHit).inuse != 0 && !(*trHit).ghoul2.is_null() {
                        //hit a non-client entity with a g2 instance
                        G_G2TraceCollide(
                            ctx,
                            &mut tr,
                            lastValidStart,
                            lastValidEnd,
                            saberTrMins,
                            saberTrMaxs,
                        );
                    }
                }
            }

            saberTraceDone = qtrue;
        }

        if (*sc).ps.saberAttackWound < ctx.world.level.time
            && (SaberAttacking(&*(self_))
                || BG_SuperBreakWinAnim((*sc).ps.torsoAnim) != 0
                || (ctx.world.cvars.d_saberSPStyleDamage.integer != 0
                    && (*sc).ps.saberInFlight != 0
                    && rSaberNum == 0)
                || (WP_SaberBladeDoTransitionDamage(
                    &mut (*sc).saber[rSaberNum as usize] as *mut saberInfo_t,
                    rBladeNum,
                ) != 0
                    && BG_SaberInTransitionAny((*sc).ps.saberMove) != 0)
                || ((*sc).ps.m_iVehicleNum != 0 && (*sc).ps.saberMove > LS_READY))
        {
            //this animation is that of the last attack movement, and so it should do full damage
            let saberInSpecial: qboolean = BG_SaberInSpecial((*sc).ps.saberMove);
            let inBackAttack = G_SaberInBackAttack((*sc).ps.saberMove);

            if ctx.world.cvars.d_saberSPStyleDamage.integer != 0 {
                let mut fDmg: f32 = 0.0f32;
                if (*sc).ps.saberInFlight != 0 {
                    let saberEnt = &mut ctx.world.g_entities[(*sc).ps.saberEntityNum as usize]
                        as *mut gentity_t;
                    if saberEnt.is_null() || (*saberEnt).s.saberInFlight == 0 {
                        //does less damage on the way back
                        fDmg = 1.0f32;
                        attackStr = FORCE_LEVEL_0;
                    } else {
                        fDmg = 2.5f32 * (*sc).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] as f32;
                        attackStr = FORCE_LEVEL_1;
                    }
                } else {
                    attackStr =
                        G_PowerLevelForSaberAnim(ctx, ctx.entity_id_of(self_), rSaberNum, qfalse);
                    if ctx.world.cvars.g_saberRealisticCombat.integer != 0 {
                        match attackStr {
                            x if x == FORCE_LEVEL_2 => {
                                fDmg = 5.0f32;
                            }
                            x if x == FORCE_LEVEL_1 || x == FORCE_LEVEL_0 => {
                                fDmg = 2.5f32;
                            }
                            _ => {
                                // default and FORCE_LEVEL_3
                                fDmg = 10.0f32;
                            }
                        }
                    } else {
                        if (*sc).ps.torsoAnim == BOTH_SPINATTACK6 as c_int
                            || (*sc).ps.torsoAnim == BOTH_SPINATTACK7 as c_int
                        {
                            //too easy to do, lower damage
                            fDmg = 2.5f32;
                        } else {
                            fDmg = 2.5f32 * attackStr as f32;
                        }
                    }
                }
                if ctx.world.cvars.g_saberRealisticCombat.integer > 1 {
                    //always do damage, and lots of it
                    if ctx.world.cvars.g_saberRealisticCombat.integer > 2 {
                        //always do damage, and lots of it
                        fDmg = 25.0f32;
                    } else if fDmg > 0.1f32 {
                        //only do super damage if we would have done damage according to normal rules
                        fDmg = 25.0f32;
                    }
                }
                if ctx.world.cvars.g_gametype.integer != GT_DUEL
                    && ctx.world.cvars.g_gametype.integer != GT_POWERDUEL
                    && ctx.world.cvars.g_gametype.integer != GT_SIEGE
                {
                    //in faster-paced games, sabers do more damage
                    fDmg *= 2.0f32;
                }
                if fDmg != 0.0 {
                    //the longer the trace, the more damage it does
                    let traceLength: f32 = Distance(*saberEnd, saberStart);
                    if tr.fraction >= 1.0f32 {
                        //allsolid?
                        dmg = (fDmg * traceLength * 0.1f32 * 0.33f32).ceil() as c_int;
                    } else {
                        //fractional hit, the sooner you hit in the trace, the more damage you did
                        dmg = (fDmg * traceLength * (1.0f32 - tr.fraction) * 0.1f32 * 0.33f32)
                            .ceil() as c_int;
                    }
                    if ctx.world.cvars.g_saberDebugBox.integer == 3
                        || ctx.world.cvars.g_saberDebugBox.integer == 4
                    {
                        G_TestLine(ctx, saberStart, *saberEnd, 0x0000ff, 50);
                    }
                }
                if (*sc).ps.torsoAnim == BOTH_A1_SPECIAL as c_int
                    || (*sc).ps.torsoAnim == BOTH_A2_SPECIAL as c_int
                    || (*sc).ps.torsoAnim == BOTH_A3_SPECIAL as c_int
                {
                    //parry/block/break-parry bonus for single-style kata moves
                    attackStr += 1;
                }
                if BG_SuperBreakWinAnim((*sc).ps.torsoAnim) != 0 {
                    trMask &= !CONTENTS_LIGHTSABER;
                }
            } else {
                dmg = SABER_HITDAMAGE;

                if (*sc).ps.fd.saberAnimLevel == saber_styles_t::SS_STAFF as c_int
                    || (*sc).ps.fd.saberAnimLevel == saber_styles_t::SS_DUAL as c_int
                {
                    if saberInSpecial != 0 {
                        //it will get auto-ramped based on the point in the attack, later on
                        if (*sc).ps.saberMove == LS_SPINATTACK
                            || (*sc).ps.saberMove == LS_SPINATTACK_DUAL
                        {
                            //these attacks are long and have the potential to hit a lot so they will do less damage.
                            dmg = 10;
                        } else {
                            if BG_KickingAnim((*sc).ps.legsAnim) != 0
                                || BG_KickingAnim((*sc).ps.torsoAnim) != 0
                            {
                                //saber shouldn't do more than min dmg during kicks
                                dmg = 2;
                            } else if BG_SaberInKata((*sc).ps.saberMove) != 0 {
                                //special kata move
                                if (*sc).ps.fd.saberAnimLevel == saber_styles_t::SS_DUAL as c_int {
                                    //this is the nasty saber twirl, do big damage cause it makes you vulnerable
                                    dmg = 90;
                                } else {
                                    //staff kata
                                    dmg = G_GetAttackDamage(
                                        ctx,
                                        ctx.entity_id_of(self_).unwrap(),
                                        60,
                                        70,
                                        0.5f32,
                                    );
                                }
                            } else {
                                //ramp from 2 to 90 by default for other specials
                                dmg = G_GetAttackDamage(
                                    ctx,
                                    ctx.entity_id_of(self_).unwrap(),
                                    2,
                                    90,
                                    0.5f32,
                                );
                            }
                        }
                    } else {
                        //otherwise we'll ramp up to 70 I guess, for both dual and staff
                        dmg =
                            G_GetAttackDamage(ctx, ctx.entity_id_of(self_).unwrap(), 2, 70, 0.5f32);
                    }
                } else if (*sc).ps.fd.saberAnimLevel == 3 {
                    //new damage-ramping system
                    if saberInSpecial == 0 && !inBackAttack {
                        dmg = G_GetAttackDamage(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            2,
                            120,
                            0.5f32,
                        );
                    } else if saberInSpecial != 0 && (*sc).ps.saberMove == LS_A_JUMP_T__B_ {
                        dmg = G_GetAttackDamage(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            2,
                            180,
                            0.65f32,
                        );
                    } else if inBackAttack {
                        dmg =
                            G_GetAttackDamage(ctx, ctx.entity_id_of(self_).unwrap(), 2, 30, 0.5f32);
                    //can hit multiple times (and almost always does), so..
                    } else {
                        dmg = 100;
                    }
                } else if (*sc).ps.fd.saberAnimLevel == 2 {
                    if saberInSpecial != 0
                        && ((*sc).ps.saberMove == LS_A_FLIP_STAB
                            || (*sc).ps.saberMove == LS_A_FLIP_SLASH)
                    {
                        //a well-timed hit with this can do a full 85
                        dmg =
                            G_GetAttackDamage(ctx, ctx.entity_id_of(self_).unwrap(), 2, 80, 0.5f32);
                    } else if inBackAttack {
                        dmg =
                            G_GetAttackDamage(ctx, ctx.entity_id_of(self_).unwrap(), 2, 25, 0.5f32);
                    } else {
                        dmg = 60;
                    }
                } else if (*sc).ps.fd.saberAnimLevel == 1 {
                    if saberInSpecial != 0 && (*sc).ps.saberMove == LS_A_LUNGE {
                        dmg = G_GetAttackDamage(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            2,
                            SABER_HITDAMAGE - 5,
                            0.3f32,
                        );
                    } else if inBackAttack {
                        dmg =
                            G_GetAttackDamage(ctx, ctx.entity_id_of(self_).unwrap(), 2, 30, 0.5f32);
                    } else {
                        dmg = SABER_HITDAMAGE;
                    }
                }

                attackStr = (*sc).ps.fd.saberAnimLevel;
            }
        } else if (*sc).ps.saberAttackWound < ctx.world.level.time
            && (*sc).ps.saberIdleWound < ctx.world.level.time
        {
            //just touching, do minimal damage and only check for it every 200ms
            if (*sc).saber[0].saberFlags2 & SFL2_NO_IDLE_EFFECT != 0 {
                //no idle damage or effects
                return true; //true cause even though we didn't get a hit, we don't want to do those extra traces because the debounce time says not to.
            }
            trMask &= !CONTENTS_LIGHTSABER;
            if ctx.world.cvars.d_saberSPStyleDamage.integer != 0 {
                if BG_SaberInReturn((*sc).ps.saberMove) != 0 {
                    dmg = SABER_NONATTACK_DAMAGE;
                } else {
                    if ctx.world.cvars.d_saberSPStyleDamage.integer == 2 {
                        dmg = SABER_NONATTACK_DAMAGE;
                    } else {
                        dmg = 0;
                    }
                }
            } else {
                dmg = SABER_NONATTACK_DAMAGE;
            }
            idleDamage = qtrue;
        } else {
            return true; //true cause even though we didn't get a hit, we don't want to do those extra traces because the debounce time says not to.
        }

        if BG_SaberInSpecial((*sc).ps.saberMove) != 0 {
            let inBackAttack = G_SaberInBackAttack((*sc).ps.saberMove);

            unblockable = qtrue;
            (*sc).ps.saberBlocked = 0;

            if ctx.world.cvars.d_saberSPStyleDamage.integer != 0 {
            } else if !inBackAttack {
                if (*sc).ps.saberMove == LS_A_JUMP_T__B_ {
                    //do extra damage for special unblockables
                    dmg += 5; //This is very tiny, because this move has a huge damage ramp
                } else if (*sc).ps.saberMove == LS_A_FLIP_STAB
                    || (*sc).ps.saberMove == LS_A_FLIP_SLASH
                {
                    dmg += 5; //ditto
                    if dmg <= 40 || G_GetAnimPoint(ctx, ctx.entity_id_of(self_).unwrap()) <= 0.4f32
                    {
                        //sort of a hack, don't want it doing big damage in the off points of the anim
                        dmg = 2;
                    }
                } else if (*sc).ps.saberMove == LS_A_LUNGE {
                    dmg += 2; //and ditto again
                    if G_GetAnimPoint(ctx, ctx.entity_id_of(self_).unwrap()) <= 0.4f32 {
                        //same as above
                        dmg = 2;
                    }
                } else if (*sc).ps.saberMove == LS_SPINATTACK
                    || (*sc).ps.saberMove == LS_SPINATTACK_DUAL
                {
                    //do a constant significant amount of damage but ramp up a little to the mid-point
                    dmg = G_GetAttackDamage(
                        ctx,
                        ctx.entity_id_of(self_).unwrap(),
                        2,
                        dmg + 3,
                        0.5f32,
                    );
                    dmg += 10;
                } else {
                    if BG_KickingAnim((*sc).ps.legsAnim) != 0
                        || BG_KickingAnim((*sc).ps.torsoAnim) != 0
                    {
                        //saber shouldn't do more than min dmg during kicks
                        dmg = 2;
                    } else {
                        //auto-ramp it I guess since it's a special we don't have a special case for.
                        dmg = G_GetAttackDamage(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            5,
                            dmg + 5,
                            0.5f32,
                        );
                    }
                }
            }
        }

        if dmg == 0 {
            if (tr.entityNum as c_int) < MAX_CLIENTS as c_int
                || (ctx.world.g_entities[tr.entityNum as usize].inuse != 0
                    && (ctx.world.g_entities[tr.entityNum as usize].r.contents
                        & CONTENTS_LIGHTSABER)
                        != 0)
            {
                return true;
            }
            return false;
        }

        if dmg > SABER_NONATTACK_DAMAGE {
            dmg = (dmg as f32 * ctx.world.cvars.g_saberDamageScale.value) as c_int;

            //see if this specific saber has a damagescale
            if WP_SaberBladeUseSecondBladeStyle(
                &mut (*sc).saber[rSaberNum as usize] as *mut saberInfo_t,
                rBladeNum,
            ) == 0
                && (*sc).saber[rSaberNum as usize].damageScale != 1.0f32
            {
                dmg = ((dmg as f32) * (*sc).saber[rSaberNum as usize].damageScale).ceil() as c_int;
            } else if WP_SaberBladeUseSecondBladeStyle(
                &mut (*sc).saber[rSaberNum as usize] as *mut saberInfo_t,
                rBladeNum,
            ) != 0
                && (*sc).saber[rSaberNum as usize].damageScale2 != 1.0f32
            {
                dmg = ((dmg as f32) * (*sc).saber[rSaberNum as usize].damageScale2).ceil() as c_int;
            }

            if ((*sc).ps.brokenLimbs & (1 << BROKENLIMB_RARM as c_int)) != 0
                || ((*sc).ps.brokenLimbs & (1 << BROKENLIMB_LARM as c_int)) != 0
            {
                //weaken it if an arm is broken
                dmg = (dmg as f64 * 0.3) as c_int;
                if dmg <= SABER_NONATTACK_DAMAGE {
                    dmg = SABER_NONATTACK_DAMAGE + 1;
                }
            }
        }

        if dmg > SABER_NONATTACK_DAMAGE && (*sc).ps.isJediMaster != 0 {
            //give the Jedi Master more saber attack power
            dmg *= 2;
        }

        if dmg > SABER_NONATTACK_DAMAGE
            && ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && (*sc).siegeClass != -1
            && ((&ctx.world.bg_state.bgSiegeClasses)[(*sc).siegeClass as usize].classflags
                & (1 << CFL_MORESABERDMG as c_int))
                != 0
        {
            //this class is flagged to do extra saber damage. I guess 2x will do for now.
            dmg *= 2;
        }

        if ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
            && (*sc).sess.duelTeam == DUELTEAM_LONE as c_int
        {
            //always x2 when we're powerdueling alone... er, so, we apparently no longer want this?  So they say.
            if ctx.world.cvars.g_duel_fraglimit.integer != 0 {
                //dmg *= 1.5 - ... (disabled in Raven)
            }
            //dmg *= 2;
        }

        if ctx.world.cvars.g_saberDebugPrint.integer > 2 && dmg > 1 {
            let s = format!("CL {} SABER DMG: {}\n", (*self_).s.number, dmg);
            Com_Printf(&s);
        }

        _VectorSubtract(*saberEnd, saberStart, &mut dir);
        VectorNormalize(&mut dir);

        if tr.entityNum as c_int == ENTITYNUM_WORLD
            || ctx.world.g_entities[tr.entityNum as usize].s.eType == ET_TERRAIN as c_int
        {
            //register this as a wall hit for jedi AI
            (*sc).ps.saberEventFlags |= SEF_HITWALL;
            ctx.world.globals.saberHitWall = qtrue;
        }

        if ctx.world.globals.saberHitWall != 0
            && ((*sc).saber[rSaberNum as usize].saberFlags & SFL_BOUNCE_ON_WALLS) != 0
            && (BG_SaberInAttackPure((*sc).ps.saberMove) != 0 //only in a normal attack anim
                || (*sc).ps.saberMove == LS_A_JUMP_T__B_)
        {
            //then bounce off
            {
                (*sc).ps.saberMove = mp_bg::bg_panimate::BG_BrokenParryForAttack(
                    &ctx.world.bg_state,
                    (*sc).ps.saberMove,
                );
                (*sc).ps.saberBlocked = BLOCKED_PARRY_BROKEN;
                if (*sc).ps.torsoAnim == (*sc).ps.legsAnim {
                    //set anim now on both parts
                    let anim =
                        ctx.world.bg_state.saberMoveData[(*sc).ps.saberMove as usize].animToUse;
                    G_SetAnim(
                        ctx,
                        ctx.entity_id_of(self_).unwrap(),
                        &mut (*sc).pers.cmd as *mut usercmd_t,
                        SETANIM_BOTH,
                        anim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        0,
                    );
                }

                //do bounce sound & force feedback
                WP_SaberBounceSound(ctx, ctx.entity_id_of(self_), rSaberNum, rBladeNum);
                //do hit effect
                let te_id = G_TempEntity(ctx, tr.endpos, EV_SABER_HIT as c_int);
                let te = ctx.entity_mut(te_id);
                te.s.otherEntityNum = ENTITYNUM_NONE; //we didn't hit anyone in particular
                te.s.otherEntityNum2 = (*self_).s.number; //send this so it knows who we are
                te.s.weapon = rSaberNum;
                te.s.legsAnim = rBladeNum;
                te.s.origin = tr.endpos;
                te.s.angles = tr.plane.normal;
                if te.s.angles[0] == 0.0 && te.s.angles[1] == 0.0 && te.s.angles[2] == 0.0 {
                    //don't let it play with no direction
                    te.s.angles[1] = 1.0;
                }
                //do radius damage/knockback, if any
                if WP_SaberBladeUseSecondBladeStyle(
                    &mut (*sc).saber[rSaberNum as usize] as *mut saberInfo_t,
                    rBladeNum,
                ) == 0
                {
                    WP_SaberRadiusDamage(
                        ctx,
                        ctx.entity_id_of(self_),
                        tr.endpos,
                        (*sc).saber[rSaberNum as usize].splashRadius,
                        (*sc).saber[rSaberNum as usize].splashDamage,
                        (*sc).saber[rSaberNum as usize].splashKnockback,
                    );
                } else {
                    WP_SaberRadiusDamage(
                        ctx,
                        ctx.entity_id_of(self_),
                        tr.endpos,
                        (*sc).saber[rSaberNum as usize].splashRadius2,
                        (*sc).saber[rSaberNum as usize].splashDamage2,
                        (*sc).saber[rSaberNum as usize].splashKnockback2,
                    );
                }

                return true;
            }
        }

        //rww - I'm saying || tr.startsolid here...
        let mut do_block_stuff = false;

        if (tr.fraction != 1.0 || tr.startsolid != 0)
            && ctx.world.g_entities[tr.entityNum as usize].takedamage != 0
            && (ctx.world.g_entities[tr.entityNum as usize].health > 0
                || (ctx.world.g_entities[tr.entityNum as usize].s.eFlags & EF_DISINTEGRATION) == 0)
            && tr.entityNum as c_int != (*self_).s.number
            && ctx.world.g_entities[tr.entityNum as usize].inuse != 0
        {
            //hit something that had health and takes damage
            let trEnt = &mut ctx.world.g_entities[tr.entityNum as usize] as *mut gentity_t;
            let trc = (*trEnt).client;

            if idleDamage != 0
                && !(*trEnt).client.is_null()
                && OnSameTeam(ctx, ctx.entity_id_of(self_), ctx.entity_id_of(trEnt)) != 0
                && ctx.world.cvars.g_friendlySaber.integer == 0
            {
                return false;
            }

            if !(*trEnt).client.is_null()
                && (*trc).ps.duelInProgress != 0
                && (*trc).ps.duelIndex != (*self_).s.number
            {
                return false;
            }

            if !(*trEnt).client.is_null()
                && (*sc).ps.duelInProgress != 0
                && (*sc).ps.duelIndex != (*trEnt).s.number
            {
                return false;
            }

            if BG_StabDownAnim((*sc).ps.torsoAnim) != 0
                && !(*trEnt).client.is_null()
                && mp_bg::bg_panimate::BG_InKnockDownOnGround(
                    &ctx.world.bg_state,
                    &mut (*trc).ps as *mut playerState_t,
                ) == 0
            {
                //stabdowns only damage people who are actually on the ground...
                return false;
            }
            (*sc).ps.saberIdleWound =
                ctx.world.level.time + ctx.world.cvars.g_saberDmgDelay_Idle.integer;

            didHit = true;

            if ctx.world.cvars.d_saberSPStyleDamage.integer == 0
                && !(*trEnt).client.is_null()
                && unblockable == 0
                && WP_SaberCanBlock(
                    ctx,
                    ctx.entity_id_of(trEnt),
                    tr.endpos,
                    0,
                    MOD_SABER as c_int,
                    false,
                    attackStr,
                ) != 0
            {
                //hit a client who blocked the attack (fake: didn't actually hit their saber)
                if dmg <= SABER_NONATTACK_DAMAGE {
                    (*sc).ps.saberIdleWound =
                        ctx.world.level.time + ctx.world.cvars.g_saberDmgDelay_Idle.integer;
                }
                ctx.world.globals.saberDoClashEffect = qtrue;
                ctx.world.globals.saberClashPos = tr.endpos;
                ctx.world.globals.saberClashNorm = tr.plane.normal;
                ctx.world.globals.saberClashEventParm = 1;

                if dmg > SABER_NONATTACK_DAMAGE {
                    let lockFactor = ctx.world.cvars.g_saberLockFactor.integer;

                    if ((*trc).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize]
                        - (*sc).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize])
                        > 1
                        && ctx.world.bg_state.rng.Q_irand(1, 10) < lockFactor * 2
                    {
                        //Just got blocked by someone with a decently higher attack level, so enter into a lock
                        if !G_ClientIdleInWorld(&*(trEnt)) {
                            if (trMask & CONTENTS_LIGHTSABER) != 0
                                && WP_SabersCheckLock(
                                    ctx,
                                    ctx.entity_id_of(self_).unwrap(),
                                    ctx.entity_id_of(trEnt).unwrap(),
                                )
                            {
                                (*sc).ps.saberBlocked = BLOCKED_NONE;
                                (*trc).ps.saberBlocked = BLOCKED_NONE;
                                return didHit;
                            }
                        }
                    } else if ctx.world.bg_state.rng.Q_irand(1, 20) < lockFactor {
                        if !G_ClientIdleInWorld(&*(trEnt)) {
                            if (trMask & CONTENTS_LIGHTSABER) != 0
                                && WP_SabersCheckLock(
                                    ctx,
                                    ctx.entity_id_of(self_).unwrap(),
                                    ctx.entity_id_of(trEnt).unwrap(),
                                )
                            {
                                (*sc).ps.saberBlocked = BLOCKED_NONE;
                                (*trc).ps.saberBlocked = BLOCKED_NONE;
                                return didHit;
                            }
                        }
                    }
                }
                otherOwner = trEnt;
                do_block_stuff = true;
            } else {
                //damage the thing we hit
                let mut doDismemberment: qboolean = qfalse;
                let mut knockbackFlags: c_int = 0;

                if !(*trEnt).client.is_null() {
                    //not a "jedi", so make them suffer more
                    if dmg > SABER_NONATTACK_DAMAGE {
                        //don't bother increasing just for idle touch damage
                        dmg = (dmg as f64 * 1.5) as c_int;
                    }
                }

                if ctx.world.cvars.d_saberSPStyleDamage.integer == 0 {
                    if !(*trEnt).client.is_null() && (*trc).ps.weapon == WP_SABER as c_int {
                        //for jedi using the saber, half the damage
                        if ctx.world.cvars.g_gametype.integer != GT_SIEGE {
                            //unless siege..
                            if dmg > SABER_NONATTACK_DAMAGE && unblockable == 0 {
                                //don't reduce damage if it's only 1, or if this is an unblockable attack
                                if dmg == SABER_HITDAMAGE {
                                    //level 1 attack
                                    dmg = (dmg as f64 * 0.7) as c_int;
                                } else {
                                    dmg = (dmg as f64 * 0.5) as c_int;
                                }
                            }
                        }
                    }
                }

                if (*self_).s.eType == ET_NPC as c_int
                    && !(*trEnt).client.is_null()
                    && (*sc).playerTeam == (*trc).playerTeam
                {
                    //Oops. Since he's an NPC, we'll be forgiving and cut the damage down.
                    dmg = (dmg as f32 * 0.2f32) as c_int;
                }

                //store the damage, we'll apply it later
                if WP_SaberBladeUseSecondBladeStyle(
                    &mut (*sc).saber[rSaberNum as usize] as *mut saberInfo_t,
                    rBladeNum,
                ) == 0
                    && ((*sc).saber[rSaberNum as usize].saberFlags2 & SFL2_NO_DISMEMBERMENT) == 0
                {
                    doDismemberment = qtrue;
                }
                if WP_SaberBladeUseSecondBladeStyle(
                    &mut (*sc).saber[rSaberNum as usize] as *mut saberInfo_t,
                    rBladeNum,
                ) != 0
                    && ((*sc).saber[rSaberNum as usize].saberFlags2 & SFL2_NO_DISMEMBERMENT) == 0
                {
                    doDismemberment = qtrue;
                }

                if WP_SaberBladeUseSecondBladeStyle(
                    &mut (*sc).saber[rSaberNum as usize] as *mut saberInfo_t,
                    rBladeNum,
                ) == 0
                    && (*sc).saber[rSaberNum as usize].knockbackScale > 0.0f32
                {
                    if rSaberNum < 1 {
                        knockbackFlags = DAMAGE_SABER_KNOCKBACK1;
                    } else {
                        knockbackFlags = DAMAGE_SABER_KNOCKBACK2;
                    }
                }

                if WP_SaberBladeUseSecondBladeStyle(
                    &mut (*sc).saber[rSaberNum as usize] as *mut saberInfo_t,
                    rBladeNum,
                ) != 0
                    && (*sc).saber[rSaberNum as usize].knockbackScale > 0.0f32
                {
                    if rSaberNum < 1 {
                        knockbackFlags = DAMAGE_SABER_KNOCKBACK1_B2;
                    } else {
                        knockbackFlags = DAMAGE_SABER_KNOCKBACK2_B2;
                    }
                }

                WP_SaberDamageAdd(
                    ctx,
                    tr.entityNum as c_int,
                    dir,
                    tr.endpos,
                    dmg,
                    doDismemberment,
                    knockbackFlags,
                );

                if !(*trEnt).client.is_null() {
                    //Let jedi AI know if it hit an enemy
                    if (*self_).enemy.is_some() && (*self_).enemy == Some(ent_id(base, trEnt)) {
                        (*sc).ps.saberEventFlags |= SEF_HITENEMY;
                    } else {
                        (*sc).ps.saberEventFlags |= SEF_HITOBJECT;
                    }
                }

                if ctx.world.cvars.d_saberSPStyleDamage.integer != 0 {
                } else {
                    (*sc).ps.saberAttackWound = ctx.world.level.time + 100;
                }
            }
        } else if (tr.fraction != 1.0 || tr.startsolid != 0)
            && (ctx.world.g_entities[tr.entityNum as usize].r.contents & CONTENTS_LIGHTSABER) != 0
            && ctx.world.g_entities[tr.entityNum as usize].r.contents != -1
            && ctx.world.g_entities[tr.entityNum as usize].inuse != 0
        {
            //saber clash
            let oo_num = ctx.world.g_entities[tr.entityNum as usize].r.ownerNum;
            otherOwner = &mut ctx.world.g_entities[oo_num as usize] as *mut gentity_t;

            if (*otherOwner).inuse == 0 || (*otherOwner).client.is_null() {
                return false;
            }

            let ooc = (*otherOwner).client;

            if !(*otherOwner).client.is_null() && (*ooc).ps.saberInFlight != 0 {
                //don't do extra collision checking vs sabers in air
            } else {
                //hit an in-hand saber, do extra collision check against it
                if ctx.world.cvars.d_saberSPStyleDamage.integer != 0 {
                    //use SP-style blade-collision test
                    if !WP_SabersIntersect(
                        ctx,
                        ctx.entity_id_of(self_),
                        rSaberNum,
                        rBladeNum,
                        ctx.entity_id_of(otherOwner),
                        qfalse,
                    ) {
                        //sabers did not actually intersect
                        return false;
                    }
                } else {
                    //MP-style
                    if !G_SaberCollide(
                        ctx,
                        ctx.entity_id_of(self_).unwrap(),
                        ctx.entity_id_of(otherOwner).unwrap(),
                        lastValidStart,
                        lastValidEnd,
                        saberTrMins,
                        saberTrMaxs,
                        tr.endpos,
                    ) {
                        //detailed collision did not produce results...
                        return false;
                    }
                }
            }

            if OnSameTeam(ctx, ctx.entity_id_of(self_), ctx.entity_id_of(otherOwner)) != 0
                && ctx.world.cvars.g_friendlySaber.integer == 0
            {
                return false;
            }

            if ((*self_).s.eType == ET_NPC as c_int || (*otherOwner).s.eType == ET_NPC as c_int)
                && (*sc).playerTeam == (*ooc).playerTeam
                && ctx.world.cvars.g_gametype.integer != GT_SIEGE
            {
                //don't hit your teammate's sabers if you are an NPC.
                return false;
            }

            if (*ooc).ps.duelInProgress != 0 && (*ooc).ps.duelIndex != (*self_).s.number {
                return false;
            }

            if (*sc).ps.duelInProgress != 0 && (*sc).ps.duelIndex != (*otherOwner).s.number {
                return false;
            }

            if ctx.world.cvars.g_debugSaberLocks.integer != 0 {
                WP_SabersCheckLock2(
                    ctx,
                    ctx.entity_id_of(self_).unwrap(),
                    ctx.entity_id_of(otherOwner).unwrap(),
                    LOCK_RANDOM,
                );
                return true;
            }
            didHit = true;
            (*sc).ps.saberIdleWound =
                ctx.world.level.time + ctx.world.cvars.g_saberDmgDelay_Idle.integer;

            if dmg <= SABER_NONATTACK_DAMAGE {
                (*sc).ps.saberIdleWound =
                    ctx.world.level.time + ctx.world.cvars.g_saberDmgDelay_Idle.integer;
            }

            ctx.world.globals.saberDoClashEffect = qtrue;
            ctx.world.globals.saberClashPos = tr.endpos;
            ctx.world.globals.saberClashNorm = tr.plane.normal;
            ctx.world.globals.saberClashEventParm = 1;

            sabersClashed = qtrue;
            ctx.world.globals.saberHitSaber = qtrue;
            ctx.world.globals.saberHitFraction = tr.fraction;

            if saberCheckKnockdown_Smashed(
                ctx,
                Some(EntityId((tr.entityNum) as u32)),
                ctx.entity_id_of(otherOwner),
                ctx.entity_id_of(self_),
                dmg,
            ) {
                //smashed it out of the air
                return false;
            }

            //is this my thrown saber?
            if (*sc).ps.saberEntityNum != 0
                && (*sc).ps.saberInFlight != 0
                && rSaberNum == 0
                && saberCheckKnockdown_Smashed(
                    ctx,
                    Some(EntityId(((*sc).ps.saberEntityNum) as u32)),
                    ctx.entity_id_of(self_),
                    ctx.entity_id_of(otherOwner),
                    dmg,
                )
            {
                //they smashed it out of the air
                return false;
            }

            do_block_stuff = true;
        }

        if do_block_stuff {
            //blockStuff:
            let ooc = (*otherOwner).client;
            otherUnblockable = qfalse;

            if !otherOwner.is_null()
                && !(*otherOwner).client.is_null()
                && (*ooc).ps.saberInFlight != 0
            {
                return false;
            }

            //this is a thrown saber, don't do any fancy saber-saber collision stuff
            if (*sc).ps.saberEntityNum != 0 && (*sc).ps.saberInFlight != 0 && rSaberNum == 0 {
                return false;
            }

            otherSaberLevel = G_SaberAttackPower(
                ctx,
                ctx.entity_id_of(otherOwner),
                SaberAttacking(&*(otherOwner)),
            );

            if dmg > SABER_NONATTACK_DAMAGE && unblockable == 0 && otherUnblockable == 0 {
                let lockFactor = ctx.world.cvars.g_saberLockFactor.integer;

                if sabersClashed != 0 && ctx.world.bg_state.rng.Q_irand(1, 20) <= lockFactor {
                    if !G_ClientIdleInWorld(&*(otherOwner)) {
                        if WP_SabersCheckLock(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            ctx.entity_id_of(otherOwner).unwrap(),
                        ) {
                            (*sc).ps.saberBlocked = BLOCKED_NONE;
                            (*ooc).ps.saberBlocked = BLOCKED_NONE;
                            return didHit;
                        }
                    }
                }
            }

            if otherOwner.is_null() || (*otherOwner).client.is_null() {
                return didHit;
            }

            if BG_SaberInSpecial((*ooc).ps.saberMove) != 0 {
                otherUnblockable = qtrue;
                (*ooc).ps.saberBlocked = 0;
            }

            if sabersClashed != 0
                && dmg > SABER_NONATTACK_DAMAGE
                && selfSaberLevel < FORCE_LEVEL_3
                && PM_SaberInBounce((*ooc).ps.saberMove) == 0
                && PM_SaberInParry((*sc).ps.saberMove) == 0
                && PM_SaberInBrokenParry((*sc).ps.saberMove) == 0
                && BG_SaberInSpecial((*sc).ps.saberMove) == 0
                && PM_SaberInBounce((*sc).ps.saberMove) == 0
                && PM_SaberInDeflect((*sc).ps.saberMove) == 0
                && PM_SaberInReflect((*sc).ps.saberMove) == 0
                && unblockable == 0
            {
                //for now, just always try a deflect. (deflect func can cause bounces too)
                if true {
                    if !WP_GetSaberDeflectionAngle(
                        ctx,
                        ctx.entity_id_of(self_),
                        ctx.entity_id_of(otherOwner),
                        tr.fraction,
                    ) {
                        tryDeflectAgain = qtrue; //Failed the deflect, try it again if we can
                    } else {
                        (*sc).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
                        didOffense = qtrue;
                    }
                } else {
                    (*sc).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                    didOffense = qtrue;

                    if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                        let s = format!(
                            "Client {} clashed into client {}'s saber, did BLOCKED_ATK_BOUNCE\n",
                            (*self_).s.number,
                            (*otherOwner).s.number
                        );
                        Com_Printf(&s);
                    }
                }
            }

            if ((selfSaberLevel < FORCE_LEVEL_3
                && ((tryDeflectAgain != 0 && ctx.world.bg_state.rng.Q_irand(1, 10) <= 3)
                    || (tryDeflectAgain == 0 && ctx.world.bg_state.rng.Q_irand(1, 10) <= 7)))
                || (ctx.world.bg_state.rng.Q_irand(1, 10) <= 1 && otherSaberLevel >= FORCE_LEVEL_3))
                && PM_SaberInBounce((*sc).ps.saberMove) == 0
                && PM_SaberInBrokenParry((*ooc).ps.saberMove) == 0
                && BG_SaberInSpecial((*ooc).ps.saberMove) == 0
                && PM_SaberInBounce((*ooc).ps.saberMove) == 0
                && PM_SaberInDeflect((*ooc).ps.saberMove) == 0
                && PM_SaberInReflect((*ooc).ps.saberMove) == 0
                && (otherSaberLevel > FORCE_LEVEL_2
                    || ((*ooc).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] >= 3
                        && ctx.world.bg_state.rng.Q_irand(0, otherSaberLevel) != 0))
                && unblockable == 0
                && otherUnblockable == 0
                && dmg > SABER_NONATTACK_DAMAGE
                && didOffense == 0
            {
                //knockaways can make fast-attacker go into a broken parry anim
                if (*sc).ps.saberEntityNum != 0 {
                    saberCheckKnockdown_BrokenParry(
                        ctx,
                        Some(EntityId(((*sc).ps.saberEntityNum) as u32)),
                        ctx.entity_id_of(self_),
                        ctx.entity_id_of(otherOwner),
                    );
                }

                if PM_SaberInParry((*ooc).ps.saberMove) == 0 {
                    WP_SaberBlockNonRandom(&*(otherOwner), tr.endpos, qfalse);
                    (*ooc).ps.saberMove = BG_KnockawayForParry((*ooc).ps.saberBlocked);
                    (*ooc).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
                } else {
                    (*ooc).ps.saberMove = G_KnockawayForParry((*ooc).ps.saberMove);
                    (*ooc).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
                }

                //make them (me) go into a broken parry
                (*sc).ps.saberMove = mp_bg::bg_panimate::BG_BrokenParryForAttack(
                    &ctx.world.bg_state,
                    (*sc).ps.saberMove,
                );
                (*sc).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;

                if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                    let s = format!(
                        "Client {} sent client {} into a reflected attack with a knockaway\n",
                        (*otherOwner).s.number,
                        (*self_).s.number
                    );
                    Com_Printf(&s);
                }

                didDefense = qtrue;
            } else if (selfSaberLevel > FORCE_LEVEL_2 || unblockable != 0)
                && ((*ooc).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] < selfSaberLevel
                    || ((*ooc).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] == selfSaberLevel
                        && (ctx.world.bg_state.rng.Q_irand(1, 10) as f64
                            >= otherSaberLevel as f64 * 1.5
                            || unblockable != 0)))
                && PM_SaberInParry((*ooc).ps.saberMove) != 0
                && PM_SaberInBrokenParry((*ooc).ps.saberMove) == 0
                && PM_SaberInParry((*sc).ps.saberMove) == 0
                && PM_SaberInBrokenParry((*sc).ps.saberMove) == 0
                && PM_SaberInBounce((*sc).ps.saberMove) == 0
                && dmg > SABER_NONATTACK_DAMAGE
                && didOffense == 0
                && otherUnblockable == 0
            {
                //they are in a parry, and we are slamming down on them
                if (*ooc).ps.saberEntityNum != 0 {
                    saberCheckKnockdown_BrokenParry(
                        ctx,
                        Some(EntityId(((*ooc).ps.saberEntityNum) as u32)),
                        ctx.entity_id_of(otherOwner),
                        ctx.entity_id_of(self_),
                    );
                }

                if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                    let s = format!(
                        "Client {} sent client {} into a broken parry\n",
                        (*self_).s.number,
                        (*otherOwner).s.number
                    );
                    Com_Printf(&s);
                }

                (*ooc).ps.saberMove =
                    BG_BrokenParryForParry(&mut ctx.world.bg_state, (*ooc).ps.saberMove);
                (*ooc).ps.saberBlocked = BLOCKED_PARRY_BROKEN;

                didDefense = qtrue;
            } else if selfSaberLevel > FORCE_LEVEL_2
                && otherSaberLevel >= FORCE_LEVEL_3
                && PM_SaberInParry((*ooc).ps.saberMove) != 0
                && PM_SaberInBrokenParry((*ooc).ps.saberMove) == 0
                && PM_SaberInParry((*sc).ps.saberMove) == 0
                && PM_SaberInBrokenParry((*sc).ps.saberMove) == 0
                && PM_SaberInBounce((*sc).ps.saberMove) == 0
                && PM_SaberInDeflect((*sc).ps.saberMove) == 0
                && PM_SaberInReflect((*sc).ps.saberMove) == 0
                && dmg > SABER_NONATTACK_DAMAGE
                && didOffense == 0
                && unblockable == 0
            {
                //they are in a parry, and we are slamming down on them
                if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                    let s = format!(
                        "Client {} bounced off of client {}'s saber\n",
                        (*self_).s.number,
                        (*otherOwner).s.number
                    );
                    Com_Printf(&s);
                }

                if tryDeflectAgain == 0 {
                    if !WP_GetSaberDeflectionAngle(
                        ctx,
                        ctx.entity_id_of(self_),
                        ctx.entity_id_of(otherOwner),
                        tr.fraction,
                    ) {
                        tryDeflectAgain = qtrue;
                    }
                }

                didOffense = qtrue;
            } else if SaberAttacking(&*(otherOwner))
                && dmg > SABER_NONATTACK_DAMAGE
                && BG_SaberInSpecial((*ooc).ps.saberMove) == 0
                && didOffense == 0
                && otherUnblockable == 0
            {
                //they were attacking and our saber hit their saber, make them bounce.
                if PM_SaberInBounce((*sc).ps.saberMove) == 0
                    && PM_SaberInBounce((*ooc).ps.saberMove) == 0
                    && PM_SaberInDeflect((*sc).ps.saberMove) == 0
                    && PM_SaberInDeflect((*ooc).ps.saberMove) == 0
                    && PM_SaberInReflect((*sc).ps.saberMove) == 0
                    && PM_SaberInReflect((*ooc).ps.saberMove) == 0
                {
                    let mut attackAdv: c_int;
                    let mut defendStr: c_int =
                        G_PowerLevelForSaberAnim(ctx, ctx.entity_id_of(otherOwner), 0, qtrue);
                    let mut attackBonus: c_int = 0;
                    if (*ooc).ps.torsoAnim == BOTH_A1_SPECIAL as c_int
                        || (*ooc).ps.torsoAnim == BOTH_A2_SPECIAL as c_int
                        || (*ooc).ps.torsoAnim == BOTH_A3_SPECIAL as c_int
                    {
                        //parry/block/break-parry bonus for single-style kata moves
                        defendStr += 1;
                    }
                    defendStr += ctx
                        .world
                        .bg_state
                        .rng
                        .Q_irand(0, (*ooc).saber[0].parryBonus);
                    if (*ooc).saber[1].model[0] != 0 && (*ooc).ps.saberHolstered == 0 {
                        defendStr += ctx
                            .world
                            .bg_state
                            .rng
                            .Q_irand(0, (*ooc).saber[1].parryBonus);
                    }

                    if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                        let s = format!(
                            "Client {} and client {} bounced off of each other's sabers\n",
                            (*self_).s.number,
                            (*otherOwner).s.number
                        );
                        Com_Printf(&s);
                    }

                    attackBonus = ctx
                        .world
                        .bg_state
                        .rng
                        .Q_irand(0, (*sc).saber[0].breakParryBonus);
                    if (*sc).saber[1].model[0] != 0 && (*sc).ps.saberHolstered == 0 {
                        attackBonus += ctx
                            .world
                            .bg_state
                            .rng
                            .Q_irand(0, (*sc).saber[1].breakParryBonus);
                    }
                    attackAdv = (attackStr
                        + attackBonus
                        + (*sc).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize])
                        - (defendStr + (*ooc).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize]);

                    if attackAdv > 1 {
                        //I won, he should knockaway
                        (*ooc).ps.saberMove = mp_bg::bg_panimate::BG_BrokenParryForAttack(
                            &ctx.world.bg_state,
                            (*ooc).ps.saberMove,
                        );
                        (*ooc).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
                    } else if attackAdv > 0 {
                        //I won, he should bounce, I should continue
                        (*ooc).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                    } else if attackAdv < 1 {
                        //I lost, I get knocked away
                        (*sc).ps.saberMove = mp_bg::bg_panimate::BG_BrokenParryForAttack(
                            &ctx.world.bg_state,
                            (*sc).ps.saberMove,
                        );
                        (*sc).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
                    } else if attackAdv < 0 {
                        //I lost, I bounce off
                        (*sc).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                    } else {
                        //even, both bounce off
                        (*sc).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                        (*ooc).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                    }

                    didOffense = qtrue;
                }
            }

            if ctx.world.cvars.d_saberGhoul2Collision.integer != 0
                && didDefense == 0
                && dmg <= SABER_NONATTACK_DAMAGE
                && otherUnblockable == 0
            {
                if PM_SaberInParry((*ooc).ps.saberMove) == 0
                    && PM_SaberInBrokenParry((*ooc).ps.saberMove) == 0
                    && BG_SaberInSpecial((*ooc).ps.saberMove) == 0
                    && PM_SaberInBounce((*ooc).ps.saberMove) == 0
                    && PM_SaberInDeflect((*ooc).ps.saberMove) == 0
                    && PM_SaberInReflect((*ooc).ps.saberMove) == 0
                {
                    WP_SaberBlockNonRandom(&*(otherOwner), tr.endpos, qfalse);
                    (*ooc).ps.saberEventFlags |= SEF_PARRIED;
                }
            } else if didDefense == 0 && dmg > SABER_NONATTACK_DAMAGE && otherUnblockable == 0 {
                //block
                if PM_SaberInParry((*ooc).ps.saberMove) == 0
                    && PM_SaberInBrokenParry((*ooc).ps.saberMove) == 0
                    && BG_SaberInSpecial((*ooc).ps.saberMove) == 0
                    && PM_SaberInBounce((*ooc).ps.saberMove) == 0
                    && PM_SaberInDeflect((*ooc).ps.saberMove) == 0
                    && PM_SaberInReflect((*ooc).ps.saberMove) == 0
                {
                    let mut crushTheParry: qboolean = qfalse;

                    if unblockable != 0 {
                        //It's unblockable. So send us into a broken parry immediately.
                        crushTheParry = qtrue;
                    }

                    if !SaberAttacking(&*(otherOwner)) {
                        let mut otherIdleStr = (*ooc).ps.fd.saberAnimLevel;
                        if otherIdleStr == saber_styles_t::SS_DUAL as c_int
                            || otherIdleStr == saber_styles_t::SS_STAFF as c_int
                        {
                            otherIdleStr = saber_styles_t::SS_MEDIUM as c_int;
                        }

                        WP_SaberBlockNonRandom(&*(otherOwner), tr.endpos, qfalse);
                        (*ooc).ps.saberEventFlags |= SEF_PARRIED;
                        (*sc).ps.saberEventFlags |= SEF_BLOCKED;

                        if attackStr + (*sc).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize]
                            > otherIdleStr + (*ooc).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize]
                        {
                            crushTheParry = qtrue;
                        } else {
                            tryDeflectAgain = qtrue;
                        }
                    } else if selfSaberLevel > otherSaberLevel
                        || (selfSaberLevel == otherSaberLevel
                            && ctx.world.bg_state.rng.Q_irand(1, 10) <= 2)
                    {
                        //they are attacking, and we managed to make them break
                        WP_SaberBlockNonRandom(&*(otherOwner), tr.endpos, qfalse);
                        crushTheParry = qtrue;

                        if (*ooc).ps.saberEntityNum != 0 {
                            saberCheckKnockdown_BrokenParry(
                                ctx,
                                Some(EntityId(((*ooc).ps.saberEntityNum) as u32)),
                                ctx.entity_id_of(otherOwner),
                                ctx.entity_id_of(self_),
                            );
                        }

                        if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                            let s = format!(
                                "Client {} forced client {} into a broken parry with a stronger attack\n",
                                (*self_).s.number,
                                (*otherOwner).s.number
                            );
                            Com_Printf(&s);
                        }
                    } else {
                        //They are attacking, so are we
                        if selfSaberLevel == otherSaberLevel {
                            //equal level, try to bounce off each other's sabers
                            if didOffense == 0
                                && PM_SaberInParry((*sc).ps.saberMove) == 0
                                && PM_SaberInBrokenParry((*sc).ps.saberMove) == 0
                                && BG_SaberInSpecial((*sc).ps.saberMove) == 0
                                && PM_SaberInBounce((*sc).ps.saberMove) == 0
                                && PM_SaberInDeflect((*sc).ps.saberMove) == 0
                                && PM_SaberInReflect((*sc).ps.saberMove) == 0
                                && unblockable == 0
                            {
                                (*sc).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                                didOffense = qtrue;
                            }
                            if didDefense == 0
                                && PM_SaberInParry((*ooc).ps.saberMove) == 0
                                && PM_SaberInBrokenParry((*ooc).ps.saberMove) == 0
                                && BG_SaberInSpecial((*ooc).ps.saberMove) == 0
                                && PM_SaberInBounce((*ooc).ps.saberMove) == 0
                                && PM_SaberInDeflect((*ooc).ps.saberMove) == 0
                                && PM_SaberInReflect((*ooc).ps.saberMove) == 0
                                && unblockable == 0
                            {
                                (*ooc).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                            }

                            if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                                let s = format!(
                                    "Equal attack level bounce/deflection for clients {} and {}\n",
                                    (*self_).s.number,
                                    (*otherOwner).s.number
                                );
                                Com_Printf(&s);
                            }

                            (*sc).ps.saberEventFlags |= SEF_DEFLECTED;
                            (*ooc).ps.saberEventFlags |= SEF_DEFLECTED;
                        } else if (ctx.world.level.time - (*ooc).lastSaberStorageTime) < 500
                            && unblockable == 0
                        {
                            //They are higher, this means they can actually smash us into a broken parry
                            (*sc).ps.saberMove = mp_bg::bg_panimate::BG_BrokenParryForAttack(
                                &ctx.world.bg_state,
                                (*sc).ps.saberMove,
                            );
                            (*sc).ps.saberBlocked = BLOCKED_PARRY_BROKEN;

                            if (*sc).ps.saberEntityNum != 0 {
                                saberCheckKnockdown_BrokenParry(
                                    ctx,
                                    Some(EntityId(((*sc).ps.saberEntityNum) as u32)),
                                    ctx.entity_id_of(self_),
                                    ctx.entity_id_of(otherOwner),
                                );
                            }

                            if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                                let s = format!(
                                    "Client {} hit client {}'s stronger attack, was forced into a broken parry\n",
                                    (*self_).s.number,
                                    (*otherOwner).s.number
                                );
                                Com_Printf(&s);
                            }

                            (*ooc).ps.saberEventFlags &= !SEF_BLOCKED;

                            didOffense = qtrue;
                        }
                    }

                    if crushTheParry != 0
                        && PM_SaberInParry(G_GetParryForBlock((*ooc).ps.saberBlocked)) != 0
                    {
                        //This means that the attack actually hit our saber, and we went to block it.
                        (*ooc).ps.saberMove = BG_BrokenParryForParry(
                            &mut ctx.world.bg_state,
                            G_GetParryForBlock((*ooc).ps.saberBlocked),
                        );
                        (*ooc).ps.saberBlocked = BLOCKED_PARRY_BROKEN;

                        (*ooc).ps.saberEventFlags &= !SEF_PARRIED;
                        (*sc).ps.saberEventFlags &= !SEF_BLOCKED;

                        if ctx.world.cvars.g_saberDebugPrint.integer != 0 {
                            let s = format!(
                                "Client {} broke through {}'s parry with a special or stronger attack\n",
                                (*self_).s.number,
                                (*otherOwner).s.number
                            );
                            Com_Printf(&s);
                        }
                    } else if PM_SaberInParry(G_GetParryForBlock((*ooc).ps.saberBlocked)) != 0
                        && didOffense == 0
                        && tryDeflectAgain != 0
                    {
                        //We want to try deflecting again
                        let preMove = (*ooc).ps.saberMove;

                        (*ooc).ps.saberMove = G_GetParryForBlock((*ooc).ps.saberBlocked);
                        WP_GetSaberDeflectionAngle(
                            ctx,
                            ctx.entity_id_of(self_),
                            ctx.entity_id_of(otherOwner),
                            tr.fraction,
                        );
                        (*ooc).ps.saberMove = preMove;
                    }
                }
            }

            (*sc).ps.saberAttackWound =
                ctx.world.level.time + ctx.world.cvars.g_saberDmgDelay_Wound.integer;
        }

        didHit
    }
}

/// Raven `G_SPSaberDamageTraceLerped`.
///
/// Source: `oracle/codemp/game/w_saber.c:5285-5480`
pub fn G_SPSaberDamageTraceLerped(
    ctx: &mut GameContext,
    self_: EntityId,
    saberNum: c_int,
    bladeNum: c_int,
    baseNew: &mut vec3_t,
    endNew: &mut vec3_t,
    clipmask: c_int,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        let ec = (*self_).client;
        // Referee probe: saber trail lerp — new base/tip vs stored trail base/tip.
        {
            let tb = &(*ec).saber[saberNum as usize].blade[bladeNum as usize].trail;
            probe!(
                "SAB_TRAIL",
                "t={} sn={} bn={} bN={:08x},{:08x},{:08x} eN={:08x},{:08x},{:08x} trB={:08x},{:08x},{:08x} trT={:08x},{:08x},{:08x} lt={}",
                ctx.world.level.time, saberNum, bladeNum,
                baseNew[0].to_bits(), baseNew[1].to_bits(), baseNew[2].to_bits(),
                endNew[0].to_bits(), endNew[1].to_bits(), endNew[2].to_bits(),
                tb.base[0].to_bits(), tb.base[1].to_bits(), tb.base[2].to_bits(),
                tb.tip[0].to_bits(), tb.tip[1].to_bits(), tb.tip[2].to_bits(),
                tb.lastTime
            );
        }
        let mut baseOld: vec3_t = [0.0; 3];
        let mut endOld: vec3_t = [0.0; 3];
        let mut mp1: vec3_t = [0.0; 3];
        let mut mp2: vec3_t = [0.0; 3];
        let mut md1: vec3_t = [0.0; 3];
        let mut md2: vec3_t = [0.0; 3];

        if (ctx.world.level.time
            - (*ec).saber[saberNum as usize].blade[bladeNum as usize]
                .trail
                .lastTime)
            > 100
        {
            //no valid last pos, use current
            baseOld = *baseNew;
            endOld = *endNew;
        } else {
            //trace from last pos
            baseOld = (*ec).saber[saberNum as usize].blade[bladeNum as usize]
                .trail
                .base;
            endOld = (*ec).saber[saberNum as usize].blade[bladeNum as usize]
                .trail
                .tip;
        }

        mp1 = baseOld;
        mp2 = *baseNew;
        _VectorSubtract(endOld, baseOld, &mut md1);
        VectorNormalize(&mut md1);
        _VectorSubtract(*endNew, *baseNew, &mut md2);
        VectorNormalize(&mut md2);

        ctx.world.globals.saberHitWall = qfalse;
        ctx.world.globals.saberHitSaber = qfalse;
        ctx.world.globals.saberHitFraction = 1.0f32;
        if VectorCompare2(baseOld, *baseNew) && VectorCompare2(endOld, *endNew) {
            //no diff
            CheckSaberDamage(
                ctx,
                ctx.entity_id_of(self_).unwrap(),
                saberNum,
                bladeNum,
                *baseNew,
                endNew,
                qfalse,
                clipmask,
                qfalse,
            );
        } else {
            //saber moved, lerp
            let mut step: f32;
            let stepsize: f32 = 8.0; //aveLength,
            let mut ma1: vec3_t = [0.0; 3];
            let mut ma2: vec3_t = [0.0; 3];
            let mut md2ang: vec3_t = [0.0; 3];
            let mut curBase1: vec3_t = [0.0; 3];
            let mut curBase2: vec3_t = [0.0; 3];
            let mut xx: c_int;
            let mut curMD1: vec3_t = [0.0; 3];
            let mut curMD2: vec3_t = [0.0; 3];
            let mut dirInc: f32;
            let mut curDirFrac: f32;
            let mut baseDiff: vec3_t = [0.0; 3];
            let mut bladePointOld: vec3_t = [0.0; 3];
            let mut bladePointNew: vec3_t = [0.0; 3];
            let mut extrapolate: qboolean = qtrue;

            //do the trace at the base first
            bladePointOld = baseOld;
            bladePointNew = *baseNew;
            CheckSaberDamage(
                ctx,
                ctx.entity_id_of(self_).unwrap(),
                saberNum,
                bladeNum,
                bladePointOld,
                &mut bladePointNew,
                qfalse,
                clipmask,
                qtrue,
            );

            //if hit a saber, shorten rest of traces to match
            if ctx.world.globals.saberHitFraction < 1.0f32 {
                //adjust muzzleDir...
                vectoangles(md1, &mut ma1);
                vectoangles(md2, &mut ma2);
                xx = 0;
                while xx < 3 {
                    md2ang[xx as usize] = LerpAngle(
                        ma1[xx as usize],
                        ma2[xx as usize],
                        ctx.world.globals.saberHitFraction,
                    );
                    xx += 1;
                }
                AngleVectors(md2ang, Some(&mut md2), None, None);
                //shorten the base pos
                _VectorSubtract(mp2, mp1, &mut baseDiff);
                _VectorMA(mp1, ctx.world.globals.saberHitFraction, baseDiff, baseNew);
                _VectorMA(
                    *baseNew,
                    (*ec).saber[saberNum as usize].blade[bladeNum as usize].lengthMax,
                    md2,
                    endNew,
                );
            }

            //If the angle diff in the blade is high, need to do it in chunks of 33 to avoid flattening of the arc
            if BG_SaberInAttack((*ec).ps.saberMove) != 0
                || BG_SaberInSpecialAttack((*ec).ps.torsoAnim) != 0
                || BG_SpinningSaberAnim((*ec).ps.torsoAnim) != 0
                || BG_InSpecialJump((*ec).ps.torsoAnim) != 0
            {
                curDirFrac = _DotProduct(md1, md2);
            } else {
                curDirFrac = 1.0f32;
            }
            //NOTE: if saber spun at least 180 degrees since last damage trace, this is not reliable...!
            if curDirFrac.abs() < 1.0f32 - MAX_SABER_SWING_INC {
                //the saber blade spun more than 33 degrees since the last damage trace
                dirInc = 1.0f32 / ((1.0f32 - curDirFrac) / MAX_SABER_SWING_INC);
                curDirFrac = dirInc;
            } else {
                curDirFrac = 1.0f32;
                dirInc = 0.0f32;
            }

            vectoangles(md1, &mut ma1);
            vectoangles(md2, &mut ma2);

            curMD2 = md1;
            curBase2 = baseOld;

            loop {
                curMD1 = curMD2;
                curBase1 = curBase2;
                if curDirFrac >= 1.0f32 {
                    curMD2 = md2;
                    curBase2 = *baseNew;
                } else {
                    xx = 0;
                    while xx < 3 {
                        md2ang[xx as usize] =
                            LerpAngle(ma1[xx as usize], ma2[xx as usize], curDirFrac);
                        xx += 1;
                    }
                    AngleVectors(md2ang, Some(&mut curMD2), None, None);
                    _VectorSubtract(*baseNew, baseOld, &mut baseDiff);
                    _VectorMA(baseOld, curDirFrac, baseDiff, &mut curBase2);
                }
                // Move up the blade in intervals of stepsize
                step = stepsize;
                while step <= (*ec).saber[saberNum as usize].blade[bladeNum as usize].lengthMax {
                    _VectorMA(curBase1, step, curMD1, &mut bladePointOld);
                    _VectorMA(curBase2, step, curMD2, &mut bladePointNew);

                    if step + stepsize
                        >= (*ec).saber[saberNum as usize].blade[bladeNum as usize].lengthMax
                    {
                        extrapolate = qfalse;
                    }
                    //do the damage trace
                    CheckSaberDamage(
                        ctx,
                        ctx.entity_id_of(self_).unwrap(),
                        saberNum,
                        bladeNum,
                        bladePointOld,
                        &mut bladePointNew,
                        qfalse,
                        clipmask,
                        extrapolate,
                    );

                    //if hit a saber, shorten rest of traces to match
                    if ctx.world.globals.saberHitFraction < 1.0f32 {
                        let mut curMA1: vec3_t = [0.0; 3];
                        let mut curMA2: vec3_t = [0.0; 3];
                        //adjust muzzle endpoint
                        _VectorSubtract(mp2, mp1, &mut baseDiff);
                        _VectorMA(mp1, ctx.world.globals.saberHitFraction, baseDiff, baseNew);
                        _VectorMA(
                            *baseNew,
                            (*ec).saber[saberNum as usize].blade[bladeNum as usize].lengthMax,
                            curMD2,
                            endNew,
                        );
                        //adjust muzzleDir...
                        vectoangles(curMD1, &mut curMA1);
                        vectoangles(curMD2, &mut curMA2);
                        xx = 0;
                        while xx < 3 {
                            md2ang[xx as usize] = LerpAngle(
                                curMA1[xx as usize],
                                curMA2[xx as usize],
                                ctx.world.globals.saberHitFraction,
                            );
                            xx += 1;
                        }
                        AngleVectors(md2ang, Some(&mut curMD2), None, None);
                        ctx.world.globals.saberHitSaber = qtrue;
                    }
                    if ctx.world.globals.saberHitWall != 0 {
                        break;
                    }
                    step += stepsize;
                }
                if ctx.world.globals.saberHitWall != 0 || ctx.world.globals.saberHitSaber != 0 {
                    break;
                }
                if curDirFrac >= 1.0f32 {
                    break;
                } else {
                    curDirFrac += dirInc;
                    if curDirFrac >= 1.0f32 {
                        curDirFrac = 1.0f32;
                    }
                }
            }
        }
    }
}

/// Raven `WP_SaberStartMissileBlockCheck`.
///
/// Source: `oracle/codemp/game/w_saber.c:5492-5883`
pub fn WP_SaberStartMissileBlockCheck(
    ctx: &mut GameContext,
    self_: EntityId,
    ucmd: *mut usercmd_t,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        let base = ctx.world.g_entities.as_mut_ptr();
        let sc = (*self_).client;
        let mut dist: f32;
        let mut ent: *mut gentity_t;
        let mut incoming: *mut gentity_t = core::ptr::null_mut();
        let mut entityList: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
        let numListedEntities: c_int;
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut i: c_int;
        let radius: f32 = 256.0;
        let mut closestDist: f32;
        let mut forward: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let mut missile_dir: vec3_t = [0.0; 3];
        let mut fwdangles: vec3_t = [0.0; 3];
        let mut trace: trace_t = core::mem::zeroed();
        let mut traceTo: vec3_t = [0.0; 3];
        let mut entDir: vec3_t = [0.0; 3];
        let mut lookTDist: f32 = -1.0;
        let mut lookT: *mut gentity_t = core::ptr::null_mut();
        let mut doFullRoutine: qboolean = qtrue;

        //keep this updated even if we don't get below
        if (*sc).ps.eFlags2 & EF2_HELD_BY_MONSTER == 0 {
            //lookTarget is set by and to the monster that's holding you, no other operations can change that
            (*sc).ps.hasLookTarget = qfalse;
        }

        if (*sc).ps.weapon != WP_SABER as c_int && (*sc).NPC_class != CLASS_BOBAFETT {
            doFullRoutine = qfalse;
        } else if (*sc).ps.saberInFlight != 0 {
            doFullRoutine = qfalse;
        } else if (*sc).ps.fd.forcePowersActive & (1 << FP_LIGHTNING) != 0 {
            //can't block while zapping
            doFullRoutine = qfalse;
        } else if (*sc).ps.fd.forcePowersActive & (1 << FP_DRAIN) != 0 {
            //can't block while draining
            doFullRoutine = qfalse;
        } else if (*sc).ps.fd.forcePowersActive & (1 << FP_PUSH) != 0 {
            //can't block while shoving
            doFullRoutine = qfalse;
        } else if (*sc).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0 {
            //can't block while gripping
            doFullRoutine = qfalse;
        }

        if (*sc).ps.weaponTime > 0 {
            //don't autoblock while busy with stuff
            return;
        }

        if (*sc).saber[0].saberFlags & SFL_NOT_ACTIVE_BLOCKING != 0 {
            //can't actively block with this saber type
            return;
        }

        if (*self_).health <= 0 {
            //dead don't try to block (NOTE: actual deflection happens in missile code)
            return;
        }
        if PM_InKnockDown(&mut (*sc).ps as *mut playerState_t) != 0 {
            //can't block when knocked down
            return;
        }

        if BG_SabersOff(&mut (*sc).ps as *mut playerState_t) != 0
            && (*sc).NPC_class != CLASS_BOBAFETT
        {
            if (*self_).s.eType != ET_NPC as c_int {
                //player doesn't auto-activate
                doFullRoutine = qfalse;
            }
        }

        if (*self_).s.eType == ET_PLAYER as c_int {
            //don't do this if already attacking!
            if (*ucmd).buttons & BUTTON_ATTACK != 0
                || BG_SaberInAttack((*sc).ps.saberMove) != 0
                || BG_SaberInSpecialAttack((*sc).ps.torsoAnim) != 0
                || BG_SaberInTransitionAny((*sc).ps.saberMove) != 0
            {
                doFullRoutine = qfalse;
            }
        }

        if (*sc).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] > ctx.world.level.time {
            //can't block while gripping
            doFullRoutine = qfalse;
        }

        fwdangles[1] = (*sc).ps.viewangles[1];
        AngleVectors(fwdangles, Some(&mut forward), None, None);

        i = 0;
        while i < 3 {
            mins[i as usize] = (*self_).r.currentOrigin[i as usize] - radius;
            maxs[i as usize] = (*self_).r.currentOrigin[i as usize] + radius;
            i += 1;
        }

        numListedEntities = trap::EntitiesInBox(
            ctx.engine,
            GEntitiesInBoxArgs::new(
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                entityList.as_mut_ptr(),
                MAX_GENTITIES as c_int,
            ),
        );

        closestDist = radius;

        for e in 0..numListedEntities as usize {
            ent = &mut ctx.world.g_entities[entityList[e] as usize] as *mut gentity_t;

            if ent == self_ {
                continue;
            }

            //as long as we're here I'm going to get a looktarget too, I guess. -rww
            let ec = (*ent).client;
            if (*self_).s.eType == ET_PLAYER as c_int
                && !(*ent).client.is_null()
                && ((*ent).s.eType == ET_NPC as c_int || (*ent).s.eType == ET_PLAYER as c_int)
                && OnSameTeam(ctx, ctx.entity_id_of(ent), ctx.entity_id_of(self_)) == 0
                && (*ec).sess.sessionTeam != TEAM_SPECTATOR
                && (*ec).ps.pm_flags & PMF_FOLLOW == 0
                && ((*ent).s.eType != ET_NPC as c_int || (*ent).s.NPC_class != CLASS_VEHICLE as c_int) //don't look at vehicle NPCs
                && (*ent).health > 0
            {
                //seems like a valid enemy to look at.
                let mut vecSub: vec3_t = [0.0; 3];
                let vecLen: f32;

                _VectorSubtract((*sc).ps.origin, (*ec).ps.origin, &mut vecSub);
                vecLen = VectorLength(vecSub);

                if lookTDist == -1.0 || vecLen < lookTDist {
                    let mut tr: trace_t = core::mem::zeroed();
                    let mut myEyes: vec3_t = [0.0; 3];

                    myEyes = (*sc).ps.origin;
                    myEyes[2] += (*sc).ps.viewheight as f32;

                    trap::Trace(
                        ctx.engine,
                        GTraceArgs::new(
                            &mut tr as *mut trace_t,
                            &myEyes as *const vec3_t,
                            core::ptr::null(),
                            core::ptr::null(),
                            &(*ec).ps.origin as *const vec3_t,
                            (*self_).s.number,
                            MASK_PLAYERSOLID,
                        ),
                    );

                    if tr.fraction == 1.0f32 || tr.entityNum as c_int == (*ent).s.number {
                        //we have a clear line of sight to him, so it's all good.
                        lookT = ent;
                        lookTDist = vecLen;
                    }
                }
            }

            if doFullRoutine == 0 {
                //don't care about the rest then
                continue;
            }

            if (*ent).r.ownerNum == (*self_).s.number {
                continue;
            }
            if (*ent).inuse == 0 {
                continue;
            }
            if (*ent).s.eType != ET_MISSILE as c_int && (*ent).s.eFlags & EF_MISSILE_STICK == 0 {
                //not a normal projectile
                if (*ent).r.ownerNum < 0 || (*ent).r.ownerNum >= ENTITYNUM_WORLD {
                    //not going to be a client then.
                    continue;
                }

                let pOwner =
                    &mut ctx.world.g_entities[(*ent).r.ownerNum as usize] as *mut gentity_t;
                let poc = (*pOwner).client;

                if (*pOwner).inuse == 0 || (*pOwner).client.is_null() {
                    continue; //not valid cl owner
                }

                if (*poc).ps.saberEntityNum == 0
                    || (*poc).ps.saberInFlight == 0
                    || (*poc).ps.saberEntityNum != (*ent).s.number
                {
                    //the saber is knocked away and/or not flying actively, or this ent is not the cl's saber ent at all
                    continue;
                }

                //If we get here then it's ok to be treated as a thrown saber, I guess.
            } else {
                if (*ent).s.pos.trType == TR_STATIONARY && (*self_).s.eType == ET_PLAYER as c_int {
                    //nothing you can do with a stationary missile if you're the player
                    continue;
                }
            }

            //see if they're in front of me
            _VectorSubtract((*ent).r.currentOrigin, (*self_).r.currentOrigin, &mut dir);
            dist = VectorNormalize(&mut dir);
            //FIXME: handle detpacks, proximity mines and tripmines
            if (*ent).s.weapon == WP_THERMAL as c_int {
                //thermal detonator!
                if !(*self_).NPC.is_null() && dist < (*ent).splashRadius as f32 {
                    if dist < (*ent).splashRadius as f32
                        && (*ent).nextthink < ctx.world.level.time + 600
                        && (*ent).count != 0
                        && (*sc).ps.groundEntityNum != ENTITYNUM_NONE
                        && ((*ent).s.pos.trType == TR_STATIONARY
                            || (*ent).s.pos.trType == TR_INTERPOLATE
                            || _DotProduct(dir, forward) < SABER_REFLECT_MISSILE_CONE
                            || !WP_ForcePowerUsable(ctx, ctx.entity_id_of(self_).unwrap(), FP_PUSH))
                    {
                        //TD is close enough to hurt me, I'm on the ground and the thing is at rest or behind me and about to blow up, or I don't have force-push so force-jump!
                        (*sc).ps.fd.forceJumpCharge = 480.0f32;
                    } else if (*sc).NPC_class != CLASS_BOBAFETT {
                        //FIXME: check forcePushRadius[NPC->client->ps.forcePowerLevel[FP_PUSH]]
                        ForceThrow(ctx, ctx.entity_id_of(self_).unwrap(), false);
                    }
                }
                continue;
            } else if (*ent).splashDamage != 0 && (*ent).splashRadius != 0 {
                //exploding missile
                if (*self_).s.eType == ET_PLAYER as c_int {
                    //players don't auto-handle these at all
                    continue;
                } else {
                    // (Raven `if (0)` placed-explosive branch dropped per §20 — never taken.)
                    if dist < (*ent).splashRadius as f32
                        && (*sc).ps.groundEntityNum != ENTITYNUM_NONE
                        && (_DotProduct(dir, forward) < SABER_REFLECT_MISSILE_CONE
                            || !WP_ForcePowerUsable(ctx, ctx.entity_id_of(self_).unwrap(), FP_PUSH))
                    {
                        //NPCs try to evade it
                        (*sc).ps.fd.forceJumpCharge = 480.0f32;
                    } else if (*sc).NPC_class != CLASS_BOBAFETT {
                        //else, try to force-throw it away
                        ForceThrow(ctx, ctx.entity_id_of(self_).unwrap(), false);
                    }
                }
                //otherwise, can't block it, so we're screwed
                continue;
            }

            if (*ent).s.weapon != WP_SABER as c_int {
                //only block shots coming from behind
                if _DotProduct(dir, forward) < SABER_REFLECT_MISSILE_CONE {
                    continue;
                }
            } else if (*self_).s.eType == ET_PLAYER as c_int {
                //player never auto-blocks thrown sabers
                continue;
            } //NPCs always try to block sabers coming from behind!

            //see if they're heading towards me
            missile_dir = (*ent).s.pos.trDelta;
            VectorNormalize(&mut missile_dir);
            if _DotProduct(dir, missile_dir) > 0.0 {
                continue;
            }

            //FIXME: must have a clear trace to me, too...
            if dist < closestDist {
                traceTo = (*self_).r.currentOrigin;
                traceTo[2] = (*self_).r.absmax[2] - 4.0;
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut trace as *mut trace_t,
                        &(*ent).r.currentOrigin as *const vec3_t,
                        &(*ent).r.mins as *const vec3_t,
                        &(*ent).r.maxs as *const vec3_t,
                        &traceTo as *const vec3_t,
                        (*ent).s.number,
                        (*ent).clipmask,
                    ),
                );
                if trace.allsolid != 0
                    || trace.startsolid != 0
                    || (trace.fraction < 1.0f32
                        && trace.entityNum as c_int != (*self_).s.number
                        && trace.entityNum as c_int != (*sc).ps.saberEntityNum)
                {
                    //okay, try one more check
                    VectorNormalize2((*ent).s.pos.trDelta, &mut entDir);
                    _VectorMA((*ent).r.currentOrigin, radius, entDir, &mut traceTo);
                    trap::Trace(
                        ctx.engine,
                        GTraceArgs::new(
                            &mut trace as *mut trace_t,
                            &(*ent).r.currentOrigin as *const vec3_t,
                            &(*ent).r.mins as *const vec3_t,
                            &(*ent).r.maxs as *const vec3_t,
                            &traceTo as *const vec3_t,
                            (*ent).s.number,
                            (*ent).clipmask,
                        ),
                    );
                    if trace.allsolid != 0
                        || trace.startsolid != 0
                        || (trace.fraction < 1.0f32
                            && trace.entityNum as c_int != (*self_).s.number
                            && trace.entityNum as c_int != (*sc).ps.saberEntityNum)
                    {
                        //can't hit me, ignore it
                        continue;
                    }
                }
                if (*self_).s.eType == ET_NPC as c_int {
                    //An NPC
                    if !(*self_).NPC.is_null()
                        && (*self_).enemy.is_none()
                        && (*ent).r.ownerNum != ENTITYNUM_NONE
                    {
                        let owner =
                            &mut ctx.world.g_entities[(*ent).r.ownerNum as usize] as *mut gentity_t;
                        let owc = (*owner).client;
                        if (*owner).health >= 0
                            && ((*owner).client.is_null() || (*owc).playerTeam != (*sc).playerTeam)
                        {
                            G_SetEnemy(
                                ctx,
                                ctx.entity_id_of(self_).unwrap(),
                                ctx.entity_id_of(owner),
                            );
                        }
                    }
                }
                //FIXME: if NPC, predict the intersection...
                closestDist = dist;
                incoming = ent;
            }
        }

        if (*self_).s.eType == ET_NPC as c_int && (*self_).localAnimIndex <= 1 {
            //humanoid NPCs don't set angles based on server angles for looking, unlike other NPCs
            if !(*self_).client.is_null() && (*sc).renderInfo.lookTarget < ENTITYNUM_WORLD {
                lookT = &mut ctx.world.g_entities[(*sc).renderInfo.lookTarget as usize]
                    as *mut gentity_t;
            }
        }

        if !lookT.is_null() {
            //we got a looktarget at some point so we'll assign it then.
            if (*sc).ps.eFlags2 & EF2_HELD_BY_MONSTER == 0 {
                //lookTarget is set by and to the monster that's holding you, no other operations can change that
                (*sc).ps.hasLookTarget = qtrue;
                (*sc).ps.lookTarget = (*lookT).s.number;
            }
        }

        if doFullRoutine == 0 {
            //then we're done now
            return;
        }

        if !incoming.is_null() {
            if !(*self_).NPC.is_null() {
                let npc = (*self_).NPC;
                if Jedi_WaitingAmbush(&*self_) != 0 {
                    Jedi_Ambush(ctx, ctx.entity_id_of(self_).unwrap());
                }
                if (*sc).NPC_class == CLASS_BOBAFETT
                    && (*sc).ps.eFlags2 & EF2_FLYING != 0
                    && (*incoming).methodOfDeath != MOD_ROCKET_HOMING as c_int
                {
                    //a hovering Boba Fett, not a tracking rocket
                    if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                        //strafe
                        (*npc).standTime = 0;
                        (*sc).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                            ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(1000, 2000);
                    }
                    if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                        //go up/down
                        let ident = cstr("heightChange");
                        let self_id = ctx.entity_id_of(self_);
                        let ident_time = ctx.world.bg_state.rng.Q_irand(1000, 3000);
                        TIMER_Set(ctx, self_id, ident.as_ptr(), ident_time);
                        (*sc).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                            ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(1000, 2000);
                    }
                } else if Jedi_SaberBlockGo(
                    ctx,
                    ctx.entity_id_of(self_).unwrap(),
                    &mut (*npc).last_ucmd as *mut usercmd_t,
                    vec3_origin,
                    vec3_origin,
                    ctx.entity_id_of(incoming),
                    0.0f32,
                ) != evasionType_t::EVASION_NONE
                {
                    //make sure to turn on your saber if it's not on
                    if (*sc).NPC_class != CLASS_BOBAFETT {
                        WP_ActivateSaber(ctx, ctx.entity_id_of(self_));
                    }
                }
            } else {
                //player
                let owner =
                    &mut ctx.world.g_entities[(*incoming).r.ownerNum as usize] as *mut gentity_t;

                WP_SaberBlockNonRandom(&*(self_), (*incoming).r.currentOrigin, qtrue);
                let owc = (*owner).client;
                let selfEnemy = ent_id::resolve(base, (*self_).enemy);
                if !(*owner).client.is_null()
                    && ((*self_).enemy.is_none() || (*selfEnemy).s.weapon != WP_SABER as c_int)
                //keep enemy jedi over shooters
                {
                    (*self_).enemy = Some(ent_id(base, owner));
                }
            }
        }
    }
}

/// Raven `CheckThrownSaberDamaged`.
///
/// Source: `oracle/codemp/game/w_saber.c:5894-6125`
pub fn CheckThrownSaberDamaged(
    ctx: &mut GameContext,
    saberent: EntityId,
    saberOwner: Option<EntityId>,
    ent: Option<EntityId>,
    dist: c_int,
    returning: c_int,
    noDCheck: qboolean,
) -> bool {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let saberent: *mut gentity_t = ctx.entity_mut(saberent);
    let saberOwner: *mut gentity_t =
        unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), saberOwner) };
    let ent: *mut gentity_t = unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), ent) };
    unsafe {
        let level_time = ctx.world.level.time;
        let mut vecsub: vec3_t;
        let mut veclen: f32;
        let base = ctx.world.g_entities.as_mut_ptr();

        let soc = (*saberOwner).client;
        if !saberOwner.is_null()
            && !(*saberOwner).client.is_null()
            && (*soc).ps.saberAttackWound > level_time
        {
            return false;
        }

        if !ent.is_null()
            && !(*ent).client.is_null()
            && (*ent).inuse != 0
            && (*ent).s.number != (*saberOwner).s.number
            && (*ent).health > 0
            && (*ent).takedamage != 0
            && trap::InPVS(
                ctx.engine,
                mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                    &(*((*ent).client)).ps.origin as *const vec3_t,
                    &(*saberent).r.currentOrigin as *const vec3_t,
                ),
            ) != 0
            && (*((*ent).client)).sess.sessionTeam != TEAM_SPECTATOR
            && ((*((*ent).client)).pers.connected != 0 || (*ent).s.eType == ET_NPC as c_int)
        {
            // hit a client
            let ec = (*ent).client;
            if (*ec).ps.duelInProgress != 0 && (*ec).ps.duelIndex != (*saberOwner).s.number {
                return false;
            }
            if (*soc).ps.duelInProgress != 0 && (*soc).ps.duelIndex != (*ent).s.number {
                return false;
            }

            vecsub = [0.0; 3];
            _VectorSubtract((*saberent).r.currentOrigin, (*ec).ps.origin, &mut vecsub);
            veclen = VectorLength(vecsub);

            if veclen < dist as f32 {
                // within range
                let mut tr: trace_t = core::mem::zeroed();
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &(*saberent).r.currentOrigin as *const vec3_t,
                        core::ptr::null(),
                        core::ptr::null(),
                        &(*ec).ps.origin as *const vec3_t,
                        (*saberent).s.number,
                        MASK_SHOT,
                    ),
                );

                if tr.fraction == 1.0 || tr.entityNum as c_int == (*ent).s.number {
                    if (*soc).ps.isJediMaster == 0
                        && WP_SaberCanBlock(
                            ctx,
                            ctx.entity_id_of(ent),
                            tr.endpos,
                            0,
                            MOD_SABER as c_int,
                            false,
                            999,
                        ) != 0
                    {
                        // they blocked it
                        WP_SaberBlockNonRandom(&*(ent), tr.endpos, qfalse);

                        let te_id = G_TempEntity(ctx, tr.endpos, EV_SABER_BLOCK as c_int);
                        let te = ctx.entity_mut(te_id);
                        te.s.origin = tr.endpos;
                        te.s.angles = tr.plane.normal;
                        if te.s.angles[0] == 0.0 && te.s.angles[1] == 0.0 && te.s.angles[2] == 0.0 {
                            te.s.angles[1] = 1.0;
                        }
                        te.s.eventParm = 1;
                        te.s.weapon = 0;
                        te.s.legsAnim = 0;

                        if saberCheckKnockdown_Thrown(
                            ctx,
                            ctx.entity_id_of(saberent),
                            ctx.entity_id_of(saberOwner),
                            ctx.entity_id_of(
                                &mut (*base.add(tr.entityNum as usize)) as *mut gentity_t,
                            ),
                        ) {
                            // it was knocked out of the air
                            return false;
                        }

                        if returning == 0 {
                            // return to owner if blocked
                            thrownSaberTouch(
                                ctx,
                                ctx.entity_id_of(saberent).unwrap(),
                                ctx.entity_id_of(saberent),
                                core::ptr::null_mut(),
                            );
                        }

                        (*soc).ps.saberAttackWound = level_time + 500;
                        return false;
                    } else {
                        // a good hit
                        let mut dir: vec3_t = [0.0; 3];
                        let mut dflags = 0;

                        _VectorSubtract(tr.endpos, (*saberent).r.currentOrigin, &mut dir);
                        VectorNormalize(&mut dir);

                        if dir[0] == 0.0 && dir[1] == 0.0 && dir[2] == 0.0 {
                            dir[1] = 1.0;
                        }

                        if ((*soc).saber[0].saberFlags2 & SFL2_NO_DISMEMBERMENT) != 0 {
                            dflags |= DAMAGE_NO_DISMEMBER;
                        }

                        if (*soc).saber[0].knockbackScale > 0.0 {
                            dflags |= DAMAGE_SABER_KNOCKBACK1;
                        }

                        if (*soc).ps.isJediMaster != 0 {
                            // 2x damage for the Jedi Master
                            G_Damage(
                                ctx,
                                ctx.entity_id_of(ent),
                                ctx.entity_id_of(saberOwner),
                                ctx.entity_id_of(saberOwner),
                                Some(&mut dir),
                                tr.endpos,
                                (*saberent).damage * 2,
                                dflags,
                                MOD_SABER as c_int,
                            );
                        } else {
                            G_Damage(
                                ctx,
                                ctx.entity_id_of(ent),
                                ctx.entity_id_of(saberOwner),
                                ctx.entity_id_of(saberOwner),
                                Some(&mut dir),
                                tr.endpos,
                                (*saberent).damage,
                                dflags,
                                MOD_SABER as c_int,
                            );
                        }

                        let te_id = G_TempEntity(ctx, tr.endpos, EV_SABER_HIT as c_int);
                        let te = ctx.entity_mut(te_id);
                        te.s.otherEntityNum = (*ent).s.number;
                        te.s.otherEntityNum2 = (*saberOwner).s.number;
                        te.s.weapon = 0;
                        te.s.legsAnim = 0;
                        te.s.origin = tr.endpos;
                        te.s.angles = tr.plane.normal;
                        if te.s.angles[0] == 0.0 && te.s.angles[1] == 0.0 && te.s.angles[2] == 0.0 {
                            te.s.angles[1] = 1.0;
                        }

                        te.s.eventParm = 1;

                        if returning == 0 {
                            thrownSaberTouch(
                                ctx,
                                ctx.entity_id_of(saberent).unwrap(),
                                ctx.entity_id_of(saberent),
                                core::ptr::null_mut(),
                            );
                        }
                    }

                    (*soc).ps.saberAttackWound = level_time + 500;
                }
            }
        } else if !ent.is_null()
            && (*ent).client.is_null()
            && (*ent).inuse != 0
            && (*ent).takedamage != 0
            && (*ent).health > 0
            && (*ent).s.number != (*saberOwner).s.number
            && (*ent).s.number != (*saberent).s.number
            && (noDCheck != 0
                || trap::InPVS(
                    ctx.engine,
                    mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                        &(*ent).r.currentOrigin as *const vec3_t,
                        &(*saberent).r.currentOrigin as *const vec3_t,
                    ),
                ) != 0)
        {
            // hit a non-client
            if noDCheck != 0 {
                veclen = 0.0;
            } else {
                vecsub = [0.0; 3];
                _VectorSubtract(
                    (*saberent).r.currentOrigin,
                    (*ent).r.currentOrigin,
                    &mut vecsub,
                );
                veclen = VectorLength(vecsub);
            }

            if veclen < dist as f32 {
                let mut tr: trace_t = core::mem::zeroed();
                let mut entOrigin: vec3_t = [0.0; 3];

                if (*ent).s.eType == ET_MOVER as c_int {
                    _VectorSubtract((*ent).r.absmax, (*ent).r.absmin, &mut entOrigin);
                    let tmp = entOrigin;
                    _VectorMA((*ent).r.absmin, 0.5, tmp, &mut entOrigin);
                    _VectorAdd((*ent).r.absmin, (*ent).r.absmax, &mut entOrigin);
                    let tmp2 = entOrigin;
                    _VectorScale(tmp2, 0.5, &mut entOrigin);
                } else {
                    entOrigin = (*ent).r.currentOrigin;
                }

                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &(*saberent).r.currentOrigin as *const vec3_t,
                        core::ptr::null(),
                        core::ptr::null(),
                        &entOrigin as *const vec3_t,
                        (*saberent).s.number,
                        MASK_SHOT,
                    ),
                );

                if tr.fraction == 1.0 || tr.entityNum as c_int == (*ent).s.number {
                    let mut dir: vec3_t = [0.0; 3];
                    let mut dflags = 0;

                    _VectorSubtract(tr.endpos, entOrigin, &mut dir);
                    VectorNormalize(&mut dir);

                    if ((*soc).saber[0].saberFlags2 & SFL2_NO_DISMEMBERMENT) != 0 {
                        dflags |= DAMAGE_NO_DISMEMBER;
                    }
                    if (*soc).saber[0].knockbackScale > 0.0 {
                        dflags |= DAMAGE_SABER_KNOCKBACK1;
                    }

                    if (*ent).s.eType == ET_NPC as c_int {
                        // an animent
                        G_Damage(
                            ctx,
                            ctx.entity_id_of(ent),
                            ctx.entity_id_of(saberOwner),
                            ctx.entity_id_of(saberOwner),
                            Some(&mut dir),
                            tr.endpos,
                            40,
                            dflags,
                            MOD_SABER as c_int,
                        );
                    } else {
                        G_Damage(
                            ctx,
                            ctx.entity_id_of(ent),
                            ctx.entity_id_of(saberOwner),
                            ctx.entity_id_of(saberOwner),
                            Some(&mut dir),
                            tr.endpos,
                            5,
                            dflags,
                            MOD_SABER as c_int,
                        );
                    }

                    let te_eid = G_TempEntity(ctx, tr.endpos, EV_SABER_HIT as c_int);
                    let te = ctx.entity_mut(te_eid);
                    te.s.otherEntityNum = ENTITYNUM_NONE;
                    te.s.otherEntityNum2 = (*saberOwner).s.number;
                    te.s.weapon = 0;
                    te.s.legsAnim = 0;
                    te.s.origin = tr.endpos;
                    te.s.angles = tr.plane.normal;
                    if te.s.angles[0] == 0.0 && te.s.angles[1] == 0.0 && te.s.angles[2] == 0.0 {
                        te.s.angles[1] = 1.0;
                    }

                    if (*ent).s.eType == ET_MOVER as c_int {
                        if !saberOwner.is_null()
                            && !(*saberOwner).client.is_null()
                            && ((*soc).saber[0].saberFlags2 & SFL2_NO_CLASH_FLARE) != 0
                        {
                            // don't do clash flare
                            G_FreeEntity(ctx, Some(te_eid));
                        } else {
                            let teS_id = G_TempEntity(ctx, tr.endpos, EV_SABER_CLASHFLARE as c_int);
                            ctx.entity_mut(teS_id).s.origin = tr.endpos;
                            ctx.entity_mut(te_eid).s.eventParm = 0;
                        }
                    } else {
                        ctx.entity_mut(te_eid).s.eventParm = 1;
                    }

                    if returning == 0 {
                        // return to owner if blocked
                        thrownSaberTouch(
                            ctx,
                            ctx.entity_id_of(saberent).unwrap(),
                            ctx.entity_id_of(saberent),
                            core::ptr::null_mut(),
                        );
                    }

                    (*soc).ps.saberAttackWound = level_time + 500;
                }
            }
        }

        true
    }
}

/// Raven `saberCheckRadiusDamage`.
///
/// Source: `oracle/codemp/game/w_saber.c:6127-6161`
// we're going to cheat and damage players within the saber's radius, just for the
// sake of doing things more "efficiently" (and because the saber entity has no
// server g2 instance)
pub fn saberCheckRadiusDamage(ctx: &mut GameContext, saberent: EntityId, returning: c_int) {
    let mut i = 0;
    let dist: c_int;
    // saberOwner is an array slot, never null; the oracle's `!saberOwner`
    // guard is vacuous in Rust.
    let owner_num = ctx.world.entity(saberent).r.ownerNum;
    let saber_owner = EntityId(owner_num as u32);

    if returning != 0 && returning != 2 {
        dist = (MIN_SABER_SLICE_RETURN_DISTANCE) as i32;
    } else {
        dist = (MIN_SABER_SLICE_DISTANCE) as i32;
    }

    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let soc = ctx.world.entity(saber_owner).client;
    if soc.is_null() {
        return;
    }

    if unsafe { (*soc).ps.saberAttackWound } > ctx.world.level.time {
        return;
    }

    while i < ctx.world.level.num_entities {
        let ent_id = EntityId(i as u32);
        CheckThrownSaberDamaged(
            ctx,
            saberent,
            Some(saber_owner),
            Some(ent_id),
            dist,
            returning,
            qfalse,
        );
        i += 1;
    }
}

/// Raven `saberMoveBack`.
///
/// Source: `oracle/codemp/game/w_saber.c:6165-6227`
pub fn saberMoveBack(ctx: &mut GameContext, ent: EntityId, goingBack: qboolean) {
    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(ent).s.pos.trType = trType_t::TR_LINEAR;

    let oldOrg = ctx.world.entity(ent).r.currentOrigin;
    let mut origin: vec3_t = [0.0; 3];
    // get current position
    BG_EvaluateTrajectory(&ctx.world.entity(ent).s.pos, level_time, &mut origin);
    // Get current angles?
    {
        let e = ctx.world.entity_mut(ent);
        BG_EvaluateTrajectory(&e.s.apos, level_time, &mut e.r.currentAngles);
    }

    // compensation test code — `THROWN_SABER_COMP` is `#define`d
    // unconditionally in the oracle, so this block is compiled in.
    if goingBack == qfalse && ctx.world.entity(ent).s.pos.trType != trType_t::TR_GRAVITY {
        // acts as a fallback in case touch code fails, keeps saber from
        // going through things between predictions
        let iCompensationLength = 32;
        let mut tr: trace_t = unsafe { core::mem::zeroed() };
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut calcComp: vec3_t = [0.0; 3];
        let mut compensatedOrigin: vec3_t = [0.0; 3];
        VectorSet(&mut mins, -24.0, -24.0, -8.0);
        VectorSet(&mut maxs, 24.0, 24.0, 8.0);

        _VectorSubtract(origin, oldOrg, &mut calcComp);
        let originalLength = VectorLength(calcComp);

        VectorNormalize(&mut calcComp);

        compensatedOrigin[0] =
            oldOrg[0] + calcComp[0] * (originalLength + iCompensationLength as f32);
        compensatedOrigin[1] =
            oldOrg[1] + calcComp[1] * (originalLength + iCompensationLength as f32);
        compensatedOrigin[2] =
            oldOrg[2] + calcComp[2] * (originalLength + iCompensationLength as f32);

        let ownerNum = ctx.world.entity(ent).r.ownerNum;
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &oldOrg as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &compensatedOrigin as *const vec3_t,
                ownerNum,
                MASK_PLAYERSOLID,
            ),
        );

        if (tr.fraction != 1.0 || tr.startsolid != 0 || tr.allsolid != 0)
            && tr.entityNum as i32 != ownerNum
            && (ctx.world.g_entities[tr.entityNum as usize].r.contents & CONTENTS_LIGHTSABER) == 0
        {
            VectorClear(&mut ctx.world.entity_mut(ent).s.pos.trDelta);

            // Unfortunately restoring `origin` would defeat the purpose of
            // the compensation; we settle for a jerk on the client.
            // we'll skip the dist check, since we just hit it physically
            CheckThrownSaberDamaged(
                ctx,
                ent,
                Some(EntityId(ownerNum as u32)),
                Some(EntityId((tr.entityNum) as u32)),
                256,
                0,
                qtrue,
            );

            if ctx.world.entity(ent).s.pos.trType == trType_t::TR_GRAVITY {
                // got blocked and knocked away in the damage func
                return;
            }

            tr.startsolid = 0;
            if tr.entityNum as i32 == ENTITYNUM_NONE {
                // it had to hit something, so we'll say it hit the world
                tr.entityNum = ENTITYNUM_WORLD as i16;
            }
            thrownSaberTouch(
                ctx,
                ent,
                Some(EntityId((tr.entityNum) as u32)),
                &mut tr as *mut trace_t,
            );
            return;
        }
    }

    ctx.world.entity_mut(ent).r.currentOrigin = origin;
}

/// Raven `SaberBounceSound`.
///
/// Source: `oracle/codemp/game/w_saber.c:6229-6233`
pub fn SaberBounceSound(self_: &mut gentity_t, other: Option<EntityId>, trace: *mut trace_t) {
    let _ = (other, trace);
    self_.s.apos.trBase = self_.r.currentAngles; // VectorCopy
    self_.s.apos.trBase[PITCH] = 90.0;
}

/// Raven `DeadSaberThink`.
///
/// Source: `oracle/codemp/game/w_saber.c:6235-6245`
pub fn DeadSaberThink(ctx: &mut GameContext, saberent: EntityId) {
    let now = ctx.world.level.time;
    if ctx.entity(saberent).speed < now as f32 {
        ctx.entity_mut(saberent).think = Some(EntThink::G_FreeEntity).into();
        ctx.entity_mut(saberent).nextthink = now;
        return;
    }

    G_RunObject(ctx, saberent);
}

/// Raven `MakeDeadSaber`.
///
/// Source: `oracle/codemp/game/w_saber.c:6247-6335`
// spawn a "dead" saber entity here so it looks like the saber fell out of the air.
// This entity removes itself after a very short time period.
pub fn MakeDeadSaber(ctx: &mut GameContext, ent: EntityId) {
    if ctx.world.cvars.g_gametype.integer == GT_JEDIMASTER {
        // never spawn a dead saber in JM, because the only saber on the level is
        // really a world object
        return;
    }

    let saberent = G_Spawn(ctx);

    let startorg: vec3_t = ctx.world.entity(ent).r.currentOrigin;
    let startang: vec3_t = ctx.world.entity(ent).r.currentAngles;
    let ent_number = ctx.world.entity(ent).s.number;
    let level_time = ctx.world.level.time;

    ctx.ent_set(saberent, PrefixSet::ClassnameStatic(c"deadsaber"));
    {
        let se = ctx.world.entity_mut(saberent);
        se.r.svFlags = SVF_USE_CURRENT_ORIGIN;
        se.r.ownerNum = ent_number;
        se.clipmask = MASK_PLAYERSOLID;
        se.r.contents = CONTENTS_TRIGGER;
        se.r.mins = [-3.0, -3.0, -1.5];
        se.r.maxs = [3.0, 3.0, 1.5];
        se.touch = Some(EntTouch::SaberBounceSound).into();
        se.think = Some(EntThink::DeadSaberThink).into();
        se.nextthink = level_time;
        se.s.pos.trBase = startorg;
        se.s.apos.trBase = startang;
        se.s.origin = startorg;
        se.s.angles = startang;
        se.r.currentOrigin = startorg;
        se.r.currentAngles = startang;
        se.s.apos.trType = trType_t::TR_GRAVITY;
    }

    let d0 = ctx.world.bg_state.rng.Q_irand(200, 800) as f32;
    let d1 = ctx.world.bg_state.rng.Q_irand(200, 800) as f32;
    let d2 = ctx.world.bg_state.rng.Q_irand(200, 800) as f32;

    {
        let se = ctx.world.entity_mut(saberent);
        se.s.apos.trDelta[0] = d0;
        se.s.apos.trDelta[1] = d1;
        se.s.apos.trDelta[2] = d2;
        se.s.apos.trTime = level_time - 50;
        se.s.pos.trType = trType_t::TR_GRAVITY;
        se.s.pos.trTime = level_time - 50;
        se.flags = FL_BOUNCE_HALF;
    }

    let owner_num = ctx.world.entity(ent).r.ownerNum;
    if owner_num >= 0 && owner_num < ENTITYNUM_WORLD {
        let owner_id = EntityId(owner_num as u32);
        // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
        let oc = ctx.world.entity(owner_id).client;
        let ok = ctx.world.entity(owner_id).inuse != 0
            && !oc.is_null()
            && unsafe { (*oc).saber[0].model[0] != 0 };
        if ok {
            let (model_ptr, skin) = unsafe { ((*oc).saber[0].model.as_ptr(), (*oc).saber[0].skin) };
            WP_SaberAddG2Model(ctx, saberent, model_ptr, skin);
        } else {
            // argh!!!!
            G_FreeEntity(ctx, Some(saberent));
            return;
        }
    }

    let trdelta = ctx.world.entity(ent).s.pos.trDelta;
    {
        let se = ctx.world.entity_mut(saberent);
        se.s.modelGhoul2 = 1;
        se.s.g2radius = 20;
        se.s.eType = ET_MISSILE as c_int;
        se.s.weapon = WP_SABER as c_int;
        se.speed = (level_time + 4000) as f32;
        se.bounceCount = 12;
        // fall off in the direction the real saber was headed
        se.s.pos.trDelta = trdelta;
    }

    saberMoveBack(ctx, saberent, qtrue);
    ctx.world.entity_mut(saberent).s.pos.trType = trType_t::TR_GRAVITY;

    let se_ptr = ctx.world.entity_mut(saberent) as *mut gentity_t;
    trap::LinkEntity(
        ctx.engine,
        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(se_ptr.cast()),
    );
}

/// Raven `DownedSaberThink`.
///
/// Source: `oracle/codemp/game/w_saber.c:6342-6480`
pub fn DownedSaberThink(ctx: &mut GameContext, saberent: EntityId) {
    let level_time = ctx.world.level.time;
    let mut notDisowned = qfalse;
    let mut pullBack = qfalse;

    ctx.world.entity_mut(saberent).nextthink = level_time;

    if ctx.world.entity(saberent).r.ownerNum == ENTITYNUM_NONE {
        MakeDeadSaber(ctx, saberent);

        let e = ctx.world.entity_mut(saberent);
        e.think = Some(EntThink::G_FreeEntity).into();
        e.nextthink = level_time;
        return;
    }

    let saberOwn_id = EntityId(ctx.world.entity(saberent).r.ownerNum as u32);
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let soc = ctx.world.entity(saberOwn_id).client;

    if ctx.world.entity(saberOwn_id).inuse == 0
        || soc.is_null()
        || unsafe { (*soc).sess.sessionTeam == TEAM_SPECTATOR }
        || unsafe { ((*soc).ps.pm_flags & PMF_FOLLOW) != 0 }
    {
        MakeDeadSaber(ctx, saberent);

        let e = ctx.world.entity_mut(saberent);
        e.think = Some(EntThink::G_FreeEntity).into();
        e.nextthink = level_time;
        return;
    }

    let saberEntityNum = unsafe { (*soc).ps.saberEntityNum };
    if saberEntityNum != 0 {
        if saberEntityNum == ctx.world.entity(saberent).s.number {
            // owner shouldn't have this set if we're thinking in here.
            notDisowned = qtrue;
        } else {
            // This should never happen, but just in case..
            debug_assert!(false, "ULTRA BAD THING");
            MakeDeadSaber(ctx, saberent);

            let e = ctx.world.entity_mut(saberent);
            e.think = Some(EntThink::G_FreeEntity).into();
            e.nextthink = level_time;
            return;
        }
    }

    if notDisowned != 0
        || ctx.world.entity(saberOwn_id).health < 1
        || unsafe { (*soc).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] } == 0
    {
        // He's dead, just go back to our normal saber status
        unsafe {
            (*soc).ps.saberEntityNum = (*soc).saberStoredIndex;
        }

        saberReactivate(ctx, saberent, saberOwn_id);

        if ctx.world.entity(saberOwn_id).health < 1 {
            unsafe {
                (*soc).ps.saberInFlight = qfalse;
            }
            MakeDeadSaber(ctx, saberent);
        }

        {
            let e = ctx.world.entity_mut(saberent);
            e.touch = Some(EntTouch::SaberGotHit).into();
            e.think = Some(EntThink::SaberUpdateSelf).into();
            e.genericValue5 = 0;
            e.nextthink = level_time;

            e.r.svFlags |= SVF_NOCLIENT;
            e.s.loopSound = 0;
            e.s.loopIsSoundset = qfalse;
        }

        if ctx.world.entity(saberOwn_id).health > 0 {
            // only set this if he's alive.
            unsafe {
                (*soc).ps.saberInFlight = qfalse;
            }
            WP_SaberRemoveG2Model(ctx, saberent);
        }
        unsafe {
            (*soc).ps.saberEntityState = 0;
            (*soc).ps.saberThrowDelay = level_time + 500;
            (*soc).ps.saberCanThrow = qfalse;
        }

        return;
    }

    unsafe {
        if (*soc).saberKnockedTime < level_time && ((*soc).pers.cmd.buttons & BUTTON_ATTACK) != 0 {
            // He wants us back
            pullBack = qtrue;
        } else if (level_time - (*soc).saberKnockedTime) > MAX_LEAVE_TIME {
            // Been sitting around for too long, go back no matter what he wants.
            pullBack = qtrue;
        }
    }

    if pullBack != 0 {
        // Get going back to the owner.
        unsafe {
            (*soc).ps.saberEntityNum = (*soc).saberStoredIndex;
        }

        saberReactivate(ctx, saberent, saberOwn_id);

        {
            let e = ctx.world.entity_mut(saberent);
            e.touch = Some(EntTouch::SaberGotHit).into();

            e.think = Some(EntThink::saberBackToOwner).into();
            e.speed = (0) as f32;
            e.genericValue5 = 0;
            e.nextthink = level_time;

            e.r.contents = CONTENTS_LIGHTSABER;
        }

        let sound = G_SoundIndex(ctx, "sound/weapons/force/pull.wav");
        G_Sound(
            ctx,
            Some(saberOwn_id),
            CHAN_BODY as c_int,
            sound,
        );
        let son0 = unsafe { (*soc).saber[0].soundOn };
        if son0 != 0 {
            G_Sound(ctx, Some(saberent), CHAN_BODY as c_int, son0);
        }
        let son1 = unsafe { (*soc).saber[1].soundOn };
        if son1 != 0 {
            G_Sound(ctx, Some(saberOwn_id), CHAN_BODY as c_int, son1);
        }

        return;
    }

    G_RunObject(ctx, saberent);
    ctx.world.entity_mut(saberent).nextthink = level_time;
}

/// Raven `saberReactivate`.
///
/// Source: `oracle/codemp/game/w_saber.c:6482-6508`
pub fn saberReactivate(ctx: &mut GameContext, saberent: EntityId, saberOwner: EntityId) {
    {
        let se = ctx.world.entity_mut(saberent);
        se.s.saberInFlight = qtrue;

        se.s.apos.trType = trType_t::TR_LINEAR;
        se.s.apos.trDelta[0] = 0.0;
        se.s.apos.trDelta[1] = 800.0;
        se.s.apos.trDelta[2] = 0.0;

        se.s.pos.trType = trType_t::TR_LINEAR;
        se.s.eType = ET_GENERAL as c_int;
        se.s.eFlags = 0;

        se.parent = Some(saberOwner);

        se.genericValue5 = 0;
    }

    SetSaberBoxSize(ctx, Some(saberent));

    {
        let se = ctx.world.entity_mut(saberent);
        se.touch = Some(EntTouch::thrownSaberTouch).into();
        se.s.weapon = WP_SABER as c_int;
    }

    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let soc = ctx.world.entity(saberOwner).client;
    unsafe {
        (*soc).ps.saberEntityState = 1;
    }

    let se_ptr = ctx.world.entity_mut(saberent) as *mut gentity_t;
    trap::LinkEntity(
        ctx.engine,
        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(se_ptr.cast()),
    );
}

/// Raven `saberKnockDown`.
///
/// Source: `oracle/codemp/game/w_saber.c:6512-6584`
pub fn saberKnockDown(
    ctx: &mut GameContext,
    saberent: EntityId,
    saberOwner: EntityId,
    other: EntityId,
) {
    let level_time = ctx.world.level.time;
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let soc = ctx.world.entity(saberOwner).client;

    unsafe {
        (*soc).ps.saberEntityNum = 0; // still stored in client->saberStoredIndex
        (*soc).saberKnockedTime = level_time + SABER_RETRIEVE_DELAY;
    }

    {
        let e = ctx.world.entity_mut(saberent);
        e.clipmask = MASK_SOLID;
        e.r.contents = CONTENTS_TRIGGER;

        e.r.mins = [-3.0, -3.0, -1.5];
        e.r.maxs = [3.0, 3.0, 1.5];

        e.s.apos.trType = trType_t::TR_GRAVITY;
    }

    let d0 = ctx.world.bg_state.rng.Q_irand(200, 800) as f32;
    let d1 = ctx.world.bg_state.rng.Q_irand(200, 800) as f32;
    let d2 = ctx.world.bg_state.rng.Q_irand(200, 800) as f32;

    {
        let e = ctx.world.entity_mut(saberent);
        e.s.apos.trDelta[0] = d0;
        e.s.apos.trDelta[1] = d1;
        e.s.apos.trDelta[2] = d2;
        e.s.apos.trTime = level_time - 50;

        e.s.pos.trType = trType_t::TR_GRAVITY;
        e.s.pos.trTime = level_time - 50;
        e.flags |= FL_BOUNCE_HALF;
    }

    let (model_ptr, skin) = unsafe { ((*soc).saber[0].model.as_ptr(), (*soc).saber[0].skin) };
    WP_SaberAddG2Model(ctx, saberent, model_ptr, skin);

    {
        let e = ctx.world.entity_mut(saberent);
        e.s.modelGhoul2 = 1;
        e.s.g2radius = 20;

        e.s.eType = ET_MISSILE as c_int;
        e.s.weapon = WP_SABER as c_int;

        e.speed = (level_time + 4000) as f32;

        e.bounceCount = -5;
    }

    saberMoveBack(ctx, saberent, qtrue);

    {
        let e = ctx.world.entity_mut(saberent);
        e.s.pos.trType = trType_t::TR_GRAVITY;

        e.s.loopSound = 0; // kill this in case it was spinning.
        e.s.loopIsSoundset = qfalse;

        e.r.svFlags &= !SVF_NOCLIENT;

        e.touch = Some(EntTouch::SaberBounceSound).into();
        e.think = Some(EntThink::DownedSaberThink).into();
        e.nextthink = level_time;
    }

    if saberOwner != other {
        // if someone knocked it out of the air and it wasn't turned off, go in
        // the direction they were facing.
        if ctx.world.entity(other).inuse != 0 && !ctx.world.entity(other).client.is_null() {
            let mut otherFwd: vec3_t = [0.0; 3];
            let deflectSpeed = 200.0f32;

            // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
            let ooc = ctx.world.entity(other).client;
            let viewangles = unsafe { (*ooc).ps.viewangles };
            AngleVectors(viewangles, Some(&mut otherFwd), None, None);

            let e = ctx.world.entity_mut(saberent);
            e.s.pos.trDelta[0] = otherFwd[0] * deflectSpeed;
            e.s.pos.trDelta[1] = otherFwd[1] * deflectSpeed;
            e.s.pos.trDelta[2] = otherFwd[2] * deflectSpeed;
        }
    }

    let se_ptr = ctx.world.entity_mut(saberent) as *mut gentity_t;
    trap::LinkEntity(
        ctx.engine,
        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(se_ptr.cast()),
    );

    unsafe {
        if (*soc).saber[0].soundOff != 0 {
            let s = (*soc).saber[0].soundOff;
            G_Sound(ctx, Some(saberent), CHAN_BODY as c_int, s);
        }

        if (*soc).saber[1].soundOff != 0 && (*soc).saber[1].model[0] != 0 {
            let s = (*soc).saber[1].soundOff;
            G_Sound(ctx, Some(saberOwner), CHAN_BODY as c_int, s);
        }
    }
}

/// Raven `WP_SaberRemoveG2Model`.
///
/// Source: `oracle/codemp/game/w_saber.c:6589-6595`
pub fn WP_SaberRemoveG2Model(ctx: &mut GameContext, saberent: EntityId) {
    if !ctx.entity(saberent).ghoul2.is_null() {
        // `ghoul2` is never dereferenced here — its address is handed to the
        // engine seam as a raw `*mut c_void`, so no `unsafe` deref is needed.
        let ghoul2 = &mut ctx.entity_mut(saberent).ghoul2 as *mut *mut c_void as *mut c_void;
        trap::G2API_RemoveGhoul2Models(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_REMOVEGHOUL2MODELS::GG2Removeghoul2ModelsArgs::new(ghoul2),
        );
    }
}

/// Raven `WP_SaberAddG2Model`.
///
/// Source: `oracle/codemp/game/w_saber.c:6597-6610`
pub fn WP_SaberAddG2Model(
    ctx: &mut GameContext,
    saberent: EntityId,
    saberModel: *const c_char,
    saberSkin: qhandle_t,
) {
    WP_SaberRemoveG2Model(ctx, saberent);
    let modelindex = if !saberModel.is_null() && unsafe { *saberModel != 0 } {
        G_ModelIndex(ctx, &(unsafe { cstr_to_str(saberModel) }))
    } else {
        G_ModelIndex(ctx, "models/weapons2/saber/saber_w.glm")
    };
    ctx.world.entity_mut(saberent).s.modelindex = modelindex;
    // FIXME(Raven): use customSkin?
    let model_name = if saberModel.is_null() {
        String::new()
    } else {
        unsafe { cstr_to_str(saberModel) }
    };
    let modelindex = ctx.entity(saberent).s.modelindex;
    // `ghoul2`'s address is handed to the engine seam as a raw pointer; it is
    // never dereferenced in module code, so no `unsafe` deref is needed.
    let ghoul2 = &mut ctx.entity_mut(saberent).ghoul2 as *mut *mut c_void;
    trap::G2API_InitGhoul2Model(
        ctx.engine,
        ghoul2,
        &model_name,
        modelindex,
        saberSkin,
        0,
        0,
        0,
    );
}

/// Raven `saberKnockOutOfHand`.
///
/// Source: `oracle/codemp/game/w_saber.c:6613-6678`
pub fn saberKnockOutOfHand(
    ctx: &mut GameContext,
    saberent: Option<EntityId>,
    saberOwner: Option<EntityId>,
    velocity: vec3_t,
) -> bool {
    let level_time = ctx.world.level.time;

    let (Some(saberent), Some(saberOwner)) = (saberent, saberOwner) else {
        return false;
    };
    if ctx.world.entity(saberent).inuse == 0
        || ctx.world.entity(saberOwner).inuse == 0
        || ctx.world.entity(saberOwner).client.is_null()
    {
        return false;
    }

    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let soc = ctx.world.entity(saberOwner).client;
    unsafe {
        if (*soc).ps.saberEntityNum == 0 {
            // already gone
            return false;
        }

        if (level_time - (*soc).lastSaberStorageTime) > 50 {
            // must have a reasonably updated saber base pos
            return false;
        }

        if (*soc).ps.saberLockTime > (level_time - 100) {
            return false;
        }
        if ((*soc).saber[0].saberFlags & SFL_NOT_DISARMABLE) != 0 {
            return false;
        }

        (*soc).ps.saberInFlight = qtrue;
        (*soc).ps.saberEntityState = 1;
    }

    {
        let e = ctx.world.entity_mut(saberent);
        e.s.saberInFlight = qfalse;

        e.s.pos.trType = trType_t::TR_LINEAR;
        e.s.eType = ET_GENERAL as c_int;
        e.s.eFlags = 0;
    }

    let (model_ptr, skin) = unsafe { ((*soc).saber[0].model.as_ptr(), (*soc).saber[0].skin) };
    WP_SaberAddG2Model(ctx, saberent, model_ptr, skin);

    let ownerNumber = ctx.world.entity(saberOwner).s.number;
    {
        let e = ctx.world.entity_mut(saberent);
        e.s.modelGhoul2 = 127;

        e.parent = Some(saberOwner);

        e.damage = SABER_THROWN_HIT_DAMAGE;
        e.methodOfDeath = MOD_SABER as c_int;
        e.splashMethodOfDeath = MOD_SABER as c_int;
        e.s.solid = 2;
        e.r.contents = CONTENTS_LIGHTSABER;

        e.genericValue5 = 0;

        e.r.mins = [-24.0, -24.0, -8.0];
        e.r.maxs = [24.0, 24.0, 8.0];

        e.s.genericenemyindex = ownerNumber + 1024;
        e.s.weapon = WP_SABER as c_int;

        e.genericValue5 = 0;
    }

    // use this as opposed to the right hand bolt, because I don't want to risk
    // reconstructing the skel again to get it here.
    let base = unsafe { (*soc).lastSaberBase_Always };
    G_SetOrigin(ctx.world.entity_mut(saberent), base);
    saberKnockDown(ctx, saberent, saberOwner, saberOwner);
    // override the velocity on the knocked away saber.
    ctx.world.entity_mut(saberent).s.pos.trDelta = velocity;

    true
}

/// Raven `saberCheckKnockdown_DuelLoss`.
///
/// Source: `oracle/codemp/game/w_saber.c:6681-6761`
pub fn saberCheckKnockdown_DuelLoss(
    ctx: &mut GameContext,
    saberent: Option<EntityId>,
    saberOwner: Option<EntityId>,
    other: Option<EntityId>,
) -> bool {
    let mut dif: vec3_t = [0.0; 3];
    let mut totalDistance = 1.0f32;
    let distScale = 6.5f32;
    let mut validMomentum = qtrue;
    let mut disarmChance = 1;

    // Raven `SABERINVALID` macro (`w_saber.c:6587`), expanded in full.
    let (Some(saberent), Some(saberOwner), Some(other)) = (saberent, saberOwner, other) else {
        return false;
    };
    if ctx.world.entity(saberent).inuse == 0
        || ctx.world.entity(saberOwner).inuse == 0
        || ctx.world.entity(other).inuse == 0
        || ctx.world.entity(saberOwner).client.is_null()
        || ctx.world.entity(other).client.is_null()
    {
        return false;
    }
    // FLAG: pool clients (NPC-capable); deref raw per recipe 2b.
    let soc = ctx.world.entity(saberOwner).client;
    let ooc = ctx.world.entity(other).client;
    unsafe {
        if (*soc).ps.saberEntityNum == 0 || (*soc).ps.saberLockTime > (ctx.world.level.time - 100) {
            return false;
        }

        let level_time = ctx.world.level.time;

        if (*ooc).olderIsValid == 0 || (level_time - (*ooc).lastSaberStorageTime) >= 200 {
            validMomentum = qfalse;
        }

        if validMomentum != 0 {
            // Get the difference
            _VectorSubtract((*ooc).lastSaberBase_Always, (*ooc).olderSaberBase, &mut dif);
            totalDistance = VectorNormalize(&mut dif);

            if totalDistance == 0.0 {
                // fine, try our own
                if (*soc).olderIsValid == 0 || (level_time - (*soc).lastSaberStorageTime) >= 200 {
                    validMomentum = qfalse;
                }
                if validMomentum != 0 {
                    _VectorSubtract((*soc).lastSaberBase_Always, (*soc).olderSaberBase, &mut dif);
                    totalDistance = VectorNormalize(&mut dif);
                }
            }

            if validMomentum != 0 {
                if totalDistance == 0.0 {
                    // try the difference between the two blades
                    _VectorSubtract(
                        (*soc).lastSaberBase_Always,
                        (*ooc).lastSaberBase_Always,
                        &mut dif,
                    );
                    totalDistance = VectorNormalize(&mut dif);
                }

                if totalDistance != 0.0 {
                    if totalDistance < 20.0 {
                        totalDistance = 20.0;
                    }
                    _VectorScale(dif, totalDistance * distScale, &mut dif);
                }
            }
        }

        (*soc).ps.saberMove = mp_bg::public::saber_move_name::LS_V1_BL;
        (*soc).ps.saberBlocked = saberBlockedType_t::BLOCKED_BOUNCE_MOVE as c_int;

        if !ctx.world.entity(other).client.is_null() {
            disarmChance += (*ooc).saber[0].disarmBonus;
            if (*ooc).saber[1].model[0] != 0 && (*ooc).ps.saberHolstered == 0 {
                // Raven no-op: `other->client->saber[1].disarmBonus;` (discarded read)
                let _ = (*ooc).saber[1].disarmBonus;
            }
        }
        if ctx.world.bg_state.rng.Q_irand(0, disarmChance) != 0 {
            saberKnockOutOfHand(ctx, Some(saberent), Some(saberOwner), dif)
        } else {
            false
        }
    }
}

/// Raven `saberCheckKnockdown_BrokenParry`.
///
/// Source: `oracle/codemp/game/w_saber.c:6765-6845`
pub fn saberCheckKnockdown_BrokenParry(
    ctx: &mut GameContext,
    saberent: Option<EntityId>,
    saberOwner: Option<EntityId>,
    other: Option<EntityId>,
) -> bool {
    let mut doKnock = qfalse;
    let mut disarmChance = 1;

    // Raven `SABERINVALID` macro (`w_saber.c:6587`), expanded in full.
    let (Some(saberent), Some(saberOwner), Some(other)) = (saberent, saberOwner, other) else {
        return false;
    };
    if ctx.world.entity(saberent).inuse == 0
        || ctx.world.entity(saberOwner).inuse == 0
        || ctx.world.entity(other).inuse == 0
        || ctx.world.entity(saberOwner).client.is_null()
        || ctx.world.entity(other).client.is_null()
    {
        return false;
    }
    // FLAG: pool clients (NPC-capable); deref raw per recipe 2b.
    let soc = ctx.world.entity(saberOwner).client;
    let ooc = ctx.world.entity(other).client;
    unsafe {
        if (*soc).ps.saberEntityNum == 0 || (*soc).ps.saberLockTime > (ctx.world.level.time - 100) {
            return false;
        }
    }

    let level_time = ctx.world.level.time;

    // Neither gets an advantage based on attack state.
    let myAttack = G_SaberAttackPower(ctx, Some(saberOwner), false);
    let otherAttack = G_SaberAttackPower(ctx, Some(other), false);

    unsafe {
        if (*ooc).olderIsValid == 0 || (level_time - (*ooc).lastSaberStorageTime) >= 200 {
            return false;
        }

        // only knock the saber out of the hand if they're in a stronger stance
        if otherAttack > myAttack + 1 && ctx.world.bg_state.rng.Q_irand(1, 10) <= 7 {
            doKnock = qtrue;
        } else if otherAttack > myAttack && ctx.world.bg_state.rng.Q_irand(1, 10) <= 3 {
            doKnock = qtrue;
        }

        if doKnock != 0 {
            let mut dif: vec3_t = [0.0; 3];
            let mut totalDistance;
            let distScale = 6.5f32;

            _VectorSubtract((*ooc).lastSaberBase_Always, (*ooc).olderSaberBase, &mut dif);
            totalDistance = VectorNormalize(&mut dif);

            if totalDistance == 0.0 {
                // fine, try our own
                if (*soc).olderIsValid == 0 || (level_time - (*soc).lastSaberStorageTime) >= 200 {
                    return false;
                }

                _VectorSubtract((*soc).lastSaberBase_Always, (*soc).olderSaberBase, &mut dif);
                totalDistance = VectorNormalize(&mut dif);
            }

            if totalDistance == 0.0 {
                // ...forget it then.
                return false;
            }

            if totalDistance < 20.0 {
                totalDistance = 20.0;
            }
            _VectorScale(dif, totalDistance * distScale, &mut dif);

            if !ctx.world.entity(other).client.is_null() {
                disarmChance += (*ooc).saber[0].disarmBonus;
                if (*ooc).saber[1].model[0] != 0 && (*ooc).ps.saberHolstered == 0 {
                    let _ = (*ooc).saber[1].disarmBonus;
                }
            }
            if ctx.world.bg_state.rng.Q_irand(0, disarmChance) != 0 {
                return saberKnockOutOfHand(ctx, Some(saberent), Some(saberOwner), dif);
            }
        }

        false
    }
}

/// Raven `saberCheckKnockdown_Smashed`.
///
/// Source: `oracle/codemp/game/w_saber.c:6852-6880`
pub fn saberCheckKnockdown_Smashed(
    ctx: &mut GameContext,
    saberent: Option<EntityId>,
    saberOwner: Option<EntityId>,
    other: Option<EntityId>,
    damage: c_int,
) -> bool {
    // Raven `SABERINVALID` macro (`w_saber.c:6587`), expanded in full.
    let (Some(saberent), Some(saberOwner), Some(other)) = (saberent, saberOwner, other) else {
        return false;
    };
    if ctx.world.entity(saberent).inuse == 0
        || ctx.world.entity(saberOwner).inuse == 0
        || ctx.world.entity(other).inuse == 0
        || ctx.world.entity(saberOwner).client.is_null()
        || ctx.world.entity(other).client.is_null()
    {
        return false;
    }
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let soc = ctx.world.entity(saberOwner).client;
    unsafe {
        if (*soc).ps.saberEntityNum == 0 || (*soc).ps.saberLockTime > (ctx.world.level.time - 100) {
            return false;
        }

        if (*soc).ps.saberInFlight == 0 {
            // can only do this if the saber is already actually in flight
            return false;
        }
    }

    if ctx.world.entity(other).inuse != 0
        && !ctx.world.entity(other).client.is_null()
        // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
        && unsafe { BG_InExtraDefenseSaberMove((*ctx.world.entity(other).client).ps.saberMove) != 0 }
    {
        // make sure the blow was strong enough
        saberKnockDown(ctx, saberent, saberOwner, other);
        return true;
    }

    if damage > 10 {
        // make sure the blow was strong enough
        saberKnockDown(ctx, saberent, saberOwner, other);
        return true;
    }

    false
}

/// Raven `saberCheckKnockdown_Thrown`.
///
/// Source: `oracle/codemp/game/w_saber.c:6884-6915`
pub fn saberCheckKnockdown_Thrown(
    ctx: &mut GameContext,
    saberent: Option<EntityId>,
    saberOwner: Option<EntityId>,
    other: Option<EntityId>,
) -> bool {
    let mut tossIt = qfalse;

    // Raven `SABERINVALID` macro (`w_saber.c:6587`), expanded in full.
    let (Some(saberent), Some(saberOwner), Some(other)) = (saberent, saberOwner, other) else {
        return false;
    };
    if ctx.world.entity(saberent).inuse == 0
        || ctx.world.entity(saberOwner).inuse == 0
        || ctx.world.entity(other).inuse == 0
        || ctx.world.entity(saberOwner).client.is_null()
        || ctx.world.entity(other).client.is_null()
    {
        return false;
    }
    // FLAG: pool clients (NPC-capable); deref raw per recipe 2b.
    let soc = ctx.world.entity(saberOwner).client;
    let ooc = ctx.world.entity(other).client;

    unsafe {
        if (*soc).ps.saberEntityNum == 0 || (*soc).ps.saberLockTime > (ctx.world.level.time - 100) {
            return false;
        }

        let defenLevel = (*ooc).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize];
        let throwLevel = (*soc).ps.fd.forcePowerLevel[FP_SABERTHROW as usize];

        if defenLevel > throwLevel {
            tossIt = qtrue;
        } else if defenLevel == throwLevel && ctx.world.bg_state.rng.Q_irand(1, 10) <= 4 {
            tossIt = qtrue;
        }
        // otherwise don't

        if tossIt != 0 {
            saberKnockDown(ctx, saberent, saberOwner, other);
            return true;
        }

        false
    }
}

/// Raven `saberBackToOwner`.
///
/// Source: `oracle/codemp/game/w_saber.c:6917-7076`
pub fn saberBackToOwner(ctx: &mut GameContext, saberent: EntityId) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let saberent: *mut gentity_t = ctx.entity_mut(saberent);
    unsafe {
        let level_time = ctx.world.level.time;
        let saberOwner =
            &mut ctx.world.g_entities[(*saberent).r.ownerNum as usize] as *mut gentity_t;
        let mut dir: vec3_t = [0.0; 3];
        let ownerLen;

        if (*saberent).r.ownerNum == ENTITYNUM_NONE {
            MakeDeadSaber(ctx, ctx.entity_id_of(saberent).unwrap());
            (*saberent).think = Some(EntThink::G_FreeEntity).into();
            (*saberent).nextthink = level_time;
            return;
        }

        if (*saberOwner).inuse == 0
            || (*saberOwner).client.is_null()
            || (*((*saberOwner).client)).sess.sessionTeam == TEAM_SPECTATOR
        {
            MakeDeadSaber(ctx, ctx.entity_id_of(saberent).unwrap());
            (*saberent).think = Some(EntThink::G_FreeEntity).into();
            (*saberent).nextthink = level_time;
            return;
        }

        let soc = (*saberOwner).client;

        if (*saberOwner).health < 1 || (*soc).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] == 0
        {
            // He's dead, just go back to our normal saber status
            (*saberent).touch = Some(EntTouch::SaberGotHit).into();
            (*saberent).think = Some(EntThink::SaberUpdateSelf).into();
            (*saberent).genericValue5 = 0;
            (*saberent).nextthink = level_time;

            if !(*saberOwner).client.is_null() && (*soc).saber[0].soundOff != 0 {
                G_Sound(
                    ctx,
                    ctx.entity_id_of(saberent),
                    CHAN_AUTO as c_int,
                    (*soc).saber[0].soundOff,
                );
            }
            MakeDeadSaber(ctx, ctx.entity_id_of(saberent).unwrap());

            (*saberent).r.svFlags |= SVF_NOCLIENT;
            (*saberent).r.contents = CONTENTS_LIGHTSABER;
            SetSaberBoxSize(ctx, ctx.entity_id_of(saberent));
            (*saberent).s.loopSound = 0;
            (*saberent).s.loopIsSoundset = qfalse;
            WP_SaberRemoveG2Model(ctx, ctx.entity_id_of(saberent).unwrap());

            (*soc).ps.saberInFlight = qfalse;
            (*soc).ps.saberEntityState = 0;
            (*soc).ps.saberThrowDelay = level_time + 500;
            (*soc).ps.saberCanThrow = qfalse;

            return;
        }

        // make sure this is set alright
        (*soc).ps.saberEntityNum = (*saberent).s.number;

        (*saberent).r.contents = CONTENTS_LIGHTSABER;

        _VectorSubtract((*saberent).pos1, (*saberent).r.currentOrigin, &mut dir);

        ownerLen = VectorLength(dir);

        if (*saberent).speed < (level_time) as f32 {
            let baseSpeed;

            VectorNormalize(&mut dir);

            saberMoveBack(ctx, ctx.entity_id_of(saberent).unwrap(), qtrue);
            (*saberent).s.pos.trBase = (*saberent).r.currentOrigin;

            if (*soc).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] >= FORCE_LEVEL_3 {
                // allow players with high saber throw rank to control return speed
                baseSpeed = 900.0f32;
                (*saberent).speed = (level_time) as f32;
            } else {
                baseSpeed = 700.0f32;
                (*saberent).speed = (level_time + 50) as f32;
            }

            // Gradually slow down as it approaches.
            if ownerLen < 64.0 {
                _VectorScale(dir, baseSpeed - 200.0, &mut (*saberent).s.pos.trDelta);
            } else if ownerLen < 128.0 {
                _VectorScale(dir, baseSpeed - 150.0, &mut (*saberent).s.pos.trDelta);
            } else if ownerLen < 256.0 {
                _VectorScale(dir, baseSpeed - 100.0, &mut (*saberent).s.pos.trDelta);
            } else {
                _VectorScale(dir, baseSpeed, &mut (*saberent).s.pos.trDelta);
            }

            (*saberent).s.pos.trTime = level_time;
        }

        // I don't really like the spin on the way back.
        if (*soc).ps.saberEntityNum == (*saberent).s.number {
            if ((*soc).saber[0].saberFlags & SFL_RETURN_DAMAGE) == 0
                || (*soc).ps.saberHolstered != 0
            {
                (*saberent).s.saberInFlight = qfalse;
            }
            (*saberent).s.loopSound = (*soc).saber[0].soundLoop;
            (*saberent).s.loopIsSoundset = qfalse;

            if ownerLen <= 32.0 {
                let sound = G_SoundIndex(ctx, "sound/weapons/saber/saber_catch.wav");
                G_Sound(
                    ctx,
                    ctx.entity_id_of(saberent),
                    CHAN_AUTO as c_int,
                    sound,
                );

                (*soc).ps.saberInFlight = qfalse;
                (*soc).ps.saberEntityState = 0;
                (*soc).ps.saberCanThrow = qfalse;
                (*soc).ps.saberThrowDelay = level_time + 300;

                (*saberent).touch = Some(EntTouch::SaberGotHit).into();

                (*saberent).think = Some(EntThink::SaberUpdateSelf).into();
                (*saberent).genericValue5 = 0;
                (*saberent).nextthink = level_time + 50;
                WP_SaberRemoveG2Model(ctx, ctx.entity_id_of(saberent).unwrap());

                return;
            }

            if (*saberent).s.saberInFlight == 0 {
                saberCheckRadiusDamage(ctx, ctx.entity_id_of(saberent).unwrap(), 1);
            } else {
                saberCheckRadiusDamage(ctx, ctx.entity_id_of(saberent).unwrap(), 2);
            }

            saberMoveBack(ctx, ctx.entity_id_of(saberent).unwrap(), qtrue);
        }

        (*saberent).nextthink = level_time;
    }
}

/// Raven `thrownSaberTouch`.
///
/// Source: `oracle/codemp/game/w_saber.c:7080-7113`
pub fn thrownSaberTouch(
    ctx: &mut GameContext,
    saberent: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    let _ = trace;
    let level_time = ctx.world.level.time;
    let mut hitEnt: Option<EntityId> = other;

    if let Some(other) = other {
        if ctx.world.entity(other).s.number == ctx.world.entity(saberent).r.ownerNum {
            return;
        }
    }
    {
        let e = ctx.world.entity_mut(saberent);
        e.s.pos.trDelta = [0.0; 3];
        e.s.pos.trTime = level_time;

        e.s.apos.trType = trType_t::TR_LINEAR;
        e.s.apos.trDelta[0] = 0.0;
        e.s.apos.trDelta[1] = 800.0;
        e.s.apos.trDelta[2] = 0.0;

        e.s.pos.trBase = e.r.currentOrigin;

        e.think = Some(EntThink::saberBackToOwner).into();
        e.nextthink = level_time;
    }

    if let Some(other) = other {
        let ownerNum = ctx.world.entity(other).r.ownerNum;
        if ownerNum < (MAX_CLIENTS) as i32
            && (ctx.world.entity(other).r.contents & CONTENTS_LIGHTSABER) != 0
            && !ctx.world.g_entities[ownerNum as usize].client.is_null()
            && ctx.world.g_entities[ownerNum as usize].inuse != 0
        {
            hitEnt = Some(EntityId(ownerNum as u32));
        }
    }

    // we'll skip the dist check, since we don't really care about that
    let saberOwnerNum = ctx.world.entity(saberent).r.ownerNum;
    CheckThrownSaberDamaged(
        ctx,
        saberent,
        Some(EntityId(saberOwnerNum as u32)),
        hitEnt,
        256,
        0,
        qtrue,
    );

    ctx.world.entity_mut(saberent).speed = (0) as f32;
}

/// Raven `saberFirstThrown`.
///
/// Source: `oracle/codemp/game/w_saber.c:7117-7257`
pub fn saberFirstThrown(ctx: &mut GameContext, saberent: EntityId) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let saberent: *mut gentity_t = ctx.entity_mut(saberent);
    unsafe {
        let level_time = ctx.world.level.time;
        let saberOwn = &mut ctx.world.g_entities[(*saberent).r.ownerNum as usize] as *mut gentity_t;

        if (*saberent).r.ownerNum == ENTITYNUM_NONE {
            MakeDeadSaber(ctx, ctx.entity_id_of(saberent).unwrap());
            (*saberent).think = Some(EntThink::G_FreeEntity).into();
            (*saberent).nextthink = level_time;
            return;
        }

        if (*saberOwn).inuse == 0
            || (*saberOwn).client.is_null()
            || (*((*saberOwn).client)).sess.sessionTeam == TEAM_SPECTATOR
        {
            MakeDeadSaber(ctx, ctx.entity_id_of(saberent).unwrap());
            (*saberent).think = Some(EntThink::G_FreeEntity).into();
            (*saberent).nextthink = level_time;
            return;
        }

        let soc = (*saberOwn).client;

        if (*saberOwn).health < 1 || (*soc).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] == 0 {
            // He's dead, just go back to our normal saber status
            (*saberent).touch = Some(EntTouch::SaberGotHit).into();
            (*saberent).think = Some(EntThink::SaberUpdateSelf).into();
            (*saberent).genericValue5 = 0;
            (*saberent).nextthink = level_time;

            if !(*saberOwn).client.is_null() && (*soc).saber[0].soundOff != 0 {
                G_Sound(
                    ctx,
                    ctx.entity_id_of(saberent),
                    CHAN_AUTO as c_int,
                    (*soc).saber[0].soundOff,
                );
            }
            MakeDeadSaber(ctx, ctx.entity_id_of(saberent).unwrap());

            (*saberent).r.svFlags |= SVF_NOCLIENT;
            (*saberent).r.contents = CONTENTS_LIGHTSABER;
            SetSaberBoxSize(ctx, ctx.entity_id_of(saberent));
            (*saberent).s.loopSound = 0;
            (*saberent).s.loopIsSoundset = qfalse;
            WP_SaberRemoveG2Model(ctx, ctx.entity_id_of(saberent).unwrap());

            (*soc).ps.saberInFlight = qfalse;
            (*soc).ps.saberEntityState = 0;
            (*soc).ps.saberThrowDelay = level_time + 500;
            (*soc).ps.saberCanThrow = qfalse;

            return;
        }

        // labeled block emulating the C `goto runMin;`
        'body: {
            if (level_time - (*soc).ps.saberDidThrowTime) > 500 {
                if ((*soc).buttons & BUTTON_ALT_ATTACK) == 0 {
                    // If owner releases altattack 500ms+ after throwing, it autoreturns
                    thrownSaberTouch(
                        ctx,
                        ctx.entity_id_of(saberent).unwrap(),
                        ctx.entity_id_of(saberent),
                        core::ptr::null_mut(),
                    );
                    break 'body;
                } else if (level_time - (*soc).ps.saberDidThrowTime) > 6000 {
                    // if it's out longer than 6 seconds, return it
                    thrownSaberTouch(
                        ctx,
                        ctx.entity_id_of(saberent).unwrap(),
                        ctx.entity_id_of(saberent),
                        core::ptr::null_mut(),
                    );
                    break 'body;
                }
            }

            if BG_HasYsalamiri(ctx.world.cvars.g_gametype.integer, &mut (*soc).ps) != 0 {
                thrownSaberTouch(
                    ctx,
                    ctx.entity_id_of(saberent).unwrap(),
                    ctx.entity_id_of(saberent),
                    core::ptr::null_mut(),
                );
                break 'body;
            }

            if BG_CanUseFPNow(
                ctx.world.cvars.g_gametype.integer,
                &mut (*soc).ps,
                level_time,
                FP_SABERTHROW,
            ) == 0
            {
                thrownSaberTouch(
                    ctx,
                    ctx.entity_id_of(saberent).unwrap(),
                    ctx.entity_id_of(saberent),
                    core::ptr::null_mut(),
                );
                break 'body;
            }

            let mut vSub: vec3_t = [0.0; 3];
            _VectorSubtract((*soc).ps.origin, (*saberent).r.currentOrigin, &mut vSub);
            let vLen = VectorLength(vSub);

            if vLen
                >= (SABER_MAX_THROW_DISTANCE
                    * (*soc).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] as f32)
            {
                thrownSaberTouch(
                    ctx,
                    ctx.entity_id_of(saberent).unwrap(),
                    ctx.entity_id_of(saberent),
                    core::ptr::null_mut(),
                );
                break 'body;
            }

            if (*soc).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] >= FORCE_LEVEL_2
                && (*saberent).speed < (level_time) as f32
            {
                // if owner is rank 3 in saber throwing, the saber goes where he points
                let mut fwd: vec3_t = [0.0; 3];
                let mut traceFrom: vec3_t = [0.0; 3];
                let mut traceTo: vec3_t = [0.0; 3];
                let mut dir: vec3_t = [0.0; 3];
                let mut tr: trace_t = core::mem::zeroed();

                AngleVectors((*soc).ps.viewangles, Some(&mut fwd), None, None);

                traceFrom = (*soc).ps.origin;
                traceFrom[2] += (*soc).ps.viewheight as f32;

                traceTo = traceFrom;
                traceTo[0] += fwd[0] * 4096.0;
                traceTo[1] += fwd[1] * 4096.0;
                traceTo[2] += fwd[2] * 4096.0;

                saberMoveBack(ctx, ctx.entity_id_of(saberent).unwrap(), qfalse);
                (*saberent).s.pos.trBase = (*saberent).r.currentOrigin;

                let mask = if (*soc).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] >= FORCE_LEVEL_3
                {
                    MASK_PLAYERSOLID
                } else {
                    MASK_SOLID
                };
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &traceFrom as *const vec3_t,
                        core::ptr::null(),
                        core::ptr::null(),
                        &traceTo as *const vec3_t,
                        (*saberOwn).s.number,
                        mask,
                    ),
                );

                _VectorSubtract(tr.endpos, (*saberent).r.currentOrigin, &mut dir);
                VectorNormalize(&mut dir);
                _VectorScale(dir, 500.0, &mut (*saberent).s.pos.trDelta);
                (*saberent).s.pos.trTime = level_time;

                if (*soc).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] >= FORCE_LEVEL_3 {
                    (*saberent).speed = (level_time + 100) as f32;
                } else {
                    (*saberent).speed = (level_time + 400) as f32;
                }
            }
        }

        // runMin:
        saberCheckRadiusDamage(ctx, ctx.entity_id_of(saberent).unwrap(), 0);
        G_RunObject(ctx, ctx.entity_id_of(saberent).unwrap());
    }
}

/// Raven `UpdateClientRenderBolts`.
///
/// Source: `oracle/codemp/game/w_saber.c:7259-7320`
pub fn UpdateClientRenderBolts(
    ctx: &mut GameContext,
    self_: EntityId,
    renderOrigin: vec3_t,
    renderAngles: vec3_t,
) {
    let level_time = ctx.world.level.time;
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let sc = ctx.world.entity(self_).client;
    // `ghoul2`/`modelScale` are stable across the bolt reads below (the seam call
    // does not mutate them), so hoist the entity reads out of the loop.
    let ghoul2 = ctx.world.entity(self_).ghoul2;
    let modelScale = ctx.world.entity(self_).modelScale;
    unsafe {
        let ri = &mut (*sc).renderInfo as *mut renderInfo_t;

        if ghoul2.is_null() {
            (*ri).headPoint = (*sc).ps.origin;
            (*ri).handRPoint = (*sc).ps.origin;
            (*ri).handLPoint = (*sc).ps.origin;
            (*ri).torsoPoint = (*sc).ps.origin;
            (*ri).crotchPoint = (*sc).ps.origin;
            (*ri).footRPoint = (*sc).ps.origin;
            (*ri).footLPoint = (*sc).ps.origin;
        } else {
            let mut get = |bolt: c_int, out: &mut vec3_t| {
                let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
                trap::G2API_GetBoltMatrix(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                        ghoul2,
                        0,
                        bolt,
                        &mut boltMatrix as *mut mdxaBone_t,
                        &renderAngles as *const vec3_t,
                        &renderOrigin as *const vec3_t,
                        level_time,
                        core::ptr::null_mut(),
                        &modelScale as *const vec3_t,
                    ),
                );
                out[0] = boltMatrix.matrix[0][3];
                out[1] = boltMatrix.matrix[1][3];
                out[2] = boltMatrix.matrix[2][3];
            };

            get((*ri).headBolt, &mut (*ri).headPoint);
            get((*ri).handRBolt, &mut (*ri).handRPoint);
            get((*ri).handLBolt, &mut (*ri).handLPoint);
            get((*ri).torsoBolt, &mut (*ri).torsoPoint);
            get((*ri).crotchBolt, &mut (*ri).crotchPoint);
            get((*ri).footRBolt, &mut (*ri).footRPoint);
            get((*ri).footLBolt, &mut (*ri).footLPoint);
        }

        (*sc).renderInfo.boltValidityTime = level_time;
    }
}

/// Raven `UpdateClientRenderinfo`.
///
/// Source: `oracle/codemp/game/w_saber.c:7322-7468`
pub fn UpdateClientRenderinfo(
    ctx: &mut GameContext,
    self_: EntityId,
    renderOrigin: vec3_t,
    renderAngles: vec3_t,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        let client = (*self_).client;
        let ri = core::ptr::addr_of_mut!((*client).renderInfo);
        if (*ri).mPCalcTime < ctx.world.level.time {
            //We're just going to give rough estimates on most of this stuff,
            //it's not like most of it matters.
            // (#if 0 block that zeroed everything each frame is omitted, as in Raven.)

            if !(*self_).ghoul2.is_null() && (*self_).ghoul2 != (*ri).lastG2 {
                //the g2 instance changed, so update all the bolts.
                //rwwFIXMEFIXME: Base on skeleton used? Assuming humanoid currently.
                (*ri).lastG2 = (*self_).ghoul2;

                if (*self_).localAnimIndex <= 1 {
                    (*ri).headBolt =
                        trap::G2API_AddBolt(ctx.engine, (*self_).ghoul2, 0, "*head_eyes");
                    (*ri).handRBolt =
                        trap::G2API_AddBolt(ctx.engine, (*self_).ghoul2, 0, "*r_hand");
                    (*ri).handLBolt =
                        trap::G2API_AddBolt(ctx.engine, (*self_).ghoul2, 0, "*l_hand");
                    (*ri).torsoBolt =
                        trap::G2API_AddBolt(ctx.engine, (*self_).ghoul2, 0, "thoracic");
                    (*ri).crotchBolt =
                        trap::G2API_AddBolt(ctx.engine, (*self_).ghoul2, 0, "pelvis");
                    (*ri).footRBolt =
                        trap::G2API_AddBolt(ctx.engine, (*self_).ghoul2, 0, "*r_leg_foot");
                    (*ri).footLBolt =
                        trap::G2API_AddBolt(ctx.engine, (*self_).ghoul2, 0, "*l_leg_foot");
                    (*ri).motionBolt = trap::G2API_AddBolt(ctx.engine, (*self_).ghoul2, 0, "Motion");
                } else {
                    (*ri).headBolt = -1;
                    (*ri).handRBolt = -1;
                    (*ri).handLBolt = -1;
                    (*ri).torsoBolt = -1;
                    (*ri).crotchBolt = -1;
                    (*ri).footRBolt = -1;
                    (*ri).footLBolt = -1;
                    (*ri).motionBolt = -1;
                }

                (*ri).lastG2 = (*self_).ghoul2;
            }

            (*ri).eyeAngles = (*client).ps.viewangles;

            //we'll just say the legs/torso are whatever the first frame of our current anim is.
            (*ri).torsoFrame = (*(&ctx.world.bg_state.bgAllAnims)[(*self_).localAnimIndex as usize]
                .anims
                .add((*client).ps.torsoAnim as usize))
            .firstFrame as c_int;
            (*ri).legsFrame = (*(&ctx.world.bg_state.bgAllAnims)[(*self_).localAnimIndex as usize]
                .anims
                .add((*client).ps.legsAnim as usize))
            .firstFrame as c_int;
            if ctx.world.cvars.g_debugServerSkel.integer != 0 {
                //Alright, I was doing this, but it's just too slow to do every frame.
                let mut boltMatrix: mdxaBone_t = core::mem::zeroed();

                if (*self_).ghoul2.is_null() {
                    (*ri).headPoint = (*client).ps.origin;
                    (*ri).handRPoint = (*client).ps.origin;
                    (*ri).handLPoint = (*client).ps.origin;
                    (*ri).torsoPoint = (*client).ps.origin;
                    (*ri).crotchPoint = (*client).ps.origin;
                    (*ri).footRPoint = (*client).ps.origin;
                    (*ri).footLPoint = (*client).ps.origin;
                } else {
                    //head
                    trap::G2API_GetBoltMatrix(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                            (*self_).ghoul2,
                            0,
                            (*ri).headBolt,
                            &mut boltMatrix as *mut mdxaBone_t,
                            &renderAngles as *const vec3_t,
                            &renderOrigin as *const vec3_t,
                            ctx.world.level.time,
                            core::ptr::null_mut(),
                            &(*self_).modelScale as *const vec3_t,
                        ),
                    );
                    (*ri).headPoint[0] = boltMatrix.matrix[0][3];
                    (*ri).headPoint[1] = boltMatrix.matrix[1][3];
                    (*ri).headPoint[2] = boltMatrix.matrix[2][3];

                    //right hand
                    trap::G2API_GetBoltMatrix(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                            (*self_).ghoul2,
                            0,
                            (*ri).handRBolt,
                            &mut boltMatrix as *mut mdxaBone_t,
                            &renderAngles as *const vec3_t,
                            &renderOrigin as *const vec3_t,
                            ctx.world.level.time,
                            core::ptr::null_mut(),
                            &(*self_).modelScale as *const vec3_t,
                        ),
                    );
                    (*ri).handRPoint[0] = boltMatrix.matrix[0][3];
                    (*ri).handRPoint[1] = boltMatrix.matrix[1][3];
                    (*ri).handRPoint[2] = boltMatrix.matrix[2][3];

                    //left hand
                    trap::G2API_GetBoltMatrix(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                            (*self_).ghoul2,
                            0,
                            (*ri).handLBolt,
                            &mut boltMatrix as *mut mdxaBone_t,
                            &renderAngles as *const vec3_t,
                            &renderOrigin as *const vec3_t,
                            ctx.world.level.time,
                            core::ptr::null_mut(),
                            &(*self_).modelScale as *const vec3_t,
                        ),
                    );
                    (*ri).handLPoint[0] = boltMatrix.matrix[0][3];
                    (*ri).handLPoint[1] = boltMatrix.matrix[1][3];
                    (*ri).handLPoint[2] = boltMatrix.matrix[2][3];

                    //chest
                    trap::G2API_GetBoltMatrix(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                            (*self_).ghoul2,
                            0,
                            (*ri).torsoBolt,
                            &mut boltMatrix as *mut mdxaBone_t,
                            &renderAngles as *const vec3_t,
                            &renderOrigin as *const vec3_t,
                            ctx.world.level.time,
                            core::ptr::null_mut(),
                            &(*self_).modelScale as *const vec3_t,
                        ),
                    );
                    (*ri).torsoPoint[0] = boltMatrix.matrix[0][3];
                    (*ri).torsoPoint[1] = boltMatrix.matrix[1][3];
                    (*ri).torsoPoint[2] = boltMatrix.matrix[2][3];

                    //crotch
                    trap::G2API_GetBoltMatrix(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                            (*self_).ghoul2,
                            0,
                            (*ri).crotchBolt,
                            &mut boltMatrix as *mut mdxaBone_t,
                            &renderAngles as *const vec3_t,
                            &renderOrigin as *const vec3_t,
                            ctx.world.level.time,
                            core::ptr::null_mut(),
                            &(*self_).modelScale as *const vec3_t,
                        ),
                    );
                    (*ri).crotchPoint[0] = boltMatrix.matrix[0][3];
                    (*ri).crotchPoint[1] = boltMatrix.matrix[1][3];
                    (*ri).crotchPoint[2] = boltMatrix.matrix[2][3];

                    //right foot
                    trap::G2API_GetBoltMatrix(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                            (*self_).ghoul2,
                            0,
                            (*ri).footRBolt,
                            &mut boltMatrix as *mut mdxaBone_t,
                            &renderAngles as *const vec3_t,
                            &renderOrigin as *const vec3_t,
                            ctx.world.level.time,
                            core::ptr::null_mut(),
                            &(*self_).modelScale as *const vec3_t,
                        ),
                    );
                    (*ri).footRPoint[0] = boltMatrix.matrix[0][3];
                    (*ri).footRPoint[1] = boltMatrix.matrix[1][3];
                    (*ri).footRPoint[2] = boltMatrix.matrix[2][3];

                    //left foot
                    trap::G2API_GetBoltMatrix(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                            (*self_).ghoul2,
                            0,
                            (*ri).footLBolt,
                            &mut boltMatrix as *mut mdxaBone_t,
                            &renderAngles as *const vec3_t,
                            &renderOrigin as *const vec3_t,
                            ctx.world.level.time,
                            core::ptr::null_mut(),
                            &(*self_).modelScale as *const vec3_t,
                        ),
                    );
                    (*ri).footLPoint[0] = boltMatrix.matrix[0][3];
                    (*ri).footLPoint[1] = boltMatrix.matrix[1][3];
                    (*ri).footLPoint[2] = boltMatrix.matrix[2][3];
                }

                //Now draw the skel for debug
                G_TestLine(ctx, (*ri).headPoint, (*ri).torsoPoint, 0x000000ff, 50);
                G_TestLine(ctx, (*ri).torsoPoint, (*ri).handRPoint, 0x000000ff, 50);
                G_TestLine(ctx, (*ri).torsoPoint, (*ri).handLPoint, 0x000000ff, 50);
                G_TestLine(ctx, (*ri).torsoPoint, (*ri).crotchPoint, 0x000000ff, 50);
                G_TestLine(ctx, (*ri).crotchPoint, (*ri).footRPoint, 0x000000ff, 50);
                G_TestLine(ctx, (*ri).crotchPoint, (*ri).footLPoint, 0x000000ff, 50);
            }

            //muzzle point calc (we are going to be cheap here)
            (*ri).muzzlePointOld = (*ri).muzzlePoint;
            (*ri).muzzlePoint = (*client).ps.origin;
            (*ri).muzzleDirOld = (*ri).muzzleDir;
            AngleVectors(
                (*client).ps.viewangles,
                Some(&mut (*ri).muzzleDir),
                None,
                None,
            );
            (*ri).mPCalcTime = ctx.world.level.time;

            (*ri).eyePoint = (*client).ps.origin;
            (*ri).eyePoint[2] += (*client).ps.viewheight as f32;
        }
    }
}

/// Raven `G_KickDownable`.
///
/// Source: `oracle/codemp/game/w_saber.c:7474-7500`
pub fn G_KickDownable(ctx: &mut GameContext, ent: Option<EntityId>) -> bool {
    if ctx.world.cvars.d_saberKickTweak.integer == 0 {
        return true;
    }

    let Some(ent) = ent else {
        return false;
    };
    if ctx.world.entity(ent).inuse == 0 || ctx.world.entity(ent).client.is_null() {
        return false;
    }
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let client = ctx.world.entity(ent).client;

    unsafe {
        if BG_InKnockDown((*client).ps.legsAnim) != 0 || BG_InKnockDown((*client).ps.torsoAnim) != 0
        {
            return false;
        }

        if (*client).ps.weaponTime <= 0
            && (*client).ps.weapon == WP_SABER as c_int
            && (*client).ps.groundEntityNum != ENTITYNUM_NONE
        {
            return false;
        }
    }

    true
}

/// Raven `G_TossTheMofo`.
///
/// Source: `oracle/codemp/game/w_saber.c:7502-7525`
// `tossDir` is read-only here (`VectorMA` input only, never written),
// so it stays by-value.
pub fn G_TossTheMofo(ctx: &mut GameContext, ent: EntityId, tossDir: vec3_t, tossStr: f32) {
    if ctx.world.entity(ent).inuse == 0 || ctx.world.entity(ent).client.is_null() {
        // no good
        return;
    }

    if ctx.world.entity(ent).s.eType == ET_NPC as c_int
        && ctx.world.entity(ent).s.NPC_class == CLASS_VEHICLE as c_int
    {
        // no, silly
        return;
    }
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let client = ctx.world.entity(ent).client;

    unsafe {
        // VectorMA(velocity, tossStr, tossDir, velocity)
        let v = (*client).ps.velocity;
        (*client).ps.velocity = [
            v[0] + tossStr * tossDir[0],
            v[1] + tossStr * tossDir[1],
            v[2] + tossStr * tossDir[2],
        ];
        (*client).ps.velocity[2] = 200.0;
        let health = ctx.world.entity(ent).health;
        if health > 0
            && (*client).ps.forceHandExtend != HANDEXTEND_KNOCKDOWN as c_int
            && BG_KnockDownable(&mut (*client).ps) != 0
            && G_KickDownable(ctx, Some(ent))
        {
            // if they are alive, knock them down I suppose
            (*client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN as c_int;
            (*client).ps.forceHandExtendTime = ctx.world.level.time + 700;
            (*client).ps.forceDodgeAnim = 0; // toggles 1/0; 1 means play the get-up anim
        }
    }
}

/// Raven `STAFF_KICK_RANGE`.
///
/// File-local `#define` in `w_saber.c` (not a header const), named at its call
/// site like the other file-local defines above.
/// Source: `oracle/codemp/game/w_saber.c:7470`
const STAFF_KICK_RANGE: c_int = 16;

/// Raven `G_KickTrace`.
///
/// Source: `oracle/codemp/game/w_saber.c:7527-7642`
pub fn G_KickTrace(
    ctx: &mut GameContext,
    ent: EntityId,
    kickDir: vec3_t,
    kickDist: f32,
    kickEnd: vec3_t,
    kickDamage: c_int,
    kickPush: f32,
) -> *mut gentity_t {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    unsafe {
        // `kickDir` is passed to `G_Damage` as an out-shaped `&mut vec3_t`; rebind
        // mutable (binding-only; the LAW-by-value type is unchanged).
        let mut kickDir = kickDir;

        let mut traceOrg: vec3_t = [0.0; 3];
        let mut traceEnd: vec3_t = [0.0; 3];
        let mut trace: trace_t = core::mem::zeroed();
        let mut hitEnt: *mut gentity_t = core::ptr::null_mut();

        // VectorSet(kickMins, -2, -2, -2); VectorSet(kickMaxs, 2, 2, 2);
        let kickMins: vec3_t = [-2.0f32, -2.0f32, -2.0f32];
        let kickMaxs: vec3_t = [2.0f32, 2.0f32, 2.0f32];

        //FIXME: variable kick height?
        // Raven null-checks the `kickEnd` array param; by-value `vec3_t` can't be
        // NULL, so callers pass `vec3_origin` for Raven NULL (same branch taken).
        if !VectorCompare(kickEnd, vec3_origin) {
            //they passed us the end point of the trace, just use that
            //this makes the trace flat
            traceOrg = [
                (*ent).r.currentOrigin[0],
                (*ent).r.currentOrigin[1],
                kickEnd[2],
            ];
            traceEnd = kickEnd;
        } else {
            //extrude
            traceOrg = [
                (*ent).r.currentOrigin[0],
                (*ent).r.currentOrigin[1],
                (*ent).r.currentOrigin[2] + (*ent).r.maxs[2] * 0.5f32,
            ];
            _VectorMA(traceOrg, kickDist, kickDir, &mut traceEnd);
        }

        if ctx.world.cvars.d_saberKickTweak.integer != 0 {
            trap::G2Trace(
                ctx.engine,
                GG2TraceArgs::new(
                    &mut trace as *mut trace_t,
                    &traceOrg as *const vec3_t,
                    &kickMins as *const vec3_t,
                    &kickMaxs as *const vec3_t,
                    &traceEnd as *const vec3_t,
                    (*ent).s.number,
                    MASK_SHOT,
                    G2TRFLAG_DOGHOULTRACE
                        | G2TRFLAG_GETSURFINDEX
                        | G2TRFLAG_THICK
                        | G2TRFLAG_HITCORPSES,
                    ctx.world.cvars.g_g2TraceLod.integer,
                ),
            );
        } else {
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut trace as *mut trace_t,
                    &traceOrg as *const vec3_t,
                    &kickMins as *const vec3_t,
                    &kickMaxs as *const vec3_t,
                    &traceEnd as *const vec3_t,
                    (*ent).s.number,
                    MASK_SHOT,
                ),
            );
        }

        //G_TestLine(traceOrg, traceEnd, 0x0000ff, 5000);
        if trace.fraction < 1.0f32 && trace.startsolid == 0 && trace.allsolid == 0 {
            let client = (*ent).client;
            if (*client).jediKickTime > ctx.world.level.time {
                if trace.entityNum as c_int == (*client).jediKickIndex {
                    //we are hitting the same ent we last hit in this same anim, don't hit it again
                    return core::ptr::null_mut();
                }
            }
            (*client).jediKickIndex = trace.entityNum as c_int;
            (*client).jediKickTime = ctx.world.level.time + (*client).ps.legsTimer;

            hitEnt = &mut ctx.world.g_entities[trace.entityNum as usize] as *mut gentity_t;
            //FIXME: regardless of what we hit, do kick hit sound and impact effect
            //G_PlayEffect( "misc/kickHit", trace.endpos, trace.plane.normal );
            if (*client).ps.torsoAnim == BOTH_A7_HILT as c_int {
                let idx = G_SoundIndex(ctx, "sound/movers/objects/saber_slam");
                G_Sound(ctx, ctx.entity_id_of(ent), CHAN_AUTO, idx);
            } else {
                let s = format!(
                    "sound/weapons/melee/punch{}",
                    ctx.world.bg_state.rng.Q_irand(1, 4)
                );
                let idx = G_SoundIndex(ctx, &s);
                G_Sound(ctx, ctx.entity_id_of(ent), CHAN_AUTO, idx);
            }
            if (*hitEnt).inuse != 0 {
                //we hit an entity
                //FIXME: don't hit same ent more than once per kick
                if (*hitEnt).takedamage != 0 {
                    //hurt it
                    if !(*hitEnt).client.is_null() {
                        let hitClient = (*hitEnt).client;
                        (*hitClient).ps.otherKiller = (*ent).s.number;
                        (*hitClient).ps.otherKillerDebounceTime = ctx.world.level.time + 10000;
                        (*hitClient).ps.otherKillerTime = ctx.world.level.time + 10000;
                        (*hitClient).otherKillerMOD = MOD_MELEE as c_int;
                        (*hitClient).otherKillerVehWeapon = 0;
                        (*hitClient).otherKillerWeaponType = WP_NONE as c_int;
                    }

                    if ctx.world.cvars.d_saberKickTweak.integer != 0 {
                        G_Damage(
                            ctx,
                            ctx.entity_id_of(hitEnt),
                            ctx.entity_id_of(ent),
                            ctx.entity_id_of(ent),
                            Some(&mut kickDir),
                            trace.endpos,
                            (kickDamage as f32 * 0.2f32) as c_int,
                            DAMAGE_NO_KNOCKBACK,
                            MOD_MELEE as c_int,
                        );
                    } else {
                        G_Damage(
                            ctx,
                            ctx.entity_id_of(hitEnt),
                            ctx.entity_id_of(ent),
                            ctx.entity_id_of(ent),
                            Some(&mut kickDir),
                            trace.endpos,
                            kickDamage,
                            DAMAGE_NO_KNOCKBACK,
                            MOD_MELEE as c_int,
                        );
                    }
                }
                if !(*hitEnt).client.is_null()
                    && ((*((*hitEnt).client)).ps.pm_flags & PMF_TIME_KNOCKBACK) == 0 //not already flying through air?  Intended to stop multiple hits, but...
                    && G_CanBeEnemy(ctx, ctx.entity_id_of(ent).unwrap(), ctx.entity_id_of(hitEnt).unwrap())
                {
                    //FIXME: this should not always work
                    if (*hitEnt).health <= 0 {
                        //we kicked a dead guy
                        //throw harder - FIXME: no matter how hard I push them, they don't go anywhere... corpses use less physics???
                        //	G_Throw( hitEnt, kickDir, kickPush*4 );
                        //see if we should play a better looking death on them
                        //	G_ThrownDeathAnimForDeathAnim( hitEnt, trace.endpos );
                        G_TossTheMofo(
                            ctx,
                            ctx.entity_id_of(hitEnt).unwrap(),
                            kickDir,
                            kickPush * 4.0f32,
                        );
                    } else {
                        /*
                        G_Throw( hitEnt, kickDir, kickPush );
                        if ( kickPush >= 75.0f && !ctx.world.bg_state.rng.Q_irand( 0, 2 ) )
                        {
                            G_Knockdown( hitEnt, ent, kickDir, 300, qtrue );
                        }
                        else
                        {
                            G_Knockdown( hitEnt, ent, kickDir, kickPush, qtrue );
                        }
                        */
                        if kickPush >= 75.0f32 && ctx.world.bg_state.rng.Q_irand(0, 2) == 0 {
                            G_TossTheMofo(
                                ctx,
                                ctx.entity_id_of(hitEnt).unwrap(),
                                kickDir,
                                300.0f32,
                            );
                        } else {
                            G_TossTheMofo(
                                ctx,
                                ctx.entity_id_of(hitEnt).unwrap(),
                                kickDir,
                                kickPush,
                            );
                        }
                    }
                }
            }
        }
        hitEnt
    }
}

/// Raven `G_KickSomeMofos`.
///
/// Source: `oracle/codemp/game/w_saber.c:7644-7956`
pub fn G_KickSomeMofos(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    unsafe {
        let client = (*ent).client;

        let mut kickDir: vec3_t = [0.0; 3];
        let mut kickEnd: vec3_t = [0.0; 3];
        let mut fwdAngs: vec3_t = [0.0; 3];
        let animLength = mp_bg::bg_panimate::BG_AnimLength(
            &ctx.world.bg_state,
            (*ent).localAnimIndex,
            (*client).ps.legsAnim,
        ) as f32;
        let elapsedTime: f32 = animLength - (*client).ps.legsTimer as f32;
        let remainingTime: f32 = animLength - elapsedTime;
        let mut kickDist: f32 = ((*ent).r.maxs[0] * 1.5f32) + STAFF_KICK_RANGE as f32 + 8.0f32; //fudge factor of 8
        let kickDamage: c_int = ctx.world.bg_state.rng.Q_irand(10, 15); //ctx.world.bg_state.rng.Q_irand( 3, 8 ); //since it can only hit a guy once now
        let mut kickPush: c_int = ctx.world.bg_state.rng.flrand(50.0f32, 100.0f32) as c_int;
        let mut doKick: qboolean = 0;

        // VectorSet(kickDir, 0,0,0); VectorSet(kickEnd, 0,0,0);
        // VectorSet(fwdAngs, 0, ps.viewangles[YAW], 0);
        fwdAngs = [0.0f32, (*client).ps.viewangles[YAW as usize], 0.0f32];

        //HMM... or maybe trace from origin to footRBolt/footLBolt?  Which one?  G2 trace?  Will do hitLoc, if so...
        if (*client).ps.torsoAnim == BOTH_A7_HILT as c_int {
            if elapsedTime >= 250.0 && remainingTime >= 250.0 {
                //front
                doKick = 1;
                if (*client).renderInfo.handRBolt != -1 {
                    //actually trace to a bolt
                    G_GetBoltPosition(
                        ctx,
                        ctx.entity_id_of(ent),
                        (*client).renderInfo.handRBolt,
                        Some(&mut kickEnd),
                        0,
                    );
                    _VectorSubtract(kickEnd, (*client).ps.origin, &mut kickDir);
                    kickDir[2] = 0.0; //ah, flatten it, I guess...
                    VectorNormalize(&mut kickDir);
                } else {
                    //guess
                    AngleVectors(fwdAngs, Some(&mut kickDir), None, None);
                }
            }
        } else {
            let legsAnim = (*client).ps.legsAnim;
            if legsAnim == BOTH_GETUP_BROLL_B as c_int
                || legsAnim == BOTH_GETUP_BROLL_F as c_int
                || legsAnim == BOTH_GETUP_FROLL_B as c_int
                || legsAnim == BOTH_GETUP_FROLL_F as c_int
            {
                if elapsedTime >= 250.0 && remainingTime >= 250.0 {
                    //front
                    doKick = 1;
                    if (*client).renderInfo.footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(
                            ctx,
                            ctx.entity_id_of(ent),
                            (*client).renderInfo.footRBolt,
                            Some(&mut kickEnd),
                            0,
                        );
                        _VectorSubtract(kickEnd, (*client).ps.origin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(fwdAngs, Some(&mut kickDir), None, None);
                    }
                }
            } else if legsAnim == BOTH_A7_KICK_F_AIR as c_int
                || legsAnim == BOTH_A7_KICK_B_AIR as c_int
                || legsAnim == BOTH_A7_KICK_R_AIR as c_int
                || legsAnim == BOTH_A7_KICK_L_AIR as c_int
            {
                if elapsedTime >= 100.0 && remainingTime >= 250.0 {
                    //air
                    doKick = 1;
                    if (*client).renderInfo.footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(
                            ctx,
                            ctx.entity_id_of(ent),
                            (*client).renderInfo.footRBolt,
                            Some(&mut kickEnd),
                            0,
                        );
                        _VectorSubtract(kickEnd, (*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(fwdAngs, Some(&mut kickDir), None, None);
                    }
                }
            } else if legsAnim == BOTH_A7_KICK_F as c_int {
                //FIXME: push forward?
                if elapsedTime >= 250.0 && remainingTime >= 250.0 {
                    //front
                    doKick = 1;
                    if (*client).renderInfo.footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(
                            ctx,
                            ctx.entity_id_of(ent),
                            (*client).renderInfo.footRBolt,
                            Some(&mut kickEnd),
                            0,
                        );
                        _VectorSubtract(kickEnd, (*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(fwdAngs, Some(&mut kickDir), None, None);
                    }
                }
            } else if legsAnim == BOTH_A7_KICK_B as c_int {
                //FIXME: push back?
                if elapsedTime >= 250.0 && remainingTime >= 250.0 {
                    //back
                    doKick = 1;
                    if (*client).renderInfo.footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(
                            ctx,
                            ctx.entity_id_of(ent),
                            (*client).renderInfo.footRBolt,
                            Some(&mut kickEnd),
                            0,
                        );
                        _VectorSubtract(kickEnd, (*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(fwdAngs, Some(&mut kickDir), None, None);
                        _VectorScale(kickDir, -1.0, &mut kickDir);
                    }
                }
            } else if legsAnim == BOTH_A7_KICK_R as c_int {
                //FIXME: push right?
                if elapsedTime >= 250.0 && remainingTime >= 250.0 {
                    //right
                    doKick = 1;
                    if (*client).renderInfo.footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(
                            ctx,
                            ctx.entity_id_of(ent),
                            (*client).renderInfo.footRBolt,
                            Some(&mut kickEnd),
                            0,
                        );
                        _VectorSubtract(kickEnd, (*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(fwdAngs, None, Some(&mut kickDir), None);
                    }
                }
            } else if legsAnim == BOTH_A7_KICK_L as c_int {
                //FIXME: push left?
                if elapsedTime >= 250.0 && remainingTime >= 250.0 {
                    //left
                    doKick = 1;
                    if (*client).renderInfo.footLBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(
                            ctx,
                            ctx.entity_id_of(ent),
                            (*client).renderInfo.footLBolt,
                            Some(&mut kickEnd),
                            0,
                        );
                        _VectorSubtract(kickEnd, (*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(fwdAngs, None, Some(&mut kickDir), None);
                        _VectorScale(kickDir, -1.0, &mut kickDir);
                    }
                }
            } else if legsAnim == BOTH_A7_KICK_S as c_int {
                kickPush = ctx.world.bg_state.rng.flrand(75.0f32, 125.0f32) as c_int;
                if (*client).renderInfo.footRBolt != -1 {
                    //actually trace to a bolt
                    if elapsedTime >= 550.0 && elapsedTime <= 1050.0 {
                        doKick = 1;
                        G_GetBoltPosition(
                            ctx,
                            ctx.entity_id_of(ent),
                            (*client).renderInfo.footRBolt,
                            Some(&mut kickEnd),
                            0,
                        );
                        _VectorSubtract(kickEnd, (*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                        //NOTE: have to fudge this a little because it's not getting enough range with the anim as-is
                        _VectorMA(kickEnd, 8.0f32, kickDir, &mut kickEnd);
                    }
                } else {
                    //guess
                    if elapsedTime >= 400.0 && elapsedTime < 500.0 {
                        //front
                        doKick = 1;
                        AngleVectors(fwdAngs, Some(&mut kickDir), None, None);
                    } else if elapsedTime >= 500.0 && elapsedTime < 600.0 {
                        //front-right?
                        doKick = 1;
                        fwdAngs[YAW as usize] += 45.0;
                        AngleVectors(fwdAngs, Some(&mut kickDir), None, None);
                    } else if elapsedTime >= 600.0 && elapsedTime < 700.0 {
                        //right
                        doKick = 1;
                        AngleVectors(fwdAngs, None, Some(&mut kickDir), None);
                    } else if elapsedTime >= 700.0 && elapsedTime < 800.0 {
                        //back-right?
                        doKick = 1;
                        fwdAngs[YAW as usize] += 45.0;
                        AngleVectors(fwdAngs, None, Some(&mut kickDir), None);
                    } else if elapsedTime >= 800.0 && elapsedTime < 900.0 {
                        //back
                        doKick = 1;
                        AngleVectors(fwdAngs, Some(&mut kickDir), None, None);
                        _VectorScale(kickDir, -1.0, &mut kickDir);
                    } else if elapsedTime >= 900.0 && elapsedTime < 1000.0 {
                        //back-left?
                        doKick = 1;
                        fwdAngs[YAW as usize] += 45.0;
                        AngleVectors(fwdAngs, Some(&mut kickDir), None, None);
                    } else if elapsedTime >= 1000.0 && elapsedTime < 1100.0 {
                        //left
                        doKick = 1;
                        AngleVectors(fwdAngs, None, Some(&mut kickDir), None);
                        _VectorScale(kickDir, -1.0, &mut kickDir);
                    } else if elapsedTime >= 1100.0 && elapsedTime < 1200.0 {
                        //front-left?
                        doKick = 1;
                        fwdAngs[YAW as usize] += 45.0;
                        AngleVectors(fwdAngs, None, Some(&mut kickDir), None);
                        _VectorScale(kickDir, -1.0, &mut kickDir);
                    }
                }
            } else if legsAnim == BOTH_A7_KICK_BF as c_int {
                kickPush = ctx.world.bg_state.rng.flrand(75.0f32, 125.0f32) as c_int;
                kickDist += 20.0f32;
                if elapsedTime < 1500.0 {
                    //auto-aim!
                    //			overridAngles = PM_AdjustAnglesForBFKick( ent, ucmd, fwdAngs, qboolean(elapsedTime<850) )?qtrue:overridAngles;
                    //FIXME: if we haven't done the back kick yet and there's no-one there to
                    //			kick anymore, go into some anim that returns us to our base stance
                }
                if (*client).renderInfo.footRBolt != -1 {
                    //actually trace to a bolt
                    if (elapsedTime >= 750.0 && elapsedTime < 850.0)
                        || (elapsedTime >= 1400.0 && elapsedTime < 1500.0)
                    {
                        //right, though either would do
                        doKick = 1;
                        G_GetBoltPosition(
                            ctx,
                            ctx.entity_id_of(ent),
                            (*client).renderInfo.footRBolt,
                            Some(&mut kickEnd),
                            0,
                        );
                        _VectorSubtract(kickEnd, (*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                        //NOTE: have to fudge this a little because it's not getting enough range with the anim as-is
                        _VectorMA(kickEnd, 8.0f32, kickDir, &mut kickEnd);
                    }
                } else {
                    //guess
                    if elapsedTime >= 250.0 && elapsedTime < 350.0 {
                        //front
                        doKick = 1;
                        AngleVectors(fwdAngs, Some(&mut kickDir), None, None);
                    } else if elapsedTime >= 350.0 && elapsedTime < 450.0 {
                        //back
                        doKick = 1;
                        AngleVectors(fwdAngs, Some(&mut kickDir), None, None);
                        _VectorScale(kickDir, -1.0, &mut kickDir);
                    }
                }
            } else if legsAnim == BOTH_A7_KICK_RL as c_int {
                kickPush = ctx.world.bg_state.rng.flrand(75.0f32, 125.0f32) as c_int;
                kickDist += 10.0f32;

                //ok, I'm tracing constantly on these things, they NEVER hit otherwise (in MP at least)

                //FIXME: auto aim at enemies on the side of us?
                //overridAngles = PM_AdjustAnglesForRLKick( ent, ucmd, fwdAngs, qboolean(elapsedTime<850) )?qtrue:overridAngles;
                //if ( elapsedTime >= 250 && elapsedTime < 350 )
                if (ctx.world.level.framenum & 1) != 0 {
                    //right
                    doKick = 1;
                    if (*client).renderInfo.footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(
                            ctx,
                            ctx.entity_id_of(ent),
                            (*client).renderInfo.footRBolt,
                            Some(&mut kickEnd),
                            0,
                        );
                        _VectorSubtract(kickEnd, (*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                        //NOTE: have to fudge this a little because it's not getting enough range with the anim as-is
                        _VectorMA(kickEnd, 8.0f32, kickDir, &mut kickEnd);
                    } else {
                        //guess
                        AngleVectors(fwdAngs, None, Some(&mut kickDir), None);
                    }
                }
                //else if ( elapsedTime >= 350 && elapsedTime < 450 )
                else {
                    //left
                    doKick = 1;
                    if (*client).renderInfo.footLBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(
                            ctx,
                            ctx.entity_id_of(ent),
                            (*client).renderInfo.footLBolt,
                            Some(&mut kickEnd),
                            0,
                        );
                        _VectorSubtract(kickEnd, (*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                        //NOTE: have to fudge this a little because it's not getting enough range with the anim as-is
                        _VectorMA(kickEnd, 8.0f32, kickDir, &mut kickEnd);
                    } else {
                        //guess
                        AngleVectors(fwdAngs, None, Some(&mut kickDir), None);
                        _VectorScale(kickDir, -1.0, &mut kickDir);
                    }
                }
            }
        }

        if doKick != 0 {
            //		G_KickTrace( ent, kickDir, kickDist, kickEnd, kickDamage, kickPush );
            // Raven passes NULL for `kickEnd`; the by-value signature takes
            // `vec3_origin`, which G_KickTrace treats identically (see its note).
            G_KickTrace(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                kickDir,
                kickDist,
                vec3_origin,
                kickDamage,
                kickPush as f32,
            );
        }
    }
}

/// Raven `G_PrettyCloseIGuess`.
///
/// Source: `oracle/codemp/game/w_saber.c:7958-7967`
pub fn G_PrettyCloseIGuess(a: f32, b: f32, tolerance: f32) -> bool {
    if (a - b) < tolerance && (a - b) > -tolerance {
        return true;
    }
    false
}

/// Raven `G_GrabSomeMofos`.
///
/// Source: `oracle/codemp/game/w_saber.c:7969-8082`
pub fn G_GrabSomeMofos(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        let client = (*self_).client;
        // `renderInfo_t *ri = &self->client->renderInfo;` — only `handRBolt` is read.
        let handRBolt = (*client).renderInfo.handRBolt;

        if (*self_).ghoul2.is_null() || handRBolt == -1 {
            // no good
            return;
        }

        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let flatAng: vec3_t = [0.0, (*client).ps.viewangles[1], 0.0];
        trap::G2API_GetBoltMatrix(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                (*self_).ghoul2,
                0,
                handRBolt,
                &mut boltMatrix as *mut mdxaBone_t,
                &flatAng as *const vec3_t,
                &(*client).ps.origin as *const vec3_t,
                ctx.world.level.time,
                core::ptr::null_mut(),
                &(*self_).modelScale as *const vec3_t,
            ),
        );
        let mut pos: vec3_t = [0.0; 3];
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut pos);

        let grabMins: vec3_t = [-4.0, -4.0, -4.0];
        let grabMaxs: vec3_t = [4.0, 4.0, 4.0];

        // trace from my origin to my hand, if we hit anyone then get 'em
        let mut trace: trace_t = core::mem::zeroed();
        trap::G2Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_G2TRACE::GG2TraceArgs::new(
                &mut trace as *mut trace_t,
                &(*client).ps.origin as *const vec3_t,
                &grabMins as *const vec3_t,
                &grabMaxs as *const vec3_t,
                &pos as *const vec3_t,
                (*self_).s.number,
                MASK_SHOT,
                G2TRFLAG_DOGHOULTRACE
                    | G2TRFLAG_GETSURFINDEX
                    | G2TRFLAG_THICK
                    | G2TRFLAG_HITCORPSES,
                ctx.world.cvars.g_g2TraceLod.integer,
            ),
        );

        if trace.fraction != 1.0 && (trace.entityNum as c_int) < ENTITYNUM_WORLD {
            let grabbed = &mut ctx.world.g_entities[trace.entityNum as usize] as *mut gentity_t;
            let gcl = (*grabbed).client;

            if (*grabbed).inuse != 0
                && ((*grabbed).s.eType == ET_PLAYER as c_int
                    || (*grabbed).s.eType == ET_NPC as c_int)
                && !(*grabbed).client.is_null()
                && (*grabbed).health > 0
                && G_CanBeEnemy(
                    ctx,
                    ctx.entity_id_of(self_).unwrap(),
                    ctx.entity_id_of(grabbed).unwrap(),
                )
                && G_PrettyCloseIGuess((*gcl).ps.origin[2], (*client).ps.origin[2], 4.0)
                && (BG_InGrappleMove((*gcl).ps.torsoAnim) == 0
                    || (*gcl).ps.torsoAnim == animNumber_t::BOTH_KYLE_GRAB as c_int)
                && (BG_InGrappleMove((*gcl).ps.legsAnim) == 0
                    || (*gcl).ps.legsAnim == animNumber_t::BOTH_KYLE_GRAB as c_int)
            {
                // grabbed an active player/npc
                let mut tortureAnim: c_int = -1;
                let mut correspondingAnim: c_int = -1;

                if (*client).pers.cmd.forwardmove > 0 {
                    // punch grab
                    tortureAnim = animNumber_t::BOTH_KYLE_PA_1 as c_int;
                    correspondingAnim = animNumber_t::BOTH_PLAYER_PA_1 as c_int;
                } else if (*client).pers.cmd.forwardmove < 0 {
                    // knee-throw
                    tortureAnim = animNumber_t::BOTH_KYLE_PA_2 as c_int;
                    correspondingAnim = animNumber_t::BOTH_PLAYER_PA_2 as c_int;
                }

                if tortureAnim == -1 || correspondingAnim == -1 {
                    if (*client).ps.torsoTimer < 300 && (*client).grappleState == 0 {
                        // you failed to grab anyone, play the "failed to grab" anim
                        G_SetAnim(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            &mut (*client).pers.cmd,
                            SETANIM_BOTH,
                            animNumber_t::BOTH_KYLE_MISS as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            0,
                        );
                        if (*client).ps.torsoAnim == animNumber_t::BOTH_KYLE_MISS as c_int {
                            // providing the anim set succeeded..
                            (*client).ps.weaponTime = (*client).ps.torsoTimer;
                        }
                    }
                    return;
                }

                (*client).grappleIndex = (*grabbed).s.number;
                (*client).grappleState = 1;

                (*gcl).grappleIndex = (*self_).s.number;
                (*gcl).grappleState = 20;

                // time to crack some heads
                G_SetAnim(
                    ctx,
                    ctx.entity_id_of(self_).unwrap(),
                    &mut (*client).pers.cmd,
                    SETANIM_BOTH,
                    tortureAnim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
                if (*client).ps.torsoAnim == tortureAnim {
                    // providing the anim set succeeded..
                    (*client).ps.weaponTime = (*client).ps.torsoTimer;
                }

                G_SetAnim(
                    ctx,
                    ctx.entity_id_of(grabbed).unwrap(),
                    &mut (*gcl).pers.cmd,
                    SETANIM_BOTH,
                    correspondingAnim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
                if (*gcl).ps.torsoAnim == correspondingAnim {
                    // providing the anim set succeeded..
                    if (*gcl).ps.weapon == WP_SABER as c_int {
                        // turn it off
                        if (*gcl).ps.saberHolstered == 0 {
                            (*gcl).ps.saberHolstered = 2;
                            if (*gcl).saber[0].soundOff != 0 {
                                G_Sound(
                                    ctx,
                                    ctx.entity_id_of(grabbed),
                                    CHAN_AUTO as c_int,
                                    (*gcl).saber[0].soundOff,
                                );
                            }
                            if (*gcl).saber[1].soundOff != 0 && (*gcl).saber[1].model[0] != 0 {
                                G_Sound(
                                    ctx,
                                    ctx.entity_id_of(grabbed),
                                    CHAN_AUTO as c_int,
                                    (*gcl).saber[1].soundOff,
                                );
                            }
                        }
                    }
                    if (*gcl).ps.torsoTimer < (*client).ps.torsoTimer {
                        // make sure they stay in the anim at least as long as the grabber
                        (*gcl).ps.torsoTimer = (*client).ps.torsoTimer;
                    }
                    (*gcl).ps.weaponTime = (*gcl).ps.torsoTimer;
                }
            }
        }

        if (*client).ps.torsoTimer < 300 && (*client).grappleState == 0 {
            // you failed to grab anyone, play the "failed to grab" anim
            G_SetAnim(
                ctx,
                ctx.entity_id_of(self_).unwrap(),
                &mut (*client).pers.cmd,
                SETANIM_BOTH,
                animNumber_t::BOTH_KYLE_MISS as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            if (*client).ps.torsoAnim == animNumber_t::BOTH_KYLE_MISS as c_int {
                // providing the anim set succeeded..
                (*client).ps.weaponTime = (*client).ps.torsoTimer;
            }
        }
    }
}

/// Raven `WP_SaberPositionUpdate`.
///
/// rww - keep the saber position as updated as possible on the server so that we
/// can try to do realistic-looking contact stuff. Also does the majority of the
/// work maintaining the server g2 client instance (updating angles/anims/etc).
///
/// Source: `oracle/codemp/game/w_saber.c:8084-9102`
pub fn WP_SaberPositionUpdate(
    ctx: &mut GameContext,
    self_: Option<EntityId>,
    ucmd: *mut usercmd_t,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t =
        unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), self_) };
    unsafe {
        let mut mySaber: *mut gentity_t = core::ptr::null_mut();
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let mut properAngles: vec3_t = [0.0; 3];
        let mut properOrigin: vec3_t = [0.0; 3];
        let mut boltAngles: vec3_t = [0.0; 3];
        let mut boltOrigin: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut legAxis: [vec3_t; 3] = [[0.0; 3]; 3];
        let mut addVel: vec3_t = [0.0; 3];
        let mut rawAngles: vec3_t = [0.0; 3];
        let mut fVSpeed: f32 = 0.0;
        let mut returnAfterUpdate: c_int = 0;
        let mut animSpeedScale: f32 = 1.0;
        let saberNum: c_int;
        let mut saberNumLocal: c_int;
        let clientOverride: qboolean;
        let mut vehEnt: *mut gentity_t = core::ptr::null_mut();
        let mut rSaberNum: c_int = 0;
        let mut rBladeNum: c_int = 0;

        // NOTE: the `#ifdef _DEBUG` g_disableServerG2 early-out is debug-only and dropped.

        if !self_.is_null() && (*self_).inuse != 0 && !(*self_).client.is_null() {
            let client = (*self_).client;
            if (*client).saberCycleQueue != 0 {
                (*client).ps.fd.saberDrawAnimLevel = (*client).saberCycleQueue;
            } else {
                (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
            }
        }

        if !self_.is_null() && (*self_).inuse != 0 && !(*self_).client.is_null() {
            let client = (*self_).client;
            if (*client).saberCycleQueue != 0
                && ((*client).ps.weaponTime <= 0 || (*self_).health < 1)
            {
                // we cycled attack levels while we were busy, so update now that we aren't
                (*client).ps.fd.saberAnimLevel = (*client).saberCycleQueue;
                (*client).saberCycleQueue = 0;
            }
        }

        if self_.is_null()
            || (*self_).inuse == 0
            || (*self_).client.is_null()
            || (*self_).ghoul2.is_null()
            || ctx.world.globals.g2SaberInstance.is_null()
        {
            return;
        }

        let client = (*self_).client;

        if BG_KickingAnim((*client).ps.legsAnim) != 0 {
            // do some kick traces and stuff if we're in the appropriate anim
            G_KickSomeMofos(ctx, ctx.entity_id_of(self_).unwrap());
        } else if (*client).ps.torsoAnim == animNumber_t::BOTH_KYLE_GRAB as c_int {
            // try to grab someone
            G_GrabSomeMofos(ctx, ctx.entity_id_of(self_).unwrap());
        } else if (*client).grappleState != 0 {
            let grappler =
                &mut ctx.world.g_entities[(*client).grappleIndex as usize] as *mut gentity_t;
            let gcl = (*grappler).client;

            if (*grappler).inuse == 0
                || gcl.is_null()
                || (*gcl).grappleIndex != (*self_).s.number
                || BG_InGrappleMove((*gcl).ps.torsoAnim) == 0
                || BG_InGrappleMove((*gcl).ps.legsAnim) == 0
                || BG_InGrappleMove((*client).ps.torsoAnim) == 0
                || BG_InGrappleMove((*client).ps.legsAnim) == 0
                || (*client).grappleState == 0
                || (*gcl).grappleState == 0
                || (*grappler).health < 1
                || (*self_).health < 1
                || !G_PrettyCloseIGuess((*client).ps.origin[2], (*gcl).ps.origin[2], 4.0f32)
            {
                (*client).grappleState = 0;
                if (BG_InGrappleMove((*client).ps.torsoAnim) != 0 && (*client).ps.torsoTimer > 100)
                    || (BG_InGrappleMove((*client).ps.legsAnim) != 0
                        && (*client).ps.legsTimer > 100)
                {
                    // if they're pretty far from finishing the anim then shove them into another anim
                    G_SetAnim(
                        ctx,
                        ctx.entity_id_of(self_).unwrap(),
                        &mut (*client).pers.cmd,
                        SETANIM_BOTH,
                        animNumber_t::BOTH_KYLE_MISS as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        0,
                    );
                    if (*client).ps.torsoAnim == animNumber_t::BOTH_KYLE_MISS as c_int {
                        // providing the anim set succeeded..
                        (*client).ps.weaponTime = (*client).ps.torsoTimer;
                    }
                }
            } else {
                let mut grapAng: vec3_t = [0.0; 3];

                _VectorSubtract((*gcl).ps.origin, (*client).ps.origin, &mut grapAng);

                if VectorLength(grapAng) > 64.0f32 {
                    // too far away, break it off
                    if (BG_InGrappleMove((*client).ps.torsoAnim) != 0
                        && (*client).ps.torsoTimer > 100)
                        || (BG_InGrappleMove((*client).ps.legsAnim) != 0
                            && (*client).ps.legsTimer > 100)
                    {
                        (*client).grappleState = 0;

                        G_SetAnim(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            &mut (*client).pers.cmd,
                            SETANIM_BOTH,
                            animNumber_t::BOTH_KYLE_MISS as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            0,
                        );
                        if (*client).ps.torsoAnim == animNumber_t::BOTH_KYLE_MISS as c_int {
                            // providing the anim set succeeded..
                            (*client).ps.weaponTime = (*client).ps.torsoTimer;
                        }
                    }
                } else {
                    vectoangles(grapAng, &mut grapAng);
                    SetClientViewAngle(&mut *self_, grapAng);

                    if (*client).grappleState >= 20 {
                        // grapplee
                        // try to position myself at the correct distance from my grappler
                        let idealDist: f32;
                        let mut gFwd: vec3_t = [0.0; 3];
                        let mut idealSpot: vec3_t = [0.0; 3];
                        let mut trace: trace_t = core::mem::zeroed();

                        if (*gcl).ps.torsoAnim == animNumber_t::BOTH_KYLE_PA_1 as c_int {
                            // grab punch
                            idealDist = 46.0f32;
                        } else {
                            // knee-throw
                            idealDist = 34.0f32;
                        }

                        AngleVectors((*gcl).ps.viewangles, Some(&mut gFwd), None, None);
                        _VectorMA((*gcl).ps.origin, idealDist, gFwd, &mut idealSpot);

                        trap::Trace(
                            ctx.engine,
                            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                &mut trace as *mut trace_t,
                                &(*client).ps.origin as *const vec3_t,
                                &(*self_).r.mins as *const vec3_t,
                                &(*self_).r.maxs as *const vec3_t,
                                &idealSpot as *const vec3_t,
                                (*self_).s.number,
                                (*self_).clipmask,
                            ),
                        );
                        if trace.startsolid == 0 && trace.allsolid == 0 && trace.fraction == 1.0f32
                        {
                            // go there
                            G_SetOrigin(&mut *(self_), idealSpot);
                            (*client).ps.origin = idealSpot;
                        }
                    } else if (*client).grappleState >= 1 {
                        // grappler
                        if (*gcl).ps.weapon == WP_SABER as c_int {
                            // make sure their saber is shut off
                            if (*gcl).ps.saberHolstered == 0 {
                                (*gcl).ps.saberHolstered = 2;
                                if (*gcl).saber[0].soundOff != 0 {
                                    G_Sound(
                                        ctx,
                                        ctx.entity_id_of(grappler),
                                        CHAN_AUTO as c_int,
                                        (*gcl).saber[0].soundOff,
                                    );
                                }
                                if (*gcl).saber[1].soundOff != 0 && (*gcl).saber[1].model[0] != 0 {
                                    G_Sound(
                                        ctx,
                                        ctx.entity_id_of(grappler),
                                        CHAN_AUTO as c_int,
                                        (*gcl).saber[1].soundOff,
                                    );
                                }
                            }
                        }

                        // check for smashy events
                        if (*client).ps.torsoAnim == animNumber_t::BOTH_KYLE_PA_1 as c_int {
                            // grab punch
                            if (*client).grappleState == 1 {
                                // smack
                                if (*client).ps.torsoTimer < 3400 {
                                    let grapplerAnim = (*gcl).ps.torsoAnim;
                                    let grapplerTime = (*gcl).ps.torsoTimer;

                                    G_Damage(
                                        ctx,
                                        ctx.entity_id_of(grappler),
                                        ctx.entity_id_of(self_),
                                        ctx.entity_id_of(self_),
                                        None,
                                        (*client).ps.origin,
                                        10,
                                        0,
                                        MOD_MELEE as c_int,
                                    );

                                    // it might try to put them into a pain anim, so override it back again
                                    if (*grappler).health > 0 {
                                        (*gcl).ps.torsoAnim = grapplerAnim;
                                        (*gcl).ps.torsoTimer = grapplerTime;
                                        (*gcl).ps.legsAnim = grapplerAnim;
                                        (*gcl).ps.legsTimer = grapplerTime;
                                        (*gcl).ps.weaponTime = grapplerTime;
                                    }
                                    (*client).grappleState += 1;
                                }
                            } else if (*client).grappleState == 2 {
                                // smack!
                                if (*client).ps.torsoTimer < 2550 {
                                    let grapplerAnim = (*gcl).ps.torsoAnim;
                                    let grapplerTime = (*gcl).ps.torsoTimer;

                                    G_Damage(
                                        ctx,
                                        ctx.entity_id_of(grappler),
                                        ctx.entity_id_of(self_),
                                        ctx.entity_id_of(self_),
                                        None,
                                        (*client).ps.origin,
                                        10,
                                        0,
                                        MOD_MELEE as c_int,
                                    );

                                    if (*grappler).health > 0 {
                                        (*gcl).ps.torsoAnim = grapplerAnim;
                                        (*gcl).ps.torsoTimer = grapplerTime;
                                        (*gcl).ps.legsAnim = grapplerAnim;
                                        (*gcl).ps.legsTimer = grapplerTime;
                                        (*gcl).ps.weaponTime = grapplerTime;
                                    }
                                    (*client).grappleState += 1;
                                }
                            } else {
                                // SMACK!
                                if (*client).ps.torsoTimer < 1300 {
                                    let mut tossDir: vec3_t = [0.0; 3];

                                    G_Damage(
                                        ctx,
                                        ctx.entity_id_of(grappler),
                                        ctx.entity_id_of(self_),
                                        ctx.entity_id_of(self_),
                                        None,
                                        (*client).ps.origin,
                                        30,
                                        0,
                                        MOD_MELEE as c_int,
                                    );

                                    (*client).grappleState = 0;

                                    _VectorSubtract(
                                        (*gcl).ps.origin,
                                        (*client).ps.origin,
                                        &mut tossDir,
                                    );
                                    VectorNormalize(&mut tossDir);
                                    _VectorScale(tossDir, 500.0f32, &mut tossDir);
                                    tossDir[2] = 200.0f32;

                                    _VectorAdd(
                                        (*gcl).ps.velocity,
                                        tossDir,
                                        &mut (*gcl).ps.velocity,
                                    );

                                    if (*grappler).health > 0 {
                                        // if still alive knock them down
                                        (*gcl).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN as c_int;
                                        (*gcl).ps.forceHandExtendTime = ctx.world.level.time + 1300;
                                    }
                                }
                            }
                        } else if (*client).ps.torsoAnim == animNumber_t::BOTH_KYLE_PA_2 as c_int {
                            // knee throw
                            if (*client).grappleState == 1 {
                                // knee to the face
                                if (*client).ps.torsoTimer < 3200 {
                                    let grapplerAnim = (*gcl).ps.torsoAnim;
                                    let grapplerTime = (*gcl).ps.torsoTimer;

                                    G_Damage(
                                        ctx,
                                        ctx.entity_id_of(grappler),
                                        ctx.entity_id_of(self_),
                                        ctx.entity_id_of(self_),
                                        None,
                                        (*client).ps.origin,
                                        20,
                                        0,
                                        MOD_MELEE as c_int,
                                    );

                                    if (*grappler).health > 0 {
                                        (*gcl).ps.torsoAnim = grapplerAnim;
                                        (*gcl).ps.torsoTimer = grapplerTime;
                                        (*gcl).ps.legsAnim = grapplerAnim;
                                        (*gcl).ps.legsTimer = grapplerTime;
                                        (*gcl).ps.weaponTime = grapplerTime;
                                    }
                                    (*client).grappleState += 1;
                                }
                            } else if (*client).grappleState == 2 {
                                // smashed on the ground
                                if (*client).ps.torsoTimer < 2000 {
                                    // don't do damage on this one, it would look very freaky if they died
                                    let sound = G_SoundIndex(ctx, "*pain100.wav");
                                    G_EntitySound(
                                        ctx,
                                        ctx.entity_id_of(grappler).unwrap(),
                                        CHAN_VOICE as c_int,
                                        sound,
                                    );
                                    (*client).grappleState += 1;
                                }
                            } else {
                                // and another smash
                                if (*client).ps.torsoTimer < 1000 {
                                    G_Damage(
                                        ctx,
                                        ctx.entity_id_of(grappler),
                                        ctx.entity_id_of(self_),
                                        ctx.entity_id_of(self_),
                                        None,
                                        (*client).ps.origin,
                                        30,
                                        0,
                                        MOD_MELEE as c_int,
                                    );

                                    if (*grappler).health > 0 {
                                        (*gcl).ps.torsoTimer = 1000;
                                        (*gcl).grappleState = 0;
                                    } else {
                                        // override death anim
                                        (*gcl).ps.torsoAnim = animNumber_t::BOTH_DEADFLOP1 as c_int;
                                        (*gcl).ps.legsAnim = animNumber_t::BOTH_DEADFLOP1 as c_int;
                                    }

                                    (*client).grappleState = 0;
                                }
                            }
                        } else {
                            // ?
                        }
                    }
                }
            }
        }

        // If this is a listen server (client+server running on same machine),
        // then lets try to steal the skeleton/etc data off the client instance
        // for this entity to save us processing time.
        clientOverride = trap::G2API_OverrideServer(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_OVERRIDESERVER::GG2OverrideserverArgs::new(
                (*self_).ghoul2,
            ),
        );

        saberNumLocal = (*client).ps.saberEntityNum;

        if saberNumLocal == 0 {
            saberNumLocal = (*client).saberStoredIndex;
        }

        'nextStep: {
            if saberNumLocal == 0 {
                returnAfterUpdate = 1;
                break 'nextStep;
            }

            mySaber = &mut ctx.world.g_entities[saberNumLocal as usize] as *mut gentity_t;

            if (*self_).health < 1 {
                // we don't want to waste CPU calculating saber positions for corpses, but we
                // want to avoid the saber ent position lagging on spawn, so keep it updated.
                if !mySaber.is_null()
                    && (((*mySaber).r.contents & CONTENTS_LIGHTSABER) != 0
                        || (*mySaber).r.contents == 0)
                    && (*client).ps.saberInFlight == 0
                {
                    // Since we haven't got a bolt position, place it on top of the player origin.
                    (*mySaber).r.currentOrigin = (*client).ps.origin;
                }
            }

            if BG_SuperBreakWinAnim((*client).ps.torsoAnim) != qfalse {
                (*client).ps.weaponstate = WEAPON_FIRING;
            }
            if (*client).ps.weapon != WP_SABER as c_int
                || (*client).ps.weaponstate == WEAPON_RAISING
                || (*client).ps.weaponstate == WEAPON_DROPPING
                || (*self_).health < 1
            {
                if (*client).ps.saberInFlight == 0 {
                    returnAfterUpdate = 1;
                }
            }

            if (*client).ps.saberThrowDelay < ctx.world.level.time {
                if ((*client).saber[0].saberFlags & SFL_NOT_THROWABLE) != 0 {
                    // cant throw it normally!
                    if ((*client).saber[0].saberFlags & SFL_SINGLE_BLADE_THROWABLE) != 0 {
                        // but can throw it if only have 1 blade on
                        if (*client).saber[0].numBlades > 1 && (*client).ps.saberHolstered == 1 {
                            // have multiple blades and only one blade on
                            (*client).ps.saberCanThrow = qtrue; // qfalse; want to be able to throw
                        } else {
                            // multiple blades on, can't throw
                            (*client).ps.saberCanThrow = qfalse;
                        }
                    } else {
                        // never can throw it
                        (*client).ps.saberCanThrow = qfalse;
                    }
                } else {
                    // can throw it!
                    (*client).ps.saberCanThrow = qtrue;
                }
            }
        }

        // nextStep:
        saberNum = saberNumLocal;

        'finalUpdate: {
            if ((*client).ps.fd.forcePowersActive & (1 << FP_RAGE as c_int)) != 0 {
                animSpeedScale = 2.0;
            }

            properOrigin = (*client).ps.origin;

            // try to predict the origin based on velocity so it's more like what the client sees
            addVel = (*client).ps.velocity;
            VectorNormalize(&mut addVel);

            if (*client).ps.velocity[0] < 0.0 {
                fVSpeed += -(*client).ps.velocity[0];
            } else {
                fVSpeed += (*client).ps.velocity[0];
            }
            if (*client).ps.velocity[1] < 0.0 {
                fVSpeed += -(*client).ps.velocity[1];
            } else {
                fVSpeed += (*client).ps.velocity[1];
            }
            if (*client).ps.velocity[2] < 0.0 {
                fVSpeed += -(*client).ps.velocity[2];
            } else {
                fVSpeed += (*client).ps.velocity[2];
            }

            fVSpeed *= 1.6f32 / ctx.world.cvars.g_svfps.value;

            // Cap it off at reasonable values so the saber box doesn't go flying ahead of us.
            if fVSpeed > 70.0 {
                fVSpeed = 70.0;
            }
            if fVSpeed < -70.0 {
                fVSpeed = -70.0;
            }

            properOrigin[0] += addVel[0] * fVSpeed;
            properOrigin[1] += addVel[1] * fVSpeed;
            properOrigin[2] += addVel[2] * fVSpeed;

            properAngles[0] = 0.0;
            if (*self_).s.number < (MAX_CLIENTS) as i32 && (*client).ps.m_iVehicleNum != 0 {
                vehEnt = &mut ctx.world.g_entities[(*client).ps.m_iVehicleNum as usize]
                    as *mut gentity_t;
                if (*vehEnt).inuse != 0
                    && !(*vehEnt).client.is_null()
                    && !(*vehEnt).m_pVehicle.is_null()
                {
                    properAngles[1] = *(*((*vehEnt).m_pVehicle)).m_vOrientation.add(YAW as usize);
                } else {
                    properAngles[1] = (*client).ps.viewangles[YAW as usize];
                    vehEnt = core::ptr::null_mut();
                }
            } else {
                properAngles[1] = (*client).ps.viewangles[YAW as usize];
            }
            properAngles[2] = 0.0;

            AnglesToAxis(properAngles, legAxis.as_mut_ptr());

            UpdateClientRenderinfo(
                ctx,
                ctx.entity_id_of(self_).unwrap(),
                properOrigin,
                properAngles,
            );

            if clientOverride == qfalse {
                // if we get the client instance we don't need to do this
                G_G2PlayerAngles(
                    ctx,
                    ctx.entity_id_of(self_),
                    legAxis.as_mut_ptr(),
                    &mut properAngles,
                );
            }

            if !vehEnt.is_null() {
                properAngles[1] = *(*((*vehEnt).m_pVehicle)).m_vOrientation.add(YAW as usize);
            }

            if returnAfterUpdate != 0 && saberNum != 0 {
                // We don't even need GetBoltMatrix if we're only here to keep the g2 server
                // instance in sync, but keep our saber entity in sync too.
                if !mySaber.is_null()
                    && (((*mySaber).r.contents & CONTENTS_LIGHTSABER) != 0
                        || (*mySaber).r.contents == 0)
                    && (*client).ps.saberInFlight == 0
                {
                    (*mySaber).r.currentOrigin = (*client).ps.origin;
                }

                break 'finalUpdate;
            }

            if returnAfterUpdate != 0 {
                break 'finalUpdate;
            }

            // We'll get data for blade 0 first no matter what and stick them into the constant
            // ("_Always") values. Later we handle going through each blade.
            trap::G2API_GetBoltMatrix(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                    (*self_).ghoul2,
                    1,
                    0,
                    &mut boltMatrix as *mut mdxaBone_t,
                    &properAngles as *const vec3_t,
                    &properOrigin as *const vec3_t,
                    ctx.world.level.time,
                    core::ptr::null_mut(),
                    &(*self_).modelScale as *const vec3_t,
                ),
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::ORIGIN as c_int,
                &mut boltOrigin,
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::NEGATIVE_Y as c_int,
                &mut boltAngles,
            );

            // immediately store these values so we don't have to recalculate this again
            if (*client).lastSaberStorageTime != 0
                && (ctx.world.level.time - (*client).lastSaberStorageTime) < 200
            {
                // alright
                (*client).olderSaberBase = (*client).lastSaberBase_Always;
                (*client).olderIsValid = qtrue;
            } else {
                (*client).olderIsValid = qfalse;
            }

            (*client).lastSaberBase_Always = boltOrigin;
            (*client).lastSaberDir_Always = boltAngles;
            (*client).lastSaberStorageTime = ctx.world.level.time;

            rawAngles = boltAngles;

            _VectorMA(
                boltOrigin,
                (*client).saber[0].blade[0].lengthMax,
                boltAngles,
                &mut end,
            );

            if (*client).ps.saberEntityNum != 0 {
                if !mySaber.is_null()
                    && (((*mySaber).r.contents & CONTENTS_LIGHTSABER) != 0
                        || (*mySaber).r.contents == 0)
                    && (*client).ps.saberInFlight == 0
                {
                    // place it roughly in the middle of the saber..
                    _VectorMA(
                        boltOrigin,
                        (*client).saber[0].blade[0].lengthMax,
                        boltAngles,
                        &mut (*mySaber).r.currentOrigin,
                    );
                }
            }

            boltAngles[YAW as usize] = (*client).ps.viewangles[YAW as usize];

            if (*client).ps.saberInFlight != 0 {
                // do the thrown-saber stuff
                let saberent = &mut ctx.world.g_entities[saberNum as usize] as *mut gentity_t;

                if !saberent.is_null() {
                    if (*client).ps.saberEntityState == 0 && (*client).ps.saberEntityNum != 0 {
                        let mut startorg: vec3_t = [0.0; 3];
                        let mut startang: vec3_t = [0.0; 3];
                        let mut dir: vec3_t = [0.0; 3];

                        (*saberent).r.currentOrigin = boltOrigin;

                        startorg = boltOrigin;
                        startang = boltAngles;

                        // Instead of forcing startang[0]=90 we fake it and slowly tilt it down on
                        // the client via a perframe method (doesn't affect where/how the saber hits)

                        (*saberent).r.svFlags &= !SVF_NOCLIENT;
                        (*saberent).s.pos.trBase = startorg;
                        (*saberent).s.apos.trBase = startang;

                        (*saberent).s.origin = startorg;
                        (*saberent).s.angles = startang;

                        (*saberent).s.saberInFlight = qtrue;

                        (*saberent).s.apos.trType = trType_t::TR_LINEAR;
                        (*saberent).s.apos.trDelta[0] = 0.0;
                        (*saberent).s.apos.trDelta[1] = 800.0;
                        (*saberent).s.apos.trDelta[2] = 0.0;

                        (*saberent).s.pos.trType = trType_t::TR_LINEAR;
                        (*saberent).s.eType = ET_GENERAL as c_int;
                        (*saberent).s.eFlags = 0;

                        WP_SaberAddG2Model(
                            ctx,
                            ctx.entity_id_of(saberent).unwrap(),
                            (*client).saber[0].model.as_ptr(),
                            (*client).saber[0].skin,
                        );

                        (*saberent).s.modelGhoul2 = 127;

                        (*saberent).parent = Some(ent_id(ctx.world.g_entities.as_mut_ptr(), self_));

                        (*client).ps.saberEntityState = 1;

                        // Projectile stuff:
                        AngleVectors((*client).ps.viewangles, Some(&mut dir), None, None);

                        (*saberent).nextthink = ctx.world.level.time + FRAMETIME;
                        (*saberent).think = Some(EntThink::saberFirstThrown).into();

                        (*saberent).damage = SABER_THROWN_HIT_DAMAGE;
                        (*saberent).methodOfDeath = MOD_SABER as c_int;
                        (*saberent).splashMethodOfDeath = MOD_SABER as c_int;
                        (*saberent).s.solid = 2;
                        (*saberent).r.contents = CONTENTS_LIGHTSABER;

                        (*saberent).genericValue5 = 0;

                        VectorSet(
                            &mut (*saberent).r.mins,
                            SABERMINS_X,
                            SABERMINS_Y,
                            SABERMINS_Z,
                        );
                        VectorSet(
                            &mut (*saberent).r.maxs,
                            SABERMAXS_X,
                            SABERMAXS_Y,
                            SABERMAXS_Z,
                        );

                        (*saberent).s.genericenemyindex = (*self_).s.number + 1024;

                        (*saberent).touch = Some(EntTouch::thrownSaberTouch).into();

                        (*saberent).s.weapon = WP_SABER as c_int;

                        _VectorScale(dir, 400.0, &mut (*saberent).s.pos.trDelta);
                        (*saberent).s.pos.trTime = ctx.world.level.time;

                        if (*client).saber[0].spinSound != 0 {
                            (*saberent).s.loopSound = (*client).saber[0].spinSound;
                        } else {
                            (*saberent).s.loopSound = ctx.world.globals.saberSpinSound;
                        }
                        (*saberent).s.loopIsSoundset = qfalse;

                        (*client).ps.saberDidThrowTime = ctx.world.level.time;

                        (*client).dangerTime = ctx.world.level.time;
                        (*client).ps.eFlags &= !EF_INVULNERABLE;
                        (*client).invulnerableTimer = 0;

                        trap::LinkEntity(
                            ctx.engine,
                            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(
                                saberent.cast(),
                            ),
                        );
                    } else if (*client).ps.saberEntityNum != 0 {
                        // only do this stuff if your saber is active and has not been knocked out of the air.
                        (*saberent).pos1 = boltOrigin;
                        trap::LinkEntity(
                            ctx.engine,
                            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(
                                saberent.cast(),
                            ),
                        );

                        if (*saberent).genericValue5 == PROPER_THROWN_VALUE {
                            // return to the owner now, this is a bad state to be in for here..
                            (*saberent).genericValue5 = 0;
                            (*saberent).think = Some(EntThink::SaberUpdateSelf).into();
                            (*saberent).nextthink = ctx.world.level.time;
                            WP_SaberRemoveG2Model(ctx, ctx.entity_id_of(saberent).unwrap());

                            (*client).ps.saberInFlight = qfalse;
                            (*client).ps.saberEntityState = 0;
                            (*client).ps.saberThrowDelay = ctx.world.level.time + 500;
                            (*client).ps.saberCanThrow = qfalse;
                        }
                    }
                }
            }

            if BG_SabersOff(&mut (*client).ps) == qfalse {
                let saberent = &mut ctx.world.g_entities[saberNum as usize] as *mut gentity_t;

                if (*client).ps.saberInFlight == 0 && !saberent.is_null() {
                    (*saberent).r.svFlags |= SVF_NOCLIENT;
                    (*saberent).r.contents = CONTENTS_LIGHTSABER;
                    SetSaberBoxSize(ctx, ctx.entity_id_of(saberent));
                    (*saberent).s.loopSound = 0;
                    (*saberent).s.loopIsSoundset = qfalse;
                }

                if (*client).ps.saberLockTime > ctx.world.level.time
                    && (*client).ps.saberEntityNum != 0
                {
                    if (*client).ps.saberIdleWound < ctx.world.level.time {
                        let saber_org = ctx.world.g_entities[saberNum as usize].r.currentOrigin;
                        let te_eid = G_TempEntity(ctx, saber_org, EV_SABER_BLOCK as c_int);
                        let mut dir: vec3_t = [0.0; 3];
                        VectorSet(&mut dir, 0.0, 1.0, 0.0);
                        let te = ctx.entity_mut(te_eid);
                        te.s.origin = saber_org;
                        te.s.angles = dir;
                        te.s.eventParm = 1;
                        te.s.weapon = 0; // saberNum
                        te.s.legsAnim = 0; // bladeNum

                        (*client).ps.saberIdleWound =
                            ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(400, 600);
                    }

                    while rSaberNum < MAX_SABERS as c_int {
                        rBladeNum = 0;
                        while rBladeNum < (*client).saber[rSaberNum as usize].numBlades {
                            // Don't bother updating the bolt for each blade for this, it's just a
                            // very rough fallback method for during saberlocks
                            // Raven indexes `saber[saberNum]` where saberNum is a saber ENTITY
                            // number (>= MAX_CLIENTS) — an OOB write past saber[MAX_SABERS]; the
                            // loop bound (rSaberNum < MAX_SABERS) and adjacent blade[rBladeNum]
                            // show the intent, so index saber[rSaberNum] (§19).
                            (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                .trail
                                .base = boltOrigin;
                            (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                .trail
                                .tip = end;
                            (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                .trail
                                .lastTime = ctx.world.level.time;

                            rBladeNum += 1;
                        }

                        rSaberNum += 1;
                    }
                    (*client).hasCurrentPosition = qtrue;

                    (*client).ps.saberBlocked = BLOCKED_NONE;

                    break 'finalUpdate;
                }

                // reset it in case we used it for cycling before
                rSaberNum = 0;
                rBladeNum = 0;

                if (*client).ps.saberInFlight != 0 {
                    // if saber is thrown then only do the standard stuff for the left hand saber
                    if (*client).ps.saberEntityNum == 0 {
                        // if saber is not in flight but rather knocked away, our left saber is off,
                        // and thus we may do nothing.
                        rSaberNum = 1; // was 2?
                    } else {
                        // thrown saber still in flight, so do damage
                        rSaberNum = 0; // was 1?
                    }
                }

                WP_SaberClearDamage(ctx);
                ctx.world.globals.saberDoClashEffect = qfalse;

                // Now cycle through each saber and each blade on the saber and do damage traces.
                while rSaberNum < MAX_SABERS as c_int {
                    if (*client).saber[rSaberNum as usize].model[0] == 0 {
                        rSaberNum += 1;
                        continue;
                    }

                    // for now I'm keeping a broken right arm swingable, it will just look and act
                    // damaged but still be useable
                    if rSaberNum == 1
                        && ((*client).ps.brokenLimbs & (1 << (BROKENLIMB_LARM as c_int))) != 0
                    {
                        // don't do saber 1 if the left arm is broken
                        break;
                    }
                    // Raven's `self->client->saber[1].model` operand is an array address
                    // (always non-NULL); only the `model[0]` byte test is load-bearing.
                    if rSaberNum > 0
                        && (*client).saber[1].model[0] != 0
                        && (*client).ps.saberHolstered == 1
                    {
                        // don't do saber 2 if it's off
                        break;
                    }
                    rBladeNum = 0;
                    while rBladeNum < (*client).saber[rSaberNum as usize].numBlades {
                        // update muzzle data for the blade
                        (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .muzzlePointOld = (*client).saber[rSaberNum as usize].blade
                            [rBladeNum as usize]
                            .muzzlePoint;
                        (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .muzzleDirOld =
                            (*client).saber[rSaberNum as usize].blade[rBladeNum as usize].muzzleDir;

                        // Raven's `!saber[1].model` operand is an always-false array-address
                        // test; only `!saber[1].model[0]` is load-bearing.
                        if rBladeNum > 0
                            && (*client).saber[1].model[0] == 0
                            && (*client).saber[rSaberNum as usize].numBlades > 1
                            && (*client).ps.saberHolstered == 1
                        {
                            // don't do extra blades if they're off
                            break;
                        }
                        // get the new data then update the bolt pos/dir. rBladeNum corresponds to the
                        // bolt index because blade bolts are added in order.
                        if rSaberNum == 0 && (*client).ps.saberInFlight != 0 {
                            if (*client).ps.saberEntityNum == 0 {
                                // dropped it... shouldn't get here, but...
                                rSaberNum += 1;
                                rBladeNum = 0;
                                continue;
                            } else {
                                let saberEnt = &mut ctx.world.g_entities
                                    [(*client).ps.saberEntityNum as usize]
                                    as *mut gentity_t;
                                let mut saberOrg: vec3_t = [0.0; 3];
                                let mut saberAngles: vec3_t = [0.0; 3];
                                if saberEnt.is_null()
                                    || (*saberEnt).inuse == 0
                                    || (*saberEnt).ghoul2.is_null()
                                {
                                    // wtf?
                                    rSaberNum += 1;
                                    rBladeNum = 0;
                                    continue;
                                }
                                // NOTE: Raven reads `saberent` (the outer saber-num entity), not
                                // `saberEnt`; ported faithfully.
                                if (*saberent).s.saberInFlight != qfalse {
                                    // spinning
                                    BG_EvaluateTrajectory(
                                        &(*saberEnt).s.pos as *const trajectory_t,
                                        ctx.world.level.time + 50,
                                        &mut saberOrg,
                                    );
                                    BG_EvaluateTrajectory(
                                        &(*saberEnt).s.apos as *const trajectory_t,
                                        ctx.world.level.time + 50,
                                        &mut saberAngles,
                                    );
                                } else {
                                    // coming right back
                                    let mut saberDir: vec3_t = [0.0; 3];
                                    BG_EvaluateTrajectory(
                                        &(*saberEnt).s.pos as *const trajectory_t,
                                        ctx.world.level.time,
                                        &mut saberOrg,
                                    );
                                    _VectorSubtract(
                                        (*self_).r.currentOrigin,
                                        saberOrg,
                                        &mut saberDir,
                                    );
                                    vectoangles(saberDir, &mut saberAngles);
                                }
                                trap::G2API_GetBoltMatrix(
                                    ctx.engine,
                                    mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                                        (*saberEnt).ghoul2,
                                        0,
                                        rBladeNum,
                                        &mut boltMatrix as *mut mdxaBone_t,
                                        &saberAngles as *const vec3_t,
                                        &saberOrg as *const vec3_t,
                                        ctx.world.level.time,
                                        core::ptr::null_mut(),
                                        &(*self_).modelScale as *const vec3_t,
                                    ),
                                );
                                BG_GiveMeVectorFromMatrix(
                                    &boltMatrix as *const mdxaBone_t,
                                    Eorientations::ORIGIN as c_int,
                                    &mut (*client).saber[rSaberNum as usize].blade
                                        [rBladeNum as usize]
                                        .muzzlePoint,
                                );
                                BG_GiveMeVectorFromMatrix(
                                    &boltMatrix as *const mdxaBone_t,
                                    Eorientations::NEGATIVE_Y as c_int,
                                    &mut (*client).saber[rSaberNum as usize].blade
                                        [rBladeNum as usize]
                                        .muzzleDir,
                                );
                                boltOrigin = (*client).saber[rSaberNum as usize].blade
                                    [rBladeNum as usize]
                                    .muzzlePoint;
                                _VectorMA(
                                    boltOrigin,
                                    (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                        .lengthMax,
                                    (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                        .muzzleDir,
                                    &mut end,
                                );
                            }
                        } else {
                            trap::G2API_GetBoltMatrix(
                                ctx.engine,
                                mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                                    (*self_).ghoul2,
                                    rSaberNum + 1,
                                    rBladeNum,
                                    &mut boltMatrix as *mut mdxaBone_t,
                                    &properAngles as *const vec3_t,
                                    &properOrigin as *const vec3_t,
                                    ctx.world.level.time,
                                    core::ptr::null_mut(),
                                    &(*self_).modelScale as *const vec3_t,
                                ),
                            );
                            BG_GiveMeVectorFromMatrix(
                                &boltMatrix as *const mdxaBone_t,
                                Eorientations::ORIGIN as c_int,
                                &mut (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                    .muzzlePoint,
                            );
                            BG_GiveMeVectorFromMatrix(
                                &boltMatrix as *const mdxaBone_t,
                                Eorientations::NEGATIVE_Y as c_int,
                                &mut (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                    .muzzleDir,
                            );
                            boltOrigin = (*client).saber[rSaberNum as usize].blade
                                [rBladeNum as usize]
                                .muzzlePoint;
                            _VectorMA(
                                boltOrigin,
                                (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                    .lengthMax,
                                (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                    .muzzleDir,
                                &mut end,
                            );
                        }

                        // Referee probe: saber blade proper origin/angles + stored muzzle point/dir.
                        {
                            let bl = &(*client).saber[rSaberNum as usize].blade[rBladeNum as usize];
                            probe!(
                                "SAB_MUZZLE",
                                "t={} sn={} bn={} pO={:08x},{:08x},{:08x} pA={:08x},{:08x},{:08x} mP={:08x},{:08x},{:08x} mD={:08x},{:08x},{:08x}",
                                ctx.world.level.time, rSaberNum, rBladeNum,
                                properOrigin[0].to_bits(), properOrigin[1].to_bits(), properOrigin[2].to_bits(),
                                properAngles[0].to_bits(), properAngles[1].to_bits(), properAngles[2].to_bits(),
                                bl.muzzlePoint[0].to_bits(), bl.muzzlePoint[1].to_bits(), bl.muzzlePoint[2].to_bits(),
                                bl.muzzleDir[0].to_bits(), bl.muzzleDir[1].to_bits(), bl.muzzleDir[2].to_bits()
                            );
                        }
                        (*client).saber[rSaberNum as usize].blade[rBladeNum as usize].storageTime =
                            ctx.world.level.time;

                        if (*client).hasCurrentPosition != qfalse
                            && ctx.world.cvars.d_saberInterpolate.integer != 0
                        {
                            if (*client).ps.weaponTime <= 0 {
                                // rww - 07/17/02 - don't bother doing the extra stuff unless actually
                                // attacking. This is in attempt to save CPU.
                                CheckSaberDamage(
                                    ctx,
                                    ctx.entity_id_of(self_).unwrap(),
                                    rSaberNum,
                                    rBladeNum,
                                    boltOrigin,
                                    &mut end,
                                    qfalse,
                                    MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT,
                                    qfalse,
                                );
                            } else if ctx.world.cvars.d_saberInterpolate.integer == 1 {
                                let mut trMask: c_int = CONTENTS_LIGHTSABER | CONTENTS_BODY;
                                let mut sN: c_int = 0;
                                let mut gotHit: qboolean = qfalse;
                                // Raven leaves clientUnlinked[MAX_CLIENTS] uninitialized and can
                                // read an entry the trace loop never wrote — UB; zero-init makes
                                // the unwritten entries read as qfalse, the defined behavior (§19).
                                let mut clientUnlinked: [qboolean; MAX_CLIENTS as usize] =
                                    [qfalse; MAX_CLIENTS as usize];
                                let mut skipSaberTrace: qboolean = qfalse;

                                if ctx.world.cvars.g_saberTraceSaberFirst.integer == 0 {
                                    skipSaberTrace = qtrue;
                                } else if ctx.world.cvars.g_saberTraceSaberFirst.integer >= 2
                                    && ctx.world.cvars.g_gametype.integer != GT_DUEL
                                    && ctx.world.cvars.g_gametype.integer != GT_POWERDUEL
                                    && (*client).ps.duelInProgress == qfalse
                                {
                                    // if value is >= 2, and not in a duel, skip
                                    skipSaberTrace = qtrue;
                                }

                                if skipSaberTrace != qfalse {
                                    // skip the saber-contents-only trace and get right to the full trace
                                    trMask = MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT;
                                } else {
                                    while sN < (MAX_CLIENTS) as i32 {
                                        if ctx.world.g_entities[sN as usize].inuse != 0
                                            && !ctx.world.g_entities[sN as usize].client.is_null()
                                            && ctx.world.g_entities[sN as usize].r.linked != qfalse
                                            && ctx.world.g_entities[sN as usize].health > 0
                                            && (ctx.world.g_entities[sN as usize].r.contents
                                                & CONTENTS_BODY)
                                                != 0
                                        {
                                            // Take this mask off before the saber trace, because we
                                            // want to hit the saber first
                                            ctx.world.g_entities[sN as usize].r.contents &=
                                                !CONTENTS_BODY;
                                            clientUnlinked[sN as usize] = qtrue;
                                        } else {
                                            clientUnlinked[sN as usize] = qfalse;
                                        }
                                        sN += 1;
                                    }
                                }

                                while gotHit == qfalse {
                                    if !CheckSaberDamage(
                                        ctx,
                                        ctx.entity_id_of(self_).unwrap(),
                                        rSaberNum,
                                        rBladeNum,
                                        boltOrigin,
                                        &mut end,
                                        qfalse,
                                        trMask,
                                        qfalse,
                                    ) {
                                        if !CheckSaberDamage(
                                            ctx,
                                            ctx.entity_id_of(self_).unwrap(),
                                            rSaberNum,
                                            rBladeNum,
                                            boltOrigin,
                                            &mut end,
                                            qtrue,
                                            trMask,
                                            qfalse,
                                        ) {
                                            let mut oldSaberStart: vec3_t = [0.0; 3];
                                            let mut oldSaberEnd: vec3_t = [0.0; 3];
                                            let mut saberAngleNow: vec3_t = [0.0; 3];
                                            let mut saberAngleBefore: vec3_t = [0.0; 3];
                                            let mut saberMidDir: vec3_t = [0.0; 3];
                                            let mut saberMidAngle: vec3_t = [0.0; 3];
                                            let mut saberMidPoint: vec3_t = [0.0; 3];
                                            let mut saberMidEnd: vec3_t = [0.0; 3];
                                            let mut saberSubBase: vec3_t = [0.0; 3];
                                            let deltaX: f32;
                                            let deltaY: f32;
                                            let deltaZ: f32;

                                            if (ctx.world.level.time
                                                - (*client).saber[rSaberNum as usize].blade
                                                    [rBladeNum as usize]
                                                    .trail
                                                    .lastTime)
                                                > 100
                                            {
                                                // no valid last pos, use current
                                                oldSaberStart = boltOrigin;
                                                oldSaberEnd = end;
                                            } else {
                                                // trace from last pos
                                                oldSaberStart = (*client).saber[rSaberNum as usize]
                                                    .blade
                                                    [rBladeNum as usize]
                                                    .trail
                                                    .base;
                                                oldSaberEnd = (*client).saber[rSaberNum as usize]
                                                    .blade
                                                    [rBladeNum as usize]
                                                    .trail
                                                    .tip;
                                            }

                                            _VectorSubtract(
                                                oldSaberEnd,
                                                oldSaberStart,
                                                &mut saberAngleBefore,
                                            );
                                            vectoangles(saberAngleBefore, &mut saberAngleBefore);

                                            _VectorSubtract(end, boltOrigin, &mut saberAngleNow);
                                            vectoangles(saberAngleNow, &mut saberAngleNow);

                                            deltaX =
                                                AngleDelta(saberAngleBefore[0], saberAngleNow[0]);
                                            deltaY =
                                                AngleDelta(saberAngleBefore[1], saberAngleNow[1]);
                                            deltaZ =
                                                AngleDelta(saberAngleBefore[2], saberAngleNow[2]);

                                            if (deltaX != 0.0 || deltaY != 0.0 || deltaZ != 0.0)
                                                && deltaX < 180.0
                                                && deltaY < 180.0
                                                && deltaZ < 180.0
                                                && (BG_SaberInAttack((*client).ps.saberMove)
                                                    != qfalse
                                                    || PM_SaberInTransition((*client).ps.saberMove)
                                                        != qfalse)
                                            {
                                                // don't go beyond here if we aren't
                                                // attacking/transitioning or the angle is too large,
                                                // and don't bother if the angle is the same
                                                saberMidAngle[0] =
                                                    saberAngleBefore[0] + (deltaX / 2.0);
                                                saberMidAngle[1] =
                                                    saberAngleBefore[1] + (deltaY / 2.0);
                                                saberMidAngle[2] =
                                                    saberAngleBefore[2] + (deltaZ / 2.0);

                                                // Now that I have the angle, I'll just say the base
                                                // is the difference between the two start points.
                                                _VectorSubtract(
                                                    boltOrigin,
                                                    oldSaberStart,
                                                    &mut saberSubBase,
                                                );
                                                saberMidPoint[0] = (boltOrigin[0] as f64
                                                    + saberSubBase[0] as f64 * 0.5)
                                                    as f32;
                                                saberMidPoint[1] = (boltOrigin[1] as f64
                                                    + saberSubBase[1] as f64 * 0.5)
                                                    as f32;
                                                saberMidPoint[2] = (boltOrigin[2] as f64
                                                    + saberSubBase[2] as f64 * 0.5)
                                                    as f32;

                                                AngleVectors(
                                                    saberMidAngle,
                                                    Some(&mut saberMidDir),
                                                    None,
                                                    None,
                                                );
                                                saberMidEnd[0] = saberMidPoint[0]
                                                    + saberMidDir[0]
                                                        * (*client).saber[rSaberNum as usize].blade
                                                            [rBladeNum as usize]
                                                            .lengthMax;
                                                saberMidEnd[1] = saberMidPoint[1]
                                                    + saberMidDir[1]
                                                        * (*client).saber[rSaberNum as usize].blade
                                                            [rBladeNum as usize]
                                                            .lengthMax;
                                                saberMidEnd[2] = saberMidPoint[2]
                                                    + saberMidDir[2]
                                                        * (*client).saber[rSaberNum as usize].blade
                                                            [rBladeNum as usize]
                                                            .lengthMax;

                                                // I'll just trace straight out and not even trace
                                                // between positions to save speed.
                                                if CheckSaberDamage(
                                                    ctx,
                                                    ctx.entity_id_of(self_).unwrap(),
                                                    rSaberNum,
                                                    rBladeNum,
                                                    saberMidPoint,
                                                    &mut saberMidEnd,
                                                    qfalse,
                                                    trMask,
                                                    qfalse,
                                                ) {
                                                    gotHit = qtrue;
                                                }
                                            }
                                        } else {
                                            gotHit = qtrue;
                                        }
                                    } else {
                                        gotHit = qtrue;
                                    }

                                    if ctx.world.cvars.g_saberTraceSaberFirst.integer != 0 {
                                        sN = 0;
                                        while sN < (MAX_CLIENTS) as i32 {
                                            if clientUnlinked[sN as usize] != qfalse {
                                                // Make clients clip properly again.
                                                if ctx.world.g_entities[sN as usize].inuse != 0
                                                    && ctx.world.g_entities[sN as usize].health > 0
                                                {
                                                    ctx.world.g_entities[sN as usize].r.contents |=
                                                        CONTENTS_BODY;
                                                }
                                            }
                                            sN += 1;
                                        }
                                    }

                                    if gotHit == qfalse {
                                        if trMask
                                            != (MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT)
                                        {
                                            trMask =
                                                MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT;
                                        } else {
                                            gotHit = qtrue; // break out of the loop
                                        }
                                    }
                                }
                            } else if ctx.world.cvars.d_saberInterpolate.integer != 0 {
                                // anything but 0 or 1, use the old plain method.
                                if !CheckSaberDamage(
                                    ctx,
                                    ctx.entity_id_of(self_).unwrap(),
                                    rSaberNum,
                                    rBladeNum,
                                    boltOrigin,
                                    &mut end,
                                    qfalse,
                                    MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT,
                                    qfalse,
                                ) {
                                    CheckSaberDamage(
                                        ctx,
                                        ctx.entity_id_of(self_).unwrap(),
                                        rSaberNum,
                                        rBladeNum,
                                        boltOrigin,
                                        &mut end,
                                        qtrue,
                                        MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT,
                                        qfalse,
                                    );
                                }
                            }
                        } else if ctx.world.cvars.d_saberSPStyleDamage.integer != 0 {
                            G_SPSaberDamageTraceLerped(
                                ctx,
                                ctx.entity_id_of(self_).unwrap(),
                                rSaberNum,
                                rBladeNum,
                                &mut boltOrigin,
                                &mut end,
                                MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT,
                            );
                        } else {
                            CheckSaberDamage(
                                ctx,
                                ctx.entity_id_of(self_).unwrap(),
                                rSaberNum,
                                rBladeNum,
                                boltOrigin,
                                &mut end,
                                qfalse,
                                MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT,
                                qfalse,
                            );
                        }

                        (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .trail
                            .base = boltOrigin;
                        (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .trail
                            .tip = end;
                        (*client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .trail
                            .lastTime = ctx.world.level.time;
                        (*client).hasCurrentPosition = qtrue;

                        // do hit effects
                        WP_SaberDoHit(ctx, ctx.entity_id_of(self_).unwrap(), rSaberNum, rBladeNum);
                        WP_SaberDoClash(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            rSaberNum,
                            rBladeNum,
                        );

                        rBladeNum += 1;
                    }

                    rSaberNum += 1;
                }

                WP_SaberApplyDamage(ctx, ctx.entity_id_of(self_).unwrap());
                // NOTE: doing one call after the 2 loops above is cheaper tempentity-wise but won't
                // use the correct saber and blade numbers, so we apply per-blade above.

                if !mySaber.is_null() && (*mySaber).inuse != 0 {
                    trap::LinkEntity(
                        ctx.engine,
                        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(mySaber.cast()),
                    );
                }

                if (*client).ps.saberInFlight == 0 {
                    (*client).ps.saberEntityState = 0;
                }
            }
        }

        // finalUpdate:
        if clientOverride != qfalse {
            // if we get the client instance we don't even need to bother setting anims and stuff
            return;
        }

        G_UpdateClientAnims(ctx, ctx.entity_id_of(self_).unwrap(), animSpeedScale);
    }
}

/// Raven `WP_MissileBlockForBlock`.
///
/// Source: `oracle/codemp/game/w_saber.c:9104-9125`
pub fn WP_MissileBlockForBlock(saberBlock: c_int) -> c_int {
    match saberBlock {
        BLOCKED_UPPER_RIGHT => BLOCKED_UPPER_RIGHT_PROJ,
        BLOCKED_UPPER_LEFT => BLOCKED_UPPER_LEFT_PROJ,
        BLOCKED_LOWER_RIGHT => BLOCKED_LOWER_RIGHT_PROJ,
        BLOCKED_LOWER_LEFT => BLOCKED_LOWER_LEFT_PROJ,
        BLOCKED_TOP => BLOCKED_TOP_PROJ,
        _ => saberBlock,
    }
}

/// Raven `WP_SaberBlockNonRandom`.
///
/// Source: `oracle/codemp/game/w_saber.c:9127-9198`
// `hitloc` is a read-only input here (VectorSubtract source, `hitloc[2]`
// read), never written — so it stays by-value `vec3_t`.
pub fn WP_SaberBlockNonRandom(self_: &gentity_t, hitloc: vec3_t, missileBlock: qboolean) {
    let client = self_.client;
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    unsafe {
        let mut clEye: vec3_t = (*client).ps.origin;
        clEye[2] += (*client).ps.viewheight as f32;

        let mut diff: vec3_t = [
            hitloc[0] - clEye[0],
            hitloc[1] - clEye[1],
            hitloc[2] - clEye[2],
        ];
        diff[2] = 0.0;
        VectorNormalize(&mut diff);

        let mut fwdangles: vec3_t = [0.0, 0.0, 0.0];
        fwdangles[1] = (*client).ps.viewangles[1];
        // Ultimately we might care if the shot was ahead or behind, but for now,
        // just quadrant is fine.
        let mut right: vec3_t = [0.0; 3];
        AngleVectors(fwdangles, None, Some(&mut right), None);

        let rightdot = right[0] * diff[0] + right[1] * diff[1] + right[2] * diff[2];
        let zdiff = hitloc[2] - clEye[2];

        if zdiff > 0.0 {
            if (rightdot as f64) > 0.3 {
                (*client).ps.saberBlocked = BLOCKED_UPPER_RIGHT;
            } else if (rightdot as f64) < -0.3 {
                (*client).ps.saberBlocked = BLOCKED_UPPER_LEFT;
            } else {
                (*client).ps.saberBlocked = BLOCKED_TOP;
            }
        } else if zdiff > -20.0 {
            if zdiff < -10.0 {
                // hmm, pretty low, but not low enough to use the low block, so
                // we need to duck
            }
            if (rightdot as f64) > 0.1 {
                (*client).ps.saberBlocked = BLOCKED_UPPER_RIGHT;
            } else if (rightdot as f64) < -0.1 {
                (*client).ps.saberBlocked = BLOCKED_UPPER_LEFT;
            } else {
                (*client).ps.saberBlocked = BLOCKED_TOP;
            }
        } else if rightdot >= 0.0 {
            (*client).ps.saberBlocked = BLOCKED_LOWER_RIGHT;
        } else {
            (*client).ps.saberBlocked = BLOCKED_LOWER_LEFT;
        }

        if missileBlock != 0 {
            (*client).ps.saberBlocked = WP_MissileBlockForBlock((*client).ps.saberBlocked);
        }
    }
}

/// Raven `WP_SaberBlock`.
///
/// Source: `oracle/codemp/game/w_saber.c:9200-9274`
// `hitloc` is a read-only input here (VectorSubtract source, `hitloc[2]`
// read), never written — so it stays by-value `vec3_t`.
pub fn WP_SaberBlock(
    ctx: &mut GameContext,
    playerent: EntityId,
    hitloc: vec3_t,
    missileBlock: qboolean,
) {
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let client = ctx.world.entity(playerent).client;
    unsafe {
        let mut diff: vec3_t = [
            hitloc[0] - (*client).ps.origin[0],
            hitloc[1] - (*client).ps.origin[1],
            hitloc[2] - (*client).ps.origin[2],
        ];
        VectorNormalize(&mut diff);

        let mut fwdangles: vec3_t = [0.0, 0.0, 0.0];
        fwdangles[1] = (*client).ps.viewangles[1];
        // Ultimately we might care if the shot was ahead or behind, but for now,
        // just quadrant is fine.
        let mut right: vec3_t = [0.0; 3];
        AngleVectors(fwdangles, None, Some(&mut right), None);

        let rightdot = (right[0] * diff[0] + right[1] * diff[1] + right[2] * diff[2])
            + RandFloat(ctx, -0.2, 0.2);
        let zdiff =
            hitloc[2] - (*client).ps.origin[2] + ctx.world.bg_state.rng.Q_irand(-8, 8) as f32;

        // Figure out what quadrant the block was in.
        if zdiff > 24.0 {
            // Attack from above
            if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                (*client).ps.saberBlocked = BLOCKED_TOP;
            } else {
                (*client).ps.saberBlocked = BLOCKED_UPPER_LEFT;
            }
        } else if zdiff > 13.0 {
            // The upper half has three viable blocks...
            if rightdot > 0.25 {
                // In the right quadrant...
                if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                    (*client).ps.saberBlocked = BLOCKED_UPPER_LEFT;
                } else {
                    (*client).ps.saberBlocked = BLOCKED_LOWER_LEFT;
                }
            } else {
                match ctx.world.bg_state.rng.Q_irand(0, 3) {
                    0 => (*client).ps.saberBlocked = BLOCKED_UPPER_RIGHT,
                    1 | 2 => (*client).ps.saberBlocked = BLOCKED_LOWER_RIGHT,
                    3 => (*client).ps.saberBlocked = BLOCKED_TOP,
                    _ => {}
                }
            }
        } else {
            // The lower half is a bit iffy as far as block coverage.  Pick one of
            // the "low" ones at random.
            if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                (*client).ps.saberBlocked = BLOCKED_LOWER_RIGHT;
            } else {
                (*client).ps.saberBlocked = BLOCKED_LOWER_LEFT;
            }
        }

        if missileBlock != 0 {
            (*client).ps.saberBlocked = WP_MissileBlockForBlock((*client).ps.saberBlocked);
        }
    }
}

/// Raven `WP_SaberCanBlock`.
///
/// Source: `oracle/codemp/game/w_saber.c:9276-9451`
// `point` is read-only (passed to InFront/WP_SaberBlockNonRandom, never
// written) — stays by-value `vec3_t`. Raven's `!point` null-guard is vestigial
// for a by-value array and is dropped.
pub fn WP_SaberCanBlock(
    ctx: &mut GameContext,
    self_: Option<EntityId>,
    point: vec3_t,
    dflags: c_int,
    r#mod: c_int,
    projectile: bool,
    mut attackStr: c_int,
) -> c_int {
    let mut thrownSaber = false;
    let mut blockFactor: f32 = 0.0;

    let Some(self_) = self_ else {
        return 0;
    };
    if ctx.world.entity(self_).client.is_null() {
        return 0;
    }
    // FLAG: pool client (NPC-capable); deref raw per recipe 2b.
    let client = ctx.world.entity(self_).client;

    unsafe {
        if attackStr == 999 {
            attackStr = 0;
            thrownSaber = true;
        }

        if BG_SaberInAttack((*client).ps.saberMove) != 0 {
            return 0;
        }

        if PM_InSaberAnim((*client).ps.torsoAnim) != 0
            && (*client).ps.saberBlocked == 0
            && (*client).ps.saberMove != LS_READY
            && (*client).ps.saberMove != LS_NONE
        {
            if (*client).ps.saberMove < LS_PARRY_UP || (*client).ps.saberMove > LS_REFLECT_LL {
                return 0;
            }
        }

        if PM_SaberInBrokenParry((*client).ps.saberMove) != 0 {
            return 0;
        }

        if (*client).ps.saberEntityNum == 0 {
            // saber is knocked away
            return 0;
        }

        if BG_SabersOff(&mut (*client).ps) != 0 {
            return 0;
        }

        if (*client).ps.weapon != WP_SABER as c_int {
            return 0;
        }

        if (*client).ps.weaponstate == WEAPON_RAISING {
            return 0;
        }

        if (*client).ps.saberInFlight != 0 {
            return 0;
        }

        if ((*client).pers.cmd.buttons & BUTTON_ATTACK) != 0 {
            // don't block when the player is trying to slash, if it's a
            // projectile or he's doing a very strong attack
            return 0;
        }

        // Removed for now (pre-1.03 block-decision code); see oracle 9342-9384.

        if SaberAttacking(ctx.world.entity(self_)) {
            // attacking, can't block now
            return 0;
        }

        if (*client).ps.saberMove != LS_READY && (*client).ps.saberBlocking == 0 {
            return 0;
        }

        if (*client).ps.saberBlockTime >= ctx.world.level.time {
            return 0;
        }

        if (*client).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            return 0;
        }

        if (*client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] == FORCE_LEVEL_3 {
            if ctx.world.cvars.d_saberGhoul2Collision.integer != 0 {
                blockFactor = 0.3;
            } else {
                blockFactor = 0.05;
            }
        } else if (*client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] == FORCE_LEVEL_2 {
            blockFactor = 0.6;
        } else if (*client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] == FORCE_LEVEL_1 {
            blockFactor = 0.9;
        } else {
            // for now we just don't get to autoblock with no def
            return 0;
        }

        if thrownSaber {
            blockFactor -= 0.25;
        }

        if attackStr != 0 {
            // blocking a saber, not a projectile.
            blockFactor -= 0.25;
        }

        if InFront(
            point,
            (*client).ps.origin,
            (*client).ps.viewangles,
            blockFactor,
        ) == 0
        {
            return 0;
        }

        if projectile {
            WP_SaberBlockNonRandom(ctx.world.entity(self_), point, projectile as qboolean);
        }
        1
    }
}

/// Raven `HasSetSaberOnly`.
///
/// Source: `oracle/codemp/game/w_saber.c:9453-9484`
pub fn HasSetSaberOnly(ctx: &mut GameContext) -> bool {
    let mut i: c_int = 0;
    let mut wDisable: c_int = 0;

    if ctx.world.cvars.g_gametype.integer == GT_JEDIMASTER {
        // set to 0
        return false;
    }

    if ctx.world.cvars.g_gametype.integer == GT_DUEL
        || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
    {
        wDisable = ctx.world.cvars.g_duelWeaponDisable.integer;
    } else {
        wDisable = ctx.world.cvars.g_weaponDisable.integer;
    }

    while i < WP_NUM_WEAPONS as c_int {
        if (wDisable & (1 << i)) == 0 && i != WP_SABER as c_int && i != WP_NONE as c_int {
            return false;
        }

        i += 1;
    }

    true
}
