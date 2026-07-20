// PORT-COMPLETE: NPC_AI_GalakMech.c
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_GalakMech.c`.
//!
//! All functions are ported: `GM_Move` reads the canonical `NIF_COLLISION`
//! bit, `GM_CheckFireState`/`NPC_BSGM_Attack`/`NPC_BSGM_Default` place their
//! `*4` state in `ctx.world.globals`, and `impactPos4` lives in
//! `ctx.world.scratch.impact_pos_4` (safe-state Stage 3).
//!
//! Safe-state **2c** (deref regime): the gentity half is converted to
//! `ctx.world.entity(id)` / `entity_mut(id)` accessor borrows — the laundered
//! `g_entities`-base raw pointers (`npc_ent`/`enemy_ent`/`cover_ent`/…) are
//! gone. Two irreducible raw-deref regimes remain, each FLAGged in-source and
//! confined to tight `unsafe` blocks through a copied pointer value: the
//! `gNPC_t` (`NPCInfo`) fields (no safe accessor) and the NPC's BG_Alloc'd pool
//! `gclient_t` (`gClPtrs`, not `level.clients`). Behavior is byte-identical,
//! referee-verified.
#![allow(non_snake_case, unused, clippy::all)]

use crate::entity::flags::{FL_NO_KNOCKBACK, FL_SHIELDED};
use crate::prelude::*;
use crate::trap;
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::public::entity_event::entity_event_t;
use mp_bg::public::means_of_death::meansOfDeath_t;
use mp_bg::public::stat_index::statIndex_t;
use mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_WALKING;

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
    // FLAG (task #7): gNPC_t (NPCInfo) has no safe accessor; deref stays raw.
    let npc = ctx.world.entity(ent).NPC;
    let behavior_state = unsafe { *((&(*npc).behaviorState) as *const bState_t as *const c_int) };
    if behavior_state != BS_CINEMATIC as c_int {
        // FLAG (task #7): NPC pool gclient_t (gClPtrs) — deref stays raw.
        let client = ctx.world.entity(ent).client;
        unsafe {
            (*client).ps.stats[statIndex_t::STAT_ARMOR as usize] = GALAK_SHIELD_HEALTH;
            (*npc).investigateCount = 0;
            (*npc).investigateDebounceTime = 0;
        }
        ctx.world.entity_mut(ent).flags |= FL_SHIELDED; // reflect normal shots
                                                        // rwwFIXMEFIXME: Support PW_GALAK_SHIELD
                                                        // ent->client->ps.powerups[PW_GALAK_SHIELD] = Q3_INFINITE; // temp, for effect
                                                        // ent->fx_time = level.time;
        ctx.world.entity_mut(ent).r.mins = [-60.0, -60.0, -24.0];
        ctx.world.entity_mut(ent).r.maxs = [60.0, 60.0, 80.0];
        ctx.world.entity_mut(ent).flags |= FL_NO_KNOCKBACK; // don't get pushed
        crate::g_timer::TIMER_Set(ctx, Some(ent), c"attackDelay".as_ptr(), 0); // FIXME: Slant for difficulty levels
        crate::g_timer::TIMER_Set(ctx, Some(ent), c"flee".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, Some(ent), c"smackTime".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, Some(ent), c"beamDelay".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, Some(ent), c"noLob".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, Some(ent), c"noRapid".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, Some(ent), c"talkDebounce".as_ptr(), 0);

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

/// Raven `GM_CreateExplosion`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:101-125`
pub fn GM_CreateExplosion(
    ctx: &mut GameContext,
    self_: EntityId,
    boltID: c_int,
    doSmall: qboolean,
) {
    if boltID >= 0 {
        let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
        let mut org: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];

        let ghoul2 = ctx.world.entity(self_).ghoul2;
        let current_angles = ctx.world.entity(self_).r.currentAngles;
        let current_origin = ctx.world.entity(self_).r.currentOrigin;
        let model_scale = ctx.world.entity(self_).modelScale;
        let level_time = ctx.world.level.time;
        trap::G2API_GetBoltMatrix(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                ghoul2,
                0,
                boltID,
                &mut boltMatrix as *mut mdxaBone_t,
                &current_angles as *const vec3_t,
                &current_origin as *const vec3_t,
                level_time,
                core::ptr::null_mut(),
                &model_scale as *const vec3_t,
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

/// Raven `GM_Dying`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:133-229`
pub fn GM_Dying(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        let level_time = ctx.world.level.time;
        // Raven `vec3_origin` — resolved via the crate prelude (pass-3 symbol
        // backfill).
        if level_time - ctx.world.entity(self_).s.time < 4000 {
            // FIXME: need a real effect
            // self->s.powerups |= ( 1 << PW_SHOCKED );
            // self->client->ps.powerups[PW_SHOCKED] = level.time + 1000;
            // FLAG (task #7): NPC pool gclient_t (gClPtrs) — deref stays raw.
            let client = ctx.world.entity(self_).client;
            (*client).ps.electrifyTime = level_time + 1000;
            if crate::g_timer::TIMER_Done(ctx, Some(self_), c"dyingExplosion".as_ptr()) != 0 {
                let mut newBolt: c_int;
                let self_id = Some(self_);
                let ghoul2 = ctx.world.entity(self_).ghoul2;
                match ctx.world.bg_state.rng.Q_irand(1, 14) {
                    // Find place to generate explosion
                    1 => {
                        if trap::G2API_GetSurfaceRenderStatus(ctx.engine, ghoul2, 0, "r_hand") == 0
                        {
                            // r_hand still there
                            let newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*flasha");
                            GM_CreateExplosion(ctx, self_, newBolt, 1);
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"r_hand".as_ptr(), TURN_OFF);
                        } else if trap::G2API_GetSurfaceRenderStatus(ctx.engine, ghoul2, 0, "r_arm_middle") == 0
                        {
                            // r_arm_middle still there
                            newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*r_arm_elbow");
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"r_arm_middle".as_ptr(), TURN_OFF);
                        }
                    }
                    2 => {
                        // FIXME: do only once?
                        if trap::G2API_GetSurfaceRenderStatus(ctx.engine, ghoul2, 0, "l_hand") == 0
                        {
                            // l_hand still there
                            let newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*flashc");
                            GM_CreateExplosion(ctx, self_, newBolt, 0);
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"l_hand".as_ptr(), TURN_OFF);
                        } else if trap::G2API_GetSurfaceRenderStatus(ctx.engine, ghoul2, 0, "l_arm_wrist") == 0
                        {
                            // l_arm_wrist still there
                            newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*l_arm_cap_l_hand");
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"l_arm_wrist".as_ptr(), TURN_OFF);
                        } else if trap::G2API_GetSurfaceRenderStatus(ctx.engine, ghoul2, 0, "l_arm_middle") == 0
                        {
                            // l_arm_middle still there
                            newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*l_arm_cap_l_hand");
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"l_arm_middle".as_ptr(), TURN_OFF);
                        } else if trap::G2API_GetSurfaceRenderStatus(ctx.engine, ghoul2, 0, "l_arm_augment") == 0
                        {
                            // l_arm_augment still there
                            newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*l_arm_elbow");
                            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_, c"l_arm_augment".as_ptr(), TURN_OFF);
                        }
                    }
                    3 | 4 => {
                        newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*hip_fr");
                        GM_CreateExplosion(ctx, self_, newBolt, 0);
                    }
                    5 | 6 => {
                        newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*shldr_l");
                        GM_CreateExplosion(ctx, self_, newBolt, 0);
                    }
                    7 | 8 => {
                        newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*uchest_r");
                        GM_CreateExplosion(ctx, self_, newBolt, 0);
                    }
                    9 | 10 => {
                        let head_bolt = (*client).renderInfo.headBolt;
                        GM_CreateExplosion(ctx, self_, head_bolt, 0);
                    }
                    11 => {
                        newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*l_leg_knee");
                        GM_CreateExplosion(ctx, self_, newBolt, 1);
                    }
                    12 => {
                        newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*r_leg_knee");
                        GM_CreateExplosion(ctx, self_, newBolt, 1);
                    }
                    13 => {
                        newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*l_leg_foot");
                        GM_CreateExplosion(ctx, self_, newBolt, 1);
                    }
                    14 => {
                        newBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*r_leg_foot");
                        GM_CreateExplosion(ctx, self_, newBolt, 1);
                    }
                    _ => {}
                }

                let delay = ctx.world.bg_state.rng.Q_irand(300, 1100);
                crate::g_timer::TIMER_Set(ctx, self_id, c"dyingExplosion".as_ptr(), delay);
            }
        } else {
            // one final, huge explosion
            let current_origin = ctx.world.entity(self_).r.currentOrigin;
            crate::g_utils::G_PlayEffectID(
                crate::g_utils::G_EffectIndex(c"galak/explode".as_ptr()),
                current_origin,
                vec3_origin,
            );
            // G_PlayEffect( "small_chunks", self->r.currentOrigin );
            // G_PlayEffect( "env/exp_trail_comp", self->r.currentOrigin, self->currentAngles );
            ctx.world.entity_mut(self_).nextthink = level_time + FRAMETIME;
            ctx.world.entity_mut(self_).think =
                Some(crate::ent_fn_enums::EntThink::G_FreeEntity).into();
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
    let inflictor = attacker;
    let hitLoc: c_int = 1; // Raven: `int hitLoc = 1;` — never reassigned in this fn
    let r#mod = ctx.world.globals.gPainMOD;
    let level_time = ctx.world.level.time;

    // FLAG (task #7): gNPC_t (NPCInfo) has no safe accessor; deref stays raw.
    let npc = ctx.world.entity(self_).NPC;
    // FLAG (task #7): NPC pool gclient_t (gClPtrs) — deref stays raw.
    let client = ctx.world.entity(self_).client;

    if ctx.world.entity(self_).lockCount == 0 && unsafe { (*client).ps.torsoTimer } <= 0 {
        // don't interrupt laser sweep attack or other special attacks/moves
        if ctx.world.entity(self_).count < 4
            && ctx.world.entity(self_).health > 100
            && hitLoc != HL_GENERIC1
        {
            if ctx.world.entity(self_).delay < level_time {
                let speech = match ctx.world.entity(self_).count {
                    1 => entity_event_t::EV_PUSHED2 as c_int,
                    2 => entity_event_t::EV_PUSHED3 as c_int,
                    3 => entity_event_t::EV_DETECTED1 as c_int,
                    _ => entity_event_t::EV_PUSHED1 as c_int,
                };
                ctx.world.entity_mut(self_).count += 1;
                let self_id = self_;
                let delay = ctx.world.bg_state.rng.Q_irand(3000, 5000);
                // §19: oracle derefs `self->NPC->blockedSpeechDebounceTime`
                // unconditionally; the null guard is defensive.
                // Source: oracle/codemp/game/NPC_AI_GalakMech.c:307
                if !npc.is_null() {
                    unsafe {
                        (*npc).blockedSpeechDebounceTime = 0;
                    }
                }
                crate::NPC_sounds::G_AddVoiceEvent(ctx, self_id, speech, delay);
                let d = level_time + ctx.world.bg_state.rng.Q_irand(5000, 7000);
                ctx.world.entity_mut(self_).delay = d;
            }
        } else {
            crate::NPC_reactions::NPC_Pain(ctx, self_, attacker, damage);
        }
    } else if hitLoc == HL_GENERIC1 {
        crate::NPC_reactions::NPC_SetPainEvent(ctx, self_);
        // self->s.powerups |= ( 1 << PW_SHOCKED );
        // self->client->ps.powerups[PW_SHOCKED] = level.time + ctx.world.bg_state.rng.Q_irand( 500, 2500 );
        let e = level_time + ctx.world.bg_state.rng.Q_irand(500, 2500);
        unsafe {
            (*client).ps.electrifyTime = e;
        }
    }

    if inflictor.is_some() && ctx.world.entity(inflictor.unwrap()).lastEnemy == Some(self_) {
        // He force-pushed my own lobfires back at me
        // FLAG (task #7): gNPC_t (NPCInfo) has no safe accessor; deref stays raw.
        let npc = ctx.world.entity(self_).NPC;
        if r#mod == meansOfDeath_t::MOD_REPEATER_ALT as c_int
            && ctx.world.bg_state.rng.Q_irand(0, 2) == 0
        {
            if crate::g_timer::TIMER_Done(ctx, Some(self_), c"noRapid".as_ptr()) != 0 {
                if !npc.is_null() {
                    unsafe {
                        (*npc).scriptFlags &= !SCF_ALT_FIRE;
                    }
                }
                ctx.world.entity_mut(self_).alt_fire = 0;
                let self_id = Some(self_);
                let delay = ctx.world.bg_state.rng.Q_irand(2000, 6000);
                crate::g_timer::TIMER_Set(ctx, self_id, c"noLob".as_ptr(), delay);
            } else {
                let self_id = Some(self_);
                let delay = ctx.world.bg_state.rng.Q_irand(1000, 2000);
                // hopefully this will make us fire the laser
                crate::g_timer::TIMER_Set(ctx, self_id, c"noLob".as_ptr(), delay);
            }
        } else if r#mod == meansOfDeath_t::MOD_REPEATER as c_int
            && ctx.world.bg_state.rng.Q_irand(0, 5) == 0
        {
            if crate::g_timer::TIMER_Done(ctx, Some(self_), c"noLob".as_ptr()) != 0 {
                if !npc.is_null() {
                    unsafe {
                        (*npc).scriptFlags |= SCF_ALT_FIRE;
                    }
                }
                ctx.world.entity_mut(self_).alt_fire = 1;
                let self_id = Some(self_);
                let delay = ctx.world.bg_state.rng.Q_irand(2000, 6000);
                crate::g_timer::TIMER_Set(ctx, self_id, c"noRapid".as_ptr(), delay);
            } else {
                let self_id = Some(self_);
                let delay = ctx.world.bg_state.rng.Q_irand(1000, 2000);
                // hopefully this will make us fire the laser
                crate::g_timer::TIMER_Set(ctx, self_id, c"noRapid".as_ptr(), delay);
            }
        }
    }
}

/// Raven `GM_HoldPosition`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:362-369`
pub fn GM_HoldPosition(ctx: &mut GameContext) {
    // `npc_ent` (globals.NPC) stays a raw pointer only for the ICARUS syscall
    // `.cast()`; no gentity field is dereffed here.
    let npc_ent = ctx.world.globals.NPC;
    // FLAG (task #7): gNPC_t (NPCInfo) has no safe accessor; derefs stay raw.
    let npc_info = ctx.world.globals.NPCInfo;
    unsafe {
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
    // `npc_ent` (globals.NPC) stays a raw pointer only for the ICARUS syscall
    // `.cast()`.
    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    // FLAG (task #7): gNPC_t (NPCInfo) has no safe accessor; deref stays raw.
    let npc_info = ctx.world.globals.NPCInfo;
    unsafe {
        (*npc_info).combatMove = qtrue; // always move straight toward our goal

        let moved = crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);

        // Get the move info
        let mut info: navInfo_t = core::mem::zeroed();
        crate::NPC_move::NAV_GetLastMove(ctx, &mut info);

        // FIXME: if we bump into another one of our guys and can't get around him, just stop!
        // If we hit our target, then stop and fire!
        if (info.flags & NIF_COLLISION) != 0 {
            if ent_id_opt(ctx.world.g_entities.as_ptr(), info.blocker)
                == ctx.world.entity(npc_id).enemy
            {
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
    // `npc_ent` (globals.NPC) stays a raw pointer only for the ICARUS syscall
    // `.cast()`.
    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    // FLAG (task #7): gNPC_t (NPCInfo) has no safe accessor; derefs stay raw.
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
    if unsafe { (*npc_info).goalEntity } != ctx.world.entity(npc_id).enemy
        && !unsafe { (*npc_info).goalEntity }.is_none()
    {
        // Did we make it?
        // Guarded by `!goalEntity.is_none()` above.
        let goal_id = unsafe { (*npc_info).goalEntity }.unwrap();
        let hit_goal = crate::g_nav::NAV_HitNavGoal(
            ctx.world.entity(npc_id).r.currentOrigin,
            ctx.world.entity(npc_id).r.mins,
            ctx.world.entity(npc_id).r.maxs,
            ctx.world.entity(goal_id).r.currentOrigin,
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
            let delay = ctx.world.bg_state.rng.Q_irand(250, 500);
            // don't attack right away
            crate::g_timer::TIMER_Set(
                ctx,
                Some(npc_id),
                c"attackDelay".as_ptr(),
                delay, // FIXME: Slant for difficulty levels
            );
            return;
        }
    }
}

/// Raven `GM_CheckFireState`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:468-556`
pub fn GM_CheckFireState(ctx: &mut GameContext) {
    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    // FLAG (task #7): gNPC_t (NPCInfo) has no safe accessor; derefs stay raw.
    let npc_info = ctx.world.globals.NPCInfo;
    unsafe {
        let level_time = ctx.world.level.time;
        let enemyCS4 = ctx.world.globals.enemyCS4;
        let hitAlly4 = ctx.world.globals.hitAlly4;
        let mut faceEnemy4 = ctx.world.globals.faceEnemy4;
        let mut shoot4 = ctx.world.globals.shoot4;

        if enemyCS4 != 0 {
            // if have a clear shot, always try
            return;
        }

        // FLAG (task #7): NPC pool gclient_t (gClPtrs) — deref stays raw.
        let client = ctx.world.entity(npc_id).client;
        if !VectorCompare((*client).ps.velocity, vec3_origin) {
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

                    crate::NPC_utils::CalcEntitySpot(ctx, Some(npc_id), SPOT_HEAD, &mut muzzle);
                    if VectorCompare(ctx.world.scratch.impact_pos_4, vec3_origin) {
                        // never checked ShotEntity this frame, so must do a trace...
                        let mut tr: trace_t = core::mem::zeroed();
                        let mut forward: vec3_t = [0.0; 3];
                        let mut end: vec3_t = [0.0; 3];
                        AngleVectors((*client).ps.viewangles, Some(&mut forward), None, None);
                        _VectorMA(muzzle, 8192.0, forward, &mut end);
                        let npc_number = ctx.world.entity(npc_id).s.number;
                        trap::Trace(
                            ctx.engine,
                            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                &mut tr as *mut trace_t,
                                &muzzle as *const vec3_t,
                                &vec3_origin as *const vec3_t,
                                &vec3_origin as *const vec3_t,
                                &end as *const vec3_t,
                                npc_number,
                                MASK_SHOT,
                            ),
                        );
                        _VectorCopy(tr.endpos, &mut ctx.world.scratch.impact_pos_4);
                    }

                    // see if impact would be too close to me
                    distThreshold = 16384.0; // 128*128, default
                    if ctx.world.entity(npc_id).s.weapon == WP_REPEATER as c_int {
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
                        if ctx.world.entity(npc_id).s.weapon == WP_REPEATER as c_int {
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
    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    let npc = ctx.world.entity(npc_id).NPC;
    if ctx.world.entity(npc_id).lockCount == 0 {
        // haven't already started a laser attack
        // warm up for the beam attack
        #[cfg(any())]
        {
            // NPC_SetAnim( NPC, SETANIM_TORSO, TORSO_RAISEWEAP2, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
        }
        // FLAG (task #7): NPC pool gclient_t (gClPtrs) — deref stays raw.
        let client = ctx.world.entity(npc_id).client;
        let torso_timer = unsafe { (*client).ps.torsoTimer };
        crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"beamDelay".as_ptr(), torso_timer);
        crate::g_timer::TIMER_Set(
            ctx,
            Some(npc_id),
            c"attackDelay".as_ptr(),
            torso_timer + 3000,
        );
        ctx.world.entity_mut(npc_id).lockCount = 1;
        // turn on warmup effect (Raven `vec3_origin` — resolved via the
        // crate prelude, pass-3 symbol backfill).
        let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
        crate::g_utils::G_PlayEffectID(
            crate::g_utils::G_EffectIndex(c"galak/beam_warmup".as_ptr()),
            current_origin,
            vec3_origin,
        );
        crate::g_utils::G_SoundOnEnt(
            ctx,
            npc_id,
            CHAN_AUTO,
            c"sound/weapons/galak/lasercharge.wav".as_ptr(),
        );
    }
}

/// Raven `GM_StartGloat`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:575-587`
pub fn GM_StartGloat(ctx: &mut GameContext) {
    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    ctx.world.entity_mut(npc_id).wait = 0.0;
    crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, npc_id, c"torso_galakface".as_ptr(), TURN_ON);
    crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, npc_id, c"torso_galakhead".as_ptr(), TURN_ON);
    crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, npc_id, c"torso_eyes_mouth".as_ptr(), TURN_ON);
    crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, npc_id, c"torso_collar".as_ptr(), TURN_ON);
    crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, npc_id, c"torso_galaktorso".as_ptr(), TURN_ON);

    crate::npc_c::NPC_SetAnim(
        ctx,
        npc_id,
        SETANIM_BOTH,
        animNumber_t::BOTH_STAND2TO1 as c_int,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
    );
    // FLAG (task #7): NPC pool gclient_t (gClPtrs) — deref stays raw.
    let client = ctx.world.entity(npc_id).client;
    unsafe {
        (*client).ps.legsTimer += 500;
        (*client).ps.torsoTimer += 500;
    }
}

/// Raven `NPC_BSGM_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_GalakMech.c:594-1229`
pub fn NPC_BSGM_Attack(ctx: &mut GameContext) {
    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    // FLAG (task #7): gNPC_t (NPCInfo) has no safe accessor; derefs stay raw.
    let npc_info = ctx.world.globals.NPCInfo;
    let level_time = ctx.world.level.time;
    // FLAG (task #7): NPC pool gclient_t (gClPtrs) — deref stays raw.
    let client = ctx.world.entity(npc_id).client;
    unsafe {
        // Don't do anything if we're hurt
        if ctx.world.entity(npc_id).painDebounceTime > level_time {
            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        // Victory animation section disabled with #if 0 in oracle

        // If we don't have an enemy, just idle
        if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qfalse) == qfalse
            || ctx.world.entity(npc_id).enemy.is_none()
        {
            ctx.world.entity_mut(npc_id).enemy = None;
            NPC_BSGM_Patrol(ctx);
            return;
        }
        // Guaranteed `Some` from here to the end of the function by the guard above.
        let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();

        // Initialize combat state
        ctx.world.globals.enemyLOS4 = qfalse;
        ctx.world.globals.enemyCS4 = qfalse;
        ctx.world.globals.move4 = qtrue;
        ctx.world.globals.faceEnemy4 = qfalse;
        ctx.world.globals.shoot4 = qfalse;
        ctx.world.globals.hitAlly4 = qfalse;
        VectorClear(&mut ctx.world.scratch.impact_pos_4);
        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
        let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
        ctx.world.globals.enemyDist4 = DistanceSquared(npc_origin, enemy_origin);

        // Melee attack logic disabled with if(0)

        // Laser beam attack logic
        if ctx.world.entity(npc_id).lockCount != 0 {
            // already shooting laser
            ctx.world.globals.shoot4 = qfalse;
            if ctx.world.entity(npc_id).lockCount == 1 {
                // charging up
                if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"beamDelay".as_ptr()) != 0 {
                    // time to start the beam
                    let laserAnim: c_int = BOTH_ATTACK2 as c_int;
                    crate::npc_c::NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_BOTH,
                        laserAnim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    let delay =
                        (*client).ps.torsoTimer + ctx.world.bg_state.rng.Q_irand(1000, 3000);
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attackDelay".as_ptr(), delay);
                    // turn on beam effect
                    ctx.world.entity_mut(npc_id).lockCount = 2;
                    let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
                    crate::g_utils::G_PlayEffectID(
                        crate::g_utils::G_EffectIndex(c"galak/trace_beam".as_ptr()),
                        current_origin,
                        vec3_origin,
                    );
                    let loop_sound = crate::g_utils::G_SoundIndex(
                        c"sound/weapons/galak/lasercutting.wav".as_ptr(),
                    );
                    ctx.world.entity_mut(npc_id).s.loopSound = loop_sound;
                    if (*npc_info).coverTarg.is_none() {
                        // for moving looping sound at end of trace
                        let cover_ent_eid = crate::g_utils::G_Spawn(ctx);
                        {
                            let cover_id = cover_ent_eid;
                            (*npc_info).coverTarg = Some(cover_id);
                            let muzzle = (*client).renderInfo.muzzlePoint;
                            crate::g_utils::G_SetOrigin(ctx.world.entity_mut(cover_id), muzzle);
                            ctx.world.entity_mut(cover_id).r.svFlags |= SVF_BROADCAST;
                            let loop_sound2 = crate::g_utils::G_SoundIndex(
                                c"sound/weapons/galak/lasercutting.wav".as_ptr(),
                            );
                            ctx.world.entity_mut(cover_id).s.loopSound = loop_sound2;
                        }
                    }
                }
            } else {
                // in the actual attack now
                if (*client).ps.torsoTimer <= 0 {
                    // attack done!
                    ctx.world.entity_mut(npc_id).lockCount = 0;
                    if let Some(cover_id) = (*npc_info).coverTarg {
                        crate::g_utils::G_FreeEntity(ctx, Some(cover_id));
                    }
                    ctx.world.entity_mut(npc_id).s.loopSound = 0;
                    let torso_timer = (*client).ps.torsoTimer;
                    crate::g_timer::TIMER_Set(
                        ctx,
                        Some(npc_id),
                        c"attackDelay".as_ptr(),
                        torso_timer,
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
                    let npc_number = ctx.world.entity(npc_id).s.number;
                    trap::Trace(
                        ctx.engine,
                        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                            &mut trace as *mut trace_t,
                            &(*client).renderInfo.muzzlePoint as *const vec3_t,
                            &mins as *const vec3_t,
                            &maxs as *const vec3_t,
                            &end as *const vec3_t,
                            npc_number,
                            MASK_SHOT,
                        ),
                    );
                    if trace.allsolid != 0 || trace.startsolid != 0 {
                        // oops, in a wall
                        if let Some(cover_id) = (*npc_info).coverTarg {
                            let muzzle = (*client).renderInfo.muzzlePoint;
                            crate::g_utils::G_SetOrigin(ctx.world.entity_mut(cover_id), muzzle);
                        }
                    } else {
                        // clear
                        if trace.fraction < 1.0 {
                            // hit something
                            let trace_id = EntityId(trace.entityNum as u32);
                            if ctx.world.entity(trace_id).takedamage != 0 {
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
                                    Some(trace_id),
                                    Some(npc_id),
                                    Some(npc_id),
                                    Some(&mut (*client).renderInfo.muzzleDir),
                                    trace.endpos,
                                    10,
                                    0,
                                    meansOfDeath_t::MOD_UNKNOWN as c_int,
                                );
                            }
                        }
                        if let Some(cover_id) = (*npc_info).coverTarg {
                            crate::g_utils::G_SetOrigin(
                                ctx.world.entity_mut(cover_id),
                                trace.endpos,
                            );
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
            // Hoisted immutable entity reads (unchanged through this branch;
            // `npc_origin`/`enemy_origin` reused from combat-state init above).
            let enemy_anim_index = ctx.world.entity(enemy_id).localAnimIndex;
            let npc_lock_count = ctx.world.entity(npc_id).lockCount;
            let npc_loc_dmg = ctx.world.entity(npc_id).locationDamage[HL_GENERIC1 as usize];
            let enemy_weapon = ctx.world.entity(enemy_id).s.weapon;
            let enemy_classname = ctx.world.entity(enemy_id).classname;
            if ctx.world.globals.enemyDist4 < MELEE_DIST_SQUARED
                && InFront(enemy_origin, npc_origin, (*client).ps.viewangles, 0.3) != 0
                && enemy_anim_index <= 1
            {
                // our shield is down, and enemy within 80, if very close, use melee attack to slap away
                if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"attackDelay".as_ptr()) != 0 {
                    // animate me
                    let swingAnim: c_int = BOTH_ATTACK1 as c_int;
                    crate::npc_c::NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_BOTH,
                        swingAnim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    let delay =
                        (*client).ps.torsoTimer + ctx.world.bg_state.rng.Q_irand(1000, 3000);
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attackDelay".as_ptr(), delay);
                    // delay the hurt until the proper point in the anim
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"smackTime".as_ptr(), 600);
                    (*npc_info).blockedDebounceTime = 0;
                }
            } else if npc_lock_count == 0
                && npc_loc_dmg > GENERATOR_HEALTH
                && crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"attackDelay".as_ptr()) != 0
                && InFront(enemy_origin, npc_origin, (*client).ps.viewangles, 0.3) != 0
                && (ctx
                    .world
                    .bg_state
                    .rng
                    .Q_irand(0, 10 * (2 - ctx.world.cvars.g_spskill.integer))
                    == 0
                    && ctx.world.globals.enemyDist4 > MIN_LOB_DIST_SQUARED
                    && ctx.world.globals.enemyDist4 < MAX_LOB_DIST_SQUARED
                    || crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"noLob".as_ptr()) == 0
                        && crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"noRapid".as_ptr()) == 0)
                && enemy_weapon != WP_TURRET as c_int
            {
                // sometimes use the laser beam attack, but only after he's taken down our generator
                ctx.world.globals.shoot4 = qfalse;
                NPC_GM_StartLaser(ctx);
            } else if ctx.world.globals.enemyDist4 < MIN_LOB_DIST_SQUARED
                && (enemy_weapon != WP_TURRET as c_int
                    || crate::q_shared::Q_stricmp(c"PAS".as_ptr(), enemy_classname) != 0)
                && crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"noRapid".as_ptr()) != 0
            {
                // enemy within 256
                if (*client).ps.weapon == WP_REPEATER as c_int
                    && ((*npc_info).scriptFlags & SCF_ALT_FIRE as i32) as c_int != 0
                {
                    // shooting an explosive, but enemy too close, switch to primary fire
                    (*npc_info).scriptFlags &= !(SCF_ALT_FIRE as i32);
                    ctx.world.entity_mut(npc_id).alt_fire = qfalse;
                    crate::NPC_combat::NPC_ChangeWeapon(WP_REPEATER);
                }
            } else if (ctx.world.globals.enemyDist4 > MAX_LOB_DIST_SQUARED
                || (enemy_weapon == WP_TURRET as c_int
                    && crate::q_shared::Q_stricmp(c"PAS".as_ptr(), enemy_classname) == 0))
                && crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"noLob".as_ptr()) != 0
            {
                // enemy more than 448 away and we are ready to try lob fire again
                if (*client).ps.weapon == WP_REPEATER as c_int
                    && ((*npc_info).scriptFlags & SCF_ALT_FIRE as i32) as c_int == 0
                {
                    // enemy far enough away to use lobby explosives
                    (*npc_info).scriptFlags |= SCF_ALT_FIRE as i32;
                    ctx.world.entity_mut(npc_id).alt_fire = qtrue;
                    crate::NPC_combat::NPC_ChangeWeapon(WP_REPEATER);
                }
            }
        }

        // can we see our target?
        if crate::NPC_utils::NPC_ClearLOS4(ctx, Some(enemy_id)) != 0 {
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
                        Some(enemy_id),
                        Some(&mut impactPos4),
                    );
                    ctx.world.scratch.impact_pos_4 = impactPos4;
                    // `hit_ent` is never NULL (base+index); the guards below survive as
                    // `client != null` pointer checks. FLAG (task #7): the hit entity's
                    // gclient_t is dereffed raw via the safe entity borrow (trap 2b).
                    let hit_id = EntityId(hit as u32);
                    let hit_client = ctx.world.entity(hit_id).client;
                    let enemy_number = ctx.world.entity(enemy_id).s.number;
                    let hit_takedamage = ctx.world.entity(hit_id).takedamage;
                    if hit == enemy_number as c_int
                        || (!hit_client.is_null()
                            && (*hit_client).playerTeam == (*client).enemyTeam)
                        || (hit_takedamage != 0)
                    {
                        // can hit enemy or will hit glass or other breakable, so shoot anyway
                        ctx.world.globals.enemyCS4 = qtrue;
                        crate::NPC_combat::NPC_AimAdjust(ctx, 2); // adjust aim better longer we have clear shot at enemy
                        let eo = ctx.world.entity(enemy_id).r.currentOrigin;
                        _VectorCopy(eo, &mut (*npc_info).enemyLastSeenLocation);
                    } else {
                        // Hmm, have to get around this bastard
                        crate::NPC_combat::NPC_AimAdjust(ctx, 1); // adjust aim better longer we can see enemy
                        if !hit_client.is_null() && (*hit_client).playerTeam == (*client).playerTeam
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
                &enemy_origin as *const vec3_t,
                &npc_origin as *const vec3_t,
            ),
        ) != 0
        {
            // C only declares hit/hitEnt here; the single NPC_ShotEntity call is below,
            // after enemyLastSeenTime is set.
            if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"talkDebounce".as_ptr()) != 0
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
                        let delay = ctx.world.bg_state.rng.Q_irand(3000, 5000);
                        crate::NPC_sounds::G_AddVoiceEvent(ctx, npc_id, speech, delay);
                        let delay_2 = ctx.world.bg_state.rng.Q_irand(5000, 7000);
                        crate::g_timer::TIMER_Set(
                            ctx,
                            Some(npc_id),
                            c"talkDebounce".as_ptr(),
                            delay_2,
                        );
                    }
                }
            }

            (*npc_info).enemyLastSeenTime = level_time;

            // `impactPos4` is a file static shared with GM_CheckFireState; copy it out, let
            // NPC_ShotEntity write through the local, then store back (matches C's file-static).
            let mut impactPos4 = ctx.world.scratch.impact_pos_4;
            let hit = crate::NPC_combat::NPC_ShotEntity(ctx, Some(enemy_id), Some(&mut impactPos4));
            ctx.world.scratch.impact_pos_4 = impactPos4;
            // `hit_ent` is never NULL (base+index). FLAG (task #7): the hit entity's
            // gclient_t is dereffed raw via the safe entity borrow (trap 2b).
            let hit_id = EntityId(hit as u32);
            let hit_client = ctx.world.entity(hit_id).client;
            let enemy_number = ctx.world.entity(enemy_id).s.number;
            let hit_takedamage = ctx.world.entity(hit_id).takedamage;
            if hit == enemy_number as c_int
                || (!hit_client.is_null() && (*hit_client).playerTeam == (*client).enemyTeam)
                || (hit_takedamage != 0)
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
                (*npc_info).goalEntity = ctx.world.entity(npc_id).enemy;
            }
            if (*npc_info).goalEntity == ctx.world.entity(npc_id).enemy {
                // for now, always chase the enemy
                ctx.world.globals.move4 = qtrue;
            }
        }
        if ctx.world.globals.enemyCS4 != 0 {
            ctx.world.globals.shoot4 = qtrue;
        } else {
            if !(*npc_info).goalEntity.is_some() {
                (*npc_info).goalEntity = ctx.world.entity(npc_id).enemy;
            }
            if (*npc_info).goalEntity == ctx.world.entity(npc_id).enemy {
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
            && crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"attackDelay".as_ptr()) != 0
        {
            let mut muzzle: vec3_t = [0.0; 3];
            let mut angles: vec3_t = [0.0; 3];
            let mut target: vec3_t = [0.0; 3];
            let mut velocity: vec3_t = [0.0; 3];
            let mins = [-REPEATER_ALT_SIZE, -REPEATER_ALT_SIZE, -REPEATER_ALT_SIZE];
            let maxs = [REPEATER_ALT_SIZE, REPEATER_ALT_SIZE, REPEATER_ALT_SIZE];

            crate::NPC_utils::CalcEntitySpot(ctx, Some(npc_id), SPOT_WEAPON, &mut muzzle);

            let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
            _VectorCopy(enemy_origin, &mut target);

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
            let npc_number = ctx.world.entity(npc_id).s.number;
            let enemy_number = ctx.world.entity(enemy_id).s.number;
            let clearshot = crate::g_weapon::WP_LobFire(
                ctx,
                npc_id,
                muzzle,
                target,
                mins,
                maxs,
                MASK_SHOT | CONTENTS_LIGHTSABER,
                &mut velocity,
                qtrue,
                npc_number,
                enemy_number as c_int,
                300.0,
                1100.0,
                1500.0,
                qtrue,
            );
            if VectorCompare(vec3_origin, velocity)
                || (clearshot == 0
                    && ctx.world.globals.enemyLOS4 != 0
                    && ctx.world.globals.enemyCS4 != 0)
            {
                // no clear lob shot and no lob shot that will hit something breakable
                if ctx.world.globals.enemyLOS4 != 0
                    && ctx.world.globals.enemyCS4 != 0
                    && crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"noRapid".as_ptr()) != 0
                {
                    // have a clear straight shot, so switch to primary
                    (*npc_info).scriptFlags &= !(SCF_ALT_FIRE as i32);
                    ctx.world.entity_mut(npc_id).alt_fire = qfalse;
                    crate::NPC_combat::NPC_ChangeWeapon(WP_REPEATER);
                    let delay = ctx.world.bg_state.rng.Q_irand(500, 1000);
                    // keep this weap for a bit
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"noLob".as_ptr(), delay);
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

        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"standTime".as_ptr()) == 0 {
            ctx.world.globals.move4 = qfalse;
        }
        if ((*npc_info).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
            // not supposed to chase my enemies
            if (*npc_info).goalEntity == ctx.world.entity(npc_id).enemy {
                // goal is my entity, so don't move
                ctx.world.globals.move4 = qfalse;
            }
        }

        if ctx.world.globals.move4 != 0 && ctx.world.entity(npc_id).lockCount == 0 {
            // move toward goal
            if (*npc_info).goalEntity.is_some() {
                ctx.world.globals.move4 = GM_Move(ctx);
            } else {
                ctx.world.globals.move4 = qfalse;
            }
        }

        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"flee".as_ptr()) == 0 {
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

        if ctx.world.entity(npc_id).enemy.is_some() && ctx.world.entity(enemy_id).enemy.is_some() {
            // The enemy's own enemy — a second EntityId hop off `enemy_id`, resolved
            // only inside this guard (guaranteed `Some` by the check above).
            let eoe_id = ctx.world.entity(enemy_id).enemy.unwrap();
            if ctx.world.entity(enemy_id).s.weapon == WP_SABER as c_int
                && ctx.world.entity(eoe_id).s.weapon == WP_SABER as c_int
            {
                // don't shoot at an enemy jedi who is fighting another jedi, for fear of injuring one or causing rogue blaster deflections (a la Obi Wan/Vader duel at end of ANH)
                ctx.world.globals.shoot4 = qfalse;
            }
        }

        if ctx.world.globals.shoot4 != 0 {
            // try to shoot if it's time
            if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"attackDelay".as_ptr()) != 0 {
                if ((*npc_info).scriptFlags & SCF_FIRE_WEAPON) == 0 {
                    // we've already fired, no need to do it again here
                    crate::NPC_combat::WeaponThink(ctx, qtrue);
                }
            }
        }

        // also:
        if ctx.world.entity(enemy_id).s.weapon == WP_TURRET as c_int
            && crate::q_shared::Q_stricmp(c"PAS".as_ptr(), ctx.world.entity(enemy_id).classname)
                == 0
        {
            // crush turrets
            if crate::NPC_goal::G_BoundsOverlap(
                ctx.world.entity(npc_id).r.absmin,
                ctx.world.entity(npc_id).r.absmax,
                ctx.world.entity(enemy_id).r.absmin,
                ctx.world.entity(enemy_id).r.absmax,
            ) != 0
            {
                // have to do this test because placed turrets are not solid to NPCs (so they don't obstruct navigation)
                if false {
                    (*client).ps.powerups[PW_BATTLESUIT as usize] = level_time + ARMOR_EFFECT_TIME;
                    let npc_origin_d = ctx.world.entity(npc_id).r.currentOrigin;
                    crate::g_combat::G_Damage(
                        ctx,
                        Some(enemy_id),
                        Some(npc_id),
                        Some(npc_id),
                        None,
                        npc_origin_d,
                        100,
                        DAMAGE_NO_KNOCKBACK,
                        meansOfDeath_t::MOD_UNKNOWN as c_int,
                    );
                } else {
                    let npc_origin_d = ctx.world.entity(npc_id).r.currentOrigin;
                    crate::g_combat::G_Damage(
                        ctx,
                        Some(enemy_id),
                        Some(npc_id),
                        Some(npc_id),
                        None,
                        npc_origin_d,
                        100,
                        DAMAGE_NO_KNOCKBACK,
                        meansOfDeath_t::MOD_CRUSH as c_int,
                    );
                }
            }
        } else if (*npc_info).touchedByPlayer.is_some()
            && (*npc_info).touchedByPlayer == ctx.world.entity(npc_id).enemy
        {
            // touched enemy
            if false {
                // zap him!
                let mut smackDir: vec3_t = [0.0; 3];
                let torso_timer = (*client).ps.torsoTimer;
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attackDelay".as_ptr(), torso_timer);
                let legs_timer = (*client).ps.legsTimer;
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"standTime".as_ptr(), legs_timer);
                (*npc_info).touchedByPlayer = None;
                (*client).ps.powerups[PW_BATTLESUIT as usize] = level_time + ARMOR_EFFECT_TIME;

                let enemy_origin_f = ctx.world.entity(enemy_id).r.currentOrigin;
                let npc_origin_f = ctx.world.entity(npc_id).r.currentOrigin;
                _VectorSubtract(enemy_origin_f, npc_origin_f, &mut smackDir);
                smackDir[2] += 30.0;
                VectorNormalize(&mut smackDir);
                let delay =
                    (ctx.world.cvars.g_spskill.integer + 1) * ctx.world.bg_state.rng.Q_irand(5, 10);
                let npc_origin_g = ctx.world.entity(npc_id).r.currentOrigin;
                crate::g_combat::G_Damage(
                    ctx,
                    Some(enemy_id),
                    Some(npc_id),
                    Some(npc_id),
                    Some(&mut smackDir),
                    npc_origin_g,
                    delay,
                    DAMAGE_NO_KNOCKBACK,
                    meansOfDeath_t::MOD_UNKNOWN as c_int,
                );
                crate::g_utils::G_Throw(ctx, enemy_id, smackDir, 100.0);
                // FLAG (task #7): enemy entity's gclient_t dereffed raw via safe
                // entity borrow (trap 2b).
                let enemy_client = ctx.world.entity(enemy_id).client;
                if enemy_client != core::ptr::null_mut() {
                    (*enemy_client).ps.electrifyTime = level_time + 1000;
                }
                ctx.world.globals.ucmd.buttons = 0;
            }
        }

        if (*npc_info).movementSpeech < 3 && (*npc_info).blockedSpeechDebounceTime <= level_time {
            if ctx.world.entity(npc_id).enemy.is_some()
                && ctx.world.entity(enemy_id).health > 0
                && ctx.world.entity(enemy_id).painDebounceTime > level_time
            {
                if ctx.world.entity(enemy_id).health < 50 && (*npc_info).movementSpeech == 2 {
                    let delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                    crate::NPC_sounds::G_AddVoiceEvent(
                        ctx,
                        npc_id,
                        entity_event_t::EV_ANGER2 as c_int,
                        delay,
                    );
                    (*npc_info).movementSpeech = 3;
                } else if ctx.world.entity(enemy_id).health < 75 && (*npc_info).movementSpeech == 1
                {
                    let delay = ctx.world.bg_state.rng.Q_irand(2000, 4000);
                    crate::NPC_sounds::G_AddVoiceEvent(
                        ctx,
                        npc_id,
                        entity_event_t::EV_ANGER1 as c_int,
                        delay,
                    );
                    (*npc_info).movementSpeech = 2;
                } else if ctx.world.entity(enemy_id).health < 100 && (*npc_info).movementSpeech == 0
                {
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
    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    // FLAG (task #7): gNPC_t (NPCInfo) has no safe accessor; derefs stay raw.
    let npc_info = ctx.world.globals.NPCInfo;
    // FLAG (task #7): NPC pool gclient_t (gClPtrs) — deref stays raw.
    let client = ctx.world.entity(npc_id).client;
    unsafe {
        if ((*npc_info).scriptFlags & SCF_FIRE_WEAPON) != 0 {
            crate::NPC_combat::WeaponThink(ctx, qtrue);
        }

        if (*client).ps.stats[statIndex_t::STAT_ARMOR as usize] <= 0 {
            // armor gone
            // if ( !NPCInfo->investigateDebounceTime )
            if false {
                // start regenerating the armor
                crate::NPC_utils::NPC_SetSurfaceOnOff(
                    ctx,
                    npc_id,
                    c"torso_shield".as_ptr(),
                    TURN_OFF,
                );
                ctx.world.entity_mut(npc_id).flags &= !FL_SHIELDED; // no more reflections
                ctx.world.entity_mut(npc_id).r.mins = [-20.0, -20.0, -24.0];
                ctx.world.entity_mut(npc_id).r.maxs = [20.0, 20.0, 64.0];
                (*client).ps.crouchheight = 64;
                (*client).ps.standheight = 64;
                if ctx.world.entity(npc_id).locationDamage[HL_GENERIC1 as usize] < GENERATOR_HEALTH
                {
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
                let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
                let npc_number = ctx.world.entity(npc_id).s.number;
                let npc_clipmask = ctx.world.entity(npc_id).clipmask;
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &current_origin as *const vec3_t,
                        &shield_mins as *const vec3_t,
                        &shield_maxs as *const vec3_t,
                        &current_origin as *const vec3_t,
                        npc_number,
                        npc_clipmask,
                    ),
                );
                if tr.startsolid == 0 {
                    ctx.world.entity_mut(npc_id).r.mins = shield_mins;
                    ctx.world.entity_mut(npc_id).r.maxs = shield_maxs;
                    (*client).ps.crouchheight = shield_maxs[2] as c_int;
                    (*client).ps.standheight = shield_maxs[2] as c_int;
                    (*client).ps.stats[statIndex_t::STAT_ARMOR as usize] = GALAK_SHIELD_HEALTH;
                    (*npc_info).investigateDebounceTime = 0;
                    ctx.world.entity_mut(npc_id).flags |= FL_SHIELDED; // reflect normal shots
                                                                       // NPC->fx_time = level.time;
                    crate::NPC_utils::NPC_SetSurfaceOnOff(
                        ctx,
                        npc_id,
                        c"torso_shield".as_ptr(),
                        TURN_ON,
                    );
                }
            }
        }

        if ctx.world.entity(npc_id).enemy.is_none() {
            // don't have an enemy, look for one
            NPC_BSGM_Patrol(ctx);
        } else {
            // have an enemy
            NPC_BSGM_Attack(ctx);
        }
    }
}
