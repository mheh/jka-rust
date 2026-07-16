// PORT-COMPLETE: NPC_AI_GalakMech.c 15/15
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_GalakMech.c`.
//!
//! All functions are ported: `GM_Move` reads the canonical `NIF_COLLISION`
//! bit, `GM_CheckFireState`/`NPC_BSGM_Attack`/`NPC_BSGM_Default` place their
//! `*4` state in `ctx.world.globals`, and `impactPos4` lives in
//! `ctx.world.scratch.impact_pos_4` (safe-state Stage 3).
//!
//! Safe-state migration **Stage 2b** (body sweep): every world reach is a
//! checked `ctx.world.…` borrow — the transitional `(*ctx.world_raw())`
//! raw-deref regime is gone. The per-body entity/`gNPC_t`/`gclient_t` re-derives
//! stay raw by design (their dissolution is out of scope), so the `// STAGE-1:`
//! markers and their `unsafe` blocks legitimately hold genuine raw ops.
//! Behavior is byte-identical, referee-verified.
#![allow(non_snake_case, unused, clippy::all)]

use crate::entity::flags::{FL_NO_KNOCKBACK, FL_SHIELDED};
use crate::prelude::*;
use crate::trap;
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::public::entity_event::entity_event_t;
use mp_bg::public::means_of_death::meansOfDeath_t;
use mp_bg::public::stat_index::statIndex_t;
use mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_WALKING;

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

// Raven's file-scope `#define`s (`NPC_AI_GalakMech.c:24-26`) — not central
// constants, ported as file-local consts matching the C values.
pub const TURN_ON: c_int = 0x00000000;
const TURN_OFF: c_int = 0x00000100;
pub const GALAK_SHIELD_HEALTH: c_int = 500;

// Raven `bState_t::BS_CINEMATIC` bare spelling (not glob-imported by the
// prelude, unlike the other Raven enums it re-exports).
// Source: `oracle/codemp/game/b_public.h`
const BS_CINEMATIC: bState_t = bState_t::BS_CINEMATIC;

// Raven `HL_GENERIC1` (`NPC_AI_GalakMech.c` uses the bare hit-location
// spelling; not glob-imported by the prelude).
// Source: `oracle/codemp/game/g_local.h`
use crate::entity::hit_location::HL_GENERIC1;

// Raven `gNPC_t::scriptFlags` bits (`SCF_*`, b_public.h:26-52) resolve to the
// canonical `crate::npc::script_flags` consts through the prelude glob. The
// former local placeholders here had guessed values (SCF_CHASE_ENEMIES 0x80,
// SCF_DONT_FIRE 0x800, SCF_FIRE_WEAPON 0x1000 vs the real 0x400/0x4000/0x40000),
// which masked the wrong scriptFlags bits — a live bug — so they were removed.

// Raven `FRAMETIME` (`g_local.h:37`) = 100. Kept local (single-owner header,
// deliberately not consolidated; see consolidation note) — value matches the oracle.
const FRAMETIME: c_int = 100;

// Raven `NIF_COLLISION` (`navInfo_t::flags` bit) resolves to the canonical
// `crate::npc::nav_info_s::NIF_COLLISION` through the prelude glob.

// Raven file-static `vec3_t impactPos4` — shared across GM_CheckFireState,
// NPC_BSGM_Attack, and others for caching impact positions. Now owned by
// `GameWorld.scratch` (safe-state Stage 3, §B3), reached as
// `ctx.world.scratch.impact_pos_4`.
// Source: `oracle/codemp/game/NPC_AI_GalakMech.c`

// Vector helpers are the canonical `crate::q_math` forms reached via the
// prelude glob: `_VectorCopy`/`_VectorSubtract`/`_VectorMA` (out-param) and
// `VectorClear`. Source: `oracle/codemp/game/q_shared.h`

// Distance constants for combat logic (derived from oracle source comments).
// Source: `oracle/codemp/game/NPC_AI_GalakMech.c` (various lines with distance checks)
const MELEE_DIST_SQUARED: f32 = 6400.0; // 80*80
const MIN_LOB_DIST_SQUARED: f32 = 65536.0; // 256*256
const MAX_LOB_DIST_SQUARED: f32 = 200704.0; // 448*448
const REPEATER_ALT_SIZE: f32 = 3.0; // half of bbox size
                                    // Raven `#define GENERATOR_HEALTH 25`.
                                    // Source: `oracle/codemp/game/NPC_AI_GalakMech.c:23`
const GENERATOR_HEALTH: c_int = 25; // Shield generator health threshold
                                    // Raven `#define ARMOR_EFFECT_TIME 500` (was a guessed 3000 — corrected).
                                    // Source: `oracle/codemp/game/w_saber.h:1`
const ARMOR_EFFECT_TIME: c_int = 500;

/// Inline helper from `oracle/codemp/game/bg_public.h:1524-1564`
/// (same local-copy precedent as `NPC_AI_Mark2.rs`'s private helper of the
/// same name — the call-surface table's "ported: NPC_AI_Mark2.rs" copy is
/// `fn`-private to that file).
#[inline]
pub(crate) fn BG_GiveMeVectorFromMatrix(
    boltMatrix: *const mdxaBone_t,
    flags: c_int,
    vec: &mut vec3_t,
) {
    pub const ORIGIN: c_int = Eorientations::ORIGIN as c_int;
    pub const NEGATIVE_Y: c_int = Eorientations::NEGATIVE_Y as c_int;
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
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:42-57`
pub fn NPC_GalakMech_Precache(ctx: &mut GameContext) {
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
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:59-98`
pub fn NPC_GalakMech_Init(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    unsafe {
        let npc = (*ent).NPC;
        let behavior_state = *((&(*npc).behaviorState) as *const bState_t as *const c_int);
        if behavior_state != BS_CINEMATIC as c_int {
            let client = (*ent).client;
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
            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(ent), c"attackDelay".as_ptr(), 0); // FIXME: Slant for difficulty levels
            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(ent), c"flee".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(ent), c"smackTime".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(ent), c"beamDelay".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(ent), c"noLob".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(ent), c"noRapid".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ctx, ctx.entity_id_of(ent), c"talkDebounce".as_ptr(), 0);

            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_shield".as_ptr(),
                TURN_ON,
            );
            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_galakface".as_ptr(),
                TURN_OFF,
            );
            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_galakhead".as_ptr(),
                TURN_OFF,
            );
            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_eyes_mouth".as_ptr(),
                TURN_OFF,
            );
            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_collar".as_ptr(),
                TURN_OFF,
            );
            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_galaktorso".as_ptr(),
                TURN_OFF,
            );
        } else {
            // NPC_SetSurfaceOnOff( ent, "helmet", TURN_OFF );
            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_shield".as_ptr(),
                TURN_OFF,
            );
            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_galakface".as_ptr(),
                TURN_ON,
            );
            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_galakhead".as_ptr(),
                TURN_ON,
            );
            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_eyes_mouth".as_ptr(),
                TURN_ON,
            );
            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_collar".as_ptr(),
                TURN_ON,
            );
            crate::NPC_utils::NPC_SetSurfaceOnOff(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                c"torso_galaktorso".as_ptr(),
                TURN_ON,
            );
        }
    }
}

/// Raven `GM_CreateExplosion`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:101-125`
pub fn GM_CreateExplosion(
    ctx: &mut GameContext,
    self_: EntityId,
    boltID: c_int,
    doSmall: qboolean,
) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
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
                    ctx.world.level.time,
                    core::ptr::null_mut(),
                    &(*self_).modelScale as *const vec3_t,
                ),
            );

            BG_GiveMeVectorFromMatrix(&boltMatrix as *const mdxaBone_t, ORIGIN as c_int, &mut org);
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
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:133-229`
pub fn GM_Dying(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        let level_time = ctx.world.level.time;
        // Raven `vec3_origin` — resolved via the crate prelude (pass-3 symbol
        // backfill).
        if level_time - (*self_).s.time < 4000 {
            // FIXME: need a real effect
            // self->s.powerups |= ( 1 << PW_SHOCKED );
            // self->client->ps.powerups[PW_SHOCKED] = level.time + 1000;
            let client = (*self_).client;
            (*client).ps.electrifyTime = level_time + 1000;
            if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(self_), c"dyingExplosion".as_ptr())
                != 0
            {
                let mut newBolt: c_int;
                let self_id = ctx.entity_id_of(self_);
                match ctx.world.bg_state.rng.Q_irand(1, 14) {
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
                            GM_CreateExplosion(ctx, ctx.entity_id_of(self_).unwrap(), newBolt, 1);
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ctx.entity_id_of(self_).unwrap(), c"r_hand".as_ptr(), TURN_OFF);
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
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ctx.entity_id_of(self_).unwrap(), c"r_arm_middle".as_ptr(), TURN_OFF);
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
                            GM_CreateExplosion(ctx, ctx.entity_id_of(self_).unwrap(), newBolt, 0);
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ctx.entity_id_of(self_).unwrap(), c"l_hand".as_ptr(), TURN_OFF);
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
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ctx.entity_id_of(self_).unwrap(), c"l_arm_wrist".as_ptr(), TURN_OFF);
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
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ctx.entity_id_of(self_).unwrap(), c"l_arm_middle".as_ptr(), TURN_OFF);
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
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, ctx.entity_id_of(self_).unwrap(), c"l_arm_augment".as_ptr(), TURN_OFF);
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
                        GM_CreateExplosion(ctx, ctx.entity_id_of(self_).unwrap(), newBolt, 0);
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
                        GM_CreateExplosion(ctx, ctx.entity_id_of(self_).unwrap(), newBolt, 0);
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
                        GM_CreateExplosion(ctx, ctx.entity_id_of(self_).unwrap(), newBolt, 0);
                    }
                    9 | 10 => {
                        let head_bolt = (*(*self_).client).renderInfo.headBolt;
                        GM_CreateExplosion(ctx, ctx.entity_id_of(self_).unwrap(), head_bolt, 0);
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
                        GM_CreateExplosion(ctx, ctx.entity_id_of(self_).unwrap(), newBolt, 1);
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
                        GM_CreateExplosion(ctx, ctx.entity_id_of(self_).unwrap(), newBolt, 1);
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
                        GM_CreateExplosion(ctx, ctx.entity_id_of(self_).unwrap(), newBolt, 1);
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
                        GM_CreateExplosion(ctx, ctx.entity_id_of(self_).unwrap(), newBolt, 1);
                    }
                    _ => {}
                }

                let delay = ctx.world.bg_state.rng.Q_irand(300, 1100);
                crate::g_timer::TIMER_Set(ctx, self_id, c"dyingExplosion".as_ptr(), delay);
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
            (*self_).think = Some(crate::ent_fn_enums::EntThink::G_FreeEntity).into();
        }
    }
}

/// Raven `NPC_GM_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:238-354`
// The disabled `if (0)`/PW_GALAK_SHIELD-shield-down branch and the dead
// `gPainPoint`/`point` store (both fully commented out in the oracle, and the
// `if ( point )` array-as-pointer check is always true) have zero observable
// effect; the `Q_irand`/`G_AddVoiceEvent`/timer paths below are the only
// live behavior, ported faithfully.
pub fn NPC_GM_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    // STAGE-1: EntityId params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let attacker: *mut gentity_t = unsafe { ent_resolve_opt(ctx, attacker) };
    unsafe {
        let inflictor = attacker;
        let hitLoc: c_int = 1; // Raven: `int hitLoc = 1;` — never reassigned in this fn
        let r#mod = ctx.world.globals.gPainMOD;
        let level_time = ctx.world.level.time;

        let npc = (*self_).NPC;

        if (*self_).lockCount == 0 && (*(*self_).client).ps.torsoTimer <= 0 {
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
                    let self_id = ctx.entity_id_of(self_).unwrap();
                    let delay = ctx.world.bg_state.rng.Q_irand(3000, 5000);
                    // §19: oracle derefs `self->NPC->blockedSpeechDebounceTime`
                    // unconditionally; the null guard is defensive.
                    // Source: oracle/codemp/game/NPC_AI_GalakMech.c:307
                    if !npc.is_null() {
                        (*npc).blockedSpeechDebounceTime = 0;
                    }
                    crate::NPC_sounds::G_AddVoiceEvent(ctx, self_id, speech, delay);
                    (*self_).delay = level_time + ctx.world.bg_state.rng.Q_irand(5000, 7000);
                }
            } else {
                crate::NPC_reactions::NPC_Pain(
                    ctx,
                    ctx.entity_id_of(self_).unwrap(),
                    ctx.entity_id_of(attacker),
                    damage,
                );
            }
        } else if hitLoc == HL_GENERIC1 {
            crate::NPC_reactions::NPC_SetPainEvent(ctx, ctx.entity_id_of(self_).unwrap());
            // self->s.powerups |= ( 1 << PW_SHOCKED );
            // self->client->ps.powerups[PW_SHOCKED] = level.time + ctx.world.bg_state.rng.Q_irand( 500, 2500 );
            (*(*self_).client).ps.electrifyTime =
                level_time + ctx.world.bg_state.rng.Q_irand(500, 2500);
        }

        if !inflictor.is_null()
            && (*inflictor).lastEnemy == ent_id_opt(ctx.world.g_entities.as_ptr(), self_)
        {
            // He force-pushed my own lobfires back at me
            let npc = (*self_).NPC;
            if r#mod == meansOfDeath_t::MOD_REPEATER_ALT as c_int
                && ctx.world.bg_state.rng.Q_irand(0, 2) == 0
            {
                if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(self_), c"noRapid".as_ptr())
                    != 0
                {
                    if !npc.is_null() {
                        (*npc).scriptFlags &= !SCF_ALT_FIRE;
                    }
                    (*self_).alt_fire = 0;
                    let self_id = ctx.entity_id_of(self_);
                    let delay = ctx.world.bg_state.rng.Q_irand(2000, 6000);
                    crate::g_timer::TIMER_Set(ctx, self_id, c"noLob".as_ptr(), delay);
                } else {
                    let self_id = ctx.entity_id_of(self_);
                    let delay = ctx.world.bg_state.rng.Q_irand(1000, 2000);
                    // hopefully this will make us fire the laser
                    crate::g_timer::TIMER_Set(ctx, self_id, c"noLob".as_ptr(), delay);
                }
            } else if r#mod == meansOfDeath_t::MOD_REPEATER as c_int
                && ctx.world.bg_state.rng.Q_irand(0, 5) == 0
            {
                if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(self_), c"noLob".as_ptr()) != 0
                {
                    if !npc.is_null() {
                        (*npc).scriptFlags |= SCF_ALT_FIRE;
                    }
                    (*self_).alt_fire = 1;
                    let self_id = ctx.entity_id_of(self_);
                    let delay = ctx.world.bg_state.rng.Q_irand(2000, 6000);
                    crate::g_timer::TIMER_Set(ctx, self_id, c"noRapid".as_ptr(), delay);
                } else {
                    let self_id = ctx.entity_id_of(self_);
                    let delay = ctx.world.bg_state.rng.Q_irand(1000, 2000);
                    // hopefully this will make us fire the laser
                    crate::g_timer::TIMER_Set(ctx, self_id, c"noRapid".as_ptr(), delay);
                }
            }
        }
    }
}

/// Raven `GM_HoldPosition`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:362-369`
pub fn GM_HoldPosition(ctx: &mut GameContext) {
    unsafe {
        let npc_ent = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        crate::NPC_combat::NPC_FreeCombatPoint(ctx, (*npc_info).combatPoint, 1);
        let pending = trap::ICARUS_TaskIDPending(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs::new(
                npc_ent.cast(),
                taskID_t::TID_MOVE_NAV as c_int,
            ),
        );
        if pending == 0 {
            // don't have a script waiting for me to get to my point, okay to stop trying and stand
            (*npc_info).goalEntity = None;
        }
    }
}

/// Raven `GM_Move`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:376-408`
pub fn GM_Move(ctx: &mut GameContext) -> qboolean {
    unsafe {
        let npc_ent = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;

        (*npc_info).combatMove = qtrue; // always move straight toward our goal

        let moved = crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);

        // Get the move info
        let mut info: navInfo_t = core::mem::zeroed();
        crate::NPC_move::NAV_GetLastMove(ctx, &mut info);

        // FIXME: if we bump into another one of our guys and can't get around him, just stop!
        // If we hit our target, then stop and fire!
        if (info.flags & NIF_COLLISION) != 0 {
            if ent_id_opt(ctx.world.g_entities.as_ptr(), info.blocker) == (*npc_ent).enemy {
                GM_HoldPosition(ctx);
            }
        }

        // If our move failed, then reset
        if moved == qfalse {
            // FIXME: if we're going to a combat point, need to pick a different one
            if trap::ICARUS_TaskIDPending(
                ctx.engine,
                mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs::new(
                    npc_ent.cast(),
                    taskID_t::TID_MOVE_NAV as c_int,
                ),
            ) == 0
            {
                // can't transfer movegoal or stop when a script we're running is waiting to complete
                GM_HoldPosition(ctx);
            }
        }

        moved
    }
}

/// Raven `NPC_BSGM_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:416-432`
pub fn NPC_BSGM_Patrol(ctx: &mut GameContext) {
    if crate::NPC_AI_Stormtrooper::NPC_CheckPlayerTeamStealth(ctx) != 0 {
        crate::NPC_utils::NPC_UpdateAngles(ctx, 1, 1);
        return;
    }

    // If we have somewhere to go, then do that
    let goal = crate::NPC_goal::UpdateGoal(ctx);
    if !goal.is_null() {
        ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
        crate::NPC_move::NPC_MoveToGoal(ctx, 1);
    }

    crate::NPC_utils::NPC_UpdateAngles(ctx, 1, 1);
}

/// Raven `GM_CheckMoveState`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:440-460`
pub fn GM_CheckMoveState(ctx: &mut GameContext) {
    unsafe {
        let npc_ent = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;

        if trap::ICARUS_TaskIDPending(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs::new(
                npc_ent.cast(),
                taskID_t::TID_MOVE_NAV as c_int,
            ),
        ) != 0
        {
            // moving toward a goal that a script is waiting on, so don't stop for anything!
            ctx.world.globals.move4 = 1;
        }

        // See if we're moving towards a goal, not the enemy
        if (*npc_info).goalEntity != (*npc_ent).enemy && !(*npc_info).goalEntity.is_none() {
            // Did we make it?
            // Guarded by `!goalEntity.is_none()` above.
            let hit_goal = crate::g_nav::NAV_HitNavGoal(
                (*npc_ent).r.currentOrigin,
                (*npc_ent).r.mins,
                (*npc_ent).r.maxs,
                ctx.world.g_entities[(*npc_info).goalEntity.unwrap().index()]
                    .r
                    .currentOrigin,
                16,
                0,
            );
            let script_pending = trap::ICARUS_TaskIDPending(
                ctx.engine,
                mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs::new(
                    npc_ent.cast(),
                    taskID_t::TID_MOVE_NAV as c_int,
                ),
            ) != 0;
            if hit_goal != 0
                || (!script_pending
                    && ctx.world.globals.enemyLOS4 != 0
                    && ctx.world.globals.enemyDist4 <= 10000.0)
            {
                // either hit our navgoal or our navgoal was not a crucial (scripted) one (maybe a
                // combat point) and we're scouting and found our enemy
                crate::NPC_goal::NPC_ReachedGoal(ctx);
                let npc_id = ctx.entity_id_of(npc_ent);
                let delay = ctx.world.bg_state.rng.Q_irand(250, 500);
                // don't attack right away
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc_id,
                    c"attackDelay".as_ptr(),
                    delay, // FIXME: Slant for difficulty levels
                );
                return;
            }
        }
    }
}

// PORT-NOTE(unresolved-callee-type): calls `NPC_ShotEntity`/
// `CalcEntitySpot`/`trap_Trace` with `trace_t`/`SPOT_HEAD` shapes plus this
// file's own `enemyCS4`/`hitAlly4`/`impactPos4`/`faceEnemy4`/`shoot4` scratch
// (GameWorld cross-frame state) — the packet's call surface resolves the callee
// signatures, but `impactPos4` is grouped as a "bg-owned/const" global
// (packet's own STATE FIELDS section, not a `ctx.world.globals` field),
// leaving its owning placement unsettled; guessing would risk silently
// wrong parity across every caller. Deferred rather than guessed per §A2.
/// Raven `GM_CheckFireState`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:468-556`
pub fn GM_CheckFireState(ctx: &mut GameContext) {
    unsafe {
        let npc_ent = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let level_time = ctx.world.level.time;
        let enemyCS4 = ctx.world.globals.enemyCS4;
        let hitAlly4 = ctx.world.globals.hitAlly4;
        let mut faceEnemy4 = ctx.world.globals.faceEnemy4;
        let mut shoot4 = ctx.world.globals.shoot4;

        if enemyCS4 != 0 {
            // if have a clear shot, always try
            return;
        }

        let client = (*npc_ent).client;
        if VectorCompare((*client).ps.velocity, vec3_origin) == qfalse {
            // if moving at all, don't do this
            return;
        }

        // See if we should continue to fire on their last position
        if hitAlly4 == 0 && (*npc_info).enemyLastSeenTime > 0 {
            if level_time - (*npc_info).enemyLastSeenTime < 10000 {
                if ctx.world.bg_state.rng.Q_irand(0, 10) == 0 {
                    // Fire on the last known position
                    let mut muzzle: vec3_t = [0.0; 3];
                    let mut dir: vec3_t = [0.0; 3];
                    let mut angles: vec3_t = [0.0; 3];
                    let mut tooClose: qboolean = qfalse;
                    let mut tooFar: qboolean = qfalse;
                    let mut distThreshold: f32;
                    let mut dist: f32;

                    crate::NPC_utils::CalcEntitySpot(
                        ctx,
                        ctx.entity_id_of(npc_ent),
                        SPOT_HEAD,
                        &mut muzzle,
                    );
                    if VectorCompare(ctx.world.scratch.impact_pos_4, vec3_origin) != 0 {
                        // never checked ShotEntity this frame, so must do a trace...
                        let mut tr: trace_t = core::mem::zeroed();
                        let mut forward: vec3_t = [0.0; 3];
                        let mut end: vec3_t = [0.0; 3];
                        AngleVectors((*client).ps.viewangles, Some(&mut forward), None, None);
                        _VectorMA(muzzle, 8192.0, forward, &mut end);
                        trap::Trace(
                            ctx.engine,
                            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                &mut tr as *mut trace_t,
                                &muzzle as *const vec3_t,
                                &vec3_origin as *const vec3_t,
                                &vec3_origin as *const vec3_t,
                                &end as *const vec3_t,
                                (*npc_ent).s.number,
                                MASK_SHOT,
                            ),
                        );
                        _VectorCopy(tr.endpos, &mut ctx.world.scratch.impact_pos_4);
                    }

                    // see if impact would be too close to me
                    distThreshold = 16384.0; // 128*128, default
                    if (*npc_ent).s.weapon == WP_REPEATER as c_int {
                        if ((*npc_info).scriptFlags & SCF_ALT_FIRE) != 0 {
                            distThreshold = 65536.0; // 256*256
                        }
                    }

                    dist = DistanceSquared(ctx.world.scratch.impact_pos_4, muzzle);

                    if dist < distThreshold {
                        // impact would be too close to me
                        tooClose = qtrue;
                    } else if level_time - (*npc_info).enemyLastSeenTime > 5000 {
                        // we've haven't seen them in the last 5 seconds
                        // see if it's too far from where he is
                        distThreshold = 65536.0; // 256*256, default
                        if (*npc_ent).s.weapon == WP_REPEATER as c_int {
                            if ((*npc_info).scriptFlags & SCF_ALT_FIRE) != 0 {
                                distThreshold = 262144.0; // 512*512
                            }
                        }
                        dist = DistanceSquared(
                            ctx.world.scratch.impact_pos_4,
                            (*npc_info).enemyLastSeenLocation,
                        );
                        if dist > distThreshold {
                            // impact would be too far from enemy
                            tooFar = qtrue;
                        }
                    }

                    if tooClose == qfalse && tooFar == qfalse {
                        // okay to shoot at last pos
                        _VectorSubtract((*npc_info).enemyLastSeenLocation, muzzle, &mut dir);
                        VectorNormalize(&mut dir);
                        vectoangles(dir, &mut angles);

                        (*npc_info).desiredYaw = angles[YAW];
                        (*npc_info).desiredPitch = angles[PITCH];

                        shoot4 = qtrue;
                        faceEnemy4 = qfalse;
                        ctx.world.globals.shoot4 = shoot4;
                        ctx.world.globals.faceEnemy4 = faceEnemy4;
                        return;
                    }
                }
            }
        }

        ctx.world.globals.faceEnemy4 = faceEnemy4;
        ctx.world.globals.shoot4 = shoot4;
    }
}

/// Raven `NPC_GM_StartLaser`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:558-573`
pub fn NPC_GM_StartLaser(ctx: &mut GameContext) {
    unsafe {
        let npc_ent = ctx.world.globals.NPC;
        let npc = (*npc_ent).NPC;
        if (*npc_ent).lockCount == 0 {
            // haven't already started a laser attack
            // warm up for the beam attack
            #[cfg(any())]
            {
                // NPC_SetAnim( NPC, SETANIM_TORSO, TORSO_RAISEWEAP2, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
            }
            let client = (*npc_ent).client;
            let torso_timer = (*client).ps.torsoTimer;
            crate::g_timer::TIMER_Set(
                ctx,
                ctx.entity_id_of(npc_ent),
                c"beamDelay".as_ptr(),
                torso_timer,
            );
            crate::g_timer::TIMER_Set(
                ctx,
                ctx.entity_id_of(npc_ent),
                c"attackDelay".as_ptr(),
                torso_timer + 3000,
            );
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
                ctx.entity_id_of(npc_ent).unwrap(),
                CHAN_AUTO,
                c"sound/weapons/galak/lasercharge.wav".as_ptr(),
            );
        }
    }
}

/// Raven `GM_StartGloat`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:575-587`
pub fn GM_StartGloat(ctx: &mut GameContext) {
    unsafe {
        let npc_ent = ctx.world.globals.NPC;
        (*npc_ent).wait = 0.0;
        crate::NPC_utils::NPC_SetSurfaceOnOff(
            ctx,
            ctx.entity_id_of(npc_ent).unwrap(),
            c"torso_galakface".as_ptr(),
            TURN_ON,
        );
        crate::NPC_utils::NPC_SetSurfaceOnOff(
            ctx,
            ctx.entity_id_of(npc_ent).unwrap(),
            c"torso_galakhead".as_ptr(),
            TURN_ON,
        );
        crate::NPC_utils::NPC_SetSurfaceOnOff(
            ctx,
            ctx.entity_id_of(npc_ent).unwrap(),
            c"torso_eyes_mouth".as_ptr(),
            TURN_ON,
        );
        crate::NPC_utils::NPC_SetSurfaceOnOff(
            ctx,
            ctx.entity_id_of(npc_ent).unwrap(),
            c"torso_collar".as_ptr(),
            TURN_ON,
        );
        crate::NPC_utils::NPC_SetSurfaceOnOff(
            ctx,
            ctx.entity_id_of(npc_ent).unwrap(),
            c"torso_galaktorso".as_ptr(),
            TURN_ON,
        );

        crate::npc_c::NPC_SetAnim(
            ctx,
            ctx.entity_id_of(npc_ent).unwrap(),
            SETANIM_BOTH,
            animNumber_t::BOTH_STAND2TO1 as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        let client = (*npc_ent).client;
        (*client).ps.legsTimer += 500;
        (*client).ps.torsoTimer += 500;
    }
}

/// Raven `NPC_BSGM_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:594-1229`
pub fn NPC_BSGM_Attack(ctx: &mut GameContext) {
    unsafe {
        let npc_ent = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let level_time = ctx.world.level.time;
        let g_entities_base = ctx.world.g_entities.as_mut_ptr();
        let client = (*npc_ent).client;

        // Don't do anything if we're hurt
        if (*npc_ent).painDebounceTime > level_time {
            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        // Victory animation section disabled with #if 0 in oracle

        // If we don't have an enemy, just idle
        if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qfalse) == qfalse || (*npc_ent).enemy.is_none()
        {
            (*npc_ent).enemy = None;
            NPC_BSGM_Patrol(ctx);
            return;
        }
        // Guaranteed `Some` from here to the end of the function by the guard above.
        let enemy_ent = g_entities_base.add((*npc_ent).enemy.unwrap().index());

        // Initialize combat state
        ctx.world.globals.enemyLOS4 = qfalse;
        ctx.world.globals.enemyCS4 = qfalse;
        ctx.world.globals.move4 = qtrue;
        ctx.world.globals.faceEnemy4 = qfalse;
        ctx.world.globals.shoot4 = qfalse;
        ctx.world.globals.hitAlly4 = qfalse;
        VectorClear(&mut ctx.world.scratch.impact_pos_4);
        ctx.world.globals.enemyDist4 =
            DistanceSquared((*npc_ent).r.currentOrigin, (*enemy_ent).r.currentOrigin);

        // Melee attack logic disabled with if(0)

        // Laser beam attack logic
        if (*npc_ent).lockCount != 0 {
            // already shooting laser
            ctx.world.globals.shoot4 = qfalse;
            if (*npc_ent).lockCount == 1 {
                // charging up
                if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc_ent), c"beamDelay".as_ptr())
                    != 0
                {
                    // time to start the beam
                    let laserAnim: c_int = BOTH_ATTACK2 as c_int;
                    crate::npc_c::NPC_SetAnim(
                        ctx,
                        ctx.entity_id_of(npc_ent).unwrap(),
                        SETANIM_BOTH,
                        laserAnim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    let npc_id = ctx.entity_id_of(npc_ent);
                    let delay =
                        (*client).ps.torsoTimer + ctx.world.bg_state.rng.Q_irand(1000, 3000);
                    crate::g_timer::TIMER_Set(ctx, npc_id, c"attackDelay".as_ptr(), delay);
                    // turn on beam effect
                    (*npc_ent).lockCount = 2;
                    crate::g_utils::G_PlayEffectID(
                        crate::g_utils::G_EffectIndex(c"galak/trace_beam".as_ptr()),
                        (*npc_ent).r.currentOrigin,
                        vec3_origin,
                    );
                    (*npc_ent).s.loopSound = crate::g_utils::G_SoundIndex(
                        c"sound/weapons/galak/lasercutting.wav".as_ptr(),
                    );
                    if (*npc_info).coverTarg.is_none() {
                        // for moving looping sound at end of trace
                        let cover_ent = crate::g_utils::G_Spawn(ctx);
                        if !cover_ent.is_null() {
                            (*npc_info).coverTarg = Some(ent_id(g_entities_base, cover_ent));
                            crate::g_utils::G_SetOrigin(
                                &mut *(cover_ent),
                                (*client).renderInfo.muzzlePoint,
                            );
                            (*cover_ent).r.svFlags |= SVF_BROADCAST;
                            (*cover_ent).s.loopSound = crate::g_utils::G_SoundIndex(
                                c"sound/weapons/galak/lasercutting.wav".as_ptr(),
                            );
                        }
                    }
                }
            } else {
                // in the actual attack now
                if (*client).ps.torsoTimer <= 0 {
                    // attack done!
                    (*npc_ent).lockCount = 0;
                    if let Some(cover_id) = (*npc_info).coverTarg {
                        let cover_ent_ptr = g_entities_base.add(cover_id.0 as usize);
                        crate::g_utils::G_FreeEntity(ctx, ctx.entity_id_of(cover_ent_ptr));
                    }
                    (*npc_ent).s.loopSound = 0;
                    crate::g_timer::TIMER_Set(
                        ctx,
                        ctx.entity_id_of(npc_ent),
                        c"attackDelay".as_ptr(),
                        (*client).ps.torsoTimer,
                    );
                } else {
                    // attack still going
                    // do the trace and damage
                    let mut trace: trace_t = core::mem::zeroed();
                    let mut end: vec3_t = [0.0; 3];
                    let mins: vec3_t = [-3.0, -3.0, -3.0];
                    let maxs: vec3_t = [3.0, 3.0, 3.0];
                    _VectorMA(
                        (*client).renderInfo.muzzlePoint,
                        1024.0,
                        (*client).renderInfo.muzzleDir,
                        &mut end,
                    );
                    trap::Trace(
                        ctx.engine,
                        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                            &mut trace as *mut trace_t,
                            &(*client).renderInfo.muzzlePoint as *const vec3_t,
                            &mins as *const vec3_t,
                            &maxs as *const vec3_t,
                            &end as *const vec3_t,
                            (*npc_ent).s.number,
                            MASK_SHOT,
                        ),
                    );
                    if trace.allsolid != 0 || trace.startsolid != 0 {
                        // oops, in a wall
                        if let Some(cover_id) = (*npc_info).coverTarg {
                            let cover_ent_ptr = g_entities_base.add(cover_id.0 as usize);
                            crate::g_utils::G_SetOrigin(
                                &mut *(cover_ent_ptr),
                                (*client).renderInfo.muzzlePoint,
                            );
                        }
                    } else {
                        // clear
                        if trace.fraction < 1.0 {
                            // hit something
                            let trace_ent = g_entities_base.add(trace.entityNum as usize);
                            if !trace_ent.is_null() && (*trace_ent).takedamage != 0 {
                                // damage it
                                crate::g_utils::G_SoundAtLoc(
                                    ctx,
                                    trace.endpos,
                                    CHAN_AUTO,
                                    crate::g_utils::G_SoundIndex(
                                        c"sound/weapons/galak/laserdamage.wav".as_ptr(),
                                    ),
                                );
                                crate::g_combat::G_Damage(
                                    ctx,
                                    ctx.entity_id_of(trace_ent),
                                    ctx.entity_id_of(npc_ent),
                                    ctx.entity_id_of(npc_ent),
                                    Some(&mut (*client).renderInfo.muzzleDir),
                                    trace.endpos,
                                    10,
                                    0,
                                    meansOfDeath_t::MOD_UNKNOWN as c_int,
                                );
                            }
                        }
                        if let Some(cover_id) = (*npc_info).coverTarg {
                            let cover_ent_ptr = g_entities_base.add(cover_id.0 as usize);
                            crate::g_utils::G_SetOrigin(&mut *(cover_ent_ptr), trace.endpos);
                        }
                        if ctx.world.bg_state.rng.Q_irand(0, 5) == 0 {
                            crate::g_utils::G_SoundAtLoc(
                                ctx,
                                trace.endpos,
                                CHAN_AUTO,
                                crate::g_utils::G_SoundIndex(
                                    c"sound/weapons/galak/laserdamage.wav".as_ptr(),
                                ),
                            );
                        }
                    }
                }
            }
        } else {
            // Okay, we're not in a special attack, see if we should switch weapons or start a special attack
            // (Raven's WP_REPEATER "he's deflecting my shots" branch is inside a
            // `/* ... */` block comment in the oracle — dead code, not transcribed;
            // its `Q_irand(0,50)`/`Q_irand(2000,6000)`/`Q_irand(0,1)` draws never run.)
            // Source: `oracle/codemp/game/NPC_AI_GalakMech.c:813-828`
            if ctx.world.globals.enemyDist4 < MELEE_DIST_SQUARED
                && InFront(
                    (*enemy_ent).r.currentOrigin,
                    (*npc_ent).r.currentOrigin,
                    (*client).ps.viewangles,
                    0.3,
                ) != 0
                && (*enemy_ent).localAnimIndex <= 1
            {
                // our shield is down, and enemy within 80, if very close, use melee attack to slap away
                if crate::g_timer::TIMER_Done(
                    ctx,
                    ctx.entity_id_of(npc_ent),
                    c"attackDelay".as_ptr(),
                ) != 0
                {
                    // animate me
                    let swingAnim: c_int = BOTH_ATTACK1 as c_int;
                    crate::npc_c::NPC_SetAnim(
                        ctx,
                        ctx.entity_id_of(npc_ent).unwrap(),
                        SETANIM_BOTH,
                        swingAnim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    let npc_id = ctx.entity_id_of(npc_ent);
                    let delay =
                        (*client).ps.torsoTimer + ctx.world.bg_state.rng.Q_irand(1000, 3000);
                    crate::g_timer::TIMER_Set(ctx, npc_id, c"attackDelay".as_ptr(), delay);
                    // delay the hurt until the proper point in the anim
                    crate::g_timer::TIMER_Set(
                        ctx,
                        ctx.entity_id_of(npc_ent),
                        c"smackTime".as_ptr(),
                        600,
                    );
                    (*npc_info).blockedDebounceTime = 0;
                }
            } else if (*npc_ent).lockCount == 0
                && (*npc_ent).locationDamage[HL_GENERIC1 as usize] > GENERATOR_HEALTH
                && crate::g_timer::TIMER_Done(
                    ctx,
                    ctx.entity_id_of(npc_ent),
                    c"attackDelay".as_ptr(),
                ) != 0
                && InFront(
                    (*enemy_ent).r.currentOrigin,
                    (*npc_ent).r.currentOrigin,
                    (*client).ps.viewangles,
                    0.3,
                ) != 0
                && (ctx
                    .world
                    .bg_state
                    .rng
                    .Q_irand(0, 10 * (2 - ctx.world.cvars.g_spskill.integer))
                    == 0
                    && ctx.world.globals.enemyDist4 > MIN_LOB_DIST_SQUARED
                    && ctx.world.globals.enemyDist4 < MAX_LOB_DIST_SQUARED
                    || crate::g_timer::TIMER_Done(
                        ctx,
                        ctx.entity_id_of(npc_ent),
                        c"noLob".as_ptr(),
                    ) == 0
                        && crate::g_timer::TIMER_Done(
                            ctx,
                            ctx.entity_id_of(npc_ent),
                            c"noRapid".as_ptr(),
                        ) == 0)
                && (*enemy_ent).s.weapon != WP_TURRET as c_int
            {
                // sometimes use the laser beam attack, but only after he's taken down our generator
                ctx.world.globals.shoot4 = qfalse;
                NPC_GM_StartLaser(ctx);
            } else if ctx.world.globals.enemyDist4 < MIN_LOB_DIST_SQUARED
                && ((*enemy_ent).s.weapon != WP_TURRET as c_int
                    || crate::q_shared::Q_stricmp(c"PAS".as_ptr(), (*enemy_ent).classname) != 0)
                && crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc_ent), c"noRapid".as_ptr())
                    != 0
            {
                // enemy within 256
                if (*client).ps.weapon == WP_REPEATER as c_int
                    && ((*npc_info).scriptFlags & SCF_ALT_FIRE as i32) as c_int != 0
                {
                    // shooting an explosive, but enemy too close, switch to primary fire
                    (*npc_info).scriptFlags &= !(SCF_ALT_FIRE as i32);
                    (*npc_ent).alt_fire = qfalse;
                    crate::NPC_combat::NPC_ChangeWeapon(WP_REPEATER);
                }
            } else if (ctx.world.globals.enemyDist4 > MAX_LOB_DIST_SQUARED
                || ((*enemy_ent).s.weapon == WP_TURRET as c_int
                    && crate::q_shared::Q_stricmp(c"PAS".as_ptr(), (*enemy_ent).classname) == 0))
                && crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc_ent), c"noLob".as_ptr())
                    != 0
            {
                // enemy more than 448 away and we are ready to try lob fire again
                if (*client).ps.weapon == WP_REPEATER as c_int
                    && ((*npc_info).scriptFlags & SCF_ALT_FIRE as i32) as c_int == 0
                {
                    // enemy far enough away to use lobby explosives
                    (*npc_info).scriptFlags |= SCF_ALT_FIRE as i32;
                    (*npc_ent).alt_fire = qtrue;
                    crate::NPC_combat::NPC_ChangeWeapon(WP_REPEATER);
                }
            }
        }

        // can we see our target?
        if crate::NPC_utils::NPC_ClearLOS4(ctx, ctx.entity_id_of(enemy_ent)) != 0 {
            (*npc_info).enemyLastSeenTime = level_time; // used here for aim debouncing, not always a clear LOS
            ctx.world.globals.enemyLOS4 = qtrue;

            if (*client).ps.weapon == WP_NONE as c_int {
                ctx.world.globals.enemyCS4 = qfalse; // not true, but should stop us from firing
                crate::NPC_combat::NPC_AimAdjust(ctx, -1);
            } else {
                // can we shoot our target?
                if (*client).ps.weapon == WP_REPEATER as c_int
                    && ((*npc_info).scriptFlags & SCF_ALT_FIRE as i32) as c_int != 0
                    && ctx.world.globals.enemyDist4 < MIN_LOB_DIST_SQUARED
                {
                    ctx.world.globals.enemyCS4 = qfalse; // not true, but should stop us from firing
                    ctx.world.globals.hitAlly4 = qtrue; // us!
                } else {
                    // `impactPos4` is a file static shared with GM_CheckFireState; copy it
                    // out, let NPC_ShotEntity write through the local, then store back so the
                    // later reads see the same value C's file-static vec3_t would hold.
                    let mut impactPos4 = ctx.world.scratch.impact_pos_4;
                    let hit = crate::NPC_combat::NPC_ShotEntity(
                        ctx,
                        ctx.entity_id_of(enemy_ent),
                        Some(&mut impactPos4),
                    );
                    ctx.world.scratch.impact_pos_4 = impactPos4;
                    let hit_ent = g_entities_base.add(hit as usize);
                    if hit == (*enemy_ent).s.number as c_int
                        || (!hit_ent.is_null()
                            && (*hit_ent).client != core::ptr::null_mut()
                            && (*((*hit_ent).client)).playerTeam == (*client).enemyTeam)
                        || (!hit_ent.is_null() && (*hit_ent).takedamage != 0)
                    {
                        // can hit enemy or will hit glass or other breakable, so shoot anyway
                        ctx.world.globals.enemyCS4 = qtrue;
                        crate::NPC_combat::NPC_AimAdjust(ctx, 2); // adjust aim better longer we have clear shot at enemy
                        _VectorCopy(
                            (*enemy_ent).r.currentOrigin,
                            &mut (*npc_info).enemyLastSeenLocation,
                        );
                    } else {
                        // Hmm, have to get around this bastard
                        crate::NPC_combat::NPC_AimAdjust(ctx, 1); // adjust aim better longer we can see enemy
                        if !hit_ent.is_null()
                            && (*hit_ent).client != core::ptr::null_mut()
                            && (*((*hit_ent).client)).playerTeam == (*client).playerTeam
                        {
                            // would hit an ally, don't fire!!!
                            ctx.world.globals.hitAlly4 = qtrue;
                        }
                    }
                }
            }
        } else if trap::InPVS(
            ctx.engine,
            mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                &(*enemy_ent).r.currentOrigin as *const vec3_t,
                &(*npc_ent).r.currentOrigin as *const vec3_t,
            ),
        ) != 0
        {
            // C only declares hit/hitEnt here; the single NPC_ShotEntity call is below,
            // after enemyLastSeenTime is set.
            if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc_ent), c"talkDebounce".as_ptr())
                != 0
                && ctx.world.bg_state.rng.Q_irand(0, 10) == 0
            {
                if (*npc_info).enemyCheckDebounceTime < 8 {
                    let speech: c_int = match (*npc_info).enemyCheckDebounceTime {
                        0 | 1 | 2 => {
                            entity_event_t::EV_CHASE1 as c_int + (*npc_info).enemyCheckDebounceTime
                        }
                        3 | 4 | 5 => {
                            entity_event_t::EV_COVER1 as c_int
                                + ((*npc_info).enemyCheckDebounceTime - 3)
                        }
                        6 | 7 => {
                            entity_event_t::EV_ESCAPING1 as c_int
                                + ((*npc_info).enemyCheckDebounceTime - 6)
                        }
                        _ => -1,
                    };
                    (*npc_info).enemyCheckDebounceTime += 1;
                    if speech != -1 {
                        let npc_id = ctx.entity_id_of(npc_ent).unwrap();
                        let delay = ctx.world.bg_state.rng.Q_irand(3000, 5000);
                        crate::NPC_sounds::G_AddVoiceEvent(ctx, npc_id, speech, delay);
                        let npc_id_2 = ctx.entity_id_of(npc_ent);
                        let delay_2 = ctx.world.bg_state.rng.Q_irand(5000, 7000);
                        crate::g_timer::TIMER_Set(ctx, npc_id_2, c"talkDebounce".as_ptr(), delay_2);
                    }
                }
            }

            (*npc_info).enemyLastSeenTime = level_time;

            // `impactPos4` is a file static shared with GM_CheckFireState; copy it out, let
            // NPC_ShotEntity write through the local, then store back (matches C's file-static).
            let mut impactPos4 = ctx.world.scratch.impact_pos_4;
            let hit = crate::NPC_combat::NPC_ShotEntity(
                ctx,
                ctx.entity_id_of(enemy_ent),
                Some(&mut impactPos4),
            );
            ctx.world.scratch.impact_pos_4 = impactPos4;
            let hit_ent = g_entities_base.add(hit as usize);
            if hit == (*enemy_ent).s.number as c_int
                || (!hit_ent.is_null()
                    && (*hit_ent).client != core::ptr::null_mut()
                    && (*((*hit_ent).client)).playerTeam == (*client).enemyTeam)
                || (!hit_ent.is_null() && (*hit_ent).takedamage != 0)
            {
                // can hit enemy or will hit glass or other breakable, so shoot anyway
                ctx.world.globals.enemyCS4 = qtrue;
            } else {
                ctx.world.globals.faceEnemy4 = qtrue;
                crate::NPC_combat::NPC_AimAdjust(ctx, -1); // adjust aim worse longer we cannot see enemy
            }
        }

        if ctx.world.globals.enemyLOS4 != 0 {
            ctx.world.globals.faceEnemy4 = qtrue;
        } else {
            if !(*npc_info).goalEntity.is_some() {
                (*npc_info).goalEntity = (*npc_ent).enemy;
            }
            if (*npc_info).goalEntity == (*npc_ent).enemy {
                // for now, always chase the enemy
                ctx.world.globals.move4 = qtrue;
            }
        }
        if ctx.world.globals.enemyCS4 != 0 {
            ctx.world.globals.shoot4 = qtrue;
        } else {
            if !(*npc_info).goalEntity.is_some() {
                (*npc_info).goalEntity = (*npc_ent).enemy;
            }
            if (*npc_info).goalEntity == (*npc_ent).enemy {
                // for now, always chase the enemy
                ctx.world.globals.move4 = qtrue;
            }
        }

        // Check for movement to take care of
        GM_CheckMoveState(ctx);

        // See if we should override shooting decision with any special considerations
        GM_CheckFireState(ctx);

        if (*client).ps.weapon == WP_REPEATER as c_int
            && ((*npc_info).scriptFlags & SCF_ALT_FIRE as i32) as c_int != 0
            && ctx.world.globals.shoot4 != 0
            && crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc_ent), c"attackDelay".as_ptr())
                != 0
        {
            let mut muzzle: vec3_t = [0.0; 3];
            let mut angles: vec3_t = [0.0; 3];
            let mut target: vec3_t = [0.0; 3];
            let mut velocity: vec3_t = [0.0; 3];
            let mins = [-REPEATER_ALT_SIZE, -REPEATER_ALT_SIZE, -REPEATER_ALT_SIZE];
            let maxs = [REPEATER_ALT_SIZE, REPEATER_ALT_SIZE, REPEATER_ALT_SIZE];

            crate::NPC_utils::CalcEntitySpot(
                ctx,
                ctx.entity_id_of(npc_ent),
                SPOT_WEAPON,
                &mut muzzle,
            );

            _VectorCopy((*enemy_ent).r.currentOrigin, &mut target);

            // C: `flrand(-5,5) + (crandom()*(6-currentAim)*2)` runs in `double`
            // (`crandom()` is `double`, `flrand`'s `float` widens); narrows to float.
            target[0] = (target[0] as f64
                + (ctx.world.bg_state.rng.flrand(-5.0, 5.0) as f64
                    + ctx.world.bg_state.rng.crandom() * (6 - (*npc_info).currentAim) as f64 * 2.0))
                as f32;
            target[1] = (target[1] as f64
                + (ctx.world.bg_state.rng.flrand(-5.0, 5.0) as f64
                    + ctx.world.bg_state.rng.crandom() * (6 - (*npc_info).currentAim) as f64 * 2.0))
                as f32;
            target[2] = (target[2] as f64
                + (ctx.world.bg_state.rng.flrand(-5.0, 5.0) as f64
                    + ctx.world.bg_state.rng.crandom() * (6 - (*npc_info).currentAim) as f64 * 2.0))
                as f32;

            // Find the desired angles
            let clearshot = crate::g_weapon::WP_LobFire(
                ctx,
                ctx.entity_id_of(npc_ent).unwrap(),
                muzzle,
                target,
                mins,
                maxs,
                MASK_SHOT | CONTENTS_LIGHTSABER,
                &mut velocity,
                qtrue,
                (*npc_ent).s.number,
                (*enemy_ent).s.number as c_int,
                300.0,
                1100.0,
                1500.0,
                qtrue,
            );
            if VectorCompare(vec3_origin, velocity) != 0
                || (clearshot == 0
                    && ctx.world.globals.enemyLOS4 != 0
                    && ctx.world.globals.enemyCS4 != 0)
            {
                // no clear lob shot and no lob shot that will hit something breakable
                if ctx.world.globals.enemyLOS4 != 0
                    && ctx.world.globals.enemyCS4 != 0
                    && crate::g_timer::TIMER_Done(
                        ctx,
                        ctx.entity_id_of(npc_ent),
                        c"noRapid".as_ptr(),
                    ) != 0
                {
                    // have a clear straight shot, so switch to primary
                    (*npc_info).scriptFlags &= !(SCF_ALT_FIRE as i32);
                    (*npc_ent).alt_fire = qfalse;
                    crate::NPC_combat::NPC_ChangeWeapon(WP_REPEATER);
                    let npc_id = ctx.entity_id_of(npc_ent);
                    let delay = ctx.world.bg_state.rng.Q_irand(500, 1000);
                    // keep this weap for a bit
                    crate::g_timer::TIMER_Set(ctx, npc_id, c"noLob".as_ptr(), delay);
                } else {
                    ctx.world.globals.shoot4 = qfalse;
                }
            } else {
                let mut vel_mut = velocity;
                vectoangles(vel_mut, &mut angles);

                (*npc_info).desiredYaw = crate::q_math::AngleNormalize360(angles[YAW]);
                (*npc_info).desiredPitch = crate::q_math::AngleNormalize360(angles[PITCH]);

                _VectorCopy(velocity, &mut (*client).hiddenDir);
                (*client).hiddenDist = VectorNormalize(&mut (*client).hiddenDir);
            }
        } else if ctx.world.globals.faceEnemy4 != 0 {
            // face the enemy
            crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);
        }

        if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc_ent), c"standTime".as_ptr()) == 0 {
            ctx.world.globals.move4 = qfalse;
        }
        if ((*npc_info).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
            // not supposed to chase my enemies
            if (*npc_info).goalEntity == (*npc_ent).enemy {
                // goal is my entity, so don't move
                ctx.world.globals.move4 = qfalse;
            }
        }

        if ctx.world.globals.move4 != 0 && (*npc_ent).lockCount == 0 {
            // move toward goal
            if (*npc_info).goalEntity.is_some() {
                ctx.world.globals.move4 = GM_Move(ctx);
            } else {
                ctx.world.globals.move4 = qfalse;
            }
        }

        if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc_ent), c"flee".as_ptr()) == 0 {
            // running away
            ctx.world.globals.faceEnemy4 = qfalse;
        }

        if ctx.world.globals.faceEnemy4 == 0 {
            // we want to face in the dir we're running
            if ctx.world.globals.move4 == 0 {
                // if we haven't moved, we should look in the direction we last looked?
                _VectorCopy((*client).ps.viewangles, &mut (*npc_info).lastPathAngles);
            }
            if ctx.world.globals.move4 != 0 {
                // don't run away and shoot
                (*npc_info).desiredYaw = (*npc_info).lastPathAngles[YAW];
                (*npc_info).desiredPitch = 0.0;
                ctx.world.globals.shoot4 = qfalse;
            }
        }
        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);

        if ((*npc_info).scriptFlags & SCF_DONT_FIRE) != 0 {
            ctx.world.globals.shoot4 = qfalse;
        }

        if (*npc_ent).enemy.is_some() && (*enemy_ent).enemy.is_some() {
            // The enemy's own enemy — a second EntityId hop off `enemy_ent`, resolved
            // only inside this guard (guaranteed `Some` by the check above).
            let enemy_of_enemy_ent = g_entities_base.add((*enemy_ent).enemy.unwrap().index());
            if (*enemy_ent).s.weapon == WP_SABER as c_int
                && (*enemy_of_enemy_ent).s.weapon == WP_SABER as c_int
            {
                // don't shoot at an enemy jedi who is fighting another jedi, for fear of injuring one or causing rogue blaster deflections (a la Obi Wan/Vader duel at end of ANH)
                ctx.world.globals.shoot4 = qfalse;
            }
        }

        if ctx.world.globals.shoot4 != 0 {
            // try to shoot if it's time
            if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc_ent), c"attackDelay".as_ptr())
                != 0
            {
                if ((*npc_info).scriptFlags & SCF_FIRE_WEAPON) == 0 {
                    // we've already fired, no need to do it again here
                    crate::NPC_combat::WeaponThink(ctx, qtrue);
                }
            }
        }

        // also:
        if (*enemy_ent).s.weapon == WP_TURRET as c_int
            && crate::q_shared::Q_stricmp(c"PAS".as_ptr(), (*enemy_ent).classname) == 0
        {
            // crush turrets
            if crate::NPC_goal::G_BoundsOverlap(
                (*npc_ent).r.absmin,
                (*npc_ent).r.absmax,
                (*enemy_ent).r.absmin,
                (*enemy_ent).r.absmax,
            ) != 0
            {
                // have to do this test because placed turrets are not solid to NPCs (so they don't obstruct navigation)
                if false {
                    (*client).ps.powerups[PW_BATTLESUIT as usize] = level_time + ARMOR_EFFECT_TIME;
                    crate::g_combat::G_Damage(
                        ctx,
                        ctx.entity_id_of(enemy_ent),
                        ctx.entity_id_of(npc_ent),
                        ctx.entity_id_of(npc_ent),
                        None,
                        (*npc_ent).r.currentOrigin,
                        100,
                        DAMAGE_NO_KNOCKBACK,
                        meansOfDeath_t::MOD_UNKNOWN as c_int,
                    );
                } else {
                    crate::g_combat::G_Damage(
                        ctx,
                        ctx.entity_id_of(enemy_ent),
                        ctx.entity_id_of(npc_ent),
                        ctx.entity_id_of(npc_ent),
                        None,
                        (*npc_ent).r.currentOrigin,
                        100,
                        DAMAGE_NO_KNOCKBACK,
                        meansOfDeath_t::MOD_CRUSH as c_int,
                    );
                }
            }
        } else if (*npc_info).touchedByPlayer.is_some()
            && (*npc_info).touchedByPlayer == (*npc_ent).enemy
        {
            // touched enemy
            if false {
                // zap him!
                let mut smackDir: vec3_t = [0.0; 3];
                crate::g_timer::TIMER_Set(
                    ctx,
                    ctx.entity_id_of(npc_ent),
                    c"attackDelay".as_ptr(),
                    (*client).ps.torsoTimer,
                );
                crate::g_timer::TIMER_Set(
                    ctx,
                    ctx.entity_id_of(npc_ent),
                    c"standTime".as_ptr(),
                    (*client).ps.legsTimer,
                );
                (*npc_info).touchedByPlayer = None;
                (*client).ps.powerups[PW_BATTLESUIT as usize] = level_time + ARMOR_EFFECT_TIME;

                _VectorSubtract(
                    (*enemy_ent).r.currentOrigin,
                    (*npc_ent).r.currentOrigin,
                    &mut smackDir,
                );
                smackDir[2] += 30.0;
                VectorNormalize(&mut smackDir);
                let enemy_id = ctx.entity_id_of(enemy_ent);
                let npc_id = ctx.entity_id_of(npc_ent);
                let npc_id_2 = ctx.entity_id_of(npc_ent);
                let delay =
                    (ctx.world.cvars.g_spskill.integer + 1) * ctx.world.bg_state.rng.Q_irand(5, 10);
                crate::g_combat::G_Damage(
                    ctx,
                    enemy_id,
                    npc_id,
                    npc_id_2,
                    Some(&mut smackDir),
                    (*npc_ent).r.currentOrigin,
                    delay,
                    DAMAGE_NO_KNOCKBACK,
                    meansOfDeath_t::MOD_UNKNOWN as c_int,
                );
                crate::g_utils::G_Throw(ctx, ctx.entity_id_of(enemy_ent).unwrap(), smackDir, 100.0);
                if (*enemy_ent).client != core::ptr::null_mut() {
                    (*((*enemy_ent).client)).ps.electrifyTime = level_time + 1000;
                }
                ctx.world.globals.ucmd.buttons = 0;
            }
        }

        if (*npc_info).movementSpeech < 3 && (*npc_info).blockedSpeechDebounceTime <= level_time {
            if (*npc_ent).enemy.is_some()
                && (*enemy_ent).health > 0
                && (*enemy_ent).painDebounceTime > level_time
            {
                if (*enemy_ent).health < 50 && (*npc_info).movementSpeech == 2 {
                    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
                    let delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                    crate::NPC_sounds::G_AddVoiceEvent(
                        ctx,
                        npc_id,
                        entity_event_t::EV_ANGER2 as c_int,
                        delay,
                    );
                    (*npc_info).movementSpeech = 3;
                } else if (*enemy_ent).health < 75 && (*npc_info).movementSpeech == 1 {
                    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
                    let delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                    crate::NPC_sounds::G_AddVoiceEvent(
                        ctx,
                        npc_id,
                        entity_event_t::EV_ANGER1 as c_int,
                        delay,
                    );
                    (*npc_info).movementSpeech = 2;
                } else if (*enemy_ent).health < 100 && (*npc_info).movementSpeech == 0 {
                    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
                    let delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                    crate::NPC_sounds::G_AddVoiceEvent(
                        ctx,
                        npc_id,
                        entity_event_t::EV_ANGER3 as c_int,
                        delay,
                    );
                    (*npc_info).movementSpeech = 1;
                }
            }
        }
    }
}

/// Raven `NPC_BSGM_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:1231-1297`
pub fn NPC_BSGM_Default(ctx: &mut GameContext) {
    unsafe {
        let npc_ent = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;

        if ((*npc_info).scriptFlags & SCF_FIRE_WEAPON) != 0 {
            crate::NPC_combat::WeaponThink(ctx, qtrue);
        }

        let client = (*npc_ent).client;
        if (*client).ps.stats[statIndex_t::STAT_ARMOR as usize] <= 0 {
            // armor gone
            // if ( !NPCInfo->investigateDebounceTime )
            if false {
                // start regenerating the armor
                crate::NPC_utils::NPC_SetSurfaceOnOff(
                    ctx,
                    ctx.entity_id_of(npc_ent).unwrap(),
                    c"torso_shield".as_ptr(),
                    TURN_OFF,
                );
                (*npc_ent).flags &= !FL_SHIELDED; // no more reflections
                (*npc_ent).r.mins = [-20.0, -20.0, -24.0];
                (*npc_ent).r.maxs = [20.0, 20.0, 64.0];
                (*client).ps.crouchheight = 64;
                (*client).ps.standheight = 64;
                if (*npc_ent).locationDamage[HL_GENERIC1 as usize] < GENERATOR_HEALTH {
                    // still have the generator bolt-on
                    if (*npc_info).investigateCount < 12 {
                        (*npc_info).investigateCount += 1;
                    }
                    (*npc_info).investigateDebounceTime = ctx.world.level.time
                        + ((*npc_info).investigateCount as c_int as i32 * 5000) as i32;
                }
            } else if (*npc_info).investigateDebounceTime < ctx.world.level.time {
                // armor regenerated, turn shield back on
                // do a trace and make sure we can turn this back on?
                let mut tr: trace_t = core::mem::zeroed();
                // file-static shieldMins/shieldMaxs, not the dead if(0) block's -20/20/64 box.
                let shield_mins = [-60.0, -60.0, -24.0];
                let shield_maxs = [60.0, 60.0, 80.0];
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &(*npc_ent).r.currentOrigin as *const vec3_t,
                        &shield_mins as *const vec3_t,
                        &shield_maxs as *const vec3_t,
                        &(*npc_ent).r.currentOrigin as *const vec3_t,
                        (*npc_ent).s.number,
                        (*npc_ent).clipmask,
                    ),
                );
                if tr.startsolid == 0 {
                    (*npc_ent).r.mins = shield_mins;
                    (*npc_ent).r.maxs = shield_maxs;
                    (*client).ps.crouchheight = shield_maxs[2] as c_int;
                    (*client).ps.standheight = shield_maxs[2] as c_int;
                    (*client).ps.stats[statIndex_t::STAT_ARMOR as usize] = GALAK_SHIELD_HEALTH;
                    (*npc_info).investigateDebounceTime = 0;
                    (*npc_ent).flags |= FL_SHIELDED; // reflect normal shots
                                                     // NPC->fx_time = level.time;
                    crate::NPC_utils::NPC_SetSurfaceOnOff(
                        ctx,
                        ctx.entity_id_of(npc_ent).unwrap(),
                        c"torso_shield".as_ptr(),
                        TURN_ON,
                    );
                }
            }
        }

        if (*npc_ent).enemy.is_none() {
            // don't have an enemy, look for one
            NPC_BSGM_Patrol(ctx);
        } else {
            // have an enemy
            NPC_BSGM_Attack(ctx);
        }
    }
}
