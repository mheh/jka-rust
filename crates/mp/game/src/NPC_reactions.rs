// PORT-COMPLETE: NPC_reactions.c 6/6
//! Port of `oracle/codemp/game/NPC_reactions.c` (jampgame mega-pass).
//!
//! Generated from `tools/closure-prototype/fnskel.py`; bodies filled per the
//! jampgame mega-pass (settled fork rulings,
//! `docs/handoffs/jampgame-fork-discovery.md`).
//!
//! SPINE (fork rulings 1/4 + `docs/architecture/engine-seam.md`, precedent
//! `w_force.rs`/`NPC_utils.rs`): logic fns that reach `level`/cvars/traps
//! thread the `GameContext<'_>` receiver (`.world: *mut GameWorld`, `.engine`)
//! as an ADDITIVE first parameter (the faithful C signature carries none).
//! `level` → `(*ctx.world_raw()).level`, cvars → `(*ctx.world_raw()).cvars`. Traps go
//! through `trap::X(ctx.engine, …)`. Cross-file callees are invoked with the
//! packet's resolved raw-pointer signatures verbatim (their own porters
//! thread the spine). Raw `gentity_t*`/`gclient_t*`/`gNPC_t*` chains are
//! transcribed as `unsafe` raw-pointer field access mirroring the C exactly
//! (`gentity_t::NPC`/`::client` are opaque `*mut c_void`, cast per the
//! `NPC_combat.rs` precedent).
//!
//! Ambient-state resolution (formerly parked topics, now bodied): the bot-AI
//! "current actor" globals Raven's `ai_main.c` think-loop sets per frame
//! (`NPC`, `NPCInfo`) are threaded as `(*ctx.world_raw()).globals.NPC` /
//! `.NPCInfo`; `NPC_ChoosePainAnimation` indexes the runtime-populated
//! `bgAllAnims`/`bgHumanoidAnimations` tables through `(*ctx.world_raw()).bg_state`;
//! and `NPC_Respond`'s droid-class `va(fmt, …)` sound-path calls are ported
//! faithfully via `format!()` (they format one `int`, so the string is
//! byte-identical to Raven's).
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`. Bodies re-derive the raw pointers verbatim at the
//! top (`// STAGE-1:` markers) — Stage-2 debt. Callers bridge at the boundary
//! via `ctx.entity_id_of(ptr)`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::trap;
use crate::world::GameContext;

use crate::entity::hit_location::HL_GENERIC1;
use crate::g_utils::{G_AddEvent, G_Sound, G_UseTargets2};
use crate::npc_c::{RestoreNPCGlobals, SaveNPCGlobals, SetNPCGlobals};
use crate::q_shared::Q_stricmp;
use crate::teams::npcteam::NPCTEAM_NEUTRAL;
use crate::NPC_combat::{G_ClearEnemy, G_SetEnemy};
use crate::NPC_utils::{G_ActivateBehavior, NPC_CheckLookTarget, NPC_SetLookTarget};
use mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::{
    GIcarusTaskidpending, GIcarusTaskidpendingArgs,
};
use mp_bg::public::entity_event::entity_event_t;
use mp_bg::public::stat_index::statIndex_t;
use mp_qshared::common::mp::qcommon::b_set_t::bSet_t;
use mp_qshared::common::mp::qcommon::task_id_t::taskID_t;

/// Resolve a stored `Option<EntityId>` field back to a `gentity_t*` (the
/// id->pointer half of the entity-id seam; `None` -> Raven's NULL).
#[inline]
unsafe fn ent_ptr(ctx: &mut GameContext, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => unsafe { &mut (*ctx.world_raw()).g_entities[i.index()] as *mut gentity_t },
        None => core::ptr::null_mut(),
    }
}

/// Raven `NPC_CheckAttacker`.
///
/// Source: `oracle/codemp/game/NPC_reactions.c:42-131`
pub fn NPC_CheckAttacker(ctx: &mut GameContext, other: Option<EntityId>, r#mod: c_int) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let other: *mut gentity_t = unsafe { ent_ptr(ctx, other) };
    unsafe {
        // `FL_NOTARGET` (crate::entity::flags) and `WP_SABER` (mp_bg weapon_t,
        // c_int const == 3) come from the prelude. `mod` is a plain c_int, so
        // keep a local c_int alias sourced from the canonical meansOfDeath_t.
        // Source: `oracle/codemp/game/bg_public.h:1046-1099`
        const MOD_SABER: c_int = meansOfDeath_t::MOD_SABER as c_int;

        // valid ent
        if other.is_null() {
            return;
        }

        let npc = (*ctx.world_raw()).globals.NPC;
        if other == npc {
            return;
        }

        if (*other).inuse == 0 {
            return;
        }

        // Don't take a target that doesn't want to be
        if ((*other).flags & FL_NOTARGET) != 0 {
            return;
        }

        // If we haven't taken a target, just get mad
        if (*npc).enemy.is_none() {
            G_SetEnemy(ctx, ctx.entity_id_of(npc).unwrap(), ctx.entity_id_of(other));
            return;
        }

        // We have an enemy, see if he's dead
        if let Some(enemy_id) = (*npc).enemy {
            let base = (*ctx.world_raw()).g_entities.as_mut_ptr();
            let enemy_ptr = base.add(enemy_id.0 as usize);
            if (*enemy_ptr).health <= 0 {
                G_ClearEnemy(ctx, ctx.entity_id_of(npc).unwrap());
                G_SetEnemy(ctx, ctx.entity_id_of(npc).unwrap(), ctx.entity_id_of(other));
                return;
            }
        }

        // Don't take the same enemy again
        let other_id = ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), other);
        if (*npc).enemy == Some(other_id) {
            return;
        }

        // Check if we're a Jedi
        let client = (*npc).client as *mut gclient_t;
        if !client.is_null() && (*client).ps.weapon == WP_SABER {
            // I'm a jedi
            if r#mod == MOD_SABER {
                // Always switch to this enemy if I'm a jedi and hit by another saber
                G_ClearEnemy(ctx, ctx.entity_id_of(npc).unwrap());
                G_SetEnemy(ctx, ctx.entity_id_of(npc).unwrap(), ctx.entity_id_of(other));
                return;
            }
        }

        // Special case player interactions
        let player = (*ctx.world_raw()).g_entities.as_mut_ptr();
        if other == player {
            // Account for the skill level to skew the results
            let luck_threshold = match (*ctx.world_raw()).cvars.g_spskill.integer {
                0 => 0.9f32, // Easiest difficulty
                1 => 0.5f32, // Medium difficulty
                _ => 0.0f32, // Hardest difficulty
            };

            // Randomly pick up the target. Raven `random()` is already in [0,1);
            // `Rng::random` matches it, so no extra /32768 normalization.
            if (*ctx.world_raw()).bg_state.rng.random() > luck_threshold {
                G_ClearEnemy(ctx, ctx.entity_id_of(other).unwrap());
                (*other).enemy = Some(ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), npc));
            }

            return;
        }
    }
}

/// Raven `NPC_SetPainEvent`.
///
/// Source: `oracle/codemp/game/NPC_reactions.c:133-149`
pub fn NPC_SetPainEvent(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        // Raven: `!self->NPC || !(self->NPC->aiFlags&NPCAI_DIE_ON_IMPACT)`.
        // NPCAI_DIE_ON_IMPACT resolves through the prelude (crate::npc::ai_flags).
        // Source: oracle/codemp/game/b_public.h:23
        if npc.is_null() || ((*npc).aiFlags & NPCAI_DIE_ON_IMPACT) == 0 {
            let client = (*self_).client as *mut gclient_t;
            let pending = trap::ICARUS_TaskIDPending(
                ctx.engine,
                GIcarusTaskidpendingArgs::new(self_, taskID_t::TID_CHAN_VOICE as c_int),
            );
            if pending == 0 && !client.is_null() {
                let stat_max_health = (*client).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize];
                let parm =
                    ((*self_).health as f32 / stat_max_health as f32 * 100.0f32).floor() as c_int;
                G_AddEvent(&mut *(self_), entity_event_t::EV_PAIN as c_int, parm);
            }
        }
    }
}

/// Raven `NPC_GetPainChance`.
///
/// Source: `oracle/codemp/game/NPC_reactions.c:157-196`
pub fn NPC_GetPainChance(ctx: &mut GameContext, self_: EntityId, damage: c_int) -> f32 {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
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

        match (*ctx.world_raw()).cvars.g_spskill.integer {
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
/// Source: `oracle/codemp/game/NPC_reactions.c:207-356`
pub fn NPC_ChoosePainAnimation(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    point: vec3_t,
    damage: c_int,
    r#mod: c_int,
    hitLoc: c_int,
    voiceEvent: c_int,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let other: *mut gentity_t = unsafe { ent_ptr(ctx, other) };
    unsafe {
        // Pain-anim numbers are `animNumber_t` variants; keep local c_int
        // aliases because `pain_anim` and `BG_PickAnim` operate in c_int.
        // Source: `oracle/codemp/game/anims.h:6-1791`
        const BOTH_PAIN1: c_int = animNumber_t::BOTH_PAIN1 as c_int;
        const BOTH_PAIN2: c_int = animNumber_t::BOTH_PAIN2 as c_int;
        const BOTH_PAIN3: c_int = animNumber_t::BOTH_PAIN3 as c_int;
        const BOTH_PAIN18: c_int = animNumber_t::BOTH_PAIN18 as c_int;
        // `mod` is a plain c_int; alias the canonical meansOfDeath_t variants.
        // Source: `oracle/codemp/game/bg_public.h:1046-1099`
        const MOD_MELEE: c_int = meansOfDeath_t::MOD_MELEE as c_int;
        const MOD_CRUSH: c_int = meansOfDeath_t::MOD_CRUSH as c_int;
        // `HL_GENERIC1` (top-of-file import), `SETANIM_*` (mp_bg set_anim),
        // `WP_SABER`/`WP_THERMAL` (mp_bg weapon_t), `NPCTEAM_PLAYER`
        // (crate::teams::npcteam) and the `CLASS_*` `class_t` variants all
        // resolve through the prelude — no local placeholders.

        // If we've already taken pain, then don't take it again
        if (*ctx.world_raw()).level.time < (*self_).painDebounceTime && r#mod != MOD_MELEE {
            return;
        }

        if (*self_).s.weapon == WP_THERMAL && (*self_).client.is_null() == false {
            let client = (*self_).client as *mut gclient_t;
            if (*client).ps.weaponTime > 0 {
                // Don't interrupt thermal throwing anim
                return;
            }
        }

        let client = (*self_).client as *mut gclient_t;
        let mut pain_chance = 0.5f32;

        if !client.is_null() && (*client).NPC_class == CLASS_GALAKMECH {
            if hitLoc == HL_GENERIC1 {
                // Hit the antenna!
                pain_chance = 1.0f32;
            } else if (*self_).health > 200 && damage < 100 {
                // Have a lot of health
                pain_chance = 0.05f32;
            } else {
                // The lower my health and greater the damage, the more likely I am to play a pain anim
                pain_chance =
                    (200.0f32 - (*self_).health as f32) / 100.0f32 + damage as f32 / 50.0f32;
            }
        } else if !client.is_null()
            && (*client).playerTeam == NPCTEAM_PLAYER
            // playerTeam is npcteam_t (== c_int); NPCTEAM_PLAYER from prelude.
            && !other.is_null()
            && (*other).s.number == 0
        {
            // Ally shot by player always complains
            pain_chance = 1.1f32;
        } else {
            if !other.is_null() && (*other).s.weapon == WP_SABER || r#mod == MOD_CRUSH {
                pain_chance = 1.0f32; // Always take pain from saber
            } else if r#mod == MOD_MELEE {
                // Higher in rank (skill) we are, less likely we are to be fazed by a punch
                let npc = (*self_).NPC as *mut gNPC_t;
                if !npc.is_null() {
                    pain_chance = 1.0f32
                        - ((RANK_CAPTAIN as c_int - (*npc).rank as c_int) as f32
                            / RANK_CAPTAIN as c_int as f32);
                } else {
                    pain_chance = 1.0f32;
                }
            } else if !client.is_null() && (*client).NPC_class == CLASS_PROTOCOL {
                pain_chance = 1.0f32;
            } else {
                pain_chance = NPC_GetPainChance(ctx, ctx.entity_id_of(self_).unwrap(), damage);
            }

            if !client.is_null() && (*client).NPC_class == CLASS_DESANN {
                pain_chance *= 0.5f32;
            }
        }

        // See if we're going to flinch. Raven `random()` is already in [0,1);
        // `Rng::random` matches it, so no extra /32768 normalization.
        if (*ctx.world_raw()).bg_state.rng.random() < pain_chance {
            let mut pain_anim = -1;

            // Pick and play our animation
            if !client.is_null()
                && (*client).ps.fd.forceGripBeingGripped < (*ctx.world_raw()).level.time as f32
            {
                // Not being force-gripped or force-drained
                let legs_anim = (*client).ps.legsAnim;
                let torso_anim = (*client).ps.torsoAnim;

                if crate::bg_panimate::PM_SpinningAnim(legs_anim) == qfalse
                    && crate::bg_panimate::BG_SaberInSpecialAttack(torso_anim) == qfalse
                    && crate::bg_panimate::PM_InKnockDown(&mut (*client).ps) == qfalse
                    && crate::bg_pmove::PM_RollingAnim(legs_anim) == qfalse
                    && !(crate::bg_panimate::BG_FlippingAnim(legs_anim) != qfalse
                        && crate::bg_panimate::PM_InCartwheel(legs_anim) == qfalse)
                {
                    // Play an anim
                    let npc = (*self_).NPC as *mut gNPC_t;
                    let local_anim_index = (*self_).localAnimIndex;

                    if !client.is_null() && (*client).NPC_class == CLASS_GALAKMECH {
                        pain_anim = BOTH_PAIN1;
                    } else if r#mod == MOD_MELEE {
                        pain_anim = crate::bg_panimate::BG_PickAnim(
                            &mut (*ctx.world_raw()).bg_state,
                            local_anim_index,
                            BOTH_PAIN2,
                            BOTH_PAIN3,
                        );
                    } else if (*self_).s.weapon == WP_SABER {
                        // These are the only 2 pain anims that look good when holding a saber
                        pain_anim = crate::bg_panimate::BG_PickAnim(
                            &mut (*ctx.world_raw()).bg_state,
                            local_anim_index,
                            BOTH_PAIN2,
                            BOTH_PAIN3,
                        );
                    }

                    if pain_anim == -1 {
                        pain_anim = crate::bg_panimate::BG_PickAnim(
                            &mut (*ctx.world_raw()).bg_state,
                            local_anim_index,
                            BOTH_PAIN1,
                            BOTH_PAIN18,
                        );
                    }

                    (*client).ps.fd.saberAnimLevel = 1; // FORCE_LEVEL_1
                    (*client).ps.saberMove = 0; // LS_READY

                    let mut parts = SETANIM_BOTH;
                    if crate::bg_panimate::BG_CrouchAnim((*client).ps.legsAnim) != qfalse
                        || crate::bg_panimate::PM_InCartwheel((*client).ps.legsAnim) != qfalse
                    {
                        parts = SETANIM_LEGS;
                    }

                    if pain_anim != -1 {
                        crate::npc_c::NPC_SetAnim(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            parts,
                            pain_anim,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    }
                }

                if voiceEvent != -1 {
                    let __h507 = ctx.entity_id_of(self_).unwrap();
                    let __h508 = (*ctx.world_raw()).bg_state.rng.Q_irand(2000, 4000);
                    crate::NPC_sounds::G_AddVoiceEvent(ctx, __h507, voiceEvent, __h508);
                } else {
                    NPC_SetPainEvent(ctx, ctx.entity_id_of(self_).unwrap());
                }
            } else {
                let __h509 = ctx.entity_id_of(self_).unwrap();
                let __h510 = (*ctx.world_raw()).bg_state.rng.Q_irand(
                    entity_event_t::EV_CHOKE1 as c_int,
                    entity_event_t::EV_CHOKE3 as c_int,
                );
                // Being force-gripped. Oracle: `Q_irand(EV_CHOKE1, EV_CHOKE3)`
                // (the BOTH_PAIN* anim numbers are unrelated).
                crate::NPC_sounds::G_AddVoiceEvent(ctx, __h509, __h510, 0);
            }

            // Setup the timing for it
            let local_anim_index = (*self_).localAnimIndex;
            let num_frames = if pain_anim >= 0 {
                // Oracle: animLength = bgAllAnims[self->localAnimIndex].anims[pain_anim].numFrames
                //   * fabs((float)(bgHumanoidAnimations[pain_anim].frameLerp));
                // numFrames comes from the skeleton-specific table, frameLerp from the
                // humanoid table (they are intentionally different tables in the C source).
                let bg = &(*ctx.world_raw()).bg_state;
                let anims = bg.bgAllAnims[local_anim_index as usize].anims;
                ((*anims.offset(pain_anim as isize)).numFrames as f32
                    * (bg.bgHumanoidAnimations[pain_anim as usize].frameLerp as f32).abs())
                    as c_int
            } else {
                0
            };

            (*self_).painDebounceTime = (*ctx.world_raw()).level.time + num_frames;
            if !client.is_null() {
                (*client).ps.weaponTime = 0;
            }
        }
    }
}

/// Raven `NPC_Pain`.
///
/// Source: `oracle/codemp/game/NPC_reactions.c:363-529`
pub fn NPC_Pain(ctx: &mut GameContext, self_: EntityId, attacker: Option<EntityId>, damage: c_int) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let attacker: *mut gentity_t = unsafe { ent_ptr(ctx, attacker) };
    unsafe {
        // `otherTeam` is npcteam_t (== c_int); keep TEAM_FREE (== 0) local.
        const TEAM_FREE: c_int = 0;
        // BSET_* are bSet_t variants but G_ActivateBehavior takes c_int; PM_DEAD
        // is a pmtype_t variant compared against the c_int `pm_type` field; and
        // EV_FFWARN is the absolute entity_event_t value G_AddVoiceEvent
        // consumes — alias each from its canonical enum so values track the port.
        // Source: bSet_t `oracle/codemp/game/g_public.h:641-664`,
        // pmtype_t `oracle/codemp/game/bg_public.h:360-370`,
        // entity_event_t `oracle/codemp/game/bg_public.h:745-990`.
        const BSET_FLEE: c_int = bSet_t::BSET_FLEE as c_int;
        const BSET_PAIN: c_int = bSet_t::BSET_PAIN as c_int;
        const BSET_FFIRE: c_int = bSet_t::BSET_FFIRE as c_int;
        const PM_DEAD: c_int = pmtype_t::PM_DEAD as c_int;
        const EV_FFWARN: c_int = entity_event_t::EV_FFWARN as c_int;

        let mut other_team = TEAM_FREE;
        let mut voice_event = -1;
        let other = attacker;
        let r#mod = (*ctx.world_raw()).globals.gPainMOD;
        let hit_loc = (*ctx.world_raw()).globals.gPainHitLoc;
        let mut point = [0.0f32; 3];
        crate::q_math::_VectorCopy((*ctx.world_raw()).globals.gPainPoint, &mut point);

        let npc = (*self_).NPC as *mut gNPC_t;
        if npc.is_null() {
            return;
        }

        if other.is_null() {
            return;
        }

        let client = (*self_).client as *mut gclient_t;
        if !client.is_null() && (*client).ps.pm_type == PM_DEAD {
            return;
        }

        if other == self_ {
            return;
        }

        // Ignore damage from your own team for now
        let other_client = (*other).client as *mut gclient_t;
        if !other_client.is_null() {
            other_team = (*other_client).playerTeam;
        }

        if !client.is_null()
            && (*client).playerTeam != 0
            && !other_client.is_null()
            && other_team == (*client).playerTeam
        {
            // Hit by a teammate. Oracle uses `self`/`other`, not the ambient
            // `NPC` global (SetNPCGlobals(self) is not called until later).
            let other_id = ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), other);
            let self_id = ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), self_);

            if (*self_).enemy != Some(other_id) && (*other).enemy != Some(self_id) {
                // We weren't already enemies
                if !(*self_).enemy.is_none() || !(*other).enemy.is_none() {
                    // If one of us actually has an enemy already, it's okay, just an accident
                    if !client.is_null() && !npc.is_null() {
                        // Run any pain instructions
                        if (*self_).health
                            <= ((*client).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize] / 3)
                            && G_ActivateBehavior(ctx, ctx.entity_id_of(self_), BSET_FLEE) != 0
                        {
                        } else {
                            G_ActivateBehavior(ctx, ctx.entity_id_of(self_), BSET_PAIN);
                        }
                    }

                    if damage != -1 {
                        // Set our proper pain animation
                        if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) != 0 {
                            NPC_ChoosePainAnimation(
                                ctx,
                                ctx.entity_id_of(self_).unwrap(),
                                ctx.entity_id_of(other),
                                point,
                                damage,
                                r#mod,
                                hit_loc,
                                EV_FFWARN,
                            );
                        } else {
                            NPC_ChoosePainAnimation(
                                ctx,
                                ctx.entity_id_of(self_).unwrap(),
                                ctx.entity_id_of(other),
                                point,
                                damage,
                                r#mod,
                                hit_loc,
                                -1,
                            );
                        }
                    }
                    return;
                } else if !npc.is_null() && (*other).s.number == 0 {
                    // NPC hit by player
                    if (*npc).charmedTime != 0 {
                        // Mindtricked
                        return;
                    } else if (*npc).ffireCount
                        < 3 + ((2 - (*ctx.world_raw()).cvars.g_spskill.integer) * 2)
                    {
                        // Not mad enough yet
                        if damage != -1 {
                            if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) != 0 {
                                NPC_ChoosePainAnimation(
                                    ctx,
                                    ctx.entity_id_of(self_).unwrap(),
                                    ctx.entity_id_of(other),
                                    point,
                                    damage,
                                    r#mod,
                                    hit_loc,
                                    EV_FFWARN,
                                );
                            } else {
                                NPC_ChoosePainAnimation(
                                    ctx,
                                    ctx.entity_id_of(self_).unwrap(),
                                    ctx.entity_id_of(other),
                                    point,
                                    damage,
                                    r#mod,
                                    hit_loc,
                                    -1,
                                );
                            }
                        }
                        return;
                    } else if G_ActivateBehavior(ctx, ctx.entity_id_of(self_), BSET_FFIRE) != 0 {
                        // We have a specific script to run
                        return;
                    } else {
                        // Turn on our ally
                        (*npc).blockedSpeechDebounceTime = 0;
                        voice_event = entity_event_t::EV_FFTURN as c_int;
                        // C chained assignment sets all three to BS_DEFAULT.
                        (*npc).defaultBehavior = bState_t::BS_DEFAULT;
                        (*npc).tempBehavior = bState_t::BS_DEFAULT;
                        (*npc).behaviorState = bState_t::BS_DEFAULT;
                        (*other).flags &= !FL_NOTARGET;
                        (*self_).r.svFlags &= !SVF_ICARUS_FREEZE;
                        G_SetEnemy(
                            ctx,
                            ctx.entity_id_of(self_).unwrap(),
                            ctx.entity_id_of(other),
                        );
                        (*npc).scriptFlags &= !(SCF_DONT_FIRE
                            | SCF_CROUCHED
                            | SCF_WALKING
                            | SCF_NO_COMBAT_TALK
                            | SCF_FORCED_MARCH);
                        (*npc).scriptFlags |= SCF_CHASE_ENEMIES | SCF_NO_MIND_TRICK;

                        if (*ctx.world_raw()).globals.killPlayerTimer == 0 {
                            (*ctx.world_raw()).globals.killPlayerTimer =
                                (*ctx.world_raw()).level.time + 10000;
                        }
                    }
                }
            }
        }

        SaveNPCGlobals(ctx);
        SetNPCGlobals(ctx, ctx.entity_id_of(self_).unwrap());

        // Do extra bits
        let npc_info_ptr = (*ctx.world_raw()).globals.NPCInfo;
        if !npc_info_ptr.is_null() && (*npc_info_ptr).ignorePain == 0 {
            (*npc_info_ptr).confusionTime = 0; // Clear any charm or confusion
            if damage != -1 {
                NPC_ChoosePainAnimation(
                    ctx,
                    ctx.entity_id_of(self_).unwrap(),
                    ctx.entity_id_of(other),
                    point,
                    damage,
                    r#mod,
                    hit_loc,
                    voice_event,
                );
            }

            // Check to take a new enemy
            let npc_ptr = (*ctx.world_raw()).globals.NPC;
            if (*npc_ptr).enemy != Some(ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), other))
                && npc_ptr != other
            {
                NPC_CheckAttacker(ctx, ctx.entity_id_of(other), r#mod);
            }
        }

        // Attempt to run any pain instructions
        if !client.is_null() && !npc.is_null() {
            if (*self_).health <= ((*client).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize] / 3)
                && G_ActivateBehavior(ctx, ctx.entity_id_of(self_), BSET_FLEE) != 0
            {
            } else {
                G_ActivateBehavior(ctx, ctx.entity_id_of(self_), BSET_PAIN);
            }
        }

        // Attempt to fire any paintargets we might have
        if !(*self_).paintarget.is_null() && *((*self_).paintarget) != 0 {
            G_UseTargets2(
                ctx,
                ctx.entity_id_of(self_),
                ctx.entity_id_of(other),
                (*self_).paintarget,
            );
        }

        RestoreNPCGlobals(ctx);
    }
}

/// Raven `NPC_Touch`.
///
/// Source: `oracle/codemp/game/NPC_reactions.c:537-653`
pub fn NPC_Touch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let other: *mut gentity_t = unsafe { ent_ptr(ctx, other) };
    unsafe {
        // MAX_CLIENTS_I32 (mp_qshared limits, == 32) and NPCAI_TOUCHED_GOAL
        // (crate::npc::ai_flags, == 0x8) resolve through the prelude.

        let npc = (*self_).NPC as *mut gNPC_t;
        if npc.is_null() {
            return;
        }

        SaveNPCGlobals(ctx);
        SetNPCGlobals(ctx, ctx.entity_id_of(self_).unwrap());

        // I am dead and carrying a key
        if !(*self_).message.is_null() && (*self_).health <= 0 {
            // Player touched me
            let other_client = (*other).client as *mut gclient_t;
            if !other.is_null() && !other_client.is_null() && (*other).s.number < MAX_CLIENTS_I32 {
                // Placeholder: would handle key pickup here (commented out in oracle)
            }
        }

        let other_client = (*other).client as *mut gclient_t;
        if !other_client.is_null() {
            // Other has a client (is a player)
            if (*other).health > 0 {
                let npc_info_ptr = (*ctx.world_raw()).globals.NPCInfo;
                if !npc_info_ptr.is_null() {
                    (*npc_info_ptr).touchedByPlayer =
                        Some(ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), other));
                }
            }

            let npc_info_ptr = (*ctx.world_raw()).globals.NPCInfo;
            if !npc_info_ptr.is_null()
                && (*npc_info_ptr).goalEntity
                    == Some(ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), other))
            {
                (*npc_info_ptr).aiFlags |= NPCAI_TOUCHED_GOAL;
            }

            // Check for enemy collision. Oracle's only active test is
            // `!(other->flags & FL_NOTARGET)`; the SVF_IGNORE_ENEMIES clause is
            // commented out there, so it is not reintroduced here.
            if ((*other).flags & FL_NOTARGET) == 0 {
                let client = (*self_).client as *mut gclient_t;
                if !client.is_null() && (*client).enemyTeam != 0 {
                    // See if we bumped into an enemy
                    if (*other_client).playerTeam == (*client).enemyTeam {
                        // Bumped into an enemy
                        let npc_info_ptr = (*ctx.world_raw()).globals.NPCInfo;
                        if !npc_info_ptr.is_null()
                            && (*npc_info_ptr).behaviorState != bState_t::BS_HUNT_AND_KILL
                            && (*npc_info_ptr).tempBehavior == bState_t::BS_DEFAULT
                        {
                            let npc_ptr = (*ctx.world_raw()).globals.NPC;
                            if (*npc_ptr).enemy
                                != Some(ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), other))
                            {
                                G_SetEnemy(
                                    ctx,
                                    ctx.entity_id_of(npc_ptr).unwrap(),
                                    ctx.entity_id_of(other),
                                );
                            }
                        }
                    }
                }
            }
        } else {
            // Other is not a client
            if (*other).health > 0 {
                // Non-NPC entity (probably an object)
                if 0 != 0 {
                    // rwwFIXMEFIXME condition always false
                    let npc_info_ptr = (*ctx.world_raw()).globals.NPCInfo;
                    if !npc_info_ptr.is_null() {
                        (*npc_info_ptr).touchedByPlayer =
                            Some(ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), other));
                    }
                }
            }

            let npc_info_ptr = (*ctx.world_raw()).globals.NPCInfo;
            if !npc_info_ptr.is_null()
                && (*npc_info_ptr).goalEntity
                    == Some(ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), other))
            {
                (*npc_info_ptr).aiFlags |= NPCAI_TOUCHED_GOAL;
            }
        }

        RestoreNPCGlobals(ctx);
    }
}

/// Raven `NPC_TempLookTarget`.
///
/// Source: `oracle/codemp/game/NPC_reactions.c:661-688`
pub fn NPC_TempLookTarget(
    ctx: &mut GameContext,
    self_: EntityId,
    lookEntNum: c_int,
    mut minLookTime: c_int,
    mut maxLookTime: c_int,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        let client = (*self_).client as *mut gclient_t;
        if client.is_null() {
            return;
        }

        // Raven `EF2_HELD_BY_MONSTER` (`playerState_t::eFlags2` bit) resolves
        // through the prelude (mp_bg::public::entity_effects).
        // Source: oracle/codemp/game/bg_public.h:616
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

        if NPC_CheckLookTarget(ctx, ctx.entity_id_of(self_).unwrap()) == 0 {
            //Not already looking at something else
            //Look at him for 1 to 3 seconds
            let level_time = (*ctx.world_raw()).level.time;
            let __h512 = level_time
                + (*ctx.world_raw())
                    .bg_state
                    .rng
                    .Q_irand(minLookTime, maxLookTime);
            let __h513 = ctx.entity_id_of(self_).unwrap();
            NPC_SetLookTarget(ctx.entity_mut(__h513), lookEntNum, __h512);
        }
    }
}

/// Raven `NPC_Respond`.
///
/// Source: `oracle/codemp/game/NPC_reactions.c:690-942`
pub fn NPC_Respond(ctx: &mut GameContext, self_: EntityId, userNum: c_int) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    unsafe {
        // The `CLASS_*` `class_t` variants resolve through the prelude; the
        // match below is on `NPC_class` (already `class_t`) directly rather than
        // a c_int cast, so no local class placeholders are needed.
        const CHAN_AUTO: c_int = 0;
        // Absolute entity_event_t values — G_AddVoiceEvent consumes the enum
        // value directly, so these must match the ported entity_event_t.
        const EV_CHASE1: c_int = 133;
        const EV_CHASE3: c_int = 135;
        const EV_OUTFLANK1: c_int = 147;
        const EV_OUTFLANK2: c_int = 148;
        const EV_COVER1: c_int = 136;
        const EV_COVER5: c_int = 140;
        const EV_SUSPICIOUS4: c_int = 167;
        const EV_SOUND1: c_int = 161;
        const EV_SOUND3: c_int = 163;
        const EV_CONFUSE1: c_int = 122;
        const EV_SIGHT1: c_int = 158;
        const EV_SIGHT2: c_int = 159;
        const EV_SIGHT3: c_int = 160;
        const EV_DETECTED1: c_int = 141;
        const EV_DETECTED5: c_int = 145;
        const EV_GIVEUP3: c_int = 154;
        const EV_GIVEUP4: c_int = 155;
        const EV_JDETECTED1: c_int = 172;
        const EV_JDETECTED2: c_int = 173;
        const EV_ANGER1: c_int = 116;
        const EV_ANGER3: c_int = 118;
        const EV_TAUNT1: c_int = 175;
        const EV_TAUNT2: c_int = 176;
        const EV_LOST1: c_int = 146;
        const EV_ESCAPING2: c_int = 150;

        let mut event = -1;

        if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) == 0 {
            // Set looktarget to them for a second or two
            NPC_TempLookTarget(ctx, ctx.entity_id_of(self_).unwrap(), userNum, 1000, 3000);
        }

        // Some last-minute hacked in responses
        let client = (*self_).client as *mut gclient_t;
        if client.is_null() {
            return;
        }

        let npc_class = (*client).NPC_class;
        let npc = (*self_).NPC as *mut gNPC_t;

        match npc_class {
            CLASS_JAN => {
                if (*self_).enemy.is_some() {
                    if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 2) == 0 {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_CHASE1, EV_CHASE3);
                    } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) != 0 {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                    } else {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_COVER1, EV_COVER5);
                    }
                } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 2) == 0 {
                    event = EV_SUSPICIOUS4;
                } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) == 0 {
                    event = EV_SOUND1;
                } else {
                    event = EV_CONFUSE1;
                }
            }
            CLASS_LANDO => {
                if (*self_).enemy.is_some() {
                    if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 2) == 0 {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_CHASE1, EV_CHASE3);
                    } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) != 0 {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                    } else {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_COVER1, EV_COVER5);
                    }
                } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 6) == 0 {
                    event = EV_SIGHT2;
                } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 5) == 0 {
                    event = EV_GIVEUP4;
                } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 4) > 1 {
                    event = (*ctx.world_raw())
                        .bg_state
                        .rng
                        .Q_irand(EV_SOUND1, EV_SOUND3);
                } else {
                    event = (*ctx.world_raw())
                        .bg_state
                        .rng
                        .Q_irand(EV_JDETECTED1, EV_JDETECTED2);
                }
            }
            CLASS_LUKE => {
                if (*self_).enemy.is_some() {
                    event = EV_COVER1;
                } else {
                    event = (*ctx.world_raw())
                        .bg_state
                        .rng
                        .Q_irand(EV_SOUND1, EV_SOUND3);
                }
            }
            CLASS_JEDI => {
                if (*self_).enemy.is_none() {
                    if 0 != 0 {
                        // rwwFIXMEFIXME: support flags!
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_ANGER1, EV_ANGER3);
                    } else {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_TAUNT1, EV_TAUNT2);
                    }
                }
            }
            CLASS_PRISONER => {
                if (*self_).enemy.is_some() {
                    if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) != 0 {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_CHASE1, EV_CHASE3);
                    } else {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                    }
                } else {
                    event = (*ctx.world_raw())
                        .bg_state
                        .rng
                        .Q_irand(EV_SOUND1, EV_SOUND3);
                }
            }
            CLASS_REBEL => {
                if (*self_).enemy.is_some() {
                    if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 2) == 0 {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_CHASE1, EV_CHASE3);
                    } else {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_DETECTED1, EV_DETECTED5);
                    }
                } else {
                    event = (*ctx.world_raw())
                        .bg_state
                        .rng
                        .Q_irand(EV_SOUND1, EV_SOUND3);
                }
            }
            CLASS_BESPIN_COP => {
                let npc_type_ptr = (*self_).NPC_type;
                let is_variant1 = if !npc_type_ptr.is_null() {
                    Q_stricmp(npc_type_ptr, cstr("bespincop").as_ptr()) == 0
                } else {
                    false
                };

                if is_variant1 {
                    // Variant 1
                    if (*self_).enemy.is_some() {
                        if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 9) > 6 {
                            event = (*ctx.world_raw())
                                .bg_state
                                .rng
                                .Q_irand(EV_CHASE1, EV_CHASE3);
                        } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 6) > 4 {
                            event = (*ctx.world_raw())
                                .bg_state
                                .rng
                                .Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                        } else {
                            event = (*ctx.world_raw())
                                .bg_state
                                .rng
                                .Q_irand(EV_COVER1, EV_COVER5);
                        }
                    } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 3) == 0 {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_SIGHT2, EV_SIGHT3);
                    } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) == 0 {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_SOUND1, EV_SOUND3);
                    } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 2) == 0 {
                        event = EV_LOST1;
                    } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) == 0 {
                        event = EV_ESCAPING2;
                    } else {
                        event = EV_GIVEUP4;
                    }
                } else {
                    // Variant 2
                    if (*self_).enemy.is_some() {
                        if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 9) > 6 {
                            event = (*ctx.world_raw())
                                .bg_state
                                .rng
                                .Q_irand(EV_CHASE1, EV_CHASE3);
                        } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 6) > 4 {
                            event = (*ctx.world_raw())
                                .bg_state
                                .rng
                                .Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                        } else {
                            event = (*ctx.world_raw())
                                .bg_state
                                .rng
                                .Q_irand(EV_COVER1, EV_COVER5);
                        }
                    } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 3) == 0 {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_SIGHT1, EV_SIGHT2);
                    } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) == 0 {
                        event = (*ctx.world_raw())
                            .bg_state
                            .rng
                            .Q_irand(EV_SOUND1, EV_SOUND3);
                    } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 2) == 0 {
                        event = EV_LOST1;
                    } else if (*ctx.world_raw()).bg_state.rng.Q_irand(0, 1) == 0 {
                        event = EV_GIVEUP3;
                    } else {
                        event = EV_CONFUSE1;
                    }
                }
            }
            CLASS_R2D2 => {
                // PORT-NOTE(va-formatting): droid sound paths use va() for dynamic formatting
                // ported to format!() with Rust string construction
                let sound_path = format!(
                    "sound/chars/r2d2/misc/r2d2talk0{}.wav",
                    (*ctx.world_raw()).bg_state.rng.Q_irand(1, 3)
                );
                let sound_index = crate::g_utils::G_SoundIndex(cstr(&sound_path).as_ptr());
                G_Sound(ctx, ctx.entity_id_of(self_), CHAN_AUTO, sound_index);
            }
            CLASS_R5D2 => {
                let sound_path = format!(
                    "sound/chars/r5d2/misc/r5talk{}.wav",
                    (*ctx.world_raw()).bg_state.rng.Q_irand(1, 4)
                );
                let sound_index = crate::g_utils::G_SoundIndex(cstr(&sound_path).as_ptr());
                G_Sound(ctx, ctx.entity_id_of(self_), CHAN_AUTO, sound_index);
            }
            CLASS_MOUSE => {
                let sound_path = format!(
                    "sound/chars/mouse/misc/mousego{}.wav",
                    (*ctx.world_raw()).bg_state.rng.Q_irand(1, 3)
                );
                let sound_index = crate::g_utils::G_SoundIndex(cstr(&sound_path).as_ptr());
                G_Sound(ctx, ctx.entity_id_of(self_), CHAN_AUTO, sound_index);
            }
            CLASS_GONK => {
                let sound_path = format!(
                    "sound/chars/gonk/misc/gonktalk{}.wav",
                    (*ctx.world_raw()).bg_state.rng.Q_irand(1, 2)
                );
                let sound_index = crate::g_utils::G_SoundIndex(cstr(&sound_path).as_ptr());
                G_Sound(ctx, ctx.entity_id_of(self_), CHAN_AUTO, sound_index);
            }
            _ => {}
        }

        if event != -1 {
            // Hack here because we reuse some "combat" and "extra" sounds
            let add_flag = if !npc.is_null() {
                ((*npc).scriptFlags & SCF_NO_COMBAT_TALK) != 0
            } else {
                false
            };

            if !npc.is_null() {
                (*npc).scriptFlags &= !SCF_NO_COMBAT_TALK;
            }

            crate::NPC_sounds::G_AddVoiceEvent(ctx, ctx.entity_id_of(self_).unwrap(), event, 3000);

            if add_flag && !npc.is_null() {
                (*npc).scriptFlags |= SCF_NO_COMBAT_TALK;
            }
        }
    }
}

/// Raven `NPC_UseResponse`.
///
/// Source: `oracle/codemp/game/NPC_reactions.c:950-999`
pub fn NPC_UseResponse(
    ctx: &mut GameContext,
    self_: EntityId,
    user: Option<EntityId>,
    useWhenDone: qboolean,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let user: *mut gentity_t = unsafe { ent_ptr(ctx, user) };
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        let client = (*self_).client as *mut gclient_t;
        if npc.is_null() || client.is_null() {
            return;
        }

        if (*user).s.number != 0 {
            //not used by the player
            if useWhenDone != 0 {
                G_ActivateBehavior(ctx, ctx.entity_id_of(self_), bSet_t::BSET_USE as c_int);
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
                G_ActivateBehavior(ctx, ctx.entity_id_of(self_), bSet_t::BSET_USE as c_int);
            }
            return;
        }

        if (*npc).blockedSpeechDebounceTime > (*ctx.world_raw()).level.time {
            //I'm not responding right now
            return;
        }

        if useWhenDone != 0 {
            G_ActivateBehavior(ctx, ctx.entity_id_of(self_), bSet_t::BSET_USE as c_int);
        } else {
            NPC_Respond(ctx, ctx.entity_id_of(self_).unwrap(), (*user).s.number);
        }
    }
}

/// Raven `NPC_Use`.
///
/// Source: `oracle/codemp/game/NPC_reactions.c:1008-1093`
pub fn NPC_Use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);
    let other: *mut gentity_t = unsafe { ent_ptr(ctx, other) };
    let activator: *mut gentity_t = unsafe { ent_ptr(ctx, activator) };
    unsafe {
        // `pm_type` is a c_int field and `BSET_USE` indexes `behaviorSet`
        // (c_int/usize), so alias both from their canonical enums.
        // `CLASS_VEHICLE`/`CLASS_GONK` are `class_t` variants from the prelude,
        // compared against `NPC_class` directly below.
        // Source: pmtype_t `oracle/codemp/game/bg_public.h:360-370`,
        // bSet_t `oracle/codemp/game/g_public.h:641-664`.
        const PM_DEAD: c_int = pmtype_t::PM_DEAD as c_int;
        const BSET_USE: c_int = bSet_t::BSET_USE as c_int;

        let client = (*self_).client as *mut gclient_t;
        if client.is_null() || (*client).ps.pm_type == PM_DEAD {
            return;
        }

        SaveNPCGlobals(ctx);
        SetNPCGlobals(ctx, ctx.entity_id_of(self_).unwrap());

        let npc = (*self_).NPC as *mut gNPC_t;
        if !client.is_null() && !npc.is_null() {
            // Check if this is a vehicle
            if (*client).NPC_class == CLASS_VEHICLE {
                // If this is a vehicle, let the other guy board it.
                let pVeh = (*self_).m_pVehicle as *mut Vehicle_t;
                if !pVeh.is_null() && !(*pVeh).m_pVehicleInfo.is_null() {
                    //if I used myself, eject everyone on me
                    if other == self_ {
                        crate::veh_dispatch::eject_all(ctx, pVeh);
                    }
                    // If other is already riding this vehicle (self), eject him.
                    else if (*other).s.owner == (*self_).s.number {
                        crate::veh_dispatch::eject(ctx, pVeh, other as *mut bgEntity_t, qfalse);
                    }
                    // Otherwise board this vehicle.
                    else {
                        crate::veh_dispatch::board(ctx, pVeh, other as *mut bgEntity_t);
                    }
                }
            } else if crate::NPC_AI_Jedi::Jedi_WaitingAmbush(&*self_) != 0 {
                crate::NPC_AI_Jedi::Jedi_Ambush(ctx, ctx.entity_id_of(self_).unwrap());
            }

            // Run any use instructions
            if !activator.is_null()
                && (*activator).s.number == 0
                && (*client).NPC_class == CLASS_GONK
            {
                // Must be using the gonk, so attempt to give battery power.
                // Oracle itself leaves the Add_Batteries call commented out
                // (`//rwwFIXMEFIXME: support for this?`), so this is a faithful
                // empty body — not a port gap.
            }

            if !(*self_).behaviorSet[BSET_USE as usize].is_null() {
                NPC_UseResponse(
                    ctx,
                    ctx.entity_id_of(self_).unwrap(),
                    ctx.entity_id_of(other),
                    1,
                );
            } else if !npc.is_null()
                && (*self_).enemy.is_none()
                && !activator.is_null()
                && (*activator).s.number == 0
                && ((*npc).scriptFlags & SCF_NO_RESPONSE) == 0
            {
                // I don't have an enemy and I was used by the player
                // (oracle gates on !(scriptFlags & SCF_NO_RESPONSE))
                NPC_UseResponse(
                    ctx,
                    ctx.entity_id_of(self_).unwrap(),
                    ctx.entity_id_of(other),
                    0,
                );
            }
        }

        RestoreNPCGlobals(ctx);
    }
}

/// Raven `NPC_CheckPlayerAim`.
///
/// Raven: body is entirely commented out (`//FIXME: need appropriate
/// dialogue`) — a dead no-op in the oracle.
/// Source: `oracle/codemp/game/NPC_reactions.c:1095-1111`
pub fn NPC_CheckPlayerAim() {}

/// Raven `NPC_CheckAllClear`.
///
/// Raven: body is entirely commented out (`//FIXME: need to make this happen
/// only once after losing enemies, not over and over again`) — a dead no-op
/// in the oracle.
/// Source: `oracle/codemp/game/NPC_reactions.c:1113-1125`
pub fn NPC_CheckAllClear() {}
