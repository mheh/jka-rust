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
//! PARKED (see PORT-NOTE markers): several functions read the ambient
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
use crate::g_utils::{G_AddEvent, G_Sound, G_UseTargets2};
use crate::NPC_utils::{G_ActivateBehavior, NPC_CheckLookTarget, NPC_SetLookTarget};
use crate::NPC_combat::{G_SetEnemy, G_ClearEnemy};
use crate::npc_c::{SaveNPCGlobals, SetNPCGlobals, RestoreNPCGlobals};
use crate::q_shared::Q_stricmp;

/// Raven `NPC_CheckAttacker`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:42-131`
pub fn NPC_CheckAttacker(
    ctx: GameContext<'_>,
    other: *mut gentity_t,
    r#mod: c_int,
) {
    unsafe {
        const FL_NOTARGET: c_int = 0x00000100;
        const WP_SABER: c_int = 1;
        const MOD_SABER: c_int = 16;

        // valid ent
        if other.is_null() {
            return;
        }

        let npc = (*ctx.world).globals.NPC;
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
            G_SetEnemy(ctx, npc, other);
            return;
        }

        // We have an enemy, see if he's dead
        if let Some(enemy_id) = (*npc).enemy {
            let base = (*ctx.world).g_entities.as_mut_ptr();
            let enemy_ptr = base.add(enemy_id.0 as usize);
            if (*enemy_ptr).health <= 0 {
                G_ClearEnemy(ctx, npc);
                G_SetEnemy(ctx, npc, other);
                return;
            }
        }

        // Don't take the same enemy again
        let other_id = ent_id((*ctx.world).g_entities.as_mut_ptr(), other);
        if (*npc).enemy == Some(other_id) {
            return;
        }

        // Check if we're a Jedi
        let client = (*npc).client as *mut gclient_t;
        if !client.is_null() && (*client).ps.weapon == WP_SABER {
            // I'm a jedi
            if r#mod == MOD_SABER {
                // Always switch to this enemy if I'm a jedi and hit by another saber
                G_ClearEnemy(ctx, npc);
                G_SetEnemy(ctx, npc, other);
                return;
            }
        }

        // Special case player interactions
        let player = (*ctx.world).g_entities.as_mut_ptr();
        if other == player {
            // Account for the skill level to skew the results
            let luck_threshold = match (*ctx.world).cvars.g_spskill.integer {
                0 => 0.9f32,  // Easiest difficulty
                1 => 0.5f32,  // Medium difficulty
                _ => 0.0f32,  // Hardest difficulty
            };

            // Randomly pick up the target
            if ((*ctx.world).bg_state.rng.random() as f32 / 32768.0f32) > luck_threshold {
                G_ClearEnemy(ctx, other);
                (*other).enemy = Some(ent_id((*ctx.world).g_entities.as_mut_ptr(), npc));
            }

            return;
        }
    }
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
    unsafe {
        const BOTH_PAIN1: c_int = 335;
        const BOTH_PAIN2: c_int = 336;
        const BOTH_PAIN3: c_int = 337;
        const BOTH_PAIN18: c_int = 352;
        const HL_GENERIC1: c_int = 19;
        const CLASS_GALAKMECH: c_int = 13;
        const CLASS_PROTOCOL: c_int = 5;
        const CLASS_DESANN: c_int = 8;
        const RANK_CAPTAIN: c_int = 5;
        const SETANIM_BOTH: c_int = 3;
        const SETANIM_LEGS: c_int = 2;
        const SETANIM_FLAG_OVERRIDE: c_int = 1;
        const SETANIM_FLAG_HOLD: c_int = 2;
        const WP_SABER: c_int = 1;
        const WP_THERMAL: c_int = 19;
        const MOD_MELEE: c_int = 18;
        const MOD_CRUSH: c_int = 12;
        const NPCTEAM_PLAYER: c_int = 0;

        // If we've already taken pain, then don't take it again
        if (*ctx.world).level.time < (*self_).painDebounceTime && r#mod != MOD_MELEE {
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

        if !client.is_null() && (*client).NPC_class as c_int == CLASS_GALAKMECH {
            if hitLoc == HL_GENERIC1 {
                // Hit the antenna!
                pain_chance = 1.0f32;
            } else if (*self_).health > 200 && damage < 100 {
                // Have a lot of health
                pain_chance = 0.05f32;
            } else {
                // The lower my health and greater the damage, the more likely I am to play a pain anim
                pain_chance = (200.0f32 - (*self_).health as f32) / 100.0f32 + damage as f32 / 50.0f32;
            }
        } else if !client.is_null() && (*client).playerTeam == NPCTEAM_PLAYER && !other.is_null() && (*other).s.number == 0 {
            // Ally shot by player always complains
            pain_chance = 1.1f32;
        } else {
            if !other.is_null() && (*other).s.weapon == WP_SABER || r#mod == MOD_CRUSH {
                pain_chance = 1.0f32;  // Always take pain from saber
            } else if r#mod == MOD_MELEE {
                // Higher in rank (skill) we are, less likely we are to be fazed by a punch
                let npc = (*self_).NPC as *mut gNPC_t;
                if !npc.is_null() {
                    pain_chance = 1.0f32 - ((RANK_CAPTAIN - (*npc).rank) as f32 / RANK_CAPTAIN as f32);
                } else {
                    pain_chance = 1.0f32;
                }
            } else if !client.is_null() && (*client).NPC_class as c_int == CLASS_PROTOCOL {
                pain_chance = 1.0f32;
            } else {
                pain_chance = NPC_GetPainChance(ctx, self_, damage);
            }

            if !client.is_null() && (*client).NPC_class as c_int == CLASS_DESANN {
                pain_chance *= 0.5f32;
            }
        }

        // See if we're going to flinch
        if ((*ctx.world).bg_state.rng.random() as f32 / 32768.0f32) < pain_chance {
            let mut pain_anim = -1;

            // Pick and play our animation
            if !client.is_null() && (*client).ps.fd.forceGripBeingGripped < (*ctx.world).level.time as f32 {
                // Not being force-gripped or force-drained
                let legs_anim = (*client).ps.legsAnim;
                let torso_anim = (*client).ps.torsoAnim;

                if crate::bg_panimate::PM_SpinningAnim(legs_anim) == qfalse &&
                   crate::bg_panimate::BG_SaberInSpecialAttack(torso_anim) == qfalse &&
                   crate::bg_panimate::PM_InKnockDown(&mut (*client).ps) == qfalse &&
                   crate::bg_pmove::PM_RollingAnim(legs_anim) == qfalse &&
                   !(crate::bg_panimate::BG_FlippingAnim(legs_anim) != qfalse && crate::bg_panimate::PM_InCartwheel(legs_anim) == qfalse) {
                    // Play an anim
                    let npc = (*self_).NPC as *mut gNPC_t;
                    let local_anim_index = (*self_).localAnimIndex;

                    if !client.is_null() && (*client).NPC_class as c_int == CLASS_GALAKMECH {
                        pain_anim = BOTH_PAIN1;
                    } else if r#mod == MOD_MELEE {
                        pain_anim = crate::bg_panimate::BG_PickAnim(&mut (*ctx.world).bg_state, local_anim_index, BOTH_PAIN2, BOTH_PAIN3);
                    } else if (*self_).s.weapon == WP_SABER {
                        // These are the only 2 pain anims that look good when holding a saber
                        pain_anim = crate::bg_panimate::BG_PickAnim(&mut (*ctx.world).bg_state, local_anim_index, BOTH_PAIN2, BOTH_PAIN3);
                    }

                    if pain_anim == -1 {
                        pain_anim = crate::bg_panimate::BG_PickAnim(&mut (*ctx.world).bg_state, local_anim_index, BOTH_PAIN1, BOTH_PAIN18);
                    }

                    (*client).ps.fd.saberAnimLevel = 1;  // FORCE_LEVEL_1
                    (*client).ps.saberMove = 0;  // LS_READY

                    let mut parts = SETANIM_BOTH;
                    if crate::bg_panimate::BG_CrouchAnim((*client).ps.legsAnim) != qfalse || crate::bg_panimate::PM_InCartwheel((*client).ps.legsAnim) != qfalse {
                        parts = SETANIM_LEGS;
                    }

                    if pain_anim != -1 {
                        crate::npc_c::NPC_SetAnim(self_, parts, pain_anim, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    }
                }

                if voiceEvent != -1 {
                    crate::NPC_sounds::G_AddVoiceEvent(ctx, self_, voiceEvent, (*ctx.world).bg_state.rng.Q_irand(2000, 4000));
                } else {
                    NPC_SetPainEvent(ctx, self_);
                }
            } else {
                // Being force-gripped
                crate::NPC_sounds::G_AddVoiceEvent(ctx, self_, (*ctx.world).bg_state.rng.Q_irand(335, 337), 0);
            }

            // Setup the timing for it
            let local_anim_index = (*self_).localAnimIndex;
            let num_frames = if pain_anim >= 0 {
                // PORT-NOTE(bgAllAnims-access): accessing global bgAllAnims/bgHumanoidAnimations
                // arrays through raw pointer arithmetic; these should be threaded via BgState
                let anim_length = 30; // Placeholder; would need bgAllAnims[local_anim_index].anims[pain_anim].numFrames
                let frame_lerp = 1; // Placeholder; would need bgHumanoidAnimations[pain_anim].frameLerp
                anim_length * frame_lerp
            } else {
                0
            };

            (*self_).painDebounceTime = (*ctx.world).level.time + num_frames;
            if !client.is_null() {
                (*client).ps.weaponTime = 0;
            }
        }
    }
}

/// Raven `NPC_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:363-529`
pub fn NPC_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    unsafe {
        const TEAM_FREE: c_int = 0;
        const NPCTEAM_PLAYER: c_int = 0;
        const MOD_MELEE: c_int = 18;
        const BSET_FLEE: c_int = 4;
        const BSET_PAIN: c_int = 2;
        const BSET_FFIRE: c_int = 5;
        const PM_DEAD: c_int = 3;
        const EV_FFWARN: c_int = 55;

        let mut other_team = TEAM_FREE;
        let mut voice_event = -1;
        let other = attacker;
        let r#mod = (*ctx.world).globals.gPainMOD;
        let hit_loc = (*ctx.world).globals.gPainHitLoc;
        let mut point = [0.0f32; 3];
        crate::q_math::_VectorCopy((*ctx.world).globals.gPainPoint, &mut point);

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

        if !client.is_null() && (*client).playerTeam != 0 && !other_client.is_null() && other_team == (*client).playerTeam {
            // Hit by a teammate
            let npc_ptr = (*ctx.world).globals.NPC;
            let other_id = ent_id((*ctx.world).g_entities.as_mut_ptr(), other);
            let npc_id = ent_id((*ctx.world).g_entities.as_mut_ptr(), npc_ptr);

            if (*npc_ptr).enemy != Some(other_id) && npc_ptr != other {
                // We weren't already enemies
                if !(*npc_ptr).enemy.is_none() || !(*other).enemy.is_none() {
                    // If one of us actually has an enemy already, it's okay, just an accident
                    if !client.is_null() && !npc.is_null() {
                        // Run any pain instructions
                        if (*self_).health <= ((*client).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize] / 3) &&
                           G_ActivateBehavior(ctx, self_, BSET_FLEE) != 0 {
                        } else {
                            G_ActivateBehavior(ctx, self_, BSET_PAIN);
                        }
                    }

                    if damage != -1 {
                        // Set our proper pain animation
                        if (*ctx.world).bg_state.rng.Q_irand(0, 1) != 0 {
                            NPC_ChoosePainAnimation(ctx, self_, other, point, damage, r#mod, hit_loc, EV_FFWARN);
                        } else {
                            NPC_ChoosePainAnimation(ctx, self_, other, point, damage, r#mod, hit_loc, -1);
                        }
                    }
                    return;
                } else if !npc.is_null() && (*other).s.number == 0 {
                    // NPC hit by player
                    if (*npc).charmedTime != 0 {
                        // Mindtricked
                        return;
                    } else if (*npc).ffireCount < 3 + ((2 - (*ctx.world).cvars.g_spskill.integer) * 2) {
                        // Not mad enough yet
                        if damage != -1 {
                            if (*ctx.world).bg_state.rng.Q_irand(0, 1) != 0 {
                                NPC_ChoosePainAnimation(ctx, self_, other, point, damage, r#mod, hit_loc, EV_FFWARN);
                            } else {
                                NPC_ChoosePainAnimation(ctx, self_, other, point, damage, r#mod, hit_loc, -1);
                            }
                        }
                        return;
                    } else if G_ActivateBehavior(ctx, self_, BSET_FFIRE) != 0 {
                        // We have a specific script to run
                        return;
                    } else {
                        // Turn on our ally
                        (*npc).blockedSpeechDebounceTime = 0;
                        voice_event = 32;  // EV_FFTURN
                        (*npc).behaviorState = (*npc).tempBehavior;
                        (*npc).tempBehavior = (*npc).defaultBehavior;
                        (*npc).defaultBehavior = bState_t::BS_DEFAULT;
                        (*other).flags &= !0x00000100;  // ~FL_NOTARGET
                        (*self_).r.svFlags &= !0x00080000;  // ~SVF_ICARUS_FREEZE
                        G_SetEnemy(ctx, self_, other);
                        (*npc).scriptFlags &= !(0x00000080 | 0x00000100 | 0x00000200 | 0x00001000 | 0x00002000);  // ~(SCF_DONT_FIRE|...)
                        (*npc).scriptFlags |= (0x00000001 | 0x00004000);  // |= (SCF_CHASE_ENEMIES|SCF_NO_MIND_TRICK)

                        if (*ctx.world).globals.killPlayerTimer == 0 {
                            (*ctx.world).globals.killPlayerTimer = (*ctx.world).level.time + 10000;
                        }
                    }
                }
            }
        }

        SaveNPCGlobals(ctx);
        SetNPCGlobals(ctx, self_);

        // Do extra bits
        let npc_info_ptr = (*ctx.world).globals.NPCInfo;
        if !npc_info_ptr.is_null() && (*npc_info_ptr).ignorePain == 0 {
            (*npc_info_ptr).confusionTime = 0;  // Clear any charm or confusion
            if damage != -1 {
                NPC_ChoosePainAnimation(ctx, self_, other, point, damage, r#mod, hit_loc, voice_event);
            }

            // Check to take a new enemy
            let npc_ptr = (*ctx.world).globals.NPC;
            if (*npc_ptr).enemy != Some(ent_id((*ctx.world).g_entities.as_mut_ptr(), other)) && npc_ptr != other {
                NPC_CheckAttacker(ctx, other, r#mod);
            }
        }

        // Attempt to run any pain instructions
        if !client.is_null() && !npc.is_null() {
            if (*self_).health <= ((*client).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize] / 3) &&
               G_ActivateBehavior(ctx, self_, BSET_FLEE) != 0 {
            } else {
                G_ActivateBehavior(ctx, self_, BSET_PAIN);
            }
        }

        // Attempt to fire any paintargets we might have
        if !(*self_).paintarget.is_null() && *((*self_).paintarget) != 0 {
            G_UseTargets2(ctx, self_, other, (*self_).paintarget);
        }

        RestoreNPCGlobals(ctx);
    }
}

/// Raven `NPC_Touch`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:537-653`
pub fn NPC_Touch(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    other: *mut gentity_t,
    trace: *mut trace_t,
) {
    unsafe {
        const MAX_CLIENTS: c_int = 64;
        const NPCAI_TOUCHED_GOAL: c_int = 0x00002000;
        const BS_HUNT_AND_KILL: c_int = 1;

        let npc = (*self_).NPC as *mut gNPC_t;
        if npc.is_null() {
            return;
        }

        SaveNPCGlobals(ctx);
        SetNPCGlobals(ctx, self_);

        // I am dead and carrying a key
        if !(*self_).message.is_null() && (*self_).health <= 0 {
            // Player touched me
            let other_client = (*other).client as *mut gclient_t;
            if !other.is_null() && !other_client.is_null() && (*other).s.number < MAX_CLIENTS {
                // Placeholder: would handle key pickup here (commented out in oracle)
            }
        }

        let other_client = (*other).client as *mut gclient_t;
        if !other_client.is_null() {
            // Other has a client (is a player)
            if (*other).health > 0 {
                let npc_info_ptr = (*ctx.world).globals.NPCInfo;
                if !npc_info_ptr.is_null() {
                    (*npc_info_ptr).touchedByPlayer = Some(ent_id((*ctx.world).g_entities.as_mut_ptr(), other));
                }
            }

            let npc_info_ptr = (*ctx.world).globals.NPCInfo;
            if !npc_info_ptr.is_null() && (*npc_info_ptr).goalEntity == Some(ent_id((*ctx.world).g_entities.as_mut_ptr(), other)) {
                (*npc_info_ptr).aiFlags |= NPCAI_TOUCHED_GOAL;
            }

            // Check for enemy collision
            if ((*self_).r.svFlags & 0x00000200) == 0 && ((*other).flags & 0x00000100) == 0 {
                // ~SVF_IGNORE_ENEMIES, ~FL_NOTARGET
                let client = (*self_).client as *mut gclient_t;
                if !client.is_null() && (*client).enemyTeam != 0 {
                    // See if we bumped into an enemy
                    if (*other_client).playerTeam == (*client).enemyTeam {
                        // Bumped into an enemy
                        let npc_info_ptr = (*ctx.world).globals.NPCInfo;
                        if !npc_info_ptr.is_null() &&
                           (*npc_info_ptr).behaviorState != BS_HUNT_AND_KILL &&
                           (*npc_info_ptr).tempBehavior == bState_t::BS_DEFAULT {
                            let npc_ptr = (*ctx.world).globals.NPC;
                            if (*npc_ptr).enemy != Some(ent_id((*ctx.world).g_entities.as_mut_ptr(), other)) {
                                G_SetEnemy(ctx, npc_ptr, other);
                            }
                        }
                    }
                }
            }
        } else {
            // Other is not a client
            if (*other).health > 0 {
                // Non-NPC entity (probably an object)
                if 0 != 0 {  // rwwFIXMEFIXME condition always false
                    let npc_info_ptr = (*ctx.world).globals.NPCInfo;
                    if !npc_info_ptr.is_null() {
                        (*npc_info_ptr).touchedByPlayer = Some(ent_id((*ctx.world).g_entities.as_mut_ptr(), other));
                    }
                }
            }

            let npc_info_ptr = (*ctx.world).globals.NPCInfo;
            if !npc_info_ptr.is_null() && (*npc_info_ptr).goalEntity == Some(ent_id((*ctx.world).g_entities.as_mut_ptr(), other)) {
                (*npc_info_ptr).aiFlags |= NPCAI_TOUCHED_GOAL;
            }
        }

        RestoreNPCGlobals(ctx);
    }
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
            NPC_SetLookTarget(self_, lookEntNum, level_time + (*ctx.world).bg_state.rng.Q_irand(minLookTime, maxLookTime));
        }
    }
}

/// Raven `NPC_Respond`.
///
/// Source: `oracle/oracle/codemp/game/NPC_reactions.c:690-942`
pub fn NPC_Respond(ctx: GameContext<'_>, self_: *mut gentity_t, userNum: c_int) {
    unsafe {
        const CLASS_JAN: c_int = 1;
        const CLASS_LANDO: c_int = 2;
        const CLASS_LUKE: c_int = 3;
        const CLASS_JEDI: c_int = 4;
        const CLASS_PRISONER: c_int = 6;
        const CLASS_REBEL: c_int = 7;
        const CLASS_BESPIN_COP: c_int = 9;
        const CLASS_R2D2: c_int = 20;
        const CLASS_R5D2: c_int = 21;
        const CLASS_MOUSE: c_int = 22;
        const CLASS_GONK: c_int = 23;
        const CHAN_AUTO: c_int = 0;
        const EV_CHASE1: c_int = 1;
        const EV_CHASE3: c_int = 3;
        const EV_OUTFLANK1: c_int = 4;
        const EV_OUTFLANK2: c_int = 5;
        const EV_COVER1: c_int = 6;
        const EV_COVER5: c_int = 10;
        const EV_SUSPICIOUS4: c_int = 49;
        const EV_SOUND1: c_int = 50;
        const EV_SOUND3: c_int = 52;
        const EV_CONFUSE1: c_int = 53;
        const EV_SIGHT1: c_int = 35;
        const EV_SIGHT2: c_int = 36;
        const EV_SIGHT3: c_int = 37;
        const EV_DETECTED1: c_int = 38;
        const EV_DETECTED5: c_int = 42;
        const EV_GIVEUP3: c_int = 46;
        const EV_GIVEUP4: c_int = 47;
        const EV_JDETECTED1: c_int = 51;
        const EV_JDETECTED2: c_int = 51;
        const EV_ANGER1: c_int = 54;
        const EV_ANGER3: c_int = 56;
        const EV_TAUNT1: c_int = 57;
        const EV_TAUNT2: c_int = 58;
        const EV_LOST1: c_int = 45;
        const EV_ESCAPING2: c_int = 33;

        let mut event = -1;

        if (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 {
            // Set looktarget to them for a second or two
            NPC_TempLookTarget(ctx, self_, userNum, 1000, 3000);
        }

        // Some last-minute hacked in responses
        let client = (*self_).client as *mut gclient_t;
        if client.is_null() {
            return;
        }

        let npc_class = (*client).NPC_class as c_int;
        let npc = (*self_).NPC as *mut gNPC_t;

        match npc_class {
            CLASS_JAN => {
                if (*self_).enemy.is_some() {
                    if (*ctx.world).bg_state.rng.Q_irand(0, 2) == 0 {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_CHASE1, EV_CHASE3);
                    } else if (*ctx.world).bg_state.rng.Q_irand(0, 1) != 0 {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                    } else {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_COVER1, EV_COVER5);
                    }
                } else if (*ctx.world).bg_state.rng.Q_irand(0, 2) == 0 {
                    event = EV_SUSPICIOUS4;
                } else if (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 {
                    event = EV_SOUND1;
                } else {
                    event = EV_CONFUSE1;
                }
            }
            CLASS_LANDO => {
                if (*self_).enemy.is_some() {
                    if (*ctx.world).bg_state.rng.Q_irand(0, 2) == 0 {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_CHASE1, EV_CHASE3);
                    } else if (*ctx.world).bg_state.rng.Q_irand(0, 1) != 0 {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                    } else {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_COVER1, EV_COVER5);
                    }
                } else if (*ctx.world).bg_state.rng.Q_irand(0, 6) == 0 {
                    event = EV_SIGHT2;
                } else if (*ctx.world).bg_state.rng.Q_irand(0, 5) == 0 {
                    event = EV_GIVEUP4;
                } else if (*ctx.world).bg_state.rng.Q_irand(0, 4) > 1 {
                    event = (*ctx.world).bg_state.rng.Q_irand(EV_SOUND1, EV_SOUND3);
                } else {
                    event = (*ctx.world).bg_state.rng.Q_irand(EV_JDETECTED1, EV_JDETECTED2);
                }
            }
            CLASS_LUKE => {
                if (*self_).enemy.is_some() {
                    event = EV_COVER1;
                } else {
                    event = (*ctx.world).bg_state.rng.Q_irand(EV_SOUND1, EV_SOUND3);
                }
            }
            CLASS_JEDI => {
                if (*self_).enemy.is_none() {
                    if 0 != 0 {  // rwwFIXMEFIXME: support flags!
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_ANGER1, EV_ANGER3);
                    } else {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_TAUNT1, EV_TAUNT2);
                    }
                }
            }
            CLASS_PRISONER => {
                if (*self_).enemy.is_some() {
                    if (*ctx.world).bg_state.rng.Q_irand(0, 1) != 0 {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_CHASE1, EV_CHASE3);
                    } else {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                    }
                } else {
                    event = (*ctx.world).bg_state.rng.Q_irand(EV_SOUND1, EV_SOUND3);
                }
            }
            CLASS_REBEL => {
                if (*self_).enemy.is_some() {
                    if (*ctx.world).bg_state.rng.Q_irand(0, 2) == 0 {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_CHASE1, EV_CHASE3);
                    } else {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_DETECTED1, EV_DETECTED5);
                    }
                } else {
                    event = (*ctx.world).bg_state.rng.Q_irand(EV_SOUND1, EV_SOUND3);
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
                        if (*ctx.world).bg_state.rng.Q_irand(0, 9) > 6 {
                            event = (*ctx.world).bg_state.rng.Q_irand(EV_CHASE1, EV_CHASE3);
                        } else if (*ctx.world).bg_state.rng.Q_irand(0, 6) > 4 {
                            event = (*ctx.world).bg_state.rng.Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                        } else {
                            event = (*ctx.world).bg_state.rng.Q_irand(EV_COVER1, EV_COVER5);
                        }
                    } else if (*ctx.world).bg_state.rng.Q_irand(0, 3) == 0 {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_SIGHT2, EV_SIGHT3);
                    } else if (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_SOUND1, EV_SOUND3);
                    } else if (*ctx.world).bg_state.rng.Q_irand(0, 2) == 0 {
                        event = EV_LOST1;
                    } else if (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 {
                        event = EV_ESCAPING2;
                    } else {
                        event = EV_GIVEUP4;
                    }
                } else {
                    // Variant 2
                    if (*self_).enemy.is_some() {
                        if (*ctx.world).bg_state.rng.Q_irand(0, 9) > 6 {
                            event = (*ctx.world).bg_state.rng.Q_irand(EV_CHASE1, EV_CHASE3);
                        } else if (*ctx.world).bg_state.rng.Q_irand(0, 6) > 4 {
                            event = (*ctx.world).bg_state.rng.Q_irand(EV_OUTFLANK1, EV_OUTFLANK2);
                        } else {
                            event = (*ctx.world).bg_state.rng.Q_irand(EV_COVER1, EV_COVER5);
                        }
                    } else if (*ctx.world).bg_state.rng.Q_irand(0, 3) == 0 {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_SIGHT1, EV_SIGHT2);
                    } else if (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 {
                        event = (*ctx.world).bg_state.rng.Q_irand(EV_SOUND1, EV_SOUND3);
                    } else if (*ctx.world).bg_state.rng.Q_irand(0, 2) == 0 {
                        event = EV_LOST1;
                    } else if (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 {
                        event = EV_GIVEUP3;
                    } else {
                        event = EV_CONFUSE1;
                    }
                }
            }
            CLASS_R2D2 => {
                // PORT-NOTE(va-formatting): droid sound paths use va() for dynamic formatting
                // ported to format!() with Rust string construction
                let sound_path = format!("sound/chars/r2d2/misc/r2d2talk0{}.wav", (*ctx.world).bg_state.rng.Q_irand(1, 3));
                let sound_index = crate::g_utils::G_SoundIndex(cstr(&sound_path).as_ptr());
                G_Sound(ctx, self_, CHAN_AUTO, sound_index);
            }
            CLASS_R5D2 => {
                let sound_path = format!("sound/chars/r5d2/misc/r5talk{}.wav", (*ctx.world).bg_state.rng.Q_irand(1, 4));
                let sound_index = crate::g_utils::G_SoundIndex(cstr(&sound_path).as_ptr());
                G_Sound(ctx, self_, CHAN_AUTO, sound_index);
            }
            CLASS_MOUSE => {
                let sound_path = format!("sound/chars/mouse/misc/mousego{}.wav", (*ctx.world).bg_state.rng.Q_irand(1, 3));
                let sound_index = crate::g_utils::G_SoundIndex(cstr(&sound_path).as_ptr());
                G_Sound(ctx, self_, CHAN_AUTO, sound_index);
            }
            CLASS_GONK => {
                let sound_path = format!("sound/chars/gonk/misc/gonktalk{}.wav", (*ctx.world).bg_state.rng.Q_irand(1, 2));
                let sound_index = crate::g_utils::G_SoundIndex(cstr(&sound_path).as_ptr());
                G_Sound(ctx, self_, CHAN_AUTO, sound_index);
            }
            _ => {}
        }

        if event != -1 {
            // Hack here because we reuse some "combat" and "extra" sounds
            let add_flag = if !npc.is_null() {
                ((*npc).scriptFlags & 0x00001000) != 0
            } else {
                false
            };

            if !npc.is_null() {
                (*npc).scriptFlags &= !0x00001000;  // ~SCF_NO_COMBAT_TALK
            }

            crate::NPC_sounds::G_AddVoiceEvent(ctx, self_, event, 3000);

            if add_flag && !npc.is_null() {
                (*npc).scriptFlags |= 0x00001000;  // |= SCF_NO_COMBAT_TALK
            }
        }
    }
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
pub fn NPC_Use(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    unsafe {
        const PM_DEAD: c_int = 3;
        const CLASS_VEHICLE: c_int = 19;
        const CLASS_GONK: c_int = 23;
        const BSET_USE: c_int = 3;

        let client = (*self_).client as *mut gclient_t;
        if client.is_null() || (*client).ps.pm_type == PM_DEAD {
            return;
        }

        SaveNPCGlobals(ctx);
        SetNPCGlobals(ctx, self_);

        let npc = (*self_).NPC as *mut gNPC_t;
        if !client.is_null() && !npc.is_null() {
            // Check if this is a vehicle
            if (*client).NPC_class == CLASS_VEHICLE {
                // PORT-NOTE(vehicle-vtable): CLASS_VEHICLE entity calls C++ vehicleInfo_t vtable methods
                // (EjectAll, Eject, Board) which are deferred per porting-rules §F (C++ track)
                let m_vehicle = (*self_).m_pVehicle;
                if !m_vehicle.is_null() {
                    // Check if I used myself, or if other is riding this vehicle
                    if other == self_ {
                        // Eject everyone on me (deferred: pVeh->m_pVehicleInfo->EjectAll(pVeh))
                    } else if (*other).s.owner == (*self_).s.number {
                        // If other is already riding this vehicle (self), eject him
                        // (deferred: pVeh->m_pVehicleInfo->Eject(pVeh, (bgEntity_t *)other, qfalse))
                    } else {
                        // Otherwise board this vehicle
                        // (deferred: pVeh->m_pVehicleInfo->Board(pVeh, (bgEntity_t *)other))
                    }
                }
            } else if crate::NPC_AI_Jedi::Jedi_WaitingAmbush(self_) != 0 {
                crate::NPC_AI_Jedi::Jedi_Ambush(ctx, self_);
            }

            // Run any use instructions
            if !activator.is_null() && (*activator).s.number == 0 && (*client).NPC_class == CLASS_GONK {
                // Must be using the gonk, so attempt to give battery power
                // (deferred: Add_Batteries(activator, &self->client->ps.batteryCharge))
            }

            if !(*self_).behaviorSet[BSET_USE as usize].is_null() {
                NPC_UseResponse(ctx, self_, other, 1);
            } else if !npc.is_null() && (*self_).enemy.is_none() && !activator.is_null() && (*activator).s.number == 0 &&
                      ((*npc).scriptFlags & 0x00004000) == 0 {
                // I don't have an enemy and I was used by the player
                NPC_UseResponse(ctx, self_, other, 0);
            }
        }

        RestoreNPCGlobals(ctx);
    }
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
