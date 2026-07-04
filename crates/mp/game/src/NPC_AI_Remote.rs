// PORT-COMPLETE: NPC_AI_Remote.c 1/10
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Remote.c`.
//!
//! One function ported; ten parked due to ambient-state infrastructure.
//! All functions except `NPC_Remote_Precache` rely on file-scope globals set
//! up by `SetNPCGlobals()` (NPC, NPCInfo, ucmd) or read other ambient state
//! (level, g_spskill). The faithful signatures carry no context parameter
//! (`&Engine`, `&mut GameWorld`), and porting-rules §B3 forbids inventing
//! `static mut` globals. How these threadless faithful signatures access the
//! ambient state is an unsettled architectural question.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

/// Raven `NPC_Remote_Precache`.
///
/// Caches sound and effect resources for Remote NPCs at map load time.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:17-22`
pub fn NPC_Remote_Precache() {
    crate::g_utils::G_SoundIndex(c"sound/chars/remote/misc/fire.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/remote/misc/hiss.wav".as_ptr());
    crate::g_utils::G_EffectIndex(c"env/small_explode".as_ptr());
}

// PORT-ESCALATION(ambient-state): calls `SetNPCGlobals()`, `Remote_Strafe()`, and reads `NPC` global; no channel to reach it from this context-free signature.
/// Raven `NPC_Remote_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:29-37`
pub fn NPC_Remote_Pain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    todo!("Port NPC_Remote_Pain — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo`, `level` globals; no channel to reach them from this context-free signature.
/// Raven `Remote_MaintainHeight`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:44-128`
pub fn Remote_MaintainHeight() {
    todo!("Port Remote_MaintainHeight — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo`, `level` globals and calls `trap_Trace`; no channel to reach them from this context-free signature.
/// Raven `Remote_Strafe`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:139-168`
pub fn Remote_Strafe() {
    todo!("Port Remote_Strafe — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo`, `level`, `g_spskill` globals; no channel to reach them from this context-free signature.
/// Raven `Remote_Hunt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:178-221`
pub fn Remote_Hunt(
    visible: qboolean,
    advance: qboolean,
    retreat: qboolean,
) {
    todo!("Port Remote_Hunt — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC` global and calls `CreateMissile()`; no channel to reach it from this context-free signature.
/// Raven `Remote_Fire`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:229-257`
pub fn Remote_Fire() {
    todo!("Port Remote_Fire — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo` globals; no channel to reach them from this context-free signature.
/// Raven `Remote_Ranged`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:264-277`
pub fn Remote_Ranged(
    visible: qboolean,
    advance: qboolean,
    retreat: qboolean,
) {
    todo!("Port Remote_Ranged — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo` globals; no channel to reach them from this context-free signature.
/// Raven `Remote_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:290-332`
pub fn Remote_Attack() {
    todo!("Port Remote_Attack — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC` global via `Remote_MaintainHeight()` and calls `NPC_BSIdle()`; no channel to reach it from this context-free signature.
/// Raven `Remote_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:339-344`
pub fn Remote_Idle() {
    todo!("Port Remote_Idle — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `ucmd` globals; no channel to reach them from this context-free signature.
/// Raven `Remote_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:351-367`
pub fn Remote_Patrol() {
    todo!("Port Remote_Patrol — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, `NPCInfo` globals; no channel to reach them from this context-free signature.
/// Raven `NPC_BSRemote_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Remote.c:375-389`
pub fn NPC_BSRemote_Default() {
    todo!("Port NPC_BSRemote_Default — parked: ambient-state")
}
