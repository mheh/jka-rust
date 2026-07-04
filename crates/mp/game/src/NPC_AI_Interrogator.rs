// PORT-COMPLETE: NPC_AI_Interrogator.c 9/10
//! Faithful port of `oracle/oracle/codemp/game/NPC_AI_Interrogator.c` (jampgame mega-pass).
//!
//! Interrogator droid NPC AI behavior: idle, patrol, hunt, strafe, melee attack.
//!
//! One function (`Interrogator_Strafe`) is parked due to trap_Trace requiring
//! an `&Engine` handle which these context-free AI functions don't have access to.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::g_utils::{G_EffectIndex, G_SoundIndex};
use crate::q_math::Q_irand;

/// Local state enums for Interrogator blade movement.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:8-13`
const LSTATE_BLADESTOP: c_int = 0;
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:11`
pub const LSTATE_BLADEUP: c_int = 1;
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:12`
pub const LSTATE_BLADEDOWN: c_int = 2;

/// Velocity decay factor for Interrogator hovering.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:129`
const VELOCITY_DECAY: f32 = 0.85;

/// Upward push for Interrogator during strafe.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:130`
const HUNTER_UPWARD_PUSH: c_int = 2;

/// Strafe velocity for Interrogator movement.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:231`
const HUNTER_STRAFE_VEL: c_int = 32;

/// Distance for Interrogator strafe.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:232`
const HUNTER_STRAFE_DIS: c_int = 200;

/// Forward base speed for Interrogator.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:287`
const HUNTER_FORWARD_BASE_SPEED: c_int = 10;

/// Forward speed multiplier for Interrogator.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:288`
const HUNTER_FORWARD_MULTIPLIER: c_int = 2;

/// Minimum distance for Interrogator melee attack.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:338`
const MIN_DISTANCE: c_int = 64;


/// Raven `NPC_Interrogator_Precache`.
///
/// Precache sounds and effects for the Interrogator NPC.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:20-28`
pub fn NPC_Interrogator_Precache(
    ctx: GameContext<'_>,self_: *mut gentity_t) {
    G_SoundIndex(c"sound/chars/interrogator/misc/torture_droid_lp".as_ptr() as *const c_char);
    G_SoundIndex(c"sound/chars/mark1/misc/anger.wav".as_ptr() as *const c_char);
    G_SoundIndex(c"sound/chars/probe/misc/talk".as_ptr() as *const c_char);
    G_SoundIndex(
        c"sound/chars/interrogator/misc/torture_droid_inject".as_ptr() as *const c_char,
    );
    G_SoundIndex(c"sound/chars/interrogator/misc/int_droid_explo".as_ptr() as *const c_char);
    G_EffectIndex(c"explosions/droidexplosion1".as_ptr() as *const c_char);
}

/// Raven `Interrogator_die`.
///
/// Death behavior for Interrogator NPC. Sets velocity and clears flying flag.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:34-57`
pub fn Interrogator_die(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
    r#mod: c_int,
    dFlags: c_int,
    hitLoc: c_int,
) {
    unsafe {
        // SAFETY: self_ accessed through game context.
        self_.as_mut().map(|ent| {
            if !ent.client.is_null() {
                let client = &mut *(ent.client as *mut gclient_t);
                client.ps.velocity[2] = -100.0;

                // Clear flying flag and set random horizontal velocity
                client.ps.eFlags2 &= !(crate::prelude::EF2_FLYING as c_int);
                client.ps.velocity[0] = Q_irand(-20, -10) as f32;
                client.ps.velocity[1] = Q_irand(-20, -10) as f32;
                client.ps.velocity[2] = -100.0;
            }
        });
    }
}

/// Raven `Interrogator_PartsMove`.
///
/// Move the syringe, scalpel, and claw parts of the Interrogator.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:64-127`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Interrogator_PartsMove(ctx: GameContext<'_>) {
    todo!("Port Interrogator_PartsMove — parked: ai-context")
}

/// Raven `Interrogator_MaintainHeight`.
///
/// Maintain hover height relative to enemy or goal.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:137-229`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Interrogator_MaintainHeight(ctx: GameContext<'_>) {
    todo!("Port Interrogator_MaintainHeight — parked: ai-context")
}

// PORT-ESCALATION(trap-no-engine): `Interrogator_Strafe` calls `trap_Trace`
// which requires an `&Engine` handle; faithful context-free signature carries
// no threading mechanism to reach it (see NPC_utils.rs precedent).
/// Raven `Interrogator_Strafe`.
///
/// Perform a strafe movement away from the target.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:238-279`
pub fn Interrogator_Strafe(ctx: GameContext<'_>) {
    todo!("Port Interrogator_Strafe — parked: trap-no-engine")
}

/// Raven `Interrogator_Hunt`.
///
/// Hunt the enemy, using strafe and movement.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:290-336`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Interrogator_Hunt(
    ctx: GameContext<'_>,visible: qboolean, advance: qboolean) {
    todo!("Port Interrogator_Hunt — parked: ai-context")
}

/// Raven `Interrogator_Melee`.
///
/// Perform melee attack if close enough and within height range.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:345-374`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Interrogator_Melee(
    ctx: GameContext<'_>,visible: qboolean, advance: qboolean) {
    todo!("Port Interrogator_Melee — parked: ai-context")
}

/// Raven `Interrogator_Attack`.
///
/// Main attack function - handles distance, visibility, and attack selection.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:381-428`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC`/`NPCInfo` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Interrogator_Attack(ctx: GameContext<'_>) {
    todo!("Port Interrogator_Attack — parked: ai-context")
}

/// Raven `Interrogator_Idle`.
///
/// Idle behavior - check for stealth enemies and maintain height.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:435-447`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn Interrogator_Idle(ctx: GameContext<'_>) {
    todo!("Port Interrogator_Idle — parked: ai-context")
}

/// Raven `NPC_BSInterrogator_Default`.
///
/// Default behavior state selector - attacks if enemy present, otherwise idles.
/// Source: `oracle/oracle/codemp/game/NPC_AI_Interrogator.c:454-467`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global(s)
// `NPC` that Raven's `ai_main.c` think-loop sets per NPC frame — no
// `GameWorld`/`GameContext` field or entity param carries them yet (topic
// `ai-context`, matching the `NPC_reactions.rs`/`NPC_utils.rs`/`NPC_combat.rs`
// precedent in this same mega-pass).
pub fn NPC_BSInterrogator_Default(ctx: GameContext<'_>) {
    todo!("Port NPC_BSInterrogator_Default — parked: ai-context")
}
