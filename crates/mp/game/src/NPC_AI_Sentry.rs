// PORT-COMPLETE: NPC_AI_Sentry.c 12/12
//! Faithful port of `oracle/oracle/codemp/game/NPC_AI_Sentry.c`.
//!
//! Sentry droid NPC AI behavior.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::NPC_reactions::NPC_Pain;
use crate::NPC_utils::G_ActivateBehavior;
use crate::bg_misc::BG_FindItemForAmmo;
use crate::entity::flags::FL_SHIELDED;
use crate::g_items::RegisterItem;
use crate::g_timer::TIMER_Set;
use crate::g_utils::{G_EffectIndex, G_Sound, G_SoundIndex};
use crate::npc_c::NPC_SetAnim;
use crate::q_math::Q_irand;
use crate::q_shared::va;
use crate::NPC_AI_Stormtrooper::NPC_CheckPlayerTeamStealth;

// Local state enums
const LSTATE_NONE: i32 = 0;
const LSTATE_ASLEEP: i32 = 1;
const LSTATE_WAKEUP: i32 = 2;
pub const LSTATE_ACTIVE: i32 = 3;
pub const LSTATE_POWERING_UP: i32 = 4;
pub const LSTATE_ATTACKING: i32 = 5;

const MIN_DISTANCE: i32 = 256;
const MIN_DISTANCE_SQR: i32 = MIN_DISTANCE * MIN_DISTANCE;

pub const SENTRY_FORWARD_BASE_SPEED: i32 = 10;
pub const SENTRY_FORWARD_MULTIPLIER: i32 = 5;

pub const SENTRY_VELOCITY_DECAY: f32 = 0.85f32;
pub const SENTRY_STRAFE_VEL: i32 = 256;
pub const SENTRY_STRAFE_DIS: i32 = 200;
pub const SENTRY_UPWARD_PUSH: i32 = 32;
pub const SENTRY_HOVER_HEIGHT: i32 = 24;

/// Raven `NPC_Sentry_Precache`.
///
/// Precache sentry sounds and effects.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:37-57`
// PORT-ESCALATION(seam-threading): faithful skeleton signature carries no
// `GameContext`/`&Engine` receiver, but this body calls a callee (or reads a
// file-scope global) that needs one (ruling 1/precedent `ai_main.rs`/
// `g_weapon.rs`) — how is state threaded in?
pub fn NPC_Sentry_Precache(ctx: GameContext<'_>) {
    todo!("Port NPC_Sentry_Precache — parked: seam-threading")
}

/// Raven `sentry_use`.
///
/// Callback when sentry is used. Activates behavior and transitions from sleep.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:64-72`
// PORT-ESCALATION(unported-type): reads/returns Raven `animNumber_t`
// (`BOTH_*`/`TORSO_*`/`LEGS_*`) enumerator(s) — this ~1500-entry enum is a
// documented deferred type-port item (`docs/type-port-todo.md`), not a
// missing `use`. Left as unresolved bare identifiers, these silently
// type-check as irrefutable match-pattern bindings (always-true), which is
// a behavioral bug, not just a compile gap — parked instead.
pub fn sentry_use(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    todo!("Port sentry_use — parked: unported-type (animNumber_t)")
}

/// Raven `NPC_Sentry_Pain`.
///
/// Handle sentry taking damage. Special handling for DEMP2 disable.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:79-105`
// PORT-ESCALATION(unported-type): reads/returns Raven `animNumber_t`
// (`BOTH_*`/`TORSO_*`/`LEGS_*`) enumerator(s) — this ~1500-entry enum is a
// documented deferred type-port item (`docs/type-port-todo.md`), not a
// missing `use`. Left as unresolved bare identifiers, these silently
// type-check as irrefutable match-pattern bindings (always-true), which is
// a behavioral bug, not just a compile gap — parked instead.
pub fn NPC_Sentry_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    todo!("Port NPC_Sentry_Pain — parked: unported-type (animNumber_t)")
}

/// Raven `Sentry_Fire`.
///
/// Fire a blaster bolt from one of three muzzles. Difficulty-scaled damage.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:112-203`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Sentry_Fire(ctx: GameContext<'_>) {
    todo!("Port Sentry_Fire — parked: ai-context")
}

/// Raven `Sentry_MaintainHeight`.
///
/// Maintain hovering height relative to enemy or goal. Apply friction to velocity.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:210-304`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Sentry_MaintainHeight(ctx: GameContext<'_>) {
    todo!("Port Sentry_MaintainHeight — parked: ai-context")
}

/// Raven `Sentry_Idle`.
///
/// Idle behavior: sleep or wake up based on local state.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:311-331`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Sentry_Idle(ctx: GameContext<'_>) {
    todo!("Port Sentry_Idle — parked: ai-context")
}

/// Raven `Sentry_Strafe`.
///
/// Strafe horizontally away from enemy. Trace to check validity.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:338-365`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Sentry_Strafe(ctx: GameContext<'_>) {
    todo!("Port Sentry_Strafe — parked: ai-context")
}

/// Raven `Sentry_Hunt`.
///
/// Hunt the enemy. Move toward or strafe, depending on visibility.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:372-411`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Sentry_Hunt(
    ctx: GameContext<'_>,visible: qboolean, advance: qboolean) {
    todo!("Port Sentry_Hunt — parked: ai-context")
}

/// Raven `Sentry_RangedAttack`.
///
/// Ranged attack: fire or close shield. Hunt if pursuing.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:418-448`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Sentry_RangedAttack(
    ctx: GameContext<'_>,visible: qboolean, advance: qboolean) {
    todo!("Port Sentry_RangedAttack — parked: ai-context")
}

/// Raven `Sentry_AttackDecision`.
///
/// Decide how to attack: maintain height, check enemy, determine if visible/in range.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:455-510`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Sentry_AttackDecision(ctx: GameContext<'_>) {
    todo!("Port Sentry_AttackDecision — parked: ai-context")
}

/// Raven `NPC_Sentry_Patrol`.
///
/// Patrol behavior: maintain height, check for stealth, update goal, talk.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:519-550`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn NPC_Sentry_Patrol(ctx: GameContext<'_>) {
    todo!("Port NPC_Sentry_Patrol — parked: ai-context")
}

/// Raven `NPC_BSSentry_Default`.
///
/// Main behavior selector: handle use callbacks, attack if enemy, patrol, or idle.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Sentry.c:557-577`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn NPC_BSSentry_Default(ctx: GameContext<'_>) {
    todo!("Port NPC_BSSentry_Default — parked: ai-context")
}
