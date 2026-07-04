// PORT-COMPLETE: NPC_AI_Sniper.c 2/13
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Sniper.c`.
//!
//! Landed from the `fnskel.py` signature skeleton. 2 functions are
//! transcribed faithfully from packet + prelude alone; the remaining 13 are
//! parked (see the `PORT-ESCALATION` topics below), matching the precedent
//! set in `NPC_AI_Jedi.rs`/`NPC_AI_Stormtrooper.rs`/`NPC_AI_GalakMech.rs`/
//! `NPC_AI_Rancor.rs`: almost every body in this file reaches the file-scope
//! AI globals (`NPC`, `NPCInfo`, `ucmd`, `level`, `g_entities`) or this
//! file's own file-statics (`enemyLOS2`/`enemyCS2`/`faceEnemy2`/`move2`/
//! `shoot2`/`enemyDist2` — fork ruling 5: genuine cross-frame state ->
//! GameWorld field) or calls a `trap_*` (needs `&Engine`). Fork ruling 1
//! makes the AI globals `GameWorld`/`GameContext` state, but these faithful
//! signatures carry no `GameContext`/`&Engine` and the resolved cross-file
//! signatures are equally context-free.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use mp_bg::public::entity_event::entity_event_t::{EV_PUSHED1, EV_PUSHED3};

// Raven's anonymous `enum { LSTATE_NONE, LSTATE_UNDERFIRE, LSTATE_INVESTIGATE }`
// (file-scope local state, `gNPC_t::localState`) — not a central type, ported
// as file-local consts matching the C values.
// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:37-42`
const LSTATE_NONE: i32 = 0;
const LSTATE_UNDERFIRE: i32 = 1;
const LSTATE_INVESTIGATE: i32 = 2;

/// Raven `Sniper_ClearTimers`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:44-58`
pub fn Sniper_ClearTimers(ent: *mut gentity_t) {
    TIMER_Set(ent, c"chatter".as_ptr(), 0);
    TIMER_Set(ent, c"duck".as_ptr(), 0);
    TIMER_Set(ent, c"stand".as_ptr(), 0);
    TIMER_Set(ent, c"shuffleTime".as_ptr(), 0);
    TIMER_Set(ent, c"sleepTime".as_ptr(), 0);
    TIMER_Set(ent, c"enemyLastVisible".as_ptr(), 0);
    TIMER_Set(ent, c"roamTime".as_ptr(), 0);
    TIMER_Set(ent, c"hideTime".as_ptr(), 0);
    // FIXME: Slant for difficulty levels (Raven comment).
    TIMER_Set(ent, c"attackDelay".as_ptr(), 0);
    TIMER_Set(ent, c"stick".as_ptr(), 0);
    TIMER_Set(ent, c"scoutTime".as_ptr(), 0);
    TIMER_Set(ent, c"flee".as_ptr(), 0);
}

// PORT-ESCALATION(constants-in-scope): needs the anonymous `squadState_e`
// value `SQUAD_IDLE` (`self->NPC->squadState = SQUAD_IDLE`); not re-exported
// by the prelude and not resolved anywhere in this crate, no import path or
// numeric value available from the packet.
/// Raven `NPC_Sniper_PlayConfusionSound`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:60-76`
pub fn NPC_Sniper_PlayConfusionSound(self_: *mut gentity_t) {
    todo!("Port NPC_Sniper_PlayConfusionSound — parked: constants-in-scope (SQUAD_IDLE)")
}

/// Raven `NPC_Sniper_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:85-98`
pub fn NPC_Sniper_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        (*npc).localState = LSTATE_UNDERFIRE;

        TIMER_Set(self_, c"duck".as_ptr(), -1);
        TIMER_Set(self_, c"stand".as_ptr(), 2000);

        NPC_Pain(self_, attacker, damage);

        if damage == 0 && (*self_).health > 0 {
            // FIXME: better way to know I was pushed (Raven comment).
            G_AddVoiceEvent(self_, Q_irand(EV_PUSHED1 as c_int, EV_PUSHED3 as c_int), 2000);
        }
    }
}

// PORT-ESCALATION(ambient-state): writes the `NPCInfo`/`NPC` ambient globals
// (`NPC_FreeCombatPoint( NPCInfo->combatPoint, qtrue ); NPCInfo->goalEntity =
// NULL;`); no channel to reach them from this context-free faithful
// signature.
/// Raven `Sniper_HoldPosition`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:106-116`
pub fn Sniper_HoldPosition() {
    todo!("Port Sniper_HoldPosition — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/`level`; no
// channel from this context-free faithful signature.
/// Raven `Sniper_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:124-177`
pub fn Sniper_Move() -> qboolean {
    todo!("Port Sniper_Move — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/`ucmd`/
// `level`; no channel from this context-free faithful signature.
/// Raven `NPC_BSSniper_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:185-275`
pub fn NPC_BSSniper_Patrol() {
    todo!("Port NPC_BSSniper_Patrol — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`level`/file-static
// `enemyDist2`/`enemyLOS2`, writes `NPCInfo`/`faceEnemy2`/`move2`; no
// channel from this context-free faithful signature.
/// Raven `Sniper_CheckMoveState`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:308-381`
pub fn Sniper_CheckMoveState() {
    todo!("Port Sniper_CheckMoveState — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/`level`; no
// channel from this context-free faithful signature.
/// Raven `Sniper_ResolveBlockedShot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:383-434`
pub fn Sniper_ResolveBlockedShot() {
    todo!("Port Sniper_ResolveBlockedShot — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/file-static
// `enemyCS2`/`shoot2`/`level`; no channel from this context-free faithful
// signature.
/// Raven `Sniper_CheckFireState`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:442-486`
pub fn Sniper_CheckFireState() {
    todo!("Port Sniper_CheckFireState — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the ambient `NPC` global and the
// ambient `g_entities` array (`hitEnt = &g_entities[hit]`); no channel from
// this context-free faithful signature.
/// Raven `Sniper_EvaluateShot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:488-506`
pub fn Sniper_EvaluateShot(hit: c_int) -> qboolean {
    todo!("Port Sniper_EvaluateShot — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/file-static
// `enemyDist2`/`enemyLOS2`/`shoot2`/`g_spskill`/`level` and calls
// `trap_Trace` (needs `&Engine`); no channel from this context-free
// faithful signature.
/// Raven `Sniper_FaceEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:508-603`
pub fn Sniper_FaceEnemy() {
    todo!("Port Sniper_FaceEnemy — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the ambient `NPC`/`NPCInfo`
// globals; no channel from this context-free faithful signature.
/// Raven `Sniper_UpdateEnemyPos`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:605-623`
pub fn Sniper_UpdateEnemyPos() {
    todo!("Port Sniper_UpdateEnemyPos — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the ambient `NPC` global; no
// channel from this context-free faithful signature.
/// Raven `Sniper_StartHide`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:631-638`
pub fn Sniper_StartHide() {
    todo!("Port Sniper_StartHide — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/`ucmd`/
// file-static `enemyCS2`/`enemyDist2`/`enemyLOS2`/`faceEnemy2`/`move2`/
// `shoot2`/`level` and calls `trap_Trace` (needs `&Engine`); no channel from
// this context-free faithful signature.
/// Raven `NPC_BSSniper_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:640-852`
pub fn NPC_BSSniper_Attack() {
    todo!("Port NPC_BSSniper_Attack — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the ambient `NPC` global (`!NPC->enemy`);
// no channel from this context-free faithful signature.
/// Raven `NPC_BSSniper_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:854-864`
pub fn NPC_BSSniper_Default() {
    todo!("Port NPC_BSSniper_Default — parked: ambient-state")
}
