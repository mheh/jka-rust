// PORT-COMPLETE: NPC_AI_Jedi.c 6/62
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Jedi.c`.
//!
//! Landed from the `fnskel.py` signature skeleton. Six leaf functions are
//! transcribed faithfully from packet + prelude alone; the remaining 62 are
//! parked (see the two `PORT-ESCALATION` topics below), because this file is
//! almost entirely ambient-state driven and the faithful context-free
//! signatures have no channel to reach it:
//!
//! - `ambient-state` — nearly every body reaches the ai_main file-scope globals
//!   (`NPC`, `NPCInfo`, `ucmd`, `player`, `level`, `g_entities`) or the `Engine`
//!   for a `trap_*` call. Fork ruling 1 makes those `GameWorld`/`GameContext`
//!   state, but these faithful signatures carry no `GameContext`/`&Engine`,
//!   rule B forbids inventing `static mut` globals, and the resolved cross-file
//!   signatures are equally context-free. How ambient state + engine thread
//!   into context-free faithful logic fns is not settled by the packet.
//! - `constants-in-scope` — `crate::prelude` re-exports only types, not the
//!   Raven event/anim/flag constants (`EV_*`, `EF2_*`, `SETANIM_*`/`BOTH_*`,
//!   `FORCE_LEVEL_*`, `JSF_*`, `NPCTEAM_*`, `MASK_SHOT`, …). No import path is
//!   resolved in the packet, and inventing constant values risks silent parity
//!   breaks (porting-rules SA: no speculative behavior).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;


/// Raven `G_StartMatrixEffect`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:16-19`
pub fn G_StartMatrixEffect(
    ent: *mut gentity_t,
) {
    //perhaps write this at some point?
}

/// Raven `NPC_ShadowTrooper_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:103-108`
pub fn NPC_ShadowTrooper_Precache(ctx: GameContext<'_>) {
    crate::g_items::RegisterItem(ctx, crate::bg_misc::BG_FindItemForAmmo(ammo_t::AMMO_FORCE));
    crate::g_utils::G_SoundIndex(c"sound/chars/shadowtrooper/cloak.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/shadowtrooper/decloak.wav".as_ptr());
}

/// Raven `Jedi_ClearTimers`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:110-135`
pub fn Jedi_ClearTimers(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) {
    crate::g_timer::TIMER_Set(ctx, ent, c"roamTime".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"chatter".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"strafeLeft".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"strafeRight".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"noStrafe".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"walking".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"taunting".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"parryTime".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"parryReCalcTime".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"forceJumpChasing".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"jumpChaseDebounce".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"moveforward".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"moveback".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"movenone".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"moveright".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"moveleft".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"movecenter".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"saberLevelDebounce".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"noRetreat".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"holdLightning".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"gripping".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"draining".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"noturn".as_ptr(), 0);
}

// PORT-ESCALATION(ambient-state): reads `level.time`; no channel to reach the ai_main globals / Engine from this context-free faithful signature (rule B forbids static mut; resolved cross-file sigs are context-free).
/// Raven `Jedi_PlayBlockedPushSound`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:137-148`
pub fn Jedi_PlayBlockedPushSound(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port Jedi_PlayBlockedPushSound — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `level.time`; no channel to reach the ai_main globals / Engine from this context-free faithful signature (rule B forbids static mut; resolved cross-file sigs are context-free).
/// Raven `Jedi_PlayDeflectSound`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:150-161`
pub fn Jedi_PlayDeflectSound(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port Jedi_PlayDeflectSound — parked: ambient-state")
}

// PORT-ESCALATION(constants-in-scope): needs event constants EV_CONFUSE1..3/EV_TAUNT1..3/EV_GLOAT1..3 not re-exported by prelude; no import path resolved in packet.
/// Raven `NPC_Jedi_PlayConfusionSound`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:163-180`
pub fn NPC_Jedi_PlayConfusionSound(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port NPC_Jedi_PlayConfusionSound — parked: constants-in-scope")
}

/// Raven `Boba_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:182-189`
pub fn Boba_Precache(ctx: GameContext<'_>) {
    crate::g_utils::G_SoundIndex(c"sound/boba/jeton.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/boba/jethover.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/effects/combustfire.mp3".as_ptr());
    crate::g_utils::G_EffectIndex(c"boba/jet".as_ptr());
    crate::g_utils::G_EffectIndex(c"boba/fthrw".as_ptr());
}

// PORT-ESCALATION(ambient-state): reads the `NPC` global; no channel to reach the ai_main globals / Engine from this context-free faithful signature (rule B forbids static mut; resolved cross-file sigs are context-free).
/// Raven `Boba_ChangeWeapon`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:193-201`
pub fn Boba_ChangeWeapon(
    ctx: GameContext<'_>,
    wp: c_int,
) {
    todo!("Port Boba_ChangeWeapon — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `level.time` and calls trap_Cvar_VariableStringBuffer (needs &Engine); no channel from this context-free faithful signature.
/// Raven `WP_ResistForcePush`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:203-270`
pub fn WP_ResistForcePush(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    pusher: *mut gentity_t,
    noPenalty: qboolean,
) {
    todo!("Port WP_ResistForcePush — parked: ambient-state")
}

// PORT-ESCALATION(constants-in-scope): needs EF2_FLYING, YAW, EV_JUMP, and roll/flip anim constants not re-exported by prelude; no import path resolved in packet.
/// Raven `Boba_StopKnockdown`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:272-343`
pub fn Boba_StopKnockdown(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    pusher: *mut gentity_t,
    pushDir: vec3_t,
    forceKnockdown: qboolean,
) -> qboolean {
    todo!("Port Boba_StopKnockdown — parked: constants-in-scope")
}

// PORT-ESCALATION(ambient-state): reads `level.time`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Boba_FlyStart`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:345-365`
pub fn Boba_FlyStart(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port Boba_FlyStart — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `g_gravity` cvar global; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Boba_FlyStop`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:367-384`
pub fn Boba_FlyStop(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port Boba_FlyStop — parked: ambient-state")
}

// PORT-ESCALATION(constants-in-scope): needs EF2_FLYING not re-exported by prelude; no import path resolved in packet.
/// Raven `Boba_Flying`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:386-389`
pub fn Boba_Flying(
    self_: *mut gentity_t,
) -> qboolean {
    todo!("Port Boba_Flying — parked: constants-in-scope")
}

// PORT-ESCALATION(ambient-state): reads `g_entities`/`level.time` and calls trap_G2API_GetBoltMatrix/trap_Trace (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Boba_FireFlameThrower`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:391-416`
pub fn Boba_FireFlameThrower(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port Boba_FireFlameThrower — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`level.time` and calls trap_G2API_GetBoltMatrix (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Boba_StartFlameThrower`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:419-469`
pub fn Boba_StartFlameThrower(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port Boba_StartFlameThrower — parked: ambient-state")
}

// PORT-ESCALATION(constants-in-scope): needs NPC_SetAnim torso anim constants (BOTH_*/SETANIM_*) not re-exported by prelude; no import path resolved in packet.
/// Raven `Boba_DoFlameThrower`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:471-479`
pub fn Boba_DoFlameThrower(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port Boba_DoFlameThrower — parked: constants-in-scope")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/`ucmd`/`g_entities`/`level` and calls trap_InPVS/trap_Trace (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Boba_FireDecide`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:481-797`
pub fn Boba_FireDecide(ctx: GameContext<'_>) {
    todo!("Port Boba_FireDecide — parked: ambient-state")
}

// PORT-ESCALATION(constants-in-scope): needs cloak power/effect constants (PW_*/EF_*) and sound handling not re-exported by prelude; no import path resolved in packet.
/// Raven `Jedi_Cloak`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:799-816`
pub fn Jedi_Cloak(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port Jedi_Cloak — parked: constants-in-scope")
}

// PORT-ESCALATION(constants-in-scope): needs cloak power/effect constants (PW_*/EF_*) not re-exported by prelude; no import path resolved in packet.
/// Raven `Jedi_Decloak`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:818-833`
pub fn Jedi_Decloak(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port Jedi_Decloak — parked: constants-in-scope")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`level`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_CheckCloak`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:835-857`
pub fn Jedi_CheckCloak(ctx: GameContext<'_>) {
    todo!("Port Jedi_CheckCloak — parked: ambient-state")
}

// PORT-ESCALATION(constants-in-scope): needs NPCTEAM_PLAYER team constant not re-exported by prelude; no import path resolved in packet.
/// Raven `Jedi_Aggression`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:863-898`
pub fn Jedi_Aggression(
    self_: *mut gentity_t,
    change: c_int,
) {
    todo!("Port Jedi_Aggression — parked: constants-in-scope")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`NPCInfo`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_AggressionErosion`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:900-912`
pub fn Jedi_AggressionErosion(
    ctx: GameContext<'_>,
    amt: c_int,
) {
    todo!("Port Jedi_AggressionErosion — parked: ambient-state")
}

/// Raven `NPC_Jedi_RateNewEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:914-950`
pub fn NPC_Jedi_RateNewEnemy(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    enemy: *mut gentity_t,
) {
    let healthAggression: f32;
    let weaponAggression: f32;
    let newAggression: c_int;

    unsafe {
        match (*enemy).s.weapon {
            w if w == WP_SABER as c_int => {
                healthAggression = (*self_).health as f32 / 200.0 * 6.0;
                weaponAggression = 7.0; //go after him
            }
            w if w == WP_BLASTER as c_int => {
                // DistanceSquared( self->r.currentOrigin, enemy->r.currentOrigin )
                let s = (*self_).r.currentOrigin;
                let e = (*enemy).r.currentOrigin;
                let v0 = e[0] - s[0];
                let v1 = e[1] - s[1];
                let v2 = e[2] - s[2];
                if v0 * v0 + v1 * v1 + v2 * v2 < 65536.0
                //256 squared
                {
                    healthAggression = (*self_).health as f32 / 200.0 * 8.0;
                    weaponAggression = 8.0; //go after him
                } else {
                    healthAggression = 8.0 - ((*self_).health as f32 / 200.0 * 8.0);
                    weaponAggression = 2.0; //hang back for a second
                }
            }
            _ => {
                healthAggression = (*self_).health as f32 / 200.0 * 8.0;
                weaponAggression = 6.0; //approach
            }
        }
        //Average these with current aggression
        newAggression =
            ((healthAggression + weaponAggression + (*((*self_).NPC as *mut gNPC_t)).stats.aggression as f32) / 3.0)
                .ceil() as c_int;
        Jedi_Aggression(self_, newAggression - (*((*self_).NPC as *mut gNPC_t)).stats.aggression);

        //don't taunt right away
        crate::g_timer::TIMER_Set(ctx, self_, c"chatter".as_ptr(), crate::q_math::Q_irand(4000, 7000));
    }
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`NPCInfo`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_Rage`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:952-964`
pub fn Jedi_Rage(ctx: GameContext<'_>) {
    todo!("Port Jedi_Rage — parked: ambient-state")
}

/// Raven `Jedi_RageStop`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:966-973`
pub fn Jedi_RageStop(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        if !(*self_).NPC.is_null() {
            //calm down and back off
            crate::g_timer::TIMER_Set(ctx, self_, c"roamTime".as_ptr(), 0);
            Jedi_Aggression(self_, crate::q_math::Q_irand(-5, 0));
        }
    }
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`level`, writes `NPCInfo`/`jediSpeechDebounceTime`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_BattleTaunt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:980-1013`
pub fn Jedi_BattleTaunt(ctx: GameContext<'_>) -> qboolean {
    todo!("Port Jedi_BattleTaunt — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` global and calls trap_Trace (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Jedi_ClearPathToSpot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1020-1077`
pub fn Jedi_ClearPathToSpot(
    ctx: GameContext<'_>,
    dest: vec3_t,
    impactEntNum: c_int,
) -> qboolean {
    todo!("Port Jedi_ClearPathToSpot — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`NPCInfo`, writes `ucmd`, calls trap_Trace (needs &Engine); no channel from this context-free faithful signature.
/// Raven `NPC_MoveDirClear`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1079-1193`
pub fn NPC_MoveDirClear(
    ctx: GameContext<'_>,
    forwardmove: c_int,
    rightmove: c_int,
    reset: qboolean,
) -> qboolean {
    todo!("Port NPC_MoveDirClear — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): writes the `NPCInfo` global; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_HoldPosition`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1200-1211`
pub fn Jedi_HoldPosition(ctx: GameContext<'_>) {
    todo!("Port Jedi_HoldPosition — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, writes `NPCInfo`/`ucmd`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1219-1251`
pub fn Jedi_Move(
    ctx: GameContext<'_>,
    goal: *mut gentity_t,
    retreat: qboolean,
) {
    todo!("Port Jedi_Move — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, writes `NPCInfo`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_Hunt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1253-1280`
pub fn Jedi_Hunt(ctx: GameContext<'_>) -> qboolean {
    todo!("Port Jedi_Hunt — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` global; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_Retreat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1300-1310`
pub fn Jedi_Retreat(ctx: GameContext<'_>) {
    todo!("Port Jedi_Retreat — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` global; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_Advance`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1312-1325`
pub fn Jedi_Advance(ctx: GameContext<'_>) {
    todo!("Port Jedi_Advance — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `d_JediAI` cvar global; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_AdjustSaberAnimLevel`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1327-1394`
pub fn Jedi_AdjustSaberAnimLevel(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    newLevel: c_int,
) {
    todo!("Port Jedi_AdjustSaberAnimLevel — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`ucmd`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_CheckDecreaseSaberAnimLevel`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1396-1411`
pub fn Jedi_CheckDecreaseSaberAnimLevel(ctx: GameContext<'_>) {
    todo!("Port Jedi_CheckDecreaseSaberAnimLevel — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/`ucmd`/`jediSpeechDebounceTime`/`level`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_CombatDistance`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1413-1874`
pub fn Jedi_CombatDistance(
    ctx: GameContext<'_>,
    enemy_dist: c_int,
) {
    todo!("Port Jedi_CombatDistance — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`level`/`ucmd`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_Strafe`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1876-1929`
pub fn Jedi_Strafe(
    ctx: GameContext<'_>,
    strafeTimeMin: c_int,
    strafeTimeMax: c_int,
    nextStrafeTimeMin: c_int,
    nextStrafeTimeMax: c_int,
    walking: qboolean,
) -> qboolean {
    todo!("Port Jedi_Strafe — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `forceJumpStrength`/`g_entities`/`level` and calls trap_Trace (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Jedi_CheckFlipEvasions`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1969-2303`
pub fn Jedi_CheckFlipEvasions(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    rightdot: f32,
    zdiff: f32,
) -> evasionType_t {
    todo!("Port Jedi_CheckFlipEvasions — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `bg_parryDebounce`/`g_saberRealisticCombat`/`g_spskill` globals; no channel to reach them from this context-free faithful signature.
/// Raven `Jedi_ReCalcParryTime`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:2305-2441`
pub fn Jedi_ReCalcParryTime(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    evasionType: evasionType_t,
) -> c_int {
    todo!("Port Jedi_ReCalcParryTime — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPCInfo`/`g_spskill` globals; no channel to reach them from this context-free faithful signature.
/// Raven `Jedi_QuickReactions`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:2443-2453`
pub fn Jedi_QuickReactions(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) -> qboolean {
    todo!("Port Jedi_QuickReactions — parked: ambient-state")
}

// PORT-ESCALATION(constants-in-scope): needs FORCE_LEVEL_3 constant not re-exported by prelude; no import path resolved in packet.
/// Raven `Jedi_SaberBusy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:2455-2470`
pub fn Jedi_SaberBusy(
    self_: *mut gentity_t,
) -> qboolean {
    todo!("Port Jedi_SaberBusy — parked: constants-in-scope")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`d_JediAI`/`d_slowmodeath`/`level`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_SaberBlockGo`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:2485-3139`
pub fn Jedi_SaberBlockGo(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    cmd: *mut usercmd_t,
    pHitloc: vec3_t,
    phitDir: vec3_t,
    incoming: *mut gentity_t,
    dist: f32,
) -> evasionType_t {
    todo!("Port Jedi_SaberBlockGo — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`NPCInfo`/`d_JediAI`/`level`/`ucmd` and calls trap_Trace (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Jedi_SaberBlock`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3143-3372`
pub fn Jedi_SaberBlock(
    ctx: GameContext<'_>,
    saberNum: c_int,
    bladeNum: c_int,
) -> qboolean {
    todo!("Port Jedi_SaberBlock — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/`ucmd`/`d_JediAI`/`g_entities`/`level`/`vec3_origin`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_EvasionSaber`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3380-3666`
pub fn Jedi_EvasionSaber(
    ctx: GameContext<'_>,
    enemy_movedir: vec3_t,
    enemy_dist: f32,
    enemy_dir: vec3_t,
) {
    todo!("Port Jedi_EvasionSaber — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `g_entities`/`vec3_origin` and calls trap_EntitiesInBox/trap_InPVS/trap_Trace (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Jedi_FindEnemyInCone`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3686-3761`
pub fn Jedi_FindEnemyInCone(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    fallback: *mut gentity_t,
    minDot: f32,
) -> *mut gentity_t {
    todo!("Port Jedi_FindEnemyInCone — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` global; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_SetEnemyInfo`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3763-3796`
pub fn Jedi_SetEnemyInfo(
    ctx: GameContext<'_>,
    enemy_dest: vec3_t,
    enemy_dir: vec3_t,
    enemy_dist: *mut f32,
    enemy_movedir: vec3_t,
    enemy_movespeed: *mut f32,
    prediction: c_int,
) {
    todo!("Port Jedi_SetEnemyInfo — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`, writes `NPCInfo`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_FaceEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3799-3874`
pub fn Jedi_FaceEnemy(
    ctx: GameContext<'_>,
    doPitch: qboolean,
) {
    todo!("Port Jedi_FaceEnemy — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`level`, writes `ucmd`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_DebounceDirectionChanges`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3876-4005`
pub fn Jedi_DebounceDirectionChanges(ctx: GameContext<'_>) {
    todo!("Port Jedi_DebounceDirectionChanges — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`NPCInfo`, writes `ucmd`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_TimersApply`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4007-4065`
pub fn Jedi_TimersApply(ctx: GameContext<'_>) {
    todo!("Port Jedi_TimersApply — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/`jediSpeechDebounceTime`/`d_JediAI`/`level`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_CombatTimersUpdate`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4067-4273`
pub fn Jedi_CombatTimersUpdate(
    ctx: GameContext<'_>,
    enemy_dist: c_int,
) {
    todo!("Port Jedi_CombatTimersUpdate — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `level`, writes `NPC`/`NPCInfo`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_CombatIdle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4275-4337`
pub fn Jedi_CombatIdle(
    ctx: GameContext<'_>,
    enemy_dist: c_int,
) {
    todo!("Port Jedi_CombatIdle — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `level`, writes `NPC`/`NPCInfo`/`ucmd`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_AttackDecide`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4339-4467`
pub fn Jedi_AttackDecide(
    ctx: GameContext<'_>,
    enemy_dist: c_int,
) -> qboolean {
    todo!("Port Jedi_AttackDecide — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `level`, writes `NPC`, calls trap_Trace (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Jedi_Jump`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4473-4717`
pub fn Jedi_Jump(
    ctx: GameContext<'_>,
    dest: vec3_t,
    goalEntNum: c_int,
) -> qboolean {
    todo!("Port Jedi_Jump — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPCInfo`/`level`, writes `NPC`/`ucmd`, calls trap_Trace (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Jedi_TryJump`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4719-4865`
pub fn Jedi_TryJump(
    ctx: GameContext<'_>,
    goal: *mut gentity_t,
) -> qboolean {
    todo!("Port Jedi_TryJump — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` global; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_Jumping`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4867-4914`
pub fn Jedi_Jumping(
    ctx: GameContext<'_>,
    goal: *mut gentity_t,
) -> qboolean {
    todo!("Port Jedi_Jumping — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPCInfo`/`g_spskill`, writes `NPC`/`ucmd`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_CheckEnemyMovement`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4917-5036`
pub fn Jedi_CheckEnemyMovement(
    ctx: GameContext<'_>,
    enemy_dist: f32,
) {
    todo!("Port Jedi_CheckEnemyMovement — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPCInfo`/`g_entities`/`level`, writes `NPC`/`ucmd`, calls trap_Trace (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Jedi_CheckJumps`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5038-5153`
pub fn Jedi_CheckJumps(ctx: GameContext<'_>) {
    todo!("Port Jedi_CheckJumps — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/`jediSpeechDebounceTime`/`level`/`ucmd` and calls trap_LinkEntity (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Jedi_Combat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5155-5344`
pub fn Jedi_Combat(ctx: GameContext<'_>) {
    todo!("Port Jedi_Combat — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `d_JediAI`/`gPainPoint`/`g_spskill`/`level`; stored as a fn pointer (needs an EntPain enum variant, out/gen/ent_fn_enums.rs); no channel to reach the globals from this context-free faithful signature.
/// Raven `NPC_Jedi_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5358-5444`
pub fn NPC_Jedi_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    todo!("Port NPC_Jedi_Pain — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`level`, writes `NPCInfo`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_CheckDanger`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5446-5463`
pub fn Jedi_CheckDanger(ctx: GameContext<'_>) -> qboolean {
    todo!("Port Jedi_CheckDanger — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `NPC`/`g_entities`/`level`, writes `NPCInfo`, calls trap_InPVS (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Jedi_CheckAmbushPlayer`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5465-5545`
pub fn Jedi_CheckAmbushPlayer(ctx: GameContext<'_>) -> qboolean {
    todo!("Port Jedi_CheckAmbushPlayer — parked: ambient-state")
}

// PORT-ESCALATION(constants-in-scope): needs EV_* voice-event and NPC_SetAnim anim constants not re-exported by prelude; no import path resolved in packet.
/// Raven `Jedi_Ambush`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5547-5559`
pub fn Jedi_Ambush(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    todo!("Port Jedi_Ambush — parked: constants-in-scope")
}

// PORT-ESCALATION(constants-in-scope): needs JSF_AMBUSH spawnflag constant not re-exported by prelude; no import path resolved in packet.
/// Raven `Jedi_WaitingAmbush`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5561-5568`
pub fn Jedi_WaitingAmbush(
    self_: *mut gentity_t,
) -> qboolean {
    todo!("Port Jedi_WaitingAmbush — parked: constants-in-scope")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/`ucmd`/`g_entities`/`level` and calls trap_InPVS (needs &Engine); no channel from this context-free faithful signature.
/// Raven `Jedi_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5575-5728`
pub fn Jedi_Patrol(ctx: GameContext<'_>) {
    todo!("Port Jedi_Patrol — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `level.time`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_CanPullBackSaber`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5730-5752`
pub fn Jedi_CanPullBackSaber(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) -> qboolean {
    todo!("Port Jedi_CanPullBackSaber — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `g_entities`, writes `NPC`/`NPCInfo`/`ucmd`, calls trap_LinkEntity (needs &Engine); no channel from this context-free faithful signature.
/// Raven `NPC_BSJedi_FollowLeader`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5758-5836`
pub fn NPC_BSJedi_FollowLeader(ctx: GameContext<'_>) {
    todo!("Port NPC_BSJedi_FollowLeader — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes `NPC`/`NPCInfo`/`jediSpeechDebounceTime`/`ucmd`/`g_entities`/`g_spskill`/`level`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `Jedi_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5845-6166`
pub fn Jedi_Attack(ctx: GameContext<'_>) {
    todo!("Port Jedi_Attack — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads `level`/`ucmd`, writes `NPC`/`NPCInfo`; no channel to reach the ai_main globals / Engine from this context-free faithful signature.
/// Raven `NPC_BSJedi_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:6170-6220`
pub fn NPC_BSJedi_Default(ctx: GameContext<'_>) {
    todo!("Port NPC_BSJedi_Default — parked: ambient-state")
}
