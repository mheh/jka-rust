// PORT-COMPLETE: NPC_AI_GalakMech.c 11/15
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_GalakMech.c`.
//!
//! Pass-2 packet port: the `NPC`/`NPCInfo`/`ucmd` ambient globals resolved to
//! real `GameGlobals` fields (`*mut gentity_t`/`*mut gNPC_t`/`usercmd_t`)
//! during this pass, so 9 of the 13 parked fns (`NPC_GalakMech_Init`,
//! `GM_CreateExplosion`, `GM_Dying`, `NPC_GM_Pain`, `GM_HoldPosition`,
//! `NPC_BSGM_Patrol`, `GM_CheckMoveState`, `NPC_GM_StartLaser`,
//! `GM_StartGloat`) are ported faithfully. 4 remain parked:
//!
//! - `GM_Move` — reads `navInfo_t`'s `NIF_COLLISION` flag bit, unresolved by
//!   this packet (same gap as `g_navnew.rs`'s `NAVNEW_AvoidCollision`).
//! - `GM_CheckFireState` — the packet's own STATE FIELDS section groups
//!   `impactPos4` as a "bg-owned/const" global rather than a
//!   `ctx.world.globals` field, leaving its placement unsettled.
//! - `NPC_BSGM_Attack`/`NPC_BSGM_Default` — 636/67 LOC bodies with further
//!   unresolved consts (`shieldMins`/`shieldMaxs`) and fn-pointer-store
//!   surface beyond this packet's scope; parked rather than guessed.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::entity::flags::{FL_NO_KNOCKBACK, FL_SHIELDED};
use crate::trap;
use mp_bg::public::entity_event::entity_event_t;
use mp_bg::public::means_of_death::meansOfDeath_t;
use mp_bg::public::stat_index::statIndex_t;
use mp_bg::public::anim_number::animNumber_t;
use mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_WALKING;

// Raven's file-scope `#define`s (`NPC_AI_GalakMech.c:24-26`) — not central
// constants, ported as file-local consts matching the C values.
const TURN_ON: c_int = 0x00000000;
const TURN_OFF: c_int = 0x00000100;
const GALAK_SHIELD_HEALTH: c_int = 500;

// Raven `bState_t::BS_CINEMATIC` bare spelling (not glob-imported by the
// prelude, unlike the other Raven enums it re-exports).
// Source: `oracle/oracle/codemp/game/b_public.h`
const BS_CINEMATIC: bState_t = bState_t::BS_CINEMATIC;

// Raven `HL_GENERIC1` (`NPC_AI_GalakMech.c` uses the bare hit-location
// spelling; not glob-imported by the prelude).
// Source: `oracle/oracle/codemp/game/g_local.h`
use crate::entity::hit_location::HL_GENERIC1;

// Raven `gNPC_t::scriptFlags` bit `SCF_ALT_FIRE` — same local-const precedent
// as `NPC_combat.rs`.
// Source: `oracle/oracle/codemp/game/b_public.h`
const SCF_ALT_FIRE: i32 = 0x00000040;

// Raven `FRAMETIME` (`bg_public.h`) — same local-const precedent as
// `g_mover.rs`.
const FRAMETIME: c_int = 100;

/// Inline helper from `oracle/oracle/codemp/game/bg_public.h:1524-1564`
/// (same local-copy precedent as `NPC_AI_Mark2.rs`'s private helper of the
/// same name — the call-surface table's "ported: NPC_AI_Mark2.rs" copy is
/// `fn`-private to that file).
#[inline]
fn BG_GiveMeVectorFromMatrix(boltMatrix: *const mdxaBone_t, flags: c_int, vec: &mut vec3_t) {
    const ORIGIN: c_int = Eorientations::ORIGIN as c_int;
    const NEGATIVE_Y: c_int = Eorientations::NEGATIVE_Y as c_int;
    unsafe {
        match flags {
            ORIGIN => {
                vec[0] = (*boltMatrix).matrix[0][3];
                vec[1] = (*boltMatrix).matrix[1][3];
                vec[2] = (*boltMatrix).matrix[2][3];
            }
            NEGATIVE_Y => {
                vec[0] = -(*boltMatrix).matrix[0][1];
                vec[1] = -(*boltMatrix).matrix[1][1];
                vec[2] = -(*boltMatrix).matrix[2][1];
            }
            _ => {}
        }
    }
}

/// Raven `NPC_GalakMech_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:42-57`
pub fn NPC_GalakMech_Precache(ctx: GameContext<'_>) {
    crate::g_utils::G_SoundIndex(c"sound/weapons/galak/skewerhit.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/weapons/galak/lasercharge.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/weapons/galak/lasercutting.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/weapons/galak/laserdamage.wav".as_ptr());

    crate::g_utils::G_EffectIndex(c"galak/trace_beam".as_ptr());
    crate::g_utils::G_EffectIndex(c"galak/beam_warmup".as_ptr());
    //	G_EffectIndex( "small_chunks");
    crate::g_utils::G_EffectIndex(c"env/med_explode2".as_ptr());
    crate::g_utils::G_EffectIndex(c"env/small_explode2".as_ptr());
    crate::g_utils::G_EffectIndex(c"galak/explode".as_ptr());
    crate::g_utils::G_EffectIndex(c"blaster/smoke_bolton".as_ptr());
    //	G_EffectIndex( "env/exp_trail_comp");
}

/// Raven `NPC_GalakMech_Init`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:59-98`
pub fn NPC_GalakMech_Init(ctx: GameContext<'_>, ent: *mut gentity_t) {
    unsafe {
        let npc = (*ent).NPC as *mut gNPC_t;
        let behavior_state = *((&(*npc).behaviorState) as *const bState_t as *const c_int);
        if behavior_state != BS_CINEMATIC as c_int {
            let client = (*ent).client as *mut gclient_t;
            (*client).ps.stats[statIndex_t::STAT_ARMOR as usize] = GALAK_SHIELD_HEALTH;
            (*npc).investigateCount = 0;
            (*npc).investigateDebounceTime = 0;
            (*ent).flags |= FL_SHIELDED; // reflect normal shots
            // rwwFIXMEFIXME: Support PW_GALAK_SHIELD
            // ent->client->ps.powerups[PW_GALAK_SHIELD] = Q3_INFINITE; // temp, for effect
            // ent->fx_time = level.time;
            (*ent).r.mins = [-60.0, -60.0, -24.0];
            (*ent).r.maxs = [60.0, 60.0, 80.0];
            (*ent).flags |= FL_NO_KNOCKBACK; // don't get pushed
            crate::g_timer::TIMER_Set(ctx, ent, c"attackDelay".as_ptr(), 0); // FIXME: Slant for difficulty levels
            crate::g_timer::TIMER_Set(ctx, ent, c"flee".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ctx, ent, c"smackTime".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ctx, ent, c"beamDelay".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ctx, ent, c"noLob".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ctx, ent, c"noRapid".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ctx, ent, c"talkDebounce".as_ptr(), 0);

            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_shield".as_ptr(), TURN_ON);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_galakface".as_ptr(), TURN_OFF);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_galakhead".as_ptr(), TURN_OFF);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_eyes_mouth".as_ptr(), TURN_OFF);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_collar".as_ptr(), TURN_OFF);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_galaktorso".as_ptr(), TURN_OFF);
        } else {
            // NPC_SetSurfaceOnOff( ent, "helmet", TURN_OFF );
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_shield".as_ptr(), TURN_OFF);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_galakface".as_ptr(), TURN_ON);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_galakhead".as_ptr(), TURN_ON);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_eyes_mouth".as_ptr(), TURN_ON);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_collar".as_ptr(), TURN_ON);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ent, c"torso_galaktorso".as_ptr(), TURN_ON);
        }
    }
}

/// Raven `GM_CreateExplosion`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:101-125`
pub fn GM_CreateExplosion(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    boltID: c_int,
    doSmall: qboolean,
) {
    unsafe {
        if boltID >= 0 {
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let mut org: vec3_t = [0.0; 3];
            let mut dir: vec3_t = [0.0; 3];

            trap::G2API_GetBoltMatrix(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                    (*self_).ghoul2,
                    0,
                    boltID,
                    &mut boltMatrix as *mut mdxaBone_t,
                    &(*self_).r.currentAngles as *const vec3_t,
                    &(*self_).r.currentOrigin as *const vec3_t,
                    (*ctx.world).level.time,
                    core::ptr::null_mut(),
                    &(*self_).modelScale as *const vec3_t,
                ),
            );

            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                ORIGIN as c_int,
                &mut org,
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                NEGATIVE_Y as c_int,
                &mut dir,
            );

            if doSmall != 0 {
                crate::g_utils::G_PlayEffectID(
                    crate::g_utils::G_EffectIndex(c"env/small_explode2".as_ptr()),
                    org,
                    dir,
                );
            } else {
                crate::g_utils::G_PlayEffectID(
                    crate::g_utils::G_EffectIndex(c"env/med_explode2".as_ptr()),
                    org,
                    dir,
                );
            }
        }
    }
}

/// Raven `GM_Dying`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:133-229`
pub fn GM_Dying(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let level_time = (*ctx.world).level.time;
        // Raven `vec3_origin` — resolved via the crate prelude (pass-3 symbol
        // backfill).
        if level_time - (*self_).s.time < 4000 {
            // FIXME: need a real effect
            // self->s.powerups |= ( 1 << PW_SHOCKED );
            // self->client->ps.powerups[PW_SHOCKED] = level.time + 1000;
            let client = (*self_).client as *mut gclient_t;
            (*client).ps.electrifyTime = level_time + 1000;
            if crate::g_timer::TIMER_Done(ctx, self_, c"dyingExplosion".as_ptr()) != 0 {
                let mut newBolt: c_int;
                match crate::q_math::Q_irand(1, 14) {
                    // Find place to generate explosion
                    1 => {
                        if trap::G2API_GetSurfaceRenderStatus(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("r_hand").unwrap(),
                            ),
                        ) == 0
                        {
                            // r_hand still there
                            let newBolt = trap::G2API_AddBolt(
                                ctx.engine,
                                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                    (*self_).ghoul2,
                                    0,
                                    std::ffi::CString::new("*flasha").unwrap(),
                                ),
                            );
                            GM_CreateExplosion(ctx, self_, newBolt, 1);
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"r_hand".as_ptr(), TURN_OFF);
                        } else if trap::G2API_GetSurfaceRenderStatus(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("r_arm_middle").unwrap(),
                            ),
                        ) == 0
                        {
                            // r_arm_middle still there
                            newBolt = trap::G2API_AddBolt(
                                ctx.engine,
                                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                    (*self_).ghoul2,
                                    0,
                                    std::ffi::CString::new("*r_arm_elbow").unwrap(),
                                ),
                            );
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"r_arm_middle".as_ptr(), TURN_OFF);
                        }
                    }
                    2 => {
                        // FIXME: do only once?
                        if trap::G2API_GetSurfaceRenderStatus(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("l_hand").unwrap(),
                            ),
                        ) == 0
                        {
                            // l_hand still there
                            let newBolt = trap::G2API_AddBolt(
                                ctx.engine,
                                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                    (*self_).ghoul2,
                                    0,
                                    std::ffi::CString::new("*flashc").unwrap(),
                                ),
                            );
                            GM_CreateExplosion(ctx, self_, newBolt, 0);
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"l_hand".as_ptr(), TURN_OFF);
                        } else if trap::G2API_GetSurfaceRenderStatus(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("l_arm_wrist").unwrap(),
                            ),
                        ) == 0
                        {
                            // l_arm_wrist still there
                            newBolt = trap::G2API_AddBolt(
                                ctx.engine,
                                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                    (*self_).ghoul2,
                                    0,
                                    std::ffi::CString::new("*l_arm_cap_l_hand").unwrap(),
                                ),
                            );
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"l_arm_wrist".as_ptr(), TURN_OFF);
                        } else if trap::G2API_GetSurfaceRenderStatus(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("l_arm_middle").unwrap(),
                            ),
                        ) == 0
                        {
                            // l_arm_middle still there
                            newBolt = trap::G2API_AddBolt(
                                ctx.engine,
                                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                    (*self_).ghoul2,
                                    0,
                                    std::ffi::CString::new("*l_arm_cap_l_hand").unwrap(),
                                ),
                            );
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"l_arm_middle".as_ptr(), TURN_OFF);
                        } else if trap::G2API_GetSurfaceRenderStatus(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("l_arm_augment").unwrap(),
                            ),
                        ) == 0
                        {
                            // l_arm_augment still there
                            newBolt = trap::G2API_AddBolt(
                                ctx.engine,
                                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                    (*self_).ghoul2,
                                    0,
                                    std::ffi::CString::new("*l_arm_elbow").unwrap(),
                                ),
                            );
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"l_arm_augment".as_ptr(), TURN_OFF);
                        }
                    }
                    3 | 4 => {
                        newBolt = trap::G2API_AddBolt(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("*hip_fr").unwrap(),
                            ),
                        );
                        GM_CreateExplosion(ctx, self_, newBolt, 0);
                    }
                    5 | 6 => {
                        newBolt = trap::G2API_AddBolt(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("*shldr_l").unwrap(),
                            ),
                        );
                        GM_CreateExplosion(ctx, self_, newBolt, 0);
                    }
                    7 | 8 => {
                        newBolt = trap::G2API_AddBolt(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("*uchest_r").unwrap(),
                            ),
                        );
                        GM_CreateExplosion(ctx, self_, newBolt, 0);
                    }
                    9 | 10 => {
                        let head_bolt = (*(*self_).client.cast::<gclient_t>()).renderInfo.headBolt;
                        GM_CreateExplosion(ctx, self_, head_bolt, 0);
                    }
                    11 => {
                        newBolt = trap::G2API_AddBolt(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("*l_leg_knee").unwrap(),
                            ),
                        );
                        GM_CreateExplosion(ctx, self_, newBolt, 1);
                    }
                    12 => {
                        newBolt = trap::G2API_AddBolt(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("*r_leg_knee").unwrap(),
                            ),
                        );
                        GM_CreateExplosion(ctx, self_, newBolt, 1);
                    }
                    13 => {
                        newBolt = trap::G2API_AddBolt(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("*l_leg_foot").unwrap(),
                            ),
                        );
                        GM_CreateExplosion(ctx, self_, newBolt, 1);
                    }
                    14 => {
                        newBolt = trap::G2API_AddBolt(
                            ctx.engine,
                            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                                (*self_).ghoul2,
                                0,
                                std::ffi::CString::new("*r_leg_foot").unwrap(),
                            ),
                        );
                        GM_CreateExplosion(ctx, self_, newBolt, 1);
                    }
                    _ => {}
                }

                crate::g_timer::TIMER_Set(
                    ctx,
                    self_,
                    c"dyingExplosion".as_ptr(),
                    crate::q_math::Q_irand(300, 1100),
                );
            }
        } else {
            // one final, huge explosion
            crate::g_utils::G_PlayEffectID(
                crate::g_utils::G_EffectIndex(c"galak/explode".as_ptr()),
                (*self_).r.currentOrigin,
                vec3_origin,
            );
            // G_PlayEffect( "small_chunks", self->r.currentOrigin );
            // G_PlayEffect( "env/exp_trail_comp", self->r.currentOrigin, self->currentAngles );
            (*self_).nextthink = level_time + FRAMETIME;
            (*self_).think = Some(crate::ent_fn_enums::EntThink::G_FreeEntity);
        }
    }
}

/// Raven `NPC_GM_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:238-354`
// The disabled `if (0)`/PW_GALAK_SHIELD-shield-down branch and the dead
// `gPainPoint`/`point` store (both fully commented out in the oracle, and the
// `if ( point )` array-as-pointer check is always true) have zero observable
// effect; the `Q_irand`/`G_AddVoiceEvent`/timer paths below are the only
// live behavior, ported faithfully.
pub fn NPC_GM_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    unsafe {
        let inflictor = attacker;
        let hitLoc: c_int = 1; // Raven: `int hitLoc = 1;` — never reassigned in this fn
        let r#mod = (*ctx.world).globals.gPainMOD;
        let level_time = (*ctx.world).level.time;

        let npc = (*self_).NPC as *mut gNPC_t;

        if (*self_).lockCount == 0
            && (*(*self_).client.cast::<gclient_t>()).ps.torsoTimer <= 0
        {
            // don't interrupt laser sweep attack or other special attacks/moves
            if (*self_).count < 4 && (*self_).health > 100 && hitLoc != HL_GENERIC1 {
                if (*self_).delay < level_time {
                    let speech = match (*self_).count {
                        1 => entity_event_t::EV_PUSHED2 as c_int,
                        2 => entity_event_t::EV_PUSHED3 as c_int,
                        3 => entity_event_t::EV_DETECTED1 as c_int,
                        _ => entity_event_t::EV_PUSHED1 as c_int,
                    };
                    (*self_).count += 1;
                    if !npc.is_null() {
                        (*npc).blockedSpeechDebounceTime = 0;
                    }
                    crate::NPC_sounds::G_AddVoiceEvent(
                        ctx,
                        self_,
                        speech,
                        crate::q_math::Q_irand(3000, 5000),
                    );
                    (*self_).delay = level_time + crate::q_math::Q_irand(5000, 7000);
                }
            } else {
                crate::NPC_reactions::NPC_Pain(ctx, self_, attacker, damage);
            }
        } else if hitLoc == HL_GENERIC1 {
            crate::NPC_reactions::NPC_SetPainEvent(ctx, self_);
            // self->s.powerups |= ( 1 << PW_SHOCKED );
            // self->client->ps.powerups[PW_SHOCKED] = level.time + Q_irand( 500, 2500 );
            (*(*self_).client.cast::<gclient_t>()).ps.electrifyTime =
                level_time + crate::q_math::Q_irand(500, 2500);
        }

        if !inflictor.is_null() && (*inflictor).lastEnemy == self_ {
            // He force-pushed my own lobfires back at me
            let npc = (*self_).NPC as *mut gNPC_t;
            if r#mod == meansOfDeath_t::MOD_REPEATER_ALT as c_int && crate::q_math::Q_irand(0, 2) == 0 {
                if crate::g_timer::TIMER_Done(ctx, self_, c"noRapid".as_ptr()) != 0 {
                    if !npc.is_null() {
                        (*npc).scriptFlags &= !SCF_ALT_FIRE;
                    }
                    (*self_).alt_fire = 0;
                    crate::g_timer::TIMER_Set(
                        ctx,
                        self_,
                        c"noLob".as_ptr(),
                        crate::q_math::Q_irand(2000, 6000),
                    );
                } else {
                    // hopefully this will make us fire the laser
                    crate::g_timer::TIMER_Set(
                        ctx,
                        self_,
                        c"noLob".as_ptr(),
                        crate::q_math::Q_irand(1000, 2000),
                    );
                }
            } else if r#mod == meansOfDeath_t::MOD_REPEATER as c_int && crate::q_math::Q_irand(0, 5) == 0 {
                if crate::g_timer::TIMER_Done(ctx, self_, c"noLob".as_ptr()) != 0 {
                    if !npc.is_null() {
                        (*npc).scriptFlags |= SCF_ALT_FIRE;
                    }
                    (*self_).alt_fire = 1;
                    crate::g_timer::TIMER_Set(
                        ctx,
                        self_,
                        c"noRapid".as_ptr(),
                        crate::q_math::Q_irand(2000, 6000),
                    );
                } else {
                    // hopefully this will make us fire the laser
                    crate::g_timer::TIMER_Set(
                        ctx,
                        self_,
                        c"noRapid".as_ptr(),
                        crate::q_math::Q_irand(1000, 2000),
                    );
                }
            }
        }
    }
}

/// Raven `GM_HoldPosition`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:362-369`
pub fn GM_HoldPosition(ctx: GameContext<'_>) {
    unsafe {
        let npc_ent = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;
        crate::NPC_combat::NPC_FreeCombatPoint(ctx, (*npc_info).combatPoint, 1);
        let pending = trap::ICARUS_TaskIDPending(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs::new(
                npc_ent,
                taskID_t::TID_MOVE_NAV as c_int,
            ),
        );
        if pending == 0 {
            // don't have a script waiting for me to get to my point, okay to stop trying and stand
            (*npc_info).goalEntity = None;
        }
    }
}

// PORT-ESCALATION(const-value): reads `navInfo_t`'s `NIF_COLLISION` flag bit
// (unresolved in this packet — same gap as `g_navnew.rs`'s
// `NAVNEW_AvoidCollision`); guessing the bit would silently corrupt
// `info.flags` parity.
/// Raven `GM_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:376-408`
pub fn GM_Move(ctx: GameContext<'_>) -> qboolean {
    todo!("Port GM_Move — parked: const-value (NIF_COLLISION)")
}

/// Raven `NPC_BSGM_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:416-432`
pub fn NPC_BSGM_Patrol(ctx: GameContext<'_>) {
    unsafe {
        if crate::NPC_AI_Stormtrooper::NPC_CheckPlayerTeamStealth(ctx) != 0 {
            crate::NPC_utils::NPC_UpdateAngles(ctx, 1, 1);
            return;
        }

        // If we have somewhere to go, then do that
        let goal = crate::NPC_goal::UpdateGoal(ctx);
        if !goal.is_null() {
            (*ctx.world).globals.ucmd.buttons |= BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, 1);
        }

        crate::NPC_utils::NPC_UpdateAngles(ctx, 1, 1);
    }
}

/// Raven `GM_CheckMoveState`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:440-460`
pub fn GM_CheckMoveState(ctx: GameContext<'_>) {
    unsafe {
        let npc_ent = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        if trap::ICARUS_TaskIDPending(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs::new(
                npc_ent,
                taskID_t::TID_MOVE_NAV as c_int,
            ),
        ) != 0
        {
            // moving toward a goal that a script is waiting on, so don't stop for anything!
            (*ctx.world).globals.move4 = 1;
        }

        // See if we're moving towards a goal, not the enemy
        if (*npc_info).goalEntity != (*npc_ent).enemy && !(*npc_info).goalEntity.is_none() {
            // Did we make it?
            let hit_goal = crate::g_nav::NAV_HitNavGoal(
                (*npc_ent).r.currentOrigin,
                (*npc_ent).r.mins,
                (*npc_ent).r.maxs,
                (*(*npc_info).goalEntity).r.currentOrigin,
                16,
                0,
            );
            let script_pending = trap::ICARUS_TaskIDPending(
                ctx.engine,
                mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs::new(
                    npc_ent,
                    taskID_t::TID_MOVE_NAV as c_int,
                ),
            ) != 0;
            if hit_goal != 0
                || (!script_pending
                    && (*ctx.world).globals.enemyLOS4 != 0
                    && (*ctx.world).globals.enemyDist4 <= 10000.0)
            {
                // either hit our navgoal or our navgoal was not a crucial (scripted) one (maybe a
                // combat point) and we're scouting and found our enemy
                crate::NPC_goal::NPC_ReachedGoal(ctx);
                // don't attack right away
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc_ent,
                    c"attackDelay".as_ptr(),
                    crate::q_math::Q_irand(250, 500), // FIXME: Slant for difficulty levels
                );
                return;
            }
        }
    }
}

// PORT-ESCALATION(unresolved-callee-type): calls `NPC_ShotEntity`/
// `CalcEntitySpot`/`trap_Trace` with `trace_t`/`SPOT_HEAD` shapes plus this
// file's own `enemyCS4`/`hitAlly4`/`impactPos4`/`faceEnemy4`/`shoot4` scratch
// (fork ruling 5) — the packet's call surface resolves the callee
// signatures, but `impactPos4` is grouped as a "bg-owned/const" global
// (packet's own STATE FIELDS section, not a `ctx.world.globals` field),
// leaving its owning placement unsettled; guessing would risk silently
// wrong parity across every caller. Deferred rather than guessed per §A2.
/// Raven `GM_CheckFireState`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:468-556`
pub fn GM_CheckFireState(ctx: GameContext<'_>) {
    todo!("Port GM_CheckFireState — parked: unresolved-callee-type (impactPos4 placement)")
}

/// Raven `NPC_GM_StartLaser`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:558-573`
pub fn NPC_GM_StartLaser(ctx: GameContext<'_>) {
    unsafe {
        let npc_ent = (*ctx.world).globals.NPC;
        let npc = (*npc_ent).NPC as *mut gNPC_t;
        if (*npc_ent).lockCount == 0 {
            // haven't already started a laser attack
            // warm up for the beam attack
            #[cfg(any())]
            {
                // NPC_SetAnim( NPC, SETANIM_TORSO, TORSO_RAISEWEAP2, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
            }
            let client = (*npc_ent).client as *mut gclient_t;
            let torso_timer = (*client).ps.torsoTimer;
            crate::g_timer::TIMER_Set(ctx, npc_ent, c"beamDelay".as_ptr(), torso_timer);
            crate::g_timer::TIMER_Set(ctx, npc_ent, c"attackDelay".as_ptr(), torso_timer + 3000);
            (*npc_ent).lockCount = 1;
            // turn on warmup effect (Raven `vec3_origin` — resolved via the
            // crate prelude, pass-3 symbol backfill).
            crate::g_utils::G_PlayEffectID(
                crate::g_utils::G_EffectIndex(c"galak/beam_warmup".as_ptr()),
                (*npc_ent).r.currentOrigin,
                vec3_origin,
            );
            crate::g_utils::G_SoundOnEnt(
                ctx,
                npc_ent,
                CHAN_AUTO,
                c"sound/weapons/galak/lasercharge.wav".as_ptr(),
            );
        }
    }
}

/// Raven `GM_StartGloat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:575-587`
pub fn GM_StartGloat(ctx: GameContext<'_>) {
    unsafe {
        let npc_ent = (*ctx.world).globals.NPC;
        (*npc_ent).wait = 0.0;
        crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, npc_ent, c"torso_galakface".as_ptr(), TURN_ON);
        crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, npc_ent, c"torso_galakhead".as_ptr(), TURN_ON);
        crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, npc_ent, c"torso_eyes_mouth".as_ptr(), TURN_ON);
        crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, npc_ent, c"torso_collar".as_ptr(), TURN_ON);
        crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, npc_ent, c"torso_galaktorso".as_ptr(), TURN_ON);

        crate::npc_c::NPC_SetAnim(
            npc_ent,
            SETANIM_BOTH,
            animNumber_t::BOTH_STAND2TO1 as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        let client = (*npc_ent).client as *mut gclient_t;
        (*client).ps.legsTimer += 500;
        (*client).ps.torsoTimer += 500;
    }
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo`/`ucmd`/
// `g_entities` ambient globals plus this file's own file-static scratch
// (`enemyCS4`/`enemyDist4`/`enemyLOS4`/`faceEnemy4`/`hitAlly4`/`move4`/
// `shoot4`/`impactPos4` — fork ruling 5) and calls trap_InPVS/trap_Trace
// (needs &Engine); no channel from this context-free faithful signature.
/// Raven `NPC_BSGM_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:594-1229`
pub fn NPC_BSGM_Attack(ctx: GameContext<'_>) {
    todo!("Port NPC_BSGM_Attack — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals and calls trap_Trace (needs &Engine); no channel from this
// context-free faithful signature.
/// Raven `NPC_BSGM_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:1231-1297`
pub fn NPC_BSGM_Default(ctx: GameContext<'_>) {
    todo!("Port NPC_BSGM_Default — parked: ambient-state")
}
