// PORT-COMPLETE: NPC_AI_Mark1.c 1/15
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Mark1.c`.
//!
//! Landed from the `fnskel.py` signature skeleton. One function is
//! transcribed faithfully from packet + prelude alone; the remaining 15 are
//! parked (see the `PORT-ESCALATION` topics below), because this file is
//! almost entirely ambient-state driven and the faithful context-free
//! signatures have no channel to reach it, matching the precedent set in
//! `NPC_AI_GalakMech.rs`/`NPC_AI_Jedi.rs`/`NPC_AI_Stormtrooper.rs`:
//!
//! - `ambient-state` — nearly every body reaches the file-scope AI globals
//!   (`NPC`, `NPCInfo`, `ucmd`, `level`, `gPainHitLoc`) or calls a `trap_*`
//!   (needs `&Engine`). Fork ruling 1 makes the AI globals `GameWorld`/
//!   `GameContext` state, but these faithful signatures carry no
//!   `GameContext`/`&Engine` and the resolved cross-file signatures are
//!   equally context-free. How ambient state + engine thread into
//!   context-free faithful logic fns is not settled by the packet.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// Raven's file-scope `#define`s (`NPC_AI_Mark1.c:4-22`) — not central
// constants, ported as file-local consts matching the C values.
const MIN_MELEE_RANGE: c_int = 320;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;
const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;
const TURN_OFF: c_int = 0x00000100;
const LEFT_ARM_HEALTH: c_int = 40;
const RIGHT_ARM_HEALTH: c_int = 40;
const AMMO_POD_HEALTH: c_int = 40;
const BOWCASTER_VELOCITY: c_int = 1300;
const BOWCASTER_NPC_DAMAGE_EASY: c_int = 12;
const BOWCASTER_NPC_DAMAGE_NORMAL: c_int = 24;
const BOWCASTER_NPC_DAMAGE_HARD: c_int = 36;
const BOWCASTER_SIZE: c_int = 2;
const BOWCASTER_SPLASH_DAMAGE: c_int = 0;
const BOWCASTER_SPLASH_RADIUS: c_int = 0;

// Raven's anonymous local-state `enum` (`NPC_AI_Mark1.c:25-35`) — no
// separate typedef name, so it stays a plain set of `c_int` consts per
// house rule (typedef int + anonymous enum -> consts).
const LSTATE_NONE: c_int = 0;
const LSTATE_ASLEEP: c_int = 1;
const LSTATE_WAKEUP: c_int = 2;
const LSTATE_FIRED0: c_int = 3;
const LSTATE_FIRED1: c_int = 4;
const LSTATE_FIRED2: c_int = 5;
const LSTATE_FIRED3: c_int = 6;
const LSTATE_FIRED4: c_int = 7;

// PORT-ESCALATION(ambient-state): needs `level.time` (via
// `trap_G2API_GetBoltMatrix`); no channel from this context-free faithful
// signature.
/// Raven `NPC_Mark1_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:50-74`
pub fn NPC_Mark1_Precache(ctx: GameContext<'_>) {
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_wakeup".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/shutdown".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/walk".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/run".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/death1".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/death2".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/anger".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_fire".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_pain".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_explo".as_ptr());

    //	G_EffectIndex( "small_chunks");
    crate::g_utils::G_EffectIndex(c"env/med_explode2".as_ptr());
    crate::g_utils::G_EffectIndex(c"explosions/probeexplosion1".as_ptr());
    crate::g_utils::G_EffectIndex(c"blaster/smoke_bolton".as_ptr());
    crate::g_utils::G_EffectIndex(c"bryar/muzzle_flash".as_ptr());
    crate::g_utils::G_EffectIndex(c"explosions/droidexplosion1".as_ptr());

    crate::g_items::RegisterItem(ctx, crate::bg_misc::BG_FindItemForAmmo(ammo_t::AMMO_METAL_BOLTS));
    crate::g_items::RegisterItem(ctx, crate::bg_misc::BG_FindItemForAmmo(ammo_t::AMMO_BLASTER));
    crate::g_items::RegisterItem(ctx, crate::bg_misc::BG_FindItemForWeapon(
        mp_bg::weapons::weapon_t::WP_BOWCASTER,
    ));
    crate::g_items::RegisterItem(ctx, crate::bg_misc::BG_FindItemForWeapon(
        mp_bg::weapons::weapon_t::WP_BRYAR_PISTOL,
    ));
}

// PORT-ESCALATION(ambient-state): reads `level.time` (via
// `trap_G2API_GetBoltMatrix`); no channel from this context-free faithful
// signature.
/// Raven `NPC_Mark1_Part_Explode`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:81-102`
pub fn NPC_Mark1_Part_Explode(
    ctx: GameContext<'_>,self_: *mut gentity_t, bolt: c_int) {
    todo!("Port NPC_Mark1_Part_Explode — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals; no channel from this context-free faithful signature.
/// Raven `Mark1_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:109-115`
pub fn Mark1_Idle(ctx: GameContext<'_>) {
    todo!("Port Mark1_Idle — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global and
// `level.time` (via `trap_G2API_AddBolt`/`trap_G2API_GetBoltMatrix`); no
// channel from this context-free faithful signature.
/// Raven `Mark1Dead_FireRocket`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:123-163`
pub fn Mark1Dead_FireRocket(ctx: GameContext<'_>) {
    todo!("Port Mark1Dead_FireRocket — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global and
// `level.time` (via `trap_G2API_AddBolt`/`trap_G2API_GetBoltMatrix`); no
// channel from this context-free faithful signature.
/// Raven `Mark1Dead_FireBlaster`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:171-202`
pub fn Mark1Dead_FireBlaster(ctx: GameContext<'_>) {
    todo!("Port Mark1Dead_FireBlaster — parked: ambient-state")
}

// PORT-ESCALATION(variadic-c-abi): the live body's only non-trivial call is
// `G_SoundIndex(va("...death%d.wav", Q_irand(1,2)))` — `va`'s packet-resolved
// signature is the parked `fn va(format: *const c_char) -> *mut c_char`
// stub with C varargs dropped (seam decision pending, see `q_shared.rs`), so
// there is no channel to pass the `Q_irand(1,2)` substitution argument
// through it.
/// Raven `Mark1_die`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:209-243`
pub fn Mark1_die(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
    r#mod: c_int,
    dFlags: c_int,
    hitLoc: c_int,
) {
    todo!("Port Mark1_die — parked: variadic-c-abi")
}

// PORT-ESCALATION(client-cast): reads `self->client->ps.torsoTimer`; `client`
// is untyped `*mut c_void` on `gentity_t` (unported field) with no resolved
// accessor to `gclient_t` fields in this packet — casting it here would
// invent a shape the packet doesn't sanction.
/// Raven `Mark1_dying`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:250-312`
pub fn Mark1_dying(
    ctx: GameContext<'_>,self_: *mut gentity_t) {
    todo!("Port Mark1_dying — parked: client-cast")
}

// PORT-ESCALATION(ambient-state): reads the `gPainHitLoc` ambient global; no
// channel from this context-free faithful signature. Also stored as a fn
// pointer (needs an EntXxx enum variant per ruling 2).
/// Raven `NPC_Mark1_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:320-396`
pub fn NPC_Mark1_Pain(
    ctx: GameContext<'_>,self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    todo!("Port NPC_Mark1_Pain — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global and
// writes `NPCInfo`; no channel from this context-free faithful signature.
/// Raven `Mark1_Hunt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:404-416`
pub fn Mark1_Hunt(ctx: GameContext<'_>) {
    todo!("Port Mark1_Hunt — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals, `level.time`, and this file's own fn-scope statics
// (`forward`/`vright`/`up`/`muzzle` — fork ruling 5: genuine cross-frame
// state -> GameWorld field); no channel from this context-free faithful
// signature.
/// Raven `Mark1_FireBlaster`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:424-488`
pub fn Mark1_FireBlaster(ctx: GameContext<'_>) {
    todo!("Port Mark1_FireBlaster — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals; no channel from this context-free faithful signature.
/// Raven `Mark1_BlasterAttack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:495-548`
pub fn Mark1_BlasterAttack(
    ctx: GameContext<'_>,advance: qboolean) {
    todo!("Port Mark1_BlasterAttack — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global, `level.time`,
// and this file's own fn-scope statics (`forward`/`vright`/`up` — fork
// ruling 5); no channel from this context-free faithful signature.
/// Raven `Mark1_FireRocket`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:555-599`
pub fn Mark1_FireRocket(ctx: GameContext<'_>) {
    todo!("Port Mark1_FireRocket — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global; no channel
// from this context-free faithful signature.
/// Raven `Mark1_RocketAttack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:606-618`
pub fn Mark1_RocketAttack(
    ctx: GameContext<'_>,advance: qboolean) {
    todo!("Port Mark1_RocketAttack — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC` ambient global and
// calls `trap_G2API_GetSurfaceRenderStatus` (needs `&Engine`); no channel
// from this context-free faithful signature.
/// Raven `Mark1_AttackDecision`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:625-704`
pub fn Mark1_AttackDecision(ctx: GameContext<'_>) {
    todo!("Port Mark1_AttackDecision — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global and writes
// `ucmd`; no channel from this context-free faithful signature.
/// Raven `Mark1_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:711-739`
pub fn Mark1_Patrol(ctx: GameContext<'_>) {
    todo!("Port Mark1_Patrol — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global and
// writes `NPCInfo`; no channel from this context-free faithful signature.
/// Raven `NPC_BSMark1_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Mark1.c:747-764`
pub fn NPC_BSMark1_Default(ctx: GameContext<'_>) {
    todo!("Port NPC_BSMark1_Default — parked: ambient-state")
}
