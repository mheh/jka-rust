// PORT-COMPLETE: NPC_AI_GalakMech.c 2/12
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_GalakMech.c`.
//!
//! Landed from the `fnskel.py` signature skeleton. Two functions are
//! transcribed faithfully from packet + prelude alone; the remaining 12 are
//! parked (see the `PORT-ESCALATION` topics below), because this file is
//! almost entirely ambient-state driven and the faithful context-free
//! signatures have no channel to reach it, matching the precedent set in
//! `NPC_AI_Jedi.rs`/`NPC_AI_Stormtrooper.rs`:
//!
//! - `ambient-state` — nearly every body reaches the file-scope AI globals
//!   (`NPC`, `NPCInfo`, `ucmd`, `level`, `g_entities`) or this file's own
//!   file-statics (`enemyLOS4`/`enemyCS4`/`hitAlly4`/`faceEnemy4`/`move4`/
//!   `enemyDist4`/`impactPos4` — fork ruling 5: genuine cross-frame state ->
//!   GameWorld field) or calls a `trap_*` (needs `&Engine`). Fork ruling 1
//!   makes the AI globals `GameWorld`/`GameContext` state, but these faithful
//!   signatures carry no `GameContext`/`&Engine` and the resolved cross-file
//!   signatures are equally context-free. How ambient state + engine thread
//!   into context-free faithful logic fns is not settled by the packet.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use mp_bg::public::stat_index::statIndex_t;

// Raven's file-scope `#define`s (`NPC_AI_GalakMech.c:24-26`) — not central
// constants, ported as file-local consts matching the C values.
const TURN_ON: c_int = 0x00000000;
const TURN_OFF: c_int = 0x00000100;
const GALAK_SHIELD_HEALTH: c_int = 500;

/// Raven `NPC_GalakMech_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:42-57`
pub fn NPC_GalakMech_Precache() {
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
pub fn NPC_GalakMech_Init(ent: *mut gentity_t) {
    unsafe {
        let npc = (*ent).NPC as *mut gNPC_t;
        let client = (*ent).client as *mut gclient_t;
        if (*npc).behaviorState != bState_t::BS_CINEMATIC {
            (*client).ps.stats[statIndex_t::STAT_ARMOR as usize] = GALAK_SHIELD_HEALTH;
            (*npc).investigateCount = 0;
            (*npc).investigateDebounceTime = 0;
            (*ent).flags |= crate::entity::flags::FL_SHIELDED; //reflect normal shots
            //rwwFIXMEFIXME: Support PW_GALAK_SHIELD
            //ent->client->ps.powerups[PW_GALAK_SHIELD] = Q3_INFINITE;//temp, for effect
            //ent->fx_time = level.time;
            (*ent).r.mins = [-60.0, -60.0, -24.0];
            (*ent).r.maxs = [60.0, 60.0, 80.0];
            (*ent).flags |= crate::entity::flags::FL_NO_KNOCKBACK; //don't get pushed
            crate::g_timer::TIMER_Set(ent, c"attackDelay".as_ptr(), 0); //FIXME: Slant for difficulty levels
            crate::g_timer::TIMER_Set(ent, c"flee".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ent, c"smackTime".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ent, c"beamDelay".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ent, c"noLob".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ent, c"noRapid".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ent, c"talkDebounce".as_ptr(), 0);

            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_shield".as_ptr(), TURN_ON);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_galakface".as_ptr(), TURN_OFF);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_galakhead".as_ptr(), TURN_OFF);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_eyes_mouth".as_ptr(), TURN_OFF);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_collar".as_ptr(), TURN_OFF);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_galaktorso".as_ptr(), TURN_OFF);
        } else {
            //		NPC_SetSurfaceOnOff( ent, "helmet", TURN_OFF );
            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_shield".as_ptr(), TURN_OFF);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_galakface".as_ptr(), TURN_ON);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_galakhead".as_ptr(), TURN_ON);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_eyes_mouth".as_ptr(), TURN_ON);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_collar".as_ptr(), TURN_ON);
            crate::NPC_utils::NPC_SetSurfaceOnOff(ent, c"torso_galaktorso".as_ptr(), TURN_ON);
        }
    }
}

// PORT-ESCALATION(ambient-state): reads `level.time` and calls
// trap_G2API_GetBoltMatrix (needs &Engine); no channel from this
// context-free faithful signature.
/// Raven `GM_CreateExplosion`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:101-125`
pub fn GM_CreateExplosion(self_: *mut gentity_t, boltID: c_int, doSmall: qboolean) {
    todo!("Port GM_CreateExplosion — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `level.time` and calls
// trap_G2API_AddBolt/trap_G2API_GetSurfaceRenderStatus (needs &Engine); no
// channel from this context-free faithful signature.
/// Raven `GM_Dying`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:133-229`
pub fn GM_Dying(self_: *mut gentity_t) {
    todo!("Port GM_Dying — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the file-scope `gPainMOD`/
// `gPainPoint` globals (ai_main) and `level.time`; no channel to reach the
// ai_main globals from this context-free faithful signature (rule B forbids
// static mut; resolved cross-file sigs are context-free). Also stored as a
// fn pointer (needs an EntXxx enum variant per ruling 2).
/// Raven `NPC_GM_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:238-354`
pub fn NPC_GM_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    todo!("Port NPC_GM_Pain — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals and calls trap_ICARUS_TaskIDPending (needs &Engine); no channel
// from this context-free faithful signature.
/// Raven `GM_HoldPosition`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:362-369`
pub fn GM_HoldPosition() {
    todo!("Port GM_HoldPosition — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals and calls trap_ICARUS_TaskIDPending (needs &Engine); no channel
// from this context-free faithful signature.
/// Raven `GM_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:376-408`
pub fn GM_Move() -> qboolean {
    todo!("Port GM_Move — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo`/`ucmd`
// ambient globals; no channel from this context-free faithful signature.
/// Raven `NPC_BSGM_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:416-432`
pub fn NPC_BSGM_Patrol() {
    todo!("Port NPC_BSGM_Patrol — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals plus this file's own file-static scratch (`enemyDist4`/
// `enemyLOS4`/`move4` — fork ruling 5) and calls trap_ICARUS_TaskIDPending
// (needs &Engine); no channel from this context-free faithful signature.
/// Raven `GM_CheckMoveState`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:440-460`
pub fn GM_CheckMoveState() {
    todo!("Port GM_CheckMoveState — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals plus this file's own file-static scratch (`enemyCS4`/`hitAlly4`/
// `impactPos4`/`faceEnemy4`/`shoot4` — fork ruling 5) and calls trap_Trace
// (needs &Engine); no channel from this context-free faithful signature.
/// Raven `GM_CheckFireState`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:468-556`
pub fn GM_CheckFireState() {
    todo!("Port GM_CheckFireState — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals; no channel from this context-free faithful signature.
/// Raven `NPC_GM_StartLaser`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:558-573`
pub fn NPC_GM_StartLaser() {
    todo!("Port NPC_GM_StartLaser — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC` ambient global; no
// channel from this context-free faithful signature.
/// Raven `GM_StartGloat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:575-587`
pub fn GM_StartGloat() {
    todo!("Port GM_StartGloat — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo`/`ucmd`/
// `g_entities` ambient globals plus this file's own file-static scratch
// (`enemyCS4`/`enemyDist4`/`enemyLOS4`/`faceEnemy4`/`hitAlly4`/`move4`/
// `shoot4`/`impactPos4` — fork ruling 5) and calls trap_InPVS/trap_Trace
// (needs &Engine); no channel from this context-free faithful signature.
/// Raven `NPC_BSGM_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:594-1229`
pub fn NPC_BSGM_Attack() {
    todo!("Port NPC_BSGM_Attack — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals and calls trap_Trace (needs &Engine); no channel from this
// context-free faithful signature.
/// Raven `NPC_BSGM_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:1231-1297`
pub fn NPC_BSGM_Default() {
    todo!("Port NPC_BSGM_Default — parked: ambient-state")
}
