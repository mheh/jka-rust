// PORT-COMPLETE: NPC_reactions.c 6/6
//! Port of `oracle/oracle/codemp/game/NPC_reactions.c` (jampgame mega-pass).
//!
//! Generated from `tools/closure-prototype/fnskel.py`; bodies filled per the
//! jampgame mega-pass (settled fork rulings,
//! `docs/handoffs/jampgame-fork-discovery.md`).
//!
//! SPINE (fork rulings 1/4 + `docs/architecture/engine-seam.md`, precedent
//! `w_force.rs`/`NPC_utils.rs`): logic fns that reach `level`/cvars/traps
//! thread the `GameContext<'_>` receiver (`.world: *mut GameWorld`, `.engine`)
//! as an ADDITIVE first parameter (the faithful C signature carries none).
//! `level` → `(*ctx.world).level`, cvars → `(*ctx.world).cvars`. Traps go
//! through `trap::X(ctx.engine, …)`. Cross-file callees are invoked with the
//! packet's resolved raw-pointer signatures verbatim (their own porters
//! thread the spine). Raw `gentity_t*`/`gclient_t*`/`gNPC_t*` chains are
//! transcribed as `unsafe` raw-pointer field access mirroring the C exactly
//! (`gentity_t::NPC`/`::client` are opaque `*mut c_void`, cast per the
//! `NPC_combat.rs` precedent).
//!
//! PARKED (see PORT-ESCALATION markers): several functions read the ambient
//! bot-AI "current actor" globals (`NPC`, `NPCInfo`) that Raven's
//! `ai_main.c` think-loop sets per NPC frame — there is no `GameWorld`/
//! `GameContext` field for them and no entity parameter to substitute (topic
//! `ai-context`, matching the `NPC_combat.rs`/`NPC_utils.rs` precedent in
//! this same mega-pass). `NPC_ChoosePainAnimation` also indexes the
//! runtime-populated `bgAllAnims`/`bgHumanoidAnimations` animation tables
//! (topic `raw-ptr-skeleton-no-world-handle`, matching `g_combat.rs`) and
//! needs the unported `rank_t` enum's `RANK_CAPTAIN` value. `NPC_Respond`'s
//! droid-class branches call `va(fmt, args…)` with real variadic arguments
//! (topic `va-varargs`; the resolved `va` signature drops the C varargs, same
//! as the `g_client.rs`/`w_force.rs`/`NPC_utils.rs` precedent) — cannot be
//! transcribed faithfully without inventing behavior.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::trap;
use crate::world::GameContext;

use mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::{GIcarusTaskidpending, GIcarusTaskidpendingArgs};
use mp_qshared::common::mp::qcommon::task_id_t::taskID_t;
use mp_qshared::common::mp::qcommon::b_set_t::bSet_t;
use mp_bg::public::stat_index::statIndex_t;
use mp_bg::public::entity_event::entity_event_t;
use crate::teams::npcteam::NPCTEAM_NEUTRAL;
use crate::g_utils::G_AddEvent;
use crate::NPC_utils::{G_ActivateBehavior, NPC_CheckLookTarget, NPC_SetLookTarget};
use crate::q_math::Q_irand;

/// Raven `NPC_CheckAttacker`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:42-131`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global `NPC`
// (compared against/against-set `other`) plus `g_entities[0]`/`g_spskill` —
// no `GameContext`/entity param carries the per-frame ambient actor (same
// unresolved fork as `NPC_combat.rs`/`NPC_utils.rs`'s `ai-context` sites).
pub fn NPC_CheckAttacker(
    ctx: GameContext<'_>,
    other: *mut gentity_t,
    r#mod: c_int,
) {
    todo!("Port NPC_CheckAttacker — parked: ai-context")
}

/// Raven `NPC_SetPainEvent`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:133-149`
pub fn NPC_SetPainEvent(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        // Raven: `!self->NPC || !(self->NPC->aiFlags&NPCAI_DIE_ON_IMPACT)`.
        // Source: oracle/oracle/codemp/game/b_public.h:23
        const NPCAI_DIE_ON_IMPACT: c_int = 0x00100000;
        if npc.is_null() || ((*npc).aiFlags & NPCAI_DIE_ON_IMPACT) == 0 {
            let client = (*self_).client as *mut gclient_t;
            let pending = trap::ICARUS_TaskIDPending(
                ctx.engine,
                GIcarusTaskidpendingArgs::new(self_, taskID_t::TID_CHAN_VOICE as c_int),
            );
            if pending == 0 && !client.is_null() {
                let stat_max_health = (*client).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize];
                let parm = ((*self_).health as f32 / stat_max_health as f32 * 100.0f32).floor() as c_int;
                G_AddEvent(self_, entity_event_t::EV_PAIN as c_int, parm);
            }
        }
    }
}

/// Raven `NPC_GetPainChance`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:157-196`
pub fn NPC_GetPainChance(ctx: GameContext<'_>, self_: *mut gentity_t, damage: c_int) -> f32 {
    unsafe {
        if (*self_).enemy.is_none() {
            //surprised, always take pain
            return 1.0f32;
        }

        let client = (*self_).client as *mut gclient_t;
        if client.is_null() {
            return 1.0f32;
        }

        let max_health = (*client).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize] as f32;
        if damage as f32 > max_health / 2.0f32 {
            return 1.0f32;
        }

        let mut pain_chance = (max_health - (*self_).health as f32) / (max_health * 2.0f32)
            + damage as f32 / (max_health / 2.0f32);

        match (*ctx.world).cvars.g_spskill.integer {
            0 => {
                //easy
            }
            1 => {
                //med
                pain_chance *= 0.5f32;
            }
            _ => {
                //hard (also default)
                pain_chance *= 0.1f32;
            }
        }
        pain_chance
    }
}

/// Raven `NPC_ChoosePainAnimation`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:207-356`
// PORT-ESCALATION(raw-ptr-skeleton-no-world-handle): reads `level.time`
// (needs `GameContext`), indexes the runtime-populated `bgAllAnims`/
// `bgHumanoidAnimations` global animation tables (no channel to reach them —
// same fork as `g_combat.rs`'s `bgAllAnims`/`bgHumanoidAnimations` sites),
// and reads `self->NPC->rank` against the unported `rank_t` enum's
// `RANK_CAPTAIN` value (`ai.h:29-41`).
pub fn NPC_ChoosePainAnimation(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    other: *mut gentity_t,
    point: vec3_t,
    damage: c_int,
    r#mod: c_int,
    hitLoc: c_int,
    voiceEvent: c_int,
) {
    todo!("Port NPC_ChoosePainAnimation — parked: raw-ptr-skeleton-no-world-handle")
}

/// Raven `NPC_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:363-529`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global `NPC`
// and writes `NPCInfo` (both per-frame ambient-actor globals, `ai_main.c`
// think loop) plus the file-scope `gPainMOD`/`gPainHitLoc`/`gPainPoint`
// globals (same unresolved fork as `NPC_AI_Jedi.rs`'s/`NPC_AI_GalakMech.rs`'s
// `ambient-state` sites) and `killPlayerTimer` — no channel to reach any of
// these from this context-free faithful signature. Also stored as a fn
// pointer (needs an `EntPain` enum variant, `out/gen/ent_fn_enums.rs`).
pub fn NPC_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    todo!("Port NPC_Pain — parked: ai-context")
}

/// Raven `NPC_Touch`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:537-653`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global `NPC`
// and writes `NPCInfo` (per-frame ambient-actor globals set by `ai_main.c`'s
// think loop) — no channel to reach them from this context-free faithful
// signature (same fork as `NPC_combat.rs`/`NPC_utils.rs`). Also stored as a
// fn pointer (needs an `EntTouch` enum variant, `out/gen/ent_fn_enums.rs`).
pub fn NPC_Touch(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    other: *mut gentity_t,
    trace: *mut trace_t,
) {
    todo!("Port NPC_Touch — parked: ai-context")
}

/// Raven `NPC_TempLookTarget`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:661-688`
pub fn NPC_TempLookTarget(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    lookEntNum: c_int,
    mut minLookTime: c_int,
    mut maxLookTime: c_int,
) {
    unsafe {
        let client = (*self_).client as *mut gclient_t;
        if client.is_null() {
            return;
        }

        // Raven `EF2_HELD_BY_MONSTER` (`playerState_t::eFlags2` bit) — not yet
        // ported as a central const; inlined here from the header value.
        // Source: oracle/oracle/codemp/game/bg_public.h:616
        const EF2_HELD_BY_MONSTER: c_int = 1 << 0;
        if ((*client).ps.eFlags2 & EF2_HELD_BY_MONSTER) != 0 {
            //lookTarget is set by and to the monster that's holding you, no other operations can change that
            return;
        }

        if minLookTime == 0 {
            minLookTime = 1000;
        }

        if maxLookTime == 0 {
            maxLookTime = 1000;
        }

        if NPC_CheckLookTarget(ctx, self_) == 0 {
            //Not already looking at something else
            //Look at him for 1 to 3 seconds
            let level_time = (*ctx.world).level.time;
            NPC_SetLookTarget(self_, lookEntNum, level_time + Q_irand(minLookTime, maxLookTime));
        }
    }
}

/// Raven `NPC_Respond`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:690-942`
// PORT-ESCALATION(va-varargs): the `CLASS_R2D2`/`CLASS_R5D2`/`CLASS_MOUSE`/
// `CLASS_GONK` branches call `G_SoundIndex(va(fmt, Q_irand(...)))` with a
// real variadic argument — the resolved `va` signature drops the C varargs
// (same fork as `g_client.rs`/`w_force.rs`/`NPC_utils.rs`'s parked
// `va(fmt, …)` call sites), so this function cannot be transcribed
// faithfully without inventing behavior.
pub fn NPC_Respond(ctx: GameContext<'_>, self_: *mut gentity_t, userNum: c_int) {
    todo!("Port NPC_Respond — parked: va-varargs")
}

/// Raven `NPC_UseResponse`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:950-999`
pub fn NPC_UseResponse(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    user: *mut gentity_t,
    useWhenDone: qboolean,
) {
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        let client = (*self_).client as *mut gclient_t;
        if npc.is_null() || client.is_null() {
            return;
        }

        if (*user).s.number != 0 {
            //not used by the player
            if useWhenDone != 0 {
                G_ActivateBehavior(ctx, self_, bSet_t::BSET_USE as c_int);
            }
            return;
        }

        let user_client = (*user).client as *mut gclient_t;
        if !user_client.is_null()
            && (*client).playerTeam != (*user_client).playerTeam
            && (*client).playerTeam != NPCTEAM_NEUTRAL
        {
            //only those on the same team react
            if useWhenDone != 0 {
                G_ActivateBehavior(ctx, self_, bSet_t::BSET_USE as c_int);
            }
            return;
        }

        if (*npc).blockedSpeechDebounceTime > (*ctx.world).level.time {
            //I'm not responding right now
            return;
        }

        if useWhenDone != 0 {
            G_ActivateBehavior(ctx, self_, bSet_t::BSET_USE as c_int);
        } else {
            NPC_Respond(ctx, self_, (*user).s.number);
        }
    }
}

/// Raven `NPC_Use`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:1008-1093`
// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global `NPC`
// directly (`Jedi_WaitingAmbush( NPC )`/`Jedi_Ambush( NPC )`) — no channel to
// reach it from this context-free faithful signature (same fork as
// `NPC_combat.rs`/`NPC_utils.rs`). The `CLASS_VEHICLE` branch also calls the
// C++ `vehicleInfo_t` vtable (`EjectAll`/`Eject`/`Board`) which is deferred
// per porting-rules §F (vehicle vtable fork, BLESSED). Also stored as a fn
// pointer (needs an `EntUse` enum variant, `out/gen/ent_fn_enums.rs`).
pub fn NPC_Use(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    todo!("Port NPC_Use — parked: ai-context")
}

/// Raven `NPC_CheckPlayerAim`.
///
/// Raven: body is entirely commented out (`//FIXME: need appropriate
/// dialogue`) — a dead no-op in the oracle.
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:1095-1111`
pub fn NPC_CheckPlayerAim() {}

/// Raven `NPC_CheckAllClear`.
///
/// Raven: body is entirely commented out (`//FIXME: need to make this happen
/// only once after losing enemies, not over and over again`) — a dead no-op
/// in the oracle.
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:1113-1125`
pub fn NPC_CheckAllClear() {}
