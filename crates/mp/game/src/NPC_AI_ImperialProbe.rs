// PORT-COMPLETE: NPC_AI_ImperialProbe.c 12/12
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c`.
//!
//! Imperial Probe droid AI behavior: idle, patrol, hunt, strafe, ranged attack.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::NPC_AI_Default::NPC_BSIdle;
use crate::NPC_reactions::{NPC_GetPainChance, NPC_Pain};
use crate::NPC_utils::NPC_UpdateAngles;
use crate::bg_lib::rand;
use crate::bg_misc::{BG_FindItemForAmmo, BG_FindItemForWeapon};
use crate::g_items::RegisterItem;
use crate::g_utils::{G_EffectIndex, G_SoundIndex};
use crate::npc_c::NPC_SetAnim;
use crate::q_math::{AngleVectors, VectorNormalize};
use crate::q_shared::va;

// Local state enums
// Source: oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:10-17
const LSTATE_NONE: i32 = 0;
const LSTATE_BACKINGUP: i32 = 1;
const LSTATE_SPINNING: i32 = 2;
const LSTATE_PAIN: i32 = 3;
const LSTATE_DROP: i32 = 4;

// Height maintenance
// Source: oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:124-127
const VELOCITY_DECAY: f32 = 0.85;

// Strafe parameters
// Source: oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:178-181
const HUNTER_STRAFE_VEL: c_int = 256;
const HUNTER_STRAFE_DIS: c_int = 200;
const HUNTER_UPWARD_PUSH: c_int = 32;

// Hunt parameters
// Source: oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:294-296
const HUNTER_FORWARD_BASE_SPEED: c_int = 10;
const HUNTER_FORWARD_MULTIPLIER: c_int = 5;

// Melee range
// Source: oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:448-452
const MIN_MELEE_RANGE: c_int = 320;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;
const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

/// Raven `NPC_Probe_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:21-40`
// PORT-ESCALATION(seam-threading): faithful skeleton signature carries no
// `GameContext`/`&Engine` receiver, but this body calls a callee (or reads a
// file-scope global) that needs one (ruling 1/precedent `ai_main.rs`/
// `g_weapon.rs`) — how is state threaded in?
pub fn NPC_Probe_Precache(ctx: GameContext<'_>) {
    todo!("Port NPC_Probe_Precache — parked: seam-threading")
}

/// Raven `ImperialProbe_MaintainHeight`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:49-170`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn ImperialProbe_MaintainHeight(ctx: GameContext<'_>) {
    todo!("Port ImperialProbe_MaintainHeight — parked: ai-context")
}

/// Raven `ImperialProbe_Strafe`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:182-209`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn ImperialProbe_Strafe(ctx: GameContext<'_>) {
    todo!("Port ImperialProbe_Strafe — parked: ai-context")
}

/// Raven `ImperialProbe_Hunt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:220-261`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn ImperialProbe_Hunt(
    ctx: GameContext<'_>,visible: qboolean, advance: qboolean) {
    todo!("Port ImperialProbe_Hunt — parked: ai-context")
}

/// Raven `ImperialProbe_FireBlaster`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:268-324`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn ImperialProbe_FireBlaster(ctx: GameContext<'_>) {
    todo!("Port ImperialProbe_FireBlaster — parked: ai-context")
}

/// Raven `ImperialProbe_Ranged`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:331-363`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn ImperialProbe_Ranged(
    ctx: GameContext<'_>,visible: qboolean, advance: qboolean) {
    todo!("Port ImperialProbe_Ranged — parked: ai-context")
}

/// Raven `ImperialProbe_AttackDecision`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:377-426`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn ImperialProbe_AttackDecision(ctx: GameContext<'_>) {
    todo!("Port ImperialProbe_AttackDecision — parked: ai-context")
}

/// Raven `NPC_Probe_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:433-498`
// PORT-ESCALATION(seam-threading): faithful skeleton signature carries no
// `GameContext`/`&Engine` receiver, but this body calls a callee (or reads a
// file-scope global) that needs one (ruling 1/precedent `ai_main.rs`/
// `g_weapon.rs`) — how is state threaded in?
pub fn NPC_Probe_Pain(
    ctx: GameContext<'_>,self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    todo!("Port NPC_Probe_Pain — parked: seam-threading")
}

/// Raven `ImperialProbe_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:506-511`
pub fn ImperialProbe_Idle(ctx: GameContext<'_>) {
    ImperialProbe_MaintainHeight(ctx);
    NPC_BSIdle(ctx);
}

/// Raven `ImperialProbe_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:518-556`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn ImperialProbe_Patrol(ctx: GameContext<'_>) {
    todo!("Port ImperialProbe_Patrol — parked: ai-context")
}

/// Raven `ImperialProbe_Wait`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:563-582`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn ImperialProbe_Wait(ctx: GameContext<'_>) {
    todo!("Port ImperialProbe_Wait — parked: ai-context")
}

/// Raven `NPC_BSImperialProbe_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_ImperialProbe.c:589-609`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn NPC_BSImperialProbe_Default(ctx: GameContext<'_>) {
    todo!("Port NPC_BSImperialProbe_Default — parked: ai-context")
}
