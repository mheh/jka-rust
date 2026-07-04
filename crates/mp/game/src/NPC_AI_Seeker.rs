// PORT-COMPLETE: NPC_AI_Seeker.c 1/10
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Seeker.c`.
//!
//! One function ported; nine parked due to ambient-state infrastructure.
//! Nearly every function in this file relies on file-scope globals set up
//! by `SetNPCGlobals()` (NPC, NPCInfo, ucmd, etc.) or reads other ambient
//! state (level, g_entities, g_spskill cvars). The faithful signatures
//! carry no context parameter (`&Engine`, `&mut GameWorld`), and porting-rules
//! §B3 forbids inventing `static mut` globals. How these threadless faithful
//! signatures access the ambient state is an unsettled architectural question.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;


/// Raven `NPC_Seeker_Precache`.
///
/// Caches sound and effect resources for Seeker NPCs at map load time.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:26-31`
pub fn NPC_Seeker_Precache() {
    crate::g_utils::G_SoundIndex(c"sound/chars/seeker/misc/fire.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/seeker/misc/hiss.wav".as_ptr());
    crate::g_utils::G_EffectIndex(c"env/small_explode".as_ptr());
}

// PORT-ESCALATION(ambient-state): reads `NPC` global (set up by SetNPCGlobals); no channel to reach it from this context-free signature.
/// Raven `NPC_Seeker_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:34-46`
pub fn NPC_Seeker_Pain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    todo!("Port NPC_Seeker_Pain — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo`, `ucmd`, `level` globals; no channel to reach them from this context-free signature.
/// Raven `Seeker_MaintainHeight`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:49-148`
pub fn Seeker_MaintainHeight() {
    todo!("Port Seeker_MaintainHeight — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo`, `level` globals; no channel to reach them from this context-free signature.
/// Raven `Seeker_Strafe`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:151-239`
pub fn Seeker_Strafe() {
    todo!("Port Seeker_Strafe — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo`, `level`, `g_spskill` globals; no channel to reach them from this context-free signature.
/// Raven `Seeker_Hunt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:242-287`
pub fn Seeker_Hunt(
    visible: qboolean,
    advance: qboolean,
) {
    todo!("Port Seeker_Hunt — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC` global; no channel to reach it from this context-free signature.
/// Raven `Seeker_Fire`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:290-317`
pub fn Seeker_Fire() {
    todo!("Port Seeker_Fire — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo` globals; no channel to reach them from this context-free signature.
/// Raven `Seeker_Ranged`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:320-347`
pub fn Seeker_Ranged(
    visible: qboolean,
    advance: qboolean,
) {
    todo!("Port Seeker_Ranged — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo` globals; no channel to reach them from this context-free signature.
/// Raven `Seeker_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:350-380`
pub fn Seeker_Attack() {
    todo!("Port Seeker_Attack — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `g_entities` globals; no channel to reach them from this context-free signature.
/// Raven `Seeker_FindEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:383-436`
pub fn Seeker_FindEnemy() {
    todo!("Port Seeker_FindEnemy — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo`, `level`, `g_entities` globals; no channel to reach them from this context-free signature.
/// Raven `Seeker_FollowOwner`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:439-520`
pub fn Seeker_FollowOwner() {
    todo!("Port Seeker_FollowOwner — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `g_entities` globals; no channel to reach them from this context-free signature.
/// Raven `NPC_BSSeeker_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Seeker.c:523-574`
pub fn NPC_BSSeeker_Default() {
    todo!("Port NPC_BSSeeker_Default — parked: ambient-state")
}
