//! FAITHFUL port of `oracle/codemp/game/g_trigger.c`.
//!
//! Functions reach file-scope game state (`level`, `g_entities`, cvars, all `GameWorld` fields) through the threaded
//! `GameContext`/`GameWorld` handle.
//! Functions reach engine traps (`trap_*`) through the same handle.
//! The shared RNG is threaded through the handle, not read from a bare `crandom`/`Q_irand`.
//! Fn-pointer fields are set through the `EntThink`/`EntTouch`/`EntUse` enums (see `out/gen/ent_fn_enums.rs`), not
//! through a bare fn pointer.
//!
//! Entity fields are reached through `ctx.world.entity(id)` / `entity_mut(id)` accessor borrows at the point of use.
//! Two raw regimes remain: the ABI seam (a raw `*mut gentity_t` handed to `trap_*`, and `CStr::from_ptr` on stored
//! `char*` fields), and the pool-`gclient_t` derefs.
//! An entity's `.client` field is a `level.clients` pointer only for a real client slot.
//! NPC and vehicle triggers read the raw pointer value through the safe entity borrow and deref it in a tight
//! `unsafe` block, the same as Raven.
//! The referee suite confirms this behavior is byte-identical to the oracle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

use crate::ent_fn_enums::{EntThink, EntTouch, EntUse};
use crate::entity::flags::FL_INACTIVE;
use crate::g_combat::{G_Damage, G_RadiusDamage};
use crate::g_main::{G_Error, G_Printf};
use crate::g_misc::TeleportPlayer;
use crate::g_mover::SP_func_rotating;
use crate::g_spawn::{G_SpawnFloat, G_SpawnInt, G_SpawnString};
use crate::g_utils::{
    G_EffectIndex, G_EntitySound, G_FreeEntity, G_PickTarget, G_PlayEffectID, G_PointInBounds,
    G_ScaleNetHealth, G_SetAngles, G_SetMovedir, G_SetOrigin, G_Sound, G_SoundIndex, G_Spawn,
    G_UseTargets,
};
use crate::q_math::vec3_origin;
use crate::trap;
use crate::NPC_utils::G_ActivateBehavior;
use native_string::{atoi_bytes, Q_stricmp};
use mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_abi::game::syscalls::G_UNLINKENTITY::GUnlinkentityArgs;
use mp_qshared::shared::trajectory::trType_t::TR_LINEAR;
use crate::q_shared;

// A resolved cross-file signature can take a `vec3_t` by value where Raven passed `NULL`, for example `G_Damage`'s
// `point` parameter.
// `vec3_origin` (the zero vector) stands in for `NULL` at these call sites. This is a value substitution, not a
// behavior change.

// Raven's `qboolean` is `c_int`. This file keeps the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

// `CONTENTS_PLAYERCLIP`/`MASK_PLAYERSOLID` (`mp_qshared::shared::surface_flags`) and `FRAMETIME` (`crate::g_items`)
// resolve through the crate prelude glob, not a local import.

// These file-scope spawnflag `#define`s are module-private. Each constant cites its own oracle source line.
// `PMF_FOLLOW`, `CS_GLOBAL_AMBIENT_SET`, `SIEGETEAM_TEAM1`, and `SIEGETEAM_TEAM2` are ported
// (`mp_qshared::common::mp::qcommon::pm_flags`, `mp_bg::public::configstring`, `mp_bg::saga::siege_team_t`) and reach
// this file through `crate::prelude::*`.
// Source: oracle/codemp/game/g_trigger.c:899
pub const PUSH_CONSTANT: c_int = 2;
// Source: oracle/codemp/game/g_trigger.c:895
pub const PUSH_LINEAR: c_int = 4;
// Source: oracle/codemp/game/g_trigger.c:896
pub const PUSH_RELATIVE: c_int = 16;
// Source: oracle/codemp/game/g_trigger.c:897
pub const PUSH_MULTIPLE: c_int = 2048;
// `HYPERSPACE_TIME` and `HYPERSPACE_TELEPORT_FRAC` are `bg_public.h` constants.
// They live in `mp_bg::public::hyperspace`, where bg consumers reach them, and reach this file through
// `crate::prelude::*`.
// Source: oracle/codemp/game/bg_public.h:1679-1680
// `EF2_HYPERSPACE` and `EF_RAG` are ported (`mp_bg::public::entity_effects`) and reach this file through
// `crate::prelude::*`.
// Source: oracle/codemp/game/g_trigger.c:1441
pub const INITIAL_SUFFOCATION_DELAY: c_int = 500;

// `atoi` is the libc-parity helper reached through the prelude (`crate::cstr_util::atoi`).
// This file carries no local extern shim for it.

/// Raven `InitTrigger`.
///
/// Source: `oracle/codemp/game/g_trigger.c:8-20`
pub fn InitTrigger(ctx: &mut GameContext, self_id: EntityId) {
    if !VectorCompare(ctx.world.entity(self_id).s.angles, vec3_origin) {
        let e = ctx.world.entity_mut(self_id);
        G_SetMovedir(&mut e.s.angles, &mut e.movedir);
    }

    let self_ptr = ctx.world.entity_mut(self_id) as *mut gentity_t;
    let model = ctx.world.entity(self_id).model.clone();
    trap::SetBrushModel(ctx.engine, self_ptr.cast(), model.as_deref().unwrap_or(""));
    ctx.world.entity_mut(self_id).r.contents = CONTENTS_TRIGGER; // replaces the -1 from trap_SetBrushModel
    ctx.world.entity_mut(self_id).r.svFlags = SVF_NOCLIENT;

    if ctx.world.entity(self_id).spawnflags & 128 != 0 {
        ctx.world.entity_mut(self_id).flags |= FL_INACTIVE;
    }
}

/// Raven `multi_wait`.
///
/// the wait time has passed, so set back up for another activation
/// Source: `oracle/codemp/game/g_trigger.c:23-25`
pub fn multi_wait(ent: &mut gentity_t) {
    ent.nextthink = 0;
}

/// Raven `multi_trigger_run`.
///
/// the trigger was just activated
/// ent->activator should be set to the activator so it can be held through a delay
/// so wait for the delay time before firing
/// Source: `oracle/codemp/game/g_trigger.c:32-94`
pub fn multi_trigger_run(ctx: &mut GameContext, ent: EntityId) {
    ctx.world.entity_mut(ent).think = FnId::NONE;

    G_ActivateBehavior(ctx, Some(ent), bSet_t::BSET_USE as c_int);

    let sound_set = ctx.world.entity(ent).soundSet.clone();
    if !sound_set.is_empty() {
        trap::SetConfigstring(ctx.engine, CS_GLOBAL_AMBIENT_SET, &sound_set);
    }

    let activator = ctx.world.entity(ent).activator;

    if ctx.world.entity(ent).genericValue4 != 0 {
        // we want to activate target3 for team1 or target4 for team2
        let gv4 = ctx.world.entity(ent).genericValue4;
        let target3 = ctx.world.entity(ent).target3.clone();
        let target4 = ctx.world.entity(ent).target4.clone();
        if gv4 == SIEGETEAM_TEAM1 && !target3.is_empty() {
            G_UseTargets2(ctx, Some(ent), activator, Some(&target3));
        } else if gv4 == SIEGETEAM_TEAM2 && !target4.is_empty() {
            G_UseTargets2(ctx, Some(ent), activator, Some(&target4));
        }

        ctx.world.entity_mut(ent).genericValue4 = 0;
    }

    G_UseTargets(ctx, Some(ent), activator);
    if ctx.world.entity(ent).noise_index != 0 {
        let ni = ctx.world.entity(ent).noise_index;
        G_Sound(ctx, activator, CHAN_AUTO, ni);
    }

    let target2 = ctx.world.entity(ent).target2.clone();
    let wait = ctx.world.entity(ent).wait;
    if target2.as_deref().is_some_and(|s| !s.is_empty()) && wait >= 0.0 {
        ctx.world.entity_mut(ent).think = Some(EntThink::trigger_cleared_fire).into();
        let nt = ctx.world.level.time + ctx.world.entity(ent).speed as c_int;
        ctx.world.entity_mut(ent).nextthink = nt;
    } else if wait > 0.0 {
        if ctx.world.entity(ent).painDebounceTime != ctx.world.level.time {
            // first ent to touch it this frame
            // C evaluates the whole right-hand side in `double`, because `crandom()` returns `double`.
            // The result truncates once into the `int` nextthink.
            let w = ctx.world.entity(ent).wait as f64;
            let r = ctx.world.entity(ent).random as f64;
            let nt = (ctx.world.level.time as f64
                + (w + r * ctx.world.bg_state.rng.crandom()) * 1000.0)
                as c_int;
            ctx.world.entity_mut(ent).nextthink = nt;
            ctx.world.entity_mut(ent).painDebounceTime = ctx.world.level.time;
        }
    } else if wait < 0.0 {
        // we can't just remove (self) here, because this is a touch function
        // called while looping through area links...
        ctx.world.entity_mut(ent).r.contents &= !CONTENTS_TRIGGER; // so the EntityContact trace doesn't have to be done against me
        ctx.world.entity_mut(ent).think = FnId::NONE;
        ctx.world.entity_mut(ent).use_ = FnId::NONE;
        // Don't remove, Icarus may barf?
    }

    if let Some(activator_id) = activator {
        // mark the trigger as being touched by the player
        let ac = ctx.world.entity(activator_id).client;
        if !ac.is_null() {
            ctx.world.entity_mut(ent).aimDebounceTime = ctx.world.level.time;
        }
    }
}

/// Raven `G_NameInTriggerClassList`.
///
/// determine if the class given is listed in the string using the | formatting
/// Source: `oracle/codemp/game/g_trigger.c:97-126`
pub fn G_NameInTriggerClassList(list: *mut c_char, str: *mut c_char) -> qboolean {
    unsafe {
        let mut cmp = [0 as c_char; 1024]; // Raven `MAX_STRING_CHARS`
        let mut i: isize = 0;
        loop {
            if *list.offset(i) == 0 {
                break;
            }
            let mut j: isize = 0;
            while *list.offset(i) != 0 && *list.offset(i) != b'|' as c_char {
                cmp[j as usize] = *list.offset(i);
                i += 1;
                j += 1;
            }
            cmp[j as usize] = 0;

            if q_shared::Q_stricmp(str, cmp.as_ptr()) == 0 {
                // found it
                return qtrue;
            }
            if *list.offset(i) != b'|' as c_char {
                // reached the end and never found it
                return qfalse;
            }
            i += 1;
        }
        qfalse
    }
}

/// Raven `multi_trigger`.
///
/// Source: `oracle/codemp/game/g_trigger.c:130-341`
pub fn multi_trigger(ctx: &mut GameContext, ent_id: EntityId, activator_id: Option<EntityId>) {
    let mut halt_trigger = false;

    if ctx.world.entity(ent_id).think.get() == Some(EntThink::multi_trigger_run) {
        // already triggered, just waiting to run
        return;
    }

    if ctx.world.cvars.g_gametype.integer == GT_SIEGE && ctx.world.globals.gSiegeRoundBegun == 0 {
        // nothing can be used til the round starts.
        return;
    }

    if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
        // FLAG: `.client` is a pool `gclient_t` for NPC activators.
        // Read the raw pointer value through the safe entity borrow and deref it in a tight `unsafe` block, the
        // same as Raven.
        let ac = match activator_id {
            Some(a) => ctx.world.entity(a).client,
            None => core::ptr::null_mut(),
        };
        if !ac.is_null()
            && ctx.world.entity(ent_id).alliedTeam != 0
            && unsafe { (*ac).sess.sessionTeam } != ctx.world.entity(ent_id).alliedTeam
        {
            // this team can't activate this trigger.
            return;
        }
    }

    let idealclass = ctx.world.entity(ent_id).idealclass.clone();
    if ctx.world.cvars.g_gametype.integer == GT_SIEGE && !idealclass.is_empty() {
        // only certain classes can activate it
        // FLAG: pool `gclient_t` deref (see above).
        let ac = match activator_id {
            Some(a) => ctx.world.entity(a).client,
            None => core::ptr::null_mut(),
        };
        if ac.is_null() || unsafe { (*ac).siegeClass } < 0 {
            // no class
            return;
        }

        let siege_class = unsafe { (*ac).siegeClass } as usize;
        let siege_class_name = cstr(&ctx.world.bg_state.bgSiegeClasses[siege_class].name);
        let idealclass_c = cstr(&idealclass);
        if G_NameInTriggerClassList(
            siege_class_name.as_ptr() as *mut c_char,
            idealclass_c.as_ptr() as *mut c_char,
        ) == 0
        {
            // wasn't in the list
            return;
        }
    }

    if ctx.world.cvars.g_gametype.integer == GT_SIEGE && ctx.world.entity(ent_id).genericValue1 != 0
    {
        halt_trigger = true;

        // FLAG: pool `gclient_t` deref (see above).
        let ac = match activator_id {
            Some(a) => ctx.world.entity(a).client,
            None => core::ptr::null_mut(),
        };
        let targetname = ctx.world.entity(ent_id).targetname_str();
        if !ac.is_null()
            && unsafe { (*ac).holdingObjectiveItem } != 0
            && targetname.as_deref().is_some_and(|s| !s.is_empty())
        {
            let obj_item = EntityId(unsafe { (*ac).holdingObjectiveItem } as u32);

            if ctx.world.entity(obj_item).inuse != 0 {
                let goaltarget = ctx.world.entity(obj_item).goaltarget.clone();
                let targetname = targetname.as_deref().unwrap();
                if !goaltarget.is_empty() && Q_stricmp(targetname, &goaltarget) == 0 {
                    let sess_team = unsafe { (*ac).sess.sessionTeam };
                    if ctx.world.entity(obj_item).genericValue7 != sess_team {
                        // The carrier of the item is not on the team which
                        // disallows objective scoring for it
                        let obj_target3 = ctx.world.entity(obj_item).target3.clone();
                        if !obj_target3.is_empty() {
                            // if it has a target3, fire it off instead of using the trigger
                            G_UseTargets2(ctx, Some(obj_item), Some(obj_item), Some(&obj_target3));

                            //3-24-03 - want to fire off the target too I guess, if we have one.
                            let tn = ctx.world.entity(ent_id).targetname_str();
                            if tn.as_deref().is_some_and(|s| !s.is_empty()) {
                                halt_trigger = false;
                            }
                        } else {
                            halt_trigger = false;
                        }

                        // now that the item has been delivered, it can go away.
                        // The g_saga port retargeted `SiegeItemRemoveOwner` to `(ctx, EntityId, Option<EntityId>)`.
                        // This file calls it directly.
                        crate::g_saga::SiegeItemRemoveOwner(ctx, obj_item, activator_id);
                        ctx.world.entity_mut(obj_item).nextthink = 0;
                        ctx.world.entity_mut(obj_item).neverFree = qfalse;
                        G_FreeEntity(ctx, Some(obj_item));
                    }
                }
            }
        }
    } else if ctx.world.entity(ent_id).genericValue1 != 0 {
        // Never activate in non-siege gametype I guess.
        return;
    }

    if ctx.world.entity(ent_id).genericValue2 != 0 {
        // has "teambalance" property
        let mut i: c_int = 0;
        let mut team1_cl_num: c_int = 0;
        let mut team2_cl_num: c_int = 0;
        let owning_team = ctx.world.entity(ent_id).genericValue3;
        let mut new_owning_team: c_int = 0;

        if ctx.world.cvars.g_gametype.integer != GT_SIEGE {
            return;
        }

        // §19: Raven derefs `activator->client` unguarded.
        // This file adds a null-activator guard.
        // FLAG: pool `gclient_t` deref (see above).
        let ac = match activator_id {
            Some(a) => ctx.world.entity(a).client,
            None => core::ptr::null_mut(),
        };
        if ac.is_null()
            || (unsafe { (*ac).sess.sessionTeam } != SIEGETEAM_TEAM1
                && unsafe { (*ac).sess.sessionTeam } != SIEGETEAM_TEAM2)
        {
            // activator must be a valid client to begin with
            return;
        }

        // Count up the number of clients standing within the bounds of the
        // trigger and the number of them on each team
        let mut entity_list = [0i32; mp_qshared::shared::MAX_GENTITIES];
        let absmin = ctx.world.entity(ent_id).r.absmin;
        let absmax = ctx.world.entity(ent_id).r.absmax;
        let num_ents = trap::EntitiesInBox(
            ctx.engine,
            GEntitiesInBoxArgs::new(
                &absmin as *const vec3_t,
                &absmax as *const vec3_t,
                entity_list.as_mut_ptr(),
                entity_list.len() as c_int,
            ),
        );
        while i < num_ents {
            if entity_list[i as usize] < MAX_CLIENTS as c_int {
                // only care about clients
                let cl = EntityId(entity_list[i as usize] as u32);
                // FLAG: pool `gclient_t` deref (see above).
                let clp = ctx.world.entity(cl).client;

                // the client is valid
                if ctx.world.entity(cl).inuse != 0
                    && !clp.is_null()
                    && (unsafe { (*clp).sess.sessionTeam } == SIEGETEAM_TEAM1
                        || unsafe { (*clp).sess.sessionTeam } == SIEGETEAM_TEAM2)
                    && ctx.world.entity(cl).health > 0
                    && (unsafe { (*clp).ps.eFlags } & EF_DEAD) == 0
                {
                    // See which team he's on
                    if unsafe { (*clp).sess.sessionTeam } == SIEGETEAM_TEAM1 {
                        team1_cl_num += 1;
                    } else {
                        team2_cl_num += 1;
                    }
                }
            }
            i += 1;
        }

        if team1_cl_num == 0 && team2_cl_num == 0 {
            // no one in the box? How did we get activated? Oh well.
            return;
        }

        if team1_cl_num == team2_cl_num {
            // if equal numbers the ownership will remain the same as it is now
            return;
        }

        // decide who owns it now
        if team1_cl_num > team2_cl_num {
            new_owning_team = SIEGETEAM_TEAM1;
        } else {
            new_owning_team = SIEGETEAM_TEAM2;
        }

        if owning_team == new_owning_team {
            // it's the same one it already was, don't care then.
            return;
        }

        // Set the new owner and set the variable which will tell us to
        // activate a team-specific target
        ctx.world.entity_mut(ent_id).genericValue3 = new_owning_team;
        ctx.world.entity_mut(ent_id).genericValue4 = new_owning_team;
    }

    if halt_trigger {
        // This is an objective trigger and the activator is not carrying an
        // objective item that matches the targetname.
        return;
    }

    if ctx.world.entity(ent_id).nextthink > ctx.world.level.time {
        if ctx.world.entity(ent_id).spawnflags & 2048 != 0 {
            // MULTIPLE - allow multiple entities to touch this trigger in a single frame
            if ctx.world.entity(ent_id).painDebounceTime != 0
                && ctx.world.entity(ent_id).painDebounceTime != ctx.world.level.time
            {
                // this should still allow subsequent ents to fire this trigger in the current frame
                return; // can't retrigger until the wait is over
            }
        } else {
            return;
        }
    }

    // if the player has already activated this trigger this frame
    if let Some(activator) = activator_id {
        if ctx.world.entity(activator).s.number == 0
            && ctx.world.entity(ent_id).aimDebounceTime == ctx.world.level.time
        {
            return;
        }
    }

    if ctx.world.entity(ent_id).flags & FL_INACTIVE != 0 {
        // Not active at this time
        return;
    }

    ctx.world.entity_mut(ent_id).activator = activator_id;

    if ctx.world.entity(ent_id).delay != 0
        && ctx.world.entity(ent_id).painDebounceTime
            < (ctx.world.level.time + ctx.world.entity(ent_id).delay)
    {
        // delay before firing trigger
        ctx.world.entity_mut(ent_id).think = Some(EntThink::multi_trigger_run).into();
        let nt = ctx.world.level.time + ctx.world.entity(ent_id).delay;
        ctx.world.entity_mut(ent_id).nextthink = nt;
        ctx.world.entity_mut(ent_id).painDebounceTime = ctx.world.level.time;
    } else {
        multi_trigger_run(ctx, ent_id);
    }
}

/// Raven `Use_Multi`.
///
/// Source: `oracle/codemp/game/g_trigger.c:343-346`
pub fn Use_Multi(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    multi_trigger(ctx, ent, activator);
}

/// Raven `Touch_Multi`.
///
/// Source: `oracle/codemp/game/g_trigger.c:350-547`
pub fn Touch_Multi(
    ctx: &mut GameContext,
    self_id: EntityId,
    other_id: Option<EntityId>,
    trace: *mut trace_t,
) {
    // `other` is the toucher.
    // Raven derefs it unconditionally.
    // A NULL toucher would crash Raven at `(*other).client` before the FL_INACTIVE check.
    // A `None` here maps to the same early exit.
    let other = match other_id {
        Some(o) => o,
        None => return,
    };
    // FLAG: `.client` is a pool `gclient_t` for NPC touchers.
    // Read the raw pointer value through the safe entity borrow and deref it in a tight `unsafe` block, the same as
    // Raven.
    let other_client = ctx.world.entity(other).client;
    if other_client.is_null() {
        return;
    }

    if ctx.world.entity(self_id).flags & FL_INACTIVE != 0 {
        // set by target_deactivate
        return;
    }

    if ctx.world.entity(self_id).alliedTeam != 0 {
        if unsafe { (*other_client).sess.sessionTeam } != ctx.world.entity(self_id).alliedTeam {
            return;
        }
    }

    // moved to just above multi_trigger because up here it just checks if
    // the trigger is not being touched we want it to check any conditions
    // set on the trigger, if one of those isn't met, the trigger is
    // considered to be "cleared"

    if ctx.world.entity(self_id).spawnflags & 1 != 0 {
        if ctx.world.entity(other).s.eType == ET_NPC as c_int {
            return;
        }
    } else {
        if ctx.world.entity(self_id).spawnflags & 16 != 0 {
            // NPCONLY
            if ctx.world.entity(other).NPC.is_null() {
                return;
            }
        }

        let npc_targetname = ctx.world.entity(self_id).NPC_targetname.clone();
        if !npc_targetname.is_empty() {
            let script_targetname = ctx.world.entity(other).script_targetname_str();
            if script_targetname.as_deref().is_some_and(|s| !s.is_empty()) {
                if Q_stricmp(&npc_targetname, script_targetname.as_deref().unwrap()) != 0 {
                    // not the right guy to fire me off
                    return;
                }
            } else {
                return;
            }
        }
    }

    if ctx.world.entity(self_id).spawnflags & 2 != 0 {
        // FACING
        let mut forward: vec3_t = [0.0; 3];
        AngleVectors(
            unsafe { (*other_client).ps.viewangles },
            Some(&mut forward),
            None,
            None,
        );

        let movedir = ctx.world.entity(self_id).movedir;
        let dot = movedir[0] * forward[0] + movedir[1] * forward[1] + movedir[2] * forward[2];
        if dot < 0.5 {
            // Not Within 45 degrees
            return;
        }
    }

    if ctx.world.entity(self_id).spawnflags & 4 != 0 {
        // USE_BUTTON
        if unsafe { (*other_client).pers.cmd.buttons } & BUTTON_USE == 0 {
            // not pressing use button
            return;
        }

        if (unsafe { (*other_client).ps.weaponTime } > 0
            && unsafe { (*other_client).ps.torsoAnim } != BOTH_BUTTON_HOLD as c_int
            && unsafe { (*other_client).ps.torsoAnim } != BOTH_CONSOLE1 as c_int)
            || ctx.world.entity(other).health < 1
            || (unsafe { (*other_client).ps.pm_flags } & PMF_FOLLOW) != 0
            || unsafe { (*other_client).sess.sessionTeam } == TEAM_SPECTATOR
            || unsafe { (*other_client).ps.forceHandExtend } != HANDEXTEND_NONE as c_int
        {
            // player has to be free of other things to use.
            return;
        }

        if ctx.world.entity(self_id).genericValue7 != 0 {
            // we have to be holding the use key in this trigger for x
            // milliseconds before firing
            let idealclass = ctx.world.entity(self_id).idealclass.clone();
            if ctx.world.cvars.g_gametype.integer == GT_SIEGE && !idealclass.is_empty() {
                // only certain classes can activate it
                if other_client.is_null() || unsafe { (*other_client).siegeClass } < 0 {
                    // no class
                    return;
                }

                let siege_class = unsafe { (*other_client).siegeClass } as usize;
                let siege_class_name = cstr(&ctx.world.bg_state.bgSiegeClasses[siege_class].name);
                let idealclass_c = cstr(&idealclass);
                if G_NameInTriggerClassList(
                    siege_class_name.as_ptr() as *mut c_char,
                    idealclass_c.as_ptr() as *mut c_char,
                ) == 0
                {
                    // wasn't in the list
                    return;
                }
            }

            let origin = unsafe { (*other_client).ps.origin };
            let absmin = ctx.world.entity(self_id).r.absmin;
            let absmax = ctx.world.entity(self_id).r.absmax;
            if G_PointInBounds(origin, absmin, absmax) == 0 {
                return;
            } else if unsafe { (*other_client).isHacking } != ctx.world.entity(self_id).s.number
                && ctx.world.entity(other).s.number < MAX_CLIENTS as c_int
            {
                // start the hack
                let self_number = ctx.world.entity(self_id).s.number;
                let gv7 = ctx.world.entity(self_id).genericValue7;
                let level_time = ctx.world.level.time;
                unsafe {
                    (*other_client).isHacking = self_number;
                    (*other_client).hackingAngles = (*other_client).ps.viewangles;
                    (*other_client).ps.hackingTime = level_time + gv7;
                    (*other_client).ps.hackingBaseTime = gv7;
                    if (*other_client).ps.hackingBaseTime > 60000 {
                        // don't allow a bit overflow
                        (*other_client).ps.hackingTime = level_time + 60000;
                        (*other_client).ps.hackingBaseTime = 60000;
                    }
                }
                return;
            } else if unsafe { (*other_client).ps.hackingTime } < ctx.world.level.time {
                // finished with the hack, reset the hacking values and let
                // it fall through
                unsafe {
                    (*other_client).isHacking = 0; // can't hack a client
                    (*other_client).ps.hackingTime = 0;
                }
            } else {
                // hack in progress
                return;
            }
        }
    }

    if ctx.world.entity(self_id).spawnflags & 8 != 0 {
        // FIRE_BUTTON
        if (unsafe { (*other_client).pers.cmd.buttons } & BUTTON_ATTACK) == 0
            && (unsafe { (*other_client).pers.cmd.buttons } & BUTTON_ALT_ATTACK) == 0
        {
            // not pressing fire button or altfire button
            return;
        }
    }

    if ctx.world.entity(self_id).radius != 0.0 {
        // Only works if your head is in it, but we allow leaning out
        // NOTE: We don't use CalcEntitySpot SPOT_HEAD because we don't want this
        // to be reliant on the physical model the player uses.
        let mut eye_spot: vec3_t = unsafe { (*other_client).ps.origin };
        eye_spot[2] += unsafe { (*other_client).ps.viewheight } as f32;

        let absmin = ctx.world.entity(self_id).r.absmin;
        let absmax = ctx.world.entity(self_id).r.absmax;
        if G_PointInBounds(eye_spot, absmin, absmax) != 0 {
            if (unsafe { (*other_client).pers.cmd.buttons } & BUTTON_ATTACK) == 0
                && (unsafe { (*other_client).pers.cmd.buttons } & BUTTON_ALT_ATTACK) == 0
            {
                // not attacking, so hiding bonus
                // Not using this, at least not yet.
                // The oracle keeps a longer commented-out FIXME block here.
                // This file does not transcribe it.
            }
        }
    }

    if ctx.world.entity(self_id).spawnflags & 4 != 0 {
        // USE_BUTTON
        if unsafe { (*other_client).ps.torsoAnim } != BOTH_BUTTON_HOLD as c_int
            && unsafe { (*other_client).ps.torsoAnim } != BOTH_CONSOLE1 as c_int
        {
            G_SetAnim(
                ctx,
                other,
                core::ptr::null_mut(),
                SETANIM_TORSO as c_int,
                BOTH_BUTTON_HOLD as c_int,
                SETANIM_FLAG_OVERRIDE as c_int | SETANIM_FLAG_HOLD as c_int,
                0,
            );
        } else {
            unsafe {
                (*other_client).ps.torsoTimer = 500;
            }
        }
        unsafe {
            (*other_client).ps.weaponTime = (*other_client).ps.torsoTimer;
        }
    }

    if ctx.world.entity(self_id).think.get() == Some(EntThink::trigger_cleared_fire) {
        // We're waiting to fire our target2 first
        let nt = ctx.world.level.time + ctx.world.entity(self_id).speed as c_int;
        ctx.world.entity_mut(self_id).nextthink = nt;
        return;
    }

    multi_trigger(ctx, self_id, other_id);
}

/// Raven `trigger_cleared_fire`.
///
/// Source: `oracle/codemp/game/g_trigger.c:549-558`
pub fn trigger_cleared_fire(ctx: &mut GameContext, self_: EntityId) {
    let activator = ctx.world.entity(self_).activator;
    let target2 = ctx.world.entity(self_).target2.clone();
    G_UseTargets2(ctx, Some(self_), activator, target2.as_deref());
    ctx.world.entity_mut(self_).think = FnId::NONE;
    // should start the wait timer now, because the trigger's just been
    // cleared, so we must "wait" from this point
    if ctx.world.entity(self_).wait > 0.0 {
        let w = ctx.world.entity(self_).wait as f64;
        let r = ctx.world.entity(self_).random as f64;
        let nt = (ctx.world.level.time as f64 + (w + r * ctx.world.bg_state.rng.crandom()) * 1000.0)
            as c_int;
        ctx.world.entity_mut(self_).nextthink = nt;
    }
}

/// Raven `SP_trigger_multiple`.
///
/// Source: `oracle/codemp/game/g_trigger.c:607-656`
pub fn SP_trigger_multiple(ctx: &mut GameContext, ent_id: EntityId) {
    let (present, s) = G_SpawnString(ctx, "noise", "");
    if present != 0 {
        if !s.is_empty() {
            ctx.world.entity_mut(ent_id).noise_index = G_SoundIndex(ctx, &s);
        } else {
            ctx.world.entity_mut(ent_id).noise_index = 0;
        }
    }

    let mut gv7: c_int = 0;
    G_SpawnInt(ctx, c"usetime".as_ptr(), c"0".as_ptr(), &mut gv7);
    ctx.world.entity_mut(ent_id).genericValue7 = gv7;

    // For siege gametype
    let mut gv1: c_int = 0;
    G_SpawnInt(ctx, c"siegetrig".as_ptr(), c"0".as_ptr(), &mut gv1);
    ctx.world.entity_mut(ent_id).genericValue1 = gv1;
    let mut gv2: c_int = 0;
    G_SpawnInt(ctx, c"teambalance".as_ptr(), c"0".as_ptr(), &mut gv2);
    ctx.world.entity_mut(ent_id).genericValue2 = gv2;

    let mut delay: c_int = 0;
    G_SpawnInt(ctx, c"delay".as_ptr(), c"0".as_ptr(), &mut delay);
    ctx.world.entity_mut(ent_id).delay = delay;

    if ctx.world.entity(ent_id).wait > 0.0
        && ctx.world.entity(ent_id).random >= ctx.world.entity(ent_id).wait
    {
        let w = ctx.world.entity(ent_id).wait;
        ctx.world.entity_mut(ent_id).random = w - FRAMETIME as f32;
        G_Printf(ctx, "^3trigger_multiple has random >= wait\n");
    }

    ctx.world.entity_mut(ent_id).delay *= 1000; // 1 = 1 msec, 1000 = 1 sec
    let target2 = ctx.world.entity(ent_id).target2.clone();
    if ctx.world.entity(ent_id).speed == 0.0 && target2.as_deref().is_some_and(|s| !s.is_empty()) {
        ctx.world.entity_mut(ent_id).speed = 1000.0;
    } else {
        ctx.world.entity_mut(ent_id).speed *= 1000.0;
    }

    ctx.world.entity_mut(ent_id).touch = Some(EntTouch::Touch_Multi).into();
    ctx.world.entity_mut(ent_id).use_ = Some(EntUse::Use_Multi).into();

    let team = ctx.world.entity(ent_id).team.clone();
    if team.as_deref().is_some_and(|s| !s.is_empty()) {
        let team = team.as_deref().unwrap();
        ctx.world.entity_mut(ent_id).alliedTeam = atoi_bytes(team.as_bytes());
        ctx.world.entity_mut(ent_id).team = None;
    }

    InitTrigger(ctx, ent_id);
    let ent_ptr = ctx.world.entity_mut(ent_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));
}

/// Raven `SP_trigger_once`.
///
/// Source: `oracle/codemp/game/g_trigger.c:694-731`
pub fn SP_trigger_once(ctx: &mut GameContext, ent_id: EntityId) {
    let (present, s) = G_SpawnString(ctx, "noise", "");
    if present != 0 {
        if !s.is_empty() {
            ctx.world.entity_mut(ent_id).noise_index = G_SoundIndex(ctx, &s);
        } else {
            ctx.world.entity_mut(ent_id).noise_index = 0;
        }
    }

    let mut gv7: c_int = 0;
    G_SpawnInt(ctx, c"usetime".as_ptr(), c"0".as_ptr(), &mut gv7);
    ctx.world.entity_mut(ent_id).genericValue7 = gv7;

    // For siege gametype
    let mut gv1: c_int = 0;
    G_SpawnInt(ctx, c"siegetrig".as_ptr(), c"0".as_ptr(), &mut gv1);
    ctx.world.entity_mut(ent_id).genericValue1 = gv1;

    let mut delay: c_int = 0;
    G_SpawnInt(ctx, c"delay".as_ptr(), c"0".as_ptr(), &mut delay);
    ctx.world.entity_mut(ent_id).delay = delay;

    ctx.world.entity_mut(ent_id).wait = -1.0;

    ctx.world.entity_mut(ent_id).touch = Some(EntTouch::Touch_Multi).into();
    ctx.world.entity_mut(ent_id).use_ = Some(EntUse::Use_Multi).into();

    let team = ctx.world.entity(ent_id).team.clone();
    if team.as_deref().is_some_and(|s| !s.is_empty()) {
        let team = team.as_deref().unwrap();
        ctx.world.entity_mut(ent_id).alliedTeam = atoi_bytes(team.as_bytes());
        ctx.world.entity_mut(ent_id).team = None;
    }

    ctx.world.entity_mut(ent_id).delay *= 1000; // 1 = 1 msec, 1000 = 1 sec

    InitTrigger(ctx, ent_id);
    let ent_ptr = ctx.world.entity_mut(ent_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));
}

/// Raven `Do_Strike`.
///
/// lightning strike trigger lightning strike event
/// Source: `oracle/codemp/game/g_trigger.c:739-786`
pub fn Do_Strike(ctx: &mut GameContext, ent: EntityId) {
    // maybe allow custom fx direction at some point?
    let fx_ang: vec3_t = [90.0, 0.0, 0.0];

    // choose a random point to strike within the bounds of the trigger
    let mut strike_point: vec3_t = [0.0; 3];
    let amin0 = ctx.world.entity(ent).r.absmin[0];
    let amax0 = ctx.world.entity(ent).r.absmax[0];
    strike_point[0] = ctx.world.bg_state.rng.flrand(amin0, amax0);
    let amin1 = ctx.world.entity(ent).r.absmin[1];
    let amax1 = ctx.world.entity(ent).r.absmax[1];
    strike_point[1] = ctx.world.bg_state.rng.flrand(amin1, amax1);
    // consider the bottom mins the ground level
    strike_point[2] = ctx.world.entity(ent).r.absmin[2];

    // set the from point
    let mut strike_from: vec3_t = [
        strike_point[0],
        strike_point[1],
        ctx.world.entity(ent).r.absmax[2] - 4.0,
    ];

    // now trace for damaging stuff, and do the effect
    // Raven passes `NULL` mins/maxs for a point trace.
    // `zero` stands in, because the resolved `GTraceArgs` takes `*const vec3_t`, not an optional.
    let zero: vec3_t = [0.0; 3];
    let mut local_trace: trace_t = unsafe { core::mem::zeroed() };
    let ent_number = ctx.world.entity(ent).s.number;
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut local_trace as *mut trace_t,
            &strike_from as *const vec3_t,
            &zero as *const vec3_t,
            &zero as *const vec3_t,
            &strike_point as *const vec3_t,
            ent_number,
            MASK_PLAYERSOLID,
        ),
    );
    strike_point = local_trace.endpos;

    if local_trace.startsolid != 0 || local_trace.allsolid != 0 {
        // got a bad spot, think again next frame to try another strike
        ctx.world.entity_mut(ent).nextthink = ctx.world.level.time;
        return;
    }

    if ctx.world.entity(ent).radius != 0.0 {
        // do a radius damage at the end pos
        let damage = ctx.world.entity(ent).damage as f32;
        let radius = ctx.world.entity(ent).radius;
        G_RadiusDamage(
            ctx,
            strike_point,
            Some(ent),
            damage,
            radius,
            Some(ent),
            None,
            MOD_SUICIDE as c_int,
        );
    } else {
        // only damage individuals
        let tr_hit = EntityId(local_trace.entityNum as u32);

        if ctx.world.entity(tr_hit).inuse != 0 && ctx.world.entity(tr_hit).takedamage != 0 {
            // damage it then
            let current_origin = ctx.world.entity(tr_hit).r.currentOrigin;
            let damage = ctx.world.entity(ent).damage;
            G_Damage(
                ctx,
                Some(tr_hit),
                Some(ent),
                Some(ent),
                None,
                current_origin,
                damage,
                0,
                MOD_SUICIDE as c_int,
            );
        }
    }

    let gv2 = ctx.world.entity(ent).genericValue2;
    G_PlayEffectID(gv2, strike_from, fx_ang);
}

/// Raven `Think_Strike`.
///
/// lightning strike trigger think loop
/// Source: `oracle/codemp/game/g_trigger.c:789-798`
pub fn Think_Strike(ctx: &mut GameContext, ent_id: EntityId) {
    if ctx.world.entity(ent_id).genericValue1 != 0 {
        // turned off currently
        return;
    }

    let wait = ctx.world.entity(ent_id).wait as c_int;
    let random = ctx.world.entity(ent_id).random as c_int;
    let nt = ctx.world.level.time + wait + ctx.world.bg_state.rng.Q_irand(0, random);
    ctx.world.entity_mut(ent_id).nextthink = nt;
    Do_Strike(ctx, ent_id);
}

/// Raven `Use_Strike`.
///
/// lightning strike trigger use event function
/// Source: `oracle/codemp/game/g_trigger.c:801-809`
pub fn Use_Strike(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let gv1 = (ctx.world.entity(ent).genericValue1 == 0) as c_int;
    ctx.world.entity_mut(ent).genericValue1 = gv1;

    if ctx.world.entity(ent).genericValue1 == 0 {
        // turn it back on
        ctx.world.entity_mut(ent).nextthink = ctx.world.level.time;
    }
}

/// Raven `SP_trigger_lightningstrike`.
///
/// Source: `oracle/codemp/game/g_trigger.c:824-861`
pub fn SP_trigger_lightningstrike(ctx: &mut GameContext, ent_id: EntityId) {
    ctx.world.entity_mut(ent_id).use_ = Some(EntUse::Use_Strike).into();
    ctx.world.entity_mut(ent_id).think = Some(EntThink::Think_Strike).into();
    ctx.world.entity_mut(ent_id).nextthink = ctx.world.level.time + 500;

    let (_, s) = G_SpawnString(ctx, "lightningfx", "");
    if s.is_empty() {
        // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
        panic!("trigger_lightningstrike with no lightningfx");
    }

    // get a configstring index for it
    ctx.world.entity_mut(ent_id).genericValue2 = G_EffectIndex(ctx, &s);

    if ctx.world.entity(ent_id).spawnflags & 1 != 0 {
        // START_OFF
        ctx.world.entity_mut(ent_id).genericValue1 = 1;
    }

    if ctx.world.entity(ent_id).wait == 0.0 {
        // default 1000
        ctx.world.entity_mut(ent_id).wait = 1000.0;
    }
    if ctx.world.entity(ent_id).random == 0.0 {
        // default 2000
        ctx.world.entity_mut(ent_id).random = 2000.0;
    }
    if ctx.world.entity(ent_id).damage == 0 {
        // default 50
        ctx.world.entity_mut(ent_id).damage = 50;
    }

    InitTrigger(ctx, ent_id);
    let ent_ptr = ctx.world.entity_mut(ent_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));
}

/// Raven `trigger_always_think`.
///
/// Source: `oracle/codemp/game/g_trigger.c:872-875`
pub fn trigger_always_think(ctx: &mut GameContext, ent: EntityId) {
    G_UseTargets(ctx, Some(ent), Some(ent));
    G_FreeEntity(ctx, Some(ent));
}

/// Raven `SP_trigger_always`.
///
/// This trigger will always fire.  It is activated by the world.
/// Source: `oracle/codemp/game/g_trigger.c:880-884`
pub fn SP_trigger_always(ctx: &mut GameContext, ent: EntityId) {
    // we must have some delay to make sure our use targets are present
    ctx.world.entity_mut(ent).nextthink = ctx.world.level.time + 300;
    ctx.world.entity_mut(ent).think = Some(EntThink::trigger_always_think).into();
}

/// Raven `trigger_push_touch`.
///
/// Source: `oracle/codemp/game/g_trigger.c:901-1029`
pub fn trigger_push_touch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    // `other` is the toucher.
    // Raven derefs it unconditionally.
    let other = match other {
        Some(o) => o,
        None => return,
    };

    if ctx.world.entity(self_).flags & FL_INACTIVE != 0 {
        // set by target_deactivate
        return;
    }

    if ctx.world.entity(self_).spawnflags & PUSH_LINEAR == 0 {
        // normal throw
        // FLAG: `.client` is a pool `gclient_t` for NPC touchers.
        // Read the raw pointer value through the safe entity borrow and deref it in a tight `unsafe` block, the same
        // as Raven.
        let other_client = ctx.world.entity(other).client;
        if other_client.is_null() {
            return;
        }
        let self_s = &raw mut ctx.world.entity_mut(self_).s;
        unsafe {
            BG_TouchJumpPad(&raw mut (*other_client).ps, self_s);
        }
        return;
    }

    // linear
    // Raven compares in float: `level.time < painDebounceTime + self->wait`.
    // `wait` is a float, so C promotes both sides to float instead of truncating `wait` to an int.
    if (ctx.world.level.time as f32)
        < ctx.world.entity(self_).painDebounceTime as f32 + ctx.world.entity(self_).wait
    {
        // normal 'wait' check
        if ctx.world.entity(self_).spawnflags & PUSH_MULTIPLE != 0 {
            // MULTIPLE - allow multiple entities to touch this trigger in one frame
            if ctx.world.entity(self_).painDebounceTime != 0
                && ctx.world.level.time > ctx.world.entity(self_).painDebounceTime
            {
                // if we haven't reached the next frame continue to let ents touch the trigger
                return;
            }
        } else {
            // only allowing one ent per frame to touch trigger
            return;
        }
    }

    // FLAG: pool `gclient_t` deref (see above).
    let other_client = ctx.world.entity(other).client;
    if other_client.is_null() {
        if ctx.world.entity(other).s.pos.trType != TR_STATIONARY
            && ctx.world.entity(other).s.pos.trType != TR_LINEAR_STOP
            && ctx.world.entity(other).s.pos.trType != TR_NONLINEAR_STOP
            && VectorLengthSquared(ctx.world.entity(other).s.pos.trDelta) != 0.0
        {
            // already moving
            let current_origin = ctx.world.entity(other).r.currentOrigin;
            ctx.world.entity_mut(other).s.pos.trBase = current_origin;
            let origin2 = ctx.world.entity(self_).s.origin2;
            ctx.world.entity_mut(other).s.pos.trDelta = origin2;
            ctx.world.entity_mut(other).s.pos.trTime = ctx.world.level.time;
        }
        return;
    }

    if unsafe { (*other_client).ps.pm_type } != PM_NORMAL as c_int
        && unsafe { (*other_client).ps.pm_type } != PM_DEAD as c_int
        && unsafe { (*other_client).ps.pm_type } != PM_FREEZE as c_int
    {
        return;
    }

    if ctx.world.entity(self_).spawnflags & PUSH_RELATIVE != 0 {
        // relative, dir to it * speed
        let origin2 = ctx.world.entity(self_).s.origin2;
        let current_origin = ctx.world.entity(other).r.currentOrigin;
        let mut dir: vec3_t = [
            origin2[0] - current_origin[0],
            origin2[1] - current_origin[1],
            origin2[2] - current_origin[2],
        ];
        if ctx.world.entity(self_).speed != 0.0 {
            VectorNormalize(&mut dir);
            let speed = ctx.world.entity(self_).speed;
            dir = [dir[0] * speed, dir[1] * speed, dir[2] * speed];
        }
        unsafe {
            (*other_client).ps.velocity = dir;
        }
    } else if ctx.world.entity(self_).spawnflags & PUSH_LINEAR != 0 {
        // linear dir * speed
        let origin2 = ctx.world.entity(self_).s.origin2;
        let speed = ctx.world.entity(self_).speed;
        unsafe {
            (*other_client).ps.velocity =
                [origin2[0] * speed, origin2[1] * speed, origin2[2] * speed];
        }
    } else {
        let origin2 = ctx.world.entity(self_).s.origin2;
        unsafe {
            (*other_client).ps.velocity = origin2;
        }
    }
    // so we don't take damage unless we land lower than we start here...
    // The oracle keeps `forceJumpZStart`, `PMF_TRIGGER_PUSHED`, and `jumpZStart` commented out there too.

    if ctx.world.entity(self_).wait == -1.0 {
        ctx.world.entity_mut(self_).touch = FnId::NONE;
    } else if ctx.world.entity(self_).wait > 0.0 {
        ctx.world.entity_mut(self_).painDebounceTime = ctx.world.level.time;
    }
    // Raven keeps the `aimDebounceTime` mark commented out too, as dead code in the oracle source.
}

/// Raven `AimAtTarget`.
///
/// Calculate origin2 so the target apogee will be hit
/// Source: `oracle/codemp/game/g_trigger.c:1039-1097`
pub fn AimAtTarget(ctx: &mut GameContext, self_: EntityId) {
    let absmin = ctx.world.entity(self_).r.absmin;
    let absmax = ctx.world.entity(self_).r.absmax;
    let mut origin: vec3_t = [
        absmin[0] + absmax[0],
        absmin[1] + absmax[1],
        absmin[2] + absmax[2],
    ];
    origin = [origin[0] * 0.5, origin[1] * 0.5, origin[2] * 0.5];

    let target = ctx.world.entity(self_).target.clone();
    let ent = G_PickTarget(ctx, target.as_deref());
    if ent.is_null() {
        G_FreeEntity(ctx, Some(self_));
        return;
    }
    let ent_id = ctx.entity_id_of(ent).unwrap();

    let classname = ctx.world.entity(self_).classname_str();
    if Q_stricmp("trigger_push", &classname) == 0 {
        if ctx.world.entity(self_).spawnflags & PUSH_RELATIVE != 0 {
            // relative, not an arc or linear
            let co = ctx.world.entity(ent_id).r.currentOrigin;
            ctx.world.entity_mut(self_).s.origin2 = co;
            return;
        } else if ctx.world.entity(self_).spawnflags & PUSH_LINEAR != 0 {
            // linear, not an arc
            let co = ctx.world.entity(ent_id).r.currentOrigin;
            ctx.world.entity_mut(self_).s.origin2 =
                [co[0] - origin[0], co[1] - origin[1], co[2] - origin[2]];
            VectorNormalize(&mut ctx.world.entity_mut(self_).s.origin2);
            return;
        }
    }

    let classname = ctx.world.entity(self_).classname_str();
    if Q_stricmp("target_push", &classname) == 0 {
        if ctx.world.entity(self_).spawnflags & PUSH_CONSTANT != 0 {
            let eo = ctx.world.entity(ent_id).s.origin;
            let so = ctx.world.entity(self_).s.origin;
            ctx.world.entity_mut(self_).s.origin2 = [eo[0] - so[0], eo[1] - so[1], eo[2] - so[2]];
            VectorNormalize(&mut ctx.world.entity_mut(self_).s.origin2);
            let o2 = ctx.world.entity(self_).s.origin2;
            let speed = ctx.world.entity(self_).speed;
            ctx.world.entity_mut(self_).s.origin2 = [o2[0] * speed, o2[1] * speed, o2[2] * speed];
            return;
        }
    }

    let height = ctx.world.entity(ent_id).s.origin[2] - origin[2];
    let gravity = ctx.world.cvars.g_gravity.value;
    // Raven: `sqrt( height / ( .5 * gravity ) )`.
    // `.5` is a double literal, so the divide promotes to double, and `sqrt` runs as the double libm call.
    // The result narrows back to float.
    let time = ((height as f64) / (0.5 * gravity as f64)).sqrt() as f32;
    if time == 0.0 {
        G_FreeEntity(ctx, Some(self_));
        return;
    }

    // set s.origin2 to the push velocity
    let eo = ctx.world.entity(ent_id).s.origin;
    ctx.world.entity_mut(self_).s.origin2 =
        [eo[0] - origin[0], eo[1] - origin[1], eo[2] - origin[2]];
    ctx.world.entity_mut(self_).s.origin2[2] = 0.0;
    let dist = VectorNormalize(&mut ctx.world.entity_mut(self_).s.origin2);

    let forward = dist / time;
    let o2 = ctx.world.entity(self_).s.origin2;
    ctx.world.entity_mut(self_).s.origin2 = [o2[0] * forward, o2[1] * forward, o2[2] * forward];

    ctx.world.entity_mut(self_).s.origin2[2] = time * gravity;
}

/// Raven `SP_trigger_push`.
///
/// Must point at a target_position, which will be the apex of the leap.
/// This will be client side predicted, unlike target_push
/// Source: `oracle/codemp/game/g_trigger.c:1112-1136`
pub fn SP_trigger_push(ctx: &mut GameContext, self_id: EntityId) {
    InitTrigger(ctx, self_id);

    // unlike other triggers, we need to send this one to the client
    ctx.world.entity_mut(self_id).r.svFlags &= !SVF_NOCLIENT;

    // make sure the client precaches this sound
    G_SoundIndex(ctx, "sound/weapons/force/jump.wav");

    ctx.world.entity_mut(self_id).s.eType = ET_PUSH_TRIGGER as c_int;

    if ctx.world.entity(self_id).spawnflags & 2 == 0 {
        // start on
        ctx.world.entity_mut(self_id).touch = Some(EntTouch::trigger_push_touch).into();
    }

    if ctx.world.entity(self_id).spawnflags & 4 != 0 {
        // linear
        ctx.world.entity_mut(self_id).speed = 1000.0;
    }

    ctx.world.entity_mut(self_id).think = Some(EntThink::AimAtTarget).into();
    ctx.world.entity_mut(self_id).nextthink = ctx.world.level.time + FRAMETIME;
    let self_ptr = ctx.world.entity_mut(self_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_ptr.cast()));
}

/// Raven `Use_target_push`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1138-1159`
pub fn Use_target_push(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // `activator` is the pusher.
    // Raven derefs it unconditionally.
    let activator = match activator {
        Some(a) => a,
        None => return,
    };
    // FLAG: `.client` is a pool `gclient_t` for NPC activators.
    // Read the raw pointer value through the safe entity borrow and deref it in a tight `unsafe` block, the same as
    // Raven.
    let client = ctx.world.entity(activator).client;
    if client.is_null() {
        return;
    }

    if unsafe { (*client).ps.pm_type } != PM_NORMAL as c_int
        && unsafe { (*client).ps.pm_type } != PM_FLOAT as c_int
    {
        return;
    }

    G_ActivateBehavior(ctx, Some(self_), bSet_t::BSET_USE as c_int);

    let origin2 = ctx.world.entity(self_).s.origin2;
    unsafe {
        (*client).ps.velocity = origin2;
    }

    // play fly sound every 1.5 seconds
    if ctx.world.entity(activator).fly_sound_debounce_time < ctx.world.level.time {
        ctx.world.entity_mut(activator).fly_sound_debounce_time = ctx.world.level.time + 1500;
        if ctx.world.entity(self_).noise_index != 0 {
            let ni = ctx.world.entity(self_).noise_index;
            G_Sound(ctx, Some(activator), CHAN_AUTO, ni);
        }
    }
}

/// Raven `SP_target_push`.
///
/// CONSTANT will push activator in direction of 'target' at constant 'speed'
/// Source: `oracle/codemp/game/g_trigger.c:1168-1187`
pub fn SP_target_push(ctx: &mut GameContext, self_: EntityId) {
    if ctx.world.entity(self_).speed == 0.0 {
        ctx.world.entity_mut(self_).speed = 1000.0;
    }
    {
        let e = ctx.world.entity_mut(self_);
        G_SetMovedir(&mut e.s.angles, &mut e.s.origin2);
    }
    let o2 = ctx.world.entity(self_).s.origin2;
    let speed = ctx.world.entity(self_).speed;
    ctx.world.entity_mut(self_).s.origin2 = [o2[0] * speed, o2[1] * speed, o2[2] * speed];

    if ctx.world.entity(self_).spawnflags & 1 != 0 {
        ctx.world.entity_mut(self_).noise_index =
            G_SoundIndex(ctx, "sound/weapons/force/jump.wav");
    } else {
        // G_SoundIndex("sound/misc/windfly.wav");
        ctx.world.entity_mut(self_).noise_index = 0;
    }
    if ctx.world.entity(self_).target.is_some() {
        let origin = ctx.world.entity(self_).s.origin;
        ctx.world.entity_mut(self_).r.absmin = origin;
        ctx.world.entity_mut(self_).r.absmax = origin;
        ctx.world.entity_mut(self_).think = Some(EntThink::AimAtTarget).into();
        ctx.world.entity_mut(self_).nextthink = ctx.world.level.time + FRAMETIME;
    }
    ctx.world.entity_mut(self_).use_ = Some(EntUse::Use_target_push).into();
}

/// Raven `trigger_teleporter_touch`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1197-1225`
pub fn trigger_teleporter_touch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    if ctx.world.entity(self_).flags & FL_INACTIVE != 0 {
        // set by target_deactivate
        return;
    }

    let other = match other {
        Some(o) => o,
        None => return,
    };
    // FLAG: `.client` is a pool `gclient_t` for NPC touchers.
    // Read the raw pointer value through the safe entity borrow and deref it in a tight `unsafe` block, the same as
    // Raven.
    let other_client = ctx.world.entity(other).client;
    if other_client.is_null() {
        return;
    }
    if unsafe { (*other_client).ps.pm_type } == pmtype_t::PM_DEAD as c_int {
        return;
    }
    // Spectators only?
    if ctx.world.entity(self_).spawnflags & 1 != 0
        && unsafe { (*other_client).sess.sessionTeam } != TEAM_SPECTATOR
    {
        return;
    }

    let target = ctx.world.entity(self_).target.clone();
    let dest = G_PickTarget(ctx, target.as_deref());
    if dest.is_null() {
        G_Printf(ctx, "Couldn't find teleporter destination\n");
        return;
    }
    let dest_id = ctx.entity_id_of(dest).unwrap();

    let origin = ctx.world.entity(dest_id).s.origin;
    let angles = ctx.world.entity(dest_id).s.angles;
    TeleportPlayer(ctx, other, origin, angles);
}

/// Raven `SP_trigger_teleport`.
///
/// Allows client side prediction of teleportation events.
/// Source: `oracle/codemp/game/g_trigger.c:1236-1254`
pub fn SP_trigger_teleport(ctx: &mut GameContext, self_id: EntityId) {
    InitTrigger(ctx, self_id);

    // unlike other triggers, we need to send this one to the client
    // unless is a spectator trigger
    if ctx.world.entity(self_id).spawnflags & 1 != 0 {
        ctx.world.entity_mut(self_id).r.svFlags |= SVF_NOCLIENT;
    } else {
        ctx.world.entity_mut(self_id).r.svFlags &= !SVF_NOCLIENT;
    }

    // make sure the client precaches this sound
    G_SoundIndex(ctx, "sound/weapons/force/speed.wav");

    ctx.world.entity_mut(self_id).s.eType = ET_TELEPORT_TRIGGER as c_int;
    ctx.world.entity_mut(self_id).touch = Some(EntTouch::trigger_teleporter_touch).into();

    let self_ptr = ctx.world.entity_mut(self_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_ptr.cast()));
}

/// Raven `hurt_use`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1280-1297`
pub fn hurt_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let activator_ok = match activator {
        Some(a) => {
            // FLAG: pool `gclient_t` deref.
            // Read the raw pointer through the safe entity borrow, the same as Raven.
            let c = ctx.world.entity(a).client;
            ctx.world.entity(a).inuse != 0 && !c.is_null()
        }
        None => false,
    };
    if activator_ok {
        ctx.world.entity_mut(self_).activator = activator;
    } else {
        ctx.world.entity_mut(self_).activator = None;
    }

    G_ActivateBehavior(ctx, Some(self_), bSet_t::BSET_USE as c_int);

    if ctx.world.entity(self_).r.linked != 0 {
        let self_ptr = ctx.world.entity_mut(self_) as *mut gentity_t;
        trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(self_ptr.cast()));
    } else {
        let self_ptr = ctx.world.entity_mut(self_) as *mut gentity_t;
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_ptr.cast()));
    }
}

/// Raven `hurt_touch`.
///
/// Any entity that touches this will be hurt.
/// Source: `oracle/codemp/game/g_trigger.c:1299-1411`
pub fn hurt_touch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    // `other` is the toucher.
    // Raven derefs it unconditionally.
    let other = match other {
        Some(o) => o,
        None => return,
    };

    let team_str = ctx.world.entity(self_).team.clone();
    if ctx.world.cvars.g_gametype.integer == GT_SIEGE
        && team_str.as_deref().is_some_and(|s| !s.is_empty())
    {
        let team = atoi_bytes(team_str.as_deref().unwrap().as_bytes());
        // FLAG: pool `gclient_t` deref.
        // Read the raw pointer through the safe entity borrow, the same as Raven.
        let oc = ctx.world.entity(other).client;

        if ctx.world.entity(other).inuse != 0
            && ctx.world.entity(other).s.number < MAX_CLIENTS as c_int
            && !oc.is_null()
            && unsafe { (*oc).sess.sessionTeam } != team
        {
            // real client don't hurt
            return;
        } else if ctx.world.entity(other).inuse != 0
            && !oc.is_null()
            && ctx.world.entity(other).s.eType == ET_NPC as c_int
            && ctx.world.entity(other).s.NPC_class == CLASS_VEHICLE as c_int
            && ctx.world.entity(other).s.teamowner != team
        {
            // vehicle owned by team don't hurt
            return;
        }
    }

    if ctx.world.entity(self_).flags & FL_INACTIVE != 0 {
        // set by target_deactivate
        return;
    }

    if ctx.world.entity(other).takedamage == 0 {
        return;
    }

    if ctx.world.entity(self_).timestamp > ctx.world.level.time {
        return;
    }

    // FLAG: pool `gclient_t` deref (see above).
    let other_client = ctx.world.entity(other).client;

    if ctx.world.entity(self_).damage == -1
        && !other_client.is_null()
        && ctx.world.entity(other).health < 1
    {
        unsafe {
            (*other_client).ps.fallingToDeath = 0;
        }
        respawn(ctx, other);
        return;
    }

    if ctx.world.entity(self_).damage == -1
        && !other_client.is_null()
        && unsafe { (*other_client).ps.fallingToDeath } != 0
    {
        return;
    }

    if ctx.world.entity(self_).spawnflags & 16 != 0 {
        ctx.world.entity_mut(self_).timestamp = ctx.world.level.time + 1000;
    } else {
        ctx.world.entity_mut(self_).timestamp = ctx.world.level.time + FRAMETIME;
    }

    // play sound
    // Raven keeps the `G_Sound`-on-touch block commented out too, as dead code in the oracle source.

    let dflags = if ctx.world.entity(self_).spawnflags & 8 != 0 {
        DAMAGE_NO_PROTECTION
    } else {
        0
    };

    if ctx.world.entity(self_).damage == -1 && !other_client.is_null() {
        let level_time = ctx.world.level.time;
        if unsafe { (*other_client).ps.otherKillerTime } > level_time {
            // we're as good as dead, so if someone pushed us into this then remember them
            unsafe {
                (*other_client).ps.otherKillerTime = level_time + 20000;
                (*other_client).ps.otherKillerDebounceTime = level_time + 10000;
                (*other_client).otherKillerMOD = MOD_FALLING as c_int;
                (*other_client).otherKillerVehWeapon = 0;
                (*other_client).otherKillerWeaponType = WP_NONE as c_int;
            }
        }
        unsafe {
            (*other_client).ps.fallingToDeath = level_time;

            // rag on the way down, this flag will automatically be cleared for us on respawn
            (*other_client).ps.eFlags |= EF_RAG;
        }

        // make sure his jetpack is off
        Jetpack_Off(ctx.world.entity_mut(other));

        if !ctx.world.entity(other).NPC.is_null() {
            // kill it now
            let mut v_dir: vec3_t = [0.0, 1.0, 0.0];
            let origin = unsafe { (*other_client).ps.origin };
            G_Damage(
                ctx,
                Some(other),
                Some(other),
                Some(other),
                Some(&mut v_dir),
                origin,
                Q3_INFINITE,
                0,
                MOD_FALLING as c_int,
            );
        } else {
            let sound = G_SoundIndex(ctx, "*falling1.wav");
            G_EntitySound(ctx, other, CHAN_VOICE, sound);
        }

        ctx.world.entity_mut(self_).timestamp = 0; // do not ignore others
    } else {
        let mut dmg = ctx.world.entity(self_).damage;

        if dmg == -1 {
            // so fall-to-blackness triggers destroy evertyhing
            dmg = 99999;
            ctx.world.entity_mut(self_).timestamp = 0;
        }
        let activator = ctx.world.entity(self_).activator;
        // FLAG: pool `gclient_t` deref (see above).
        let activator_ok = match activator {
            Some(a) => {
                let c = ctx.world.entity(a).client;
                ctx.world.entity(a).inuse != 0 && !c.is_null()
            }
            None => false,
        };
        if activator_ok {
            let activator = activator.unwrap();
            G_Damage(
                ctx,
                Some(other),
                Some(activator),
                Some(activator),
                None,
                vec3_origin,
                dmg,
                dflags | DAMAGE_NO_PROTECTION,
                MOD_TRIGGER_HURT as c_int,
            );
        } else {
            G_Damage(
                ctx,
                Some(other),
                Some(self_),
                Some(self_),
                None,
                vec3_origin,
                dmg,
                dflags | DAMAGE_NO_PROTECTION,
                MOD_TRIGGER_HURT as c_int,
            );
        }
    }
}

/// Raven `SP_trigger_hurt`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1413-1439`
pub fn SP_trigger_hurt(ctx: &mut GameContext, self_id: EntityId) {
    InitTrigger(ctx, self_id);

    ctx.world.globals.gTrigFallSound = G_SoundIndex(ctx, "*falling1.wav");

    ctx.world.entity_mut(self_id).noise_index =
        G_SoundIndex(ctx, "sound/weapons/force/speed.wav");
    ctx.world.entity_mut(self_id).touch = Some(EntTouch::hurt_touch).into();

    if ctx.world.entity(self_id).damage == 0 {
        ctx.world.entity_mut(self_id).damage = 5;
    }

    ctx.world.entity_mut(self_id).r.contents = CONTENTS_TRIGGER;

    if ctx.world.entity(self_id).spawnflags & 2 != 0 {
        ctx.world.entity_mut(self_id).use_ = Some(EntUse::hurt_use).into();
    }

    // link in to the world if starting active
    if ctx.world.entity(self_id).spawnflags & 1 == 0 {
        let self_ptr = ctx.world.entity_mut(self_id) as *mut gentity_t;
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_ptr.cast()));
    } else if ctx.world.entity(self_id).r.linked != 0 {
        let self_ptr = ctx.world.entity_mut(self_id) as *mut gentity_t;
        trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(self_ptr.cast()));
    }
}

/// Raven `space_touch`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1442-1478`
pub fn space_touch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    // NOTE: we need vehicles to know this, too...
    // || other->s.number >= MAX_CLIENTS)
    let other = match other {
        Some(o) => o,
        None => return,
    };
    if ctx.world.entity(other).inuse == 0 {
        return;
    }
    // FLAG: `.client` is a pool `gclient_t` for NPC/vehicle touchers.
    // Read the raw pointer through the safe entity borrow and deref it in a tight `unsafe` block, the same as Raven.
    let other_client = ctx.world.entity(other).client;
    if other_client.is_null() {
        return;
    }

    let m_iVehicleNum = unsafe { (*other_client).ps.m_iVehicleNum };
    if ctx.world.entity(other).s.number < MAX_CLIENTS as c_int // player
        && m_iVehicleNum != 0 // in a vehicle
        && m_iVehicleNum >= MAX_CLIENTS as c_int
    {
        // a player client inside a vehicle
        let veh = EntityId(m_iVehicleNum as u32);

        if ctx.world.entity(veh).inuse != 0
            && !ctx.world.entity(veh).client.is_null()
            && !ctx.world.entity(veh).m_pVehicle.is_null()
        {
            let p_veh = ctx.world.entity(veh).m_pVehicle;
            // §19: Raven derefs `m_pVehicleInfo` unguarded.
            // This file guards the null value that would crash Raven.
            let veh_info = unsafe { (*p_veh).m_pVehicleInfo };
            if !veh_info.is_null() && unsafe { (*veh_info).hideRider } != 0 {
                // if they are "inside" a vehicle, then let that protect them from THE HORRORS OF SPACE.
                unsafe {
                    (*other_client).inSpaceSuffocation = 0;
                    (*other_client).inSpaceIndex = ENTITYNUM_NONE;
                }
                return;
            }
        }
    }

    let origin = unsafe { (*other_client).ps.origin };
    let absmin = ctx.world.entity(self_).r.absmin;
    let absmax = ctx.world.entity(self_).r.absmax;
    if G_PointInBounds(origin, absmin, absmax) == 0 {
        // his origin must be inside the trigger
        return;
    }

    if unsafe { (*other_client).inSpaceIndex } == 0
        || unsafe { (*other_client).inSpaceIndex } == ENTITYNUM_NONE
    {
        // freshly entering space
        let t = ctx.world.level.time + INITIAL_SUFFOCATION_DELAY;
        unsafe {
            (*other_client).inSpaceSuffocation = t;
        }
    }

    let self_number = ctx.world.entity(self_).s.number;
    unsafe {
        (*other_client).inSpaceIndex = self_number;
    }
}

/// Raven `SP_trigger_space`.
///
/// causes human clients to suffocate and have no gravity.
/// Source: `oracle/codemp/game/g_trigger.c:1484-1492`
pub fn SP_trigger_space(ctx: &mut GameContext, self_id: EntityId) {
    InitTrigger(ctx, self_id);
    ctx.world.entity_mut(self_id).r.contents = CONTENTS_TRIGGER;

    ctx.world.entity_mut(self_id).touch = Some(EntTouch::space_touch).into();

    let self_ptr = ctx.world.entity_mut(self_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_ptr.cast()));
}

/// Raven `shipboundary_touch`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1494-1531`
pub fn shipboundary_touch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    let other = match other {
        Some(o) => o,
        None => return,
    };
    if ctx.world.entity(other).inuse == 0
        || ctx.world.entity(other).client.is_null()
        || ctx.world.entity(other).s.number < MAX_CLIENTS as c_int
        || ctx.world.entity(other).m_pVehicle.is_null()
    {
        // only let vehicles touch
        return;
    }

    // FLAG: pool `gclient_t` deref.
    // Read the raw pointer through the safe entity borrow, the same as Raven.
    let other_client = ctx.world.entity(other).client;

    if unsafe { (*other_client).ps.hyperSpaceTime } != 0
        && ctx.world.level.time - unsafe { (*other_client).ps.hyperSpaceTime } < HYPERSPACE_TIME
    {
        // don't interfere with hyperspacing ships
        return;
    }

    // A NULL `target` never matches in Raven's `G_Find`.
    // This file keeps that behavior by not searching, rather than reading NULL as a string.
    let target = ctx.world.entity(self_).target.clone();
    let ent = match target.as_deref() {
        Some(target) => G_Find(ctx, None, EntFindField::Targetname, target),
        None => core::ptr::null_mut(),
    };
    let ent_id = ctx.entity_id_of(ent);
    if ent_id.is_none() || ctx.world.entity(ent_id.unwrap()).inuse == 0 {
        // this is bad
        G_Error(ctx, "trigger_shipboundary has invalid target '%s'\n");
        return;
    }
    let ent_id = ent_id.unwrap();

    let veh = ctx.world.entity(other).m_pVehicle;
    if unsafe { (*other_client).ps.m_iVehicleNum } == 0 || unsafe { (*veh).m_iRemovedSurfaces } != 0
    {
        // if a vehicle touches a boundary without a pilot in it or with parts missing, just blow the thing up
        let origin = unsafe { (*other_client).ps.origin };
        G_Damage(
            ctx,
            Some(other),
            Some(other),
            Some(other),
            None,
            origin,
            99999,
            DAMAGE_NO_PROTECTION,
            MOD_SUICIDE as c_int,
        );
        return;
    }

    // make sure this sucker is linked so the prediction knows where to go
    let ent_ptr = ctx.world.entity_mut(ent_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));

    let ent_number = ctx.world.entity(ent_id).s.number;
    let gv1 = ctx.world.entity(self_).genericValue1;
    let level_time = ctx.world.level.time;
    unsafe {
        (*other_client).ps.vehTurnaroundIndex = ent_number;
        (*other_client).ps.vehTurnaroundTime = level_time + gv1 * 2;
    }

    // keep up the detailed checks for another 2 seconds
    ctx.world.entity_mut(self_).genericValue7 = ctx.world.level.time + 2000;
}

/// Raven `shipboundary_think`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1533-1565`
pub fn shipboundary_think(ctx: &mut GameContext, ent_id: EntityId) {
    ctx.world.entity_mut(ent_id).nextthink = ctx.world.level.time + 100;

    if ctx.world.entity(ent_id).genericValue7 < ctx.world.level.time {
        // don't need to be doing this check, no one has touched recently
        return;
    }

    let mut entity_list = [0i32; mp_qshared::shared::MAX_GENTITIES];
    let absmin = ctx.world.entity(ent_id).r.absmin;
    let absmax = ctx.world.entity(ent_id).r.absmax;
    let num_listed = trap::EntitiesInBox(
        ctx.engine,
        GEntitiesInBoxArgs::new(
            &absmin as *const vec3_t,
            &absmax as *const vec3_t,
            entity_list.as_mut_ptr(),
            entity_list.len() as c_int,
        ),
    );

    let mut i = 0;
    while i < num_listed {
        let listed_ent = EntityId(entity_list[i as usize] as u32);
        // FLAG: pool `gclient_t` deref.
        // Read the raw pointer through the safe entity borrow, the same as Raven.
        let clp = ctx.world.entity(listed_ent).client;
        if ctx.world.entity(listed_ent).inuse != 0
            && !clp.is_null()
            && unsafe { (*clp).ps.m_iVehicleNum } != 0
        {
            if ctx.world.entity(listed_ent).s.eType == entityType_t::ET_NPC as c_int
                && ctx.world.entity(listed_ent).s.NPC_class == CLASS_VEHICLE as c_int
            {
                let p_veh = ctx.world.entity(listed_ent).m_pVehicle;
                // §19: Raven derefs `m_pVehicleInfo` unguarded.
                // This file guards the null value that would crash Raven.
                if !p_veh.is_null()
                    && !unsafe { (*p_veh).m_pVehicleInfo }.is_null()
                    && unsafe { (*(*p_veh).m_pVehicleInfo).r#type } == VH_FIGHTER
                {
                    shipboundary_touch(ctx, ent_id, Some(listed_ent), core::ptr::null_mut());
                }
            }
        }
        i += 1;
    }
}

/// Raven `SP_trigger_shipboundary`.
///
/// causes vehicle to turn toward target and travel in that direction for a set time when hit.
/// Source: `oracle/codemp/game/g_trigger.c:1574-1595`
pub fn SP_trigger_shipboundary(ctx: &mut GameContext, self_id: EntityId) {
    InitTrigger(ctx, self_id);
    ctx.world.entity_mut(self_id).r.contents = CONTENTS_TRIGGER;

    let target = ctx.world.entity(self_id).target.clone();
    if !target.as_deref().is_some_and(|s| !s.is_empty()) {
        G_Error(ctx, "trigger_shipboundary without a target.");
    }
    let mut gv1: c_int = 0;
    G_SpawnInt(ctx, c"traveltime".as_ptr(), c"0".as_ptr(), &mut gv1);
    ctx.world.entity_mut(self_id).genericValue1 = gv1;

    if ctx.world.entity(self_id).genericValue1 == 0 {
        G_Error(ctx, "trigger_shipboundary without traveltime.");
    }

    ctx.world.entity_mut(self_id).think = Some(EntThink::shipboundary_think).into();
    ctx.world.entity_mut(self_id).nextthink = ctx.world.level.time + 500;
    ctx.world.entity_mut(self_id).touch = Some(EntTouch::shipboundary_touch).into();

    let self_ptr = ctx.world.entity_mut(self_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_ptr.cast()));
}

/// Raven `hyperspace_touch`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1597-1680`
pub fn hyperspace_touch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    let other = match other {
        Some(o) => o,
        None => return,
    };
    if ctx.world.entity(other).inuse == 0
        || ctx.world.entity(other).client.is_null()
        || ctx.world.entity(other).s.number < MAX_CLIENTS as c_int
        || ctx.world.entity(other).m_pVehicle.is_null()
    {
        // only let vehicles touch
        return;
    }

    // FLAG: pool `gclient_t` deref.
    // Read the raw pointer through the safe entity borrow, the same as Raven.
    let other_client = ctx.world.entity(other).client;

    if unsafe { (*other_client).ps.hyperSpaceTime } != 0
        && ctx.world.level.time - unsafe { (*other_client).ps.hyperSpaceTime } < HYPERSPACE_TIME
    {
        // already hyperspacing, just keep us moving
        if unsafe { (*other_client).ps.eFlags2 } & EF2_HYPERSPACE != 0 {
            // they've started the hyperspace but haven't been teleported yet
            let time_frac = (ctx.world.level.time - unsafe { (*other_client).ps.hyperSpaceTime })
                as f32
                / HYPERSPACE_TIME as f32;
            if time_frac >= HYPERSPACE_TELEPORT_FRAC {
                // half-way, now teleport them!
                // take off the flag so we only do this once
                unsafe {
                    (*other_client).ps.eFlags2 &= !EF2_HYPERSPACE;
                }
                // Get the offset from the local position
                // A NULL `target` never matches in Raven's `G_Find`.
                // This file keeps that behavior by not searching, rather than reading NULL as a string.
                let target = ctx.world.entity(self_).target.clone();
                let ent = match target.as_deref() {
                    Some(target) => G_Find(ctx, None, EntFindField::Targetname, target),
                    None => core::ptr::null_mut(),
                };
                let ent_id = ctx.entity_id_of(ent);
                if ent_id.is_none() || ctx.world.entity(ent_id.unwrap()).inuse == 0 {
                    // this is bad
                    G_Error(ctx, "trigger_hyperspace has invalid target '%s'\n");
                    return;
                }
                let ent_id = ent_id.unwrap();
                let origin = unsafe { (*other_client).ps.origin };
                let ent_origin = ctx.world.entity(ent_id).s.origin;
                let diff: vec3_t = [
                    origin[0] - ent_origin[0],
                    origin[1] - ent_origin[1],
                    origin[2] - ent_origin[2],
                ];
                let mut fwd: vec3_t = [0.0; 3];
                let mut right: vec3_t = [0.0; 3];
                let mut up: vec3_t = [0.0; 3];
                let angles = ctx.world.entity(ent_id).s.angles;
                AngleVectors(angles, Some(&mut fwd), Some(&mut right), Some(&mut up));
                let f_diff = fwd[0] * diff[0] + fwd[1] * diff[1] + fwd[2] * diff[2];
                let r_diff = right[0] * diff[0] + right[1] * diff[1] + right[2] * diff[2];
                let u_diff = up[0] * diff[0] + up[1] * diff[1] + up[2] * diff[2];

                // Now get the base position of the destination
                // A NULL `target2` never matches in Raven's `G_Find`.
                // This file keeps that behavior by not searching, rather than reading NULL as a string.
                let target2 = ctx.world.entity(self_).target2.clone();
                let ent = match target2.as_deref() {
                    Some(target2) => G_Find(ctx, None, EntFindField::Targetname, target2),
                    None => core::ptr::null_mut(),
                };
                let ent_id = ctx.entity_id_of(ent);
                if ent_id.is_none() || ctx.world.entity(ent_id.unwrap()).inuse == 0 {
                    // this is bad
                    G_Error(ctx, "trigger_hyperspace has invalid target2 '%s'\n");
                    return;
                }
                let ent_id = ent_id.unwrap();
                let mut new_org: vec3_t = ctx.world.entity(ent_id).s.origin;
                // finally, add the offset into the new origin
                let angles = ctx.world.entity(ent_id).s.angles;
                AngleVectors(angles, Some(&mut fwd), Some(&mut right), Some(&mut up));
                let radius = ctx.world.entity(self_).radius;
                let f_scale = f_diff * radius;
                new_org = [
                    new_org[0] + f_scale * fwd[0],
                    new_org[1] + f_scale * fwd[1],
                    new_org[2] + f_scale * fwd[2],
                ];
                let r_scale = r_diff * radius;
                new_org = [
                    new_org[0] + r_scale * right[0],
                    new_org[1] + r_scale * right[1],
                    new_org[2] + r_scale * right[2],
                ];
                let u_scale = u_diff * radius;
                new_org = [
                    new_org[0] + u_scale * up[0],
                    new_org[1] + u_scale * up[1],
                    new_org[2] + u_scale * up[2],
                ];
                // now put them in the offset position, facing the angles that position wants them to be facing
                let ent_angles = ctx.world.entity(ent_id).s.angles;
                TeleportPlayer(ctx, other, new_org, ent_angles);
                let veh = ctx.world.entity(other).m_pVehicle;
                if !veh.is_null() && !unsafe { (*veh).m_pPilot }.is_null() {
                    // teleport the pilot, too
                    let pilot = unsafe { (*veh).m_pPilot } as *mut gentity_t;
                    let pilot_id = ctx.entity_id_of(pilot).unwrap();
                    let ent_angles = ctx.world.entity(ent_id).s.angles;
                    TeleportPlayer(ctx, pilot_id, new_org, ent_angles);
                    // FIXME: and the passengers?
                }
                // make them face the new angle
                let ent_angles = ctx.world.entity(ent_id).s.angles;
                unsafe {
                    (*other_client).ps.hyperSpaceAngles = ent_angles;
                }
                // sound
                let sound = G_SoundIndex(ctx, "sound/vehicles/common/hyperend.wav");
                G_Sound(ctx, Some(other), CHAN_LOCAL, sound);
            }
        }
        return;
    } else {
        // A NULL `target` never matches in Raven's `G_Find`.
        // This file keeps that behavior by not searching, rather than reading NULL as a string.
        let target = ctx.world.entity(self_).target.clone();
        let ent = match target.as_deref() {
            Some(target) => G_Find(ctx, None, EntFindField::Targetname, target),
            None => core::ptr::null_mut(),
        };
        let ent_id = ctx.entity_id_of(ent);
        if ent_id.is_none() || ctx.world.entity(ent_id.unwrap()).inuse == 0 {
            // this is bad
            G_Error(ctx, "trigger_hyperspace has invalid target '%s'\n");
            return;
        }
        let ent_id = ent_id.unwrap();

        let veh = ctx.world.entity(other).m_pVehicle;
        if unsafe { (*other_client).ps.m_iVehicleNum } == 0
            || unsafe { (*veh).m_iRemovedSurfaces } != 0
        {
            // if a vehicle touches a boundary without a pilot in it or with parts missing, just blow the thing up
            let origin = unsafe { (*other_client).ps.origin };
            G_Damage(
                ctx,
                Some(other),
                Some(other),
                Some(other),
                None,
                origin,
                99999,
                DAMAGE_NO_PROTECTION,
                MOD_SUICIDE as c_int,
            );
            return;
        }
        let ent_angles = ctx.world.entity(ent_id).s.angles;
        unsafe {
            (*other_client).ps.hyperSpaceAngles = ent_angles;
            (*other_client).ps.hyperSpaceTime = ctx.world.level.time;
        }
    }
}

/// Raven `SP_trigger_hyperspace`.
///
/// Ship will turn to face the angles of the first target_position then fly forward,
/// playing the hyperspace effect, then pop out at a relative point around the target
/// Source: `oracle/codemp/game/g_trigger.c:1709-1736`
pub fn SP_trigger_hyperspace(ctx: &mut GameContext, self_id: EntityId) {
    let mut radius: f32 = 0.0;
    G_SpawnFloat(ctx, c"exitscale".as_ptr(), c"1".as_ptr(), &mut radius);
    ctx.world.entity_mut(self_id).radius = radius;

    // register the hyperspace end sound (start sounds are customized)
    G_SoundIndex(ctx, "sound/vehicles/common/hyperend.wav");

    InitTrigger(ctx, self_id);
    ctx.world.entity_mut(self_id).r.contents = CONTENTS_TRIGGER;

    let target = ctx.world.entity(self_id).target.clone();
    if !target.as_deref().is_some_and(|s| !s.is_empty()) {
        G_Error(ctx, "trigger_hyperspace without a target.");
    }
    let target2 = ctx.world.entity(self_id).target2.clone();
    if !target2.as_deref().is_some_and(|s| !s.is_empty()) {
        G_Error(ctx, "trigger_hyperspace without a target2.");
    }

    let absmax = ctx.world.entity(self_id).r.absmax;
    let absmin = ctx.world.entity(self_id).r.absmin;
    ctx.world.entity_mut(self_id).delay = Distance(absmax, absmin) as c_int; // my size

    ctx.world.entity_mut(self_id).touch = Some(EntTouch::hyperspace_touch).into();

    let self_ptr = ctx.world.entity_mut(self_id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_ptr.cast()));

    // self->think = trigger_hyperspace_find_targets;
    // self->nextthink = level.time + FRAMETIME;
}

/// Raven `func_timer_think`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1757-1761`
pub fn func_timer_think(ctx: &mut GameContext, self_: EntityId) {
    let activator = ctx.world.entity(self_).activator;
    G_UseTargets(ctx, Some(self_), activator);
    // set time before next firing
    let w = ctx.world.entity(self_).wait as f64;
    let r = ctx.world.entity(self_).random as f64;
    let nt = (ctx.world.level.time as f64 + 1000.0 * (w + ctx.world.bg_state.rng.crandom() * r))
        as c_int;
    ctx.world.entity_mut(self_).nextthink = nt;
}

/// Raven `func_timer_use`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1763-1776`
pub fn func_timer_use(
    ctx: &mut GameContext,
    self_id: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    ctx.world.entity_mut(self_id).activator = activator;

    G_ActivateBehavior(ctx, Some(self_id), bSet_t::BSET_USE as c_int);

    // if on, turn it off
    if ctx.world.entity(self_id).nextthink != 0 {
        ctx.world.entity_mut(self_id).nextthink = 0;
        return;
    }

    // turn it on
    func_timer_think(ctx, self_id);
}

/// Raven `SP_func_timer`.
///
/// This should be renamed trigger_timer...
/// Repeatedly fires its targets.
/// Can be turned on or off by using.
/// Source: `oracle/codemp/game/g_trigger.c:1778-1796`
pub fn SP_func_timer(ctx: &mut GameContext, self_: EntityId) {
    let mut random: f32 = 0.0;
    G_SpawnFloat(ctx, c"random".as_ptr(), c"1".as_ptr(), &mut random);
    ctx.world.entity_mut(self_).random = random;
    let mut wait: f32 = 0.0;
    G_SpawnFloat(ctx, c"wait".as_ptr(), c"1".as_ptr(), &mut wait);
    ctx.world.entity_mut(self_).wait = wait;

    ctx.world.entity_mut(self_).use_ = Some(EntUse::func_timer_use).into();
    ctx.world.entity_mut(self_).think = Some(EntThink::func_timer_think).into();

    if ctx.world.entity(self_).random >= ctx.world.entity(self_).wait {
        let w = ctx.world.entity(self_).wait;
        ctx.world.entity_mut(self_).random = w - 1.0; // NOTE: was - FRAMETIME, but FRAMETIME is
                                                      // in msec (100) and these numbers are in
                                                      // *seconds*!
                                                      // `G_Printf`'s call site is a fixed format string with no vararg substitution.
                                                      // `vtos(self->s.origin)` is dropped from the message.
                                                      // This matches other `G_Printf`/`G_Error` sites in this file.
        G_Printf(ctx, "func_timer at (unresolved-vtos) has random >= wait\n");
    }

    if ctx.world.entity(self_).spawnflags & 1 != 0 {
        ctx.world.entity_mut(self_).nextthink = ctx.world.level.time + FRAMETIME;
        ctx.world.entity_mut(self_).activator = Some(self_);
    }

    ctx.world.entity_mut(self_).r.svFlags = SVF_NOCLIENT;
}

/// Raven `asteroid_pick_random_asteroid`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1798-1841`
pub fn asteroid_pick_random_asteroid(ctx: &mut GameContext, self_: EntityId) -> *mut gentity_t {
    // The return type stays `*mut gentity_t`. The caller re-derives its id from this raw handle.
    // The function body reaches entity fields through the safe accessors.
    let mut t_count: c_int = 0;
    let mut t: *mut gentity_t = core::ptr::null_mut();
    // A NULL `target` never matches in Raven's `G_Find`.
    // This file reads it as `Option<String>`, so a NULL target yields no matches instead of dereferencing NULL.
    let target = ctx.world.entity(self_).target.clone();

    loop {
        let t_id = ctx.entity_id_of(t);
        t = match &target {
            Some(target) => G_Find(ctx, t_id, EntFindField::Targetname, target),
            None => core::ptr::null_mut(),
        };
        if t.is_null() {
            break;
        }
        if ctx.entity_id_of(t) != Some(self_) {
            t_count += 1;
        }
    }

    if t_count == 0 {
        return core::ptr::null_mut();
    }

    if t_count == 1 {
        return match &target {
            Some(target) => G_Find(ctx, None, EntFindField::Targetname, target),
            None => core::ptr::null_mut(),
        };
    }

    // FIXME: need a seed
    let pick = ctx.world.bg_state.rng.Q_irand(1, t_count);
    t_count = 0;
    t = core::ptr::null_mut();
    loop {
        let t_id = ctx.entity_id_of(t);
        t = match &target {
            Some(target) => G_Find(ctx, t_id, EntFindField::Targetname, target),
            None => core::ptr::null_mut(),
        };
        if t.is_null() {
            break;
        }
        if ctx.entity_id_of(t) != Some(self_) {
            t_count += 1;
        } else {
            continue;
        }

        if t_count == pick {
            return t;
        }
    }
    core::ptr::null_mut()
}

/// Raven `asteroid_count_num_asteroids`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1843-1859`
pub fn asteroid_count_num_asteroids(ctx: &mut GameContext, self_: EntityId) -> c_int {
    let mut count: c_int = 0;
    let mut i = MAX_CLIENTS as c_int;
    let self_number = ctx.world.entity(self_).s.number;
    while i < ENTITYNUM_WORLD as c_int {
        let e = EntityId(i as u32);
        if ctx.world.entity(e).inuse == 0 {
            i += 1;
            continue;
        }
        if ctx.world.entity(e).r.ownerNum == self_number {
            count += 1;
        }
        i += 1;
    }
    count
}

/// Raven `asteroid_move_to_start2`.
///
/// move asteroid to a new start position
/// Source: `oracle/codemp/game/g_trigger.c:1864-1920`
pub fn asteroid_move_to_start2(
    ctx: &mut GameContext,
    self_: EntityId,
    ownerTrigger: Option<EntityId>,
) {
    if let Some(owner) = ownerTrigger {
        // move it
        let self_speed = ctx.world.entity(self_).speed;
        let speed = ctx
            .world
            .bg_state
            .rng
            .flrand(self_speed * 0.25, self_speed * 2.0);
        let cap_axis = ctx.world.bg_state.rng.Q_irand(0, 2);

        let mut start_spot: vec3_t = [0.0; 3];
        let mut end_spot: vec3_t = [0.0; 3];

        for axis in 0..3usize {
            if axis as c_int == cap_axis {
                if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                    start_spot[axis] = ctx.world.entity(owner).r.mins[axis];
                    end_spot[axis] = ctx.world.entity(owner).r.maxs[axis];
                } else {
                    start_spot[axis] = ctx.world.entity(owner).r.maxs[axis];
                    end_spot[axis] = ctx.world.entity(owner).r.mins[axis];
                }
            } else {
                let mins = ctx.world.entity(owner).r.mins[axis];
                let maxs = ctx.world.entity(owner).r.maxs[axis];
                start_spot[axis] = mins + (ctx.world.bg_state.rng.flrand(0.0, 1.0) * (maxs - mins));
                end_spot[axis] = mins + (ctx.world.bg_state.rng.flrand(0.0, 1.0) * (maxs - mins));
            }
        }
        // FIXME: maybe trace from start to end to make sure nothing is in the way?  How big of a trace?

        G_SetOrigin(ctx.world.entity_mut(self_), start_spot);
        let dist = crate::q_math::Distance(end_spot, start_spot);
        let time = ((dist / speed).ceil() as c_int) * 1000;
        let self_number = ctx.world.entity(self_).s.number;
        crate::g_ICARUScb::Q3_Lerp2Origin(ctx, -1, self_number, end_spot, time as f32);

        // spin it
        let start_angles: vec3_t = [
            ctx.world.bg_state.rng.flrand(-360.0, 360.0),
            ctx.world.bg_state.rng.flrand(-360.0, 360.0),
            ctx.world.bg_state.rng.flrand(-360.0, 360.0),
        ];
        G_SetAngles(ctx.world.entity_mut(self_), start_angles);
        let tr_delta: vec3_t = [
            ctx.world.bg_state.rng.flrand(-100.0, 100.0),
            ctx.world.bg_state.rng.flrand(-100.0, 100.0),
            ctx.world.bg_state.rng.flrand(-100.0, 100.0),
        ];
        ctx.world.entity_mut(self_).s.apos.trDelta = tr_delta;
        ctx.world.entity_mut(self_).s.apos.trTime = ctx.world.level.time;
        ctx.world.entity_mut(self_).s.apos.trType = TR_LINEAR;
        // move itownerTrigger back to a new start when done
        ctx.world.entity_mut(self_).think = Some(EntThink::asteroid_move_to_start).into();
        ctx.world.entity_mut(self_).nextthink = ctx.world.level.time + time;
    } else {
        // crap, go bye-bye
        ctx.world.entity_mut(self_).think = Some(EntThink::G_FreeEntity).into();
        ctx.world.entity_mut(self_).nextthink = ctx.world.level.time + FRAMETIME;
    }
}

/// Raven `asteroid_move_to_start`.
///
/// move asteroid to a new start position
/// Source: `oracle/codemp/game/g_trigger.c:1922-1925`
pub fn asteroid_move_to_start(ctx: &mut GameContext, self_id: EntityId) {
    let owner_num = ctx.world.entity(self_id).r.ownerNum;
    asteroid_move_to_start2(ctx, self_id, Some(EntityId(owner_num as u32)));
}

/// Raven `asteroid_field_think`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1927-1979`
pub fn asteroid_field_think(ctx: &mut GameContext, self_id: EntityId) {
    let num_asteroids = asteroid_count_num_asteroids(ctx, self_id);

    ctx.world.entity_mut(self_id).nextthink = ctx.world.level.time + 500;

    if num_asteroids < ctx.world.entity(self_id).count {
        // need to spawn a new asteroid
        let new_asteroid_eid = G_Spawn(ctx);
        let new_asteroid = ctx.entity_mut(new_asteroid_eid) as *mut gentity_t;
        if !new_asteroid.is_null() {
            let new_id = new_asteroid_eid;
            let copy_asteroid = asteroid_pick_random_asteroid(ctx, self_id);
            if !copy_asteroid.is_null() {
                let copy_id = ctx.entity_id_of(copy_asteroid).unwrap();

                let c_model = ctx.world.entity(copy_id).model.clone();
                ctx.world.entity_mut(new_id).model = c_model;
                let c_model2 = ctx.world.entity(copy_id).model2.clone();
                ctx.world.entity_mut(new_id).model2 = c_model2;
                let c_health = ctx.world.entity(copy_id).health;
                ctx.world.entity_mut(new_id).health = c_health;
                let c_spawnflags = ctx.world.entity(copy_id).spawnflags;
                ctx.world.entity_mut(new_id).spawnflags = c_spawnflags;
                let c_mass = ctx.world.entity(copy_id).mass;
                ctx.world.entity_mut(new_id).mass = c_mass;
                let c_damage = ctx.world.entity(copy_id).damage;
                ctx.world.entity_mut(new_id).damage = c_damage;
                let c_speed = ctx.world.entity(copy_id).speed;
                ctx.world.entity_mut(new_id).speed = c_speed;

                let c_origin = ctx.world.entity(copy_id).s.origin;
                G_SetOrigin(ctx.world.entity_mut(new_id), c_origin);
                let c_angles = ctx.world.entity(copy_id).s.angles;
                G_SetAngles(ctx.world.entity_mut(new_id), c_angles);
                ctx.ent_set(new_id, PrefixSet::ClassnameStatic(c"func_rotating"));

                SP_func_rotating(ctx, new_id);

                let c_gv15 = ctx.world.entity(copy_id).genericValue15;
                ctx.world.entity_mut(new_id).genericValue15 = c_gv15;
                let c_imodelscale = ctx.world.entity(copy_id).s.iModelScale;
                ctx.world.entity_mut(new_id).s.iModelScale = c_imodelscale;
                let new_health = ctx.world.entity(new_id).health;
                ctx.world.entity_mut(new_id).maxHealth = new_health;
                G_ScaleNetHealth(ctx.world.entity_mut(new_id));
                let c_radius = ctx.world.entity(copy_id).radius;
                ctx.world.entity_mut(new_id).radius = c_radius;
                let c_material = ctx.world.entity(copy_id).material;
                ctx.world.entity_mut(new_id).material = c_material;
                // CacheChunkEffects( self->material );

                // keep track of it
                let self_number = ctx.world.entity(self_id).s.number;
                ctx.world.entity_mut(new_id).r.ownerNum = self_number;

                // position it
                asteroid_move_to_start2(ctx, new_id, Some(self_id));

                // think again sooner if need even more
                if num_asteroids + 1 < ctx.world.entity(self_id).count {
                    // still need at least one more
                    // spawn it in 100ms
                    ctx.world.entity_mut(self_id).nextthink = ctx.world.level.time + 100;
                }
            }
        }
    }
}

/// Raven `SP_trigger_asteroid_field`.
///
/// Source: `oracle/codemp/game/g_trigger.c:1986-2007`
pub fn SP_trigger_asteroid_field(ctx: &mut GameContext, self_: EntityId) {
    let self_ptr = ctx.world.entity_mut(self_) as *mut gentity_t;
    let model = ctx.world.entity(self_).model.clone();
    trap::SetBrushModel(ctx.engine, self_ptr.cast(), model.as_deref().unwrap_or(""));
    // self->r.contents = CONTENTS_TRIGGER; // replaces the -1 from trap_SetBrushModel
    ctx.world.entity_mut(self_).r.contents = 0;
    ctx.world.entity_mut(self_).r.svFlags = SVF_NOCLIENT;

    if ctx.world.entity(self_).count == 0 {
        ctx.world.entity_mut(self_).health = 20;
    }

    if ctx.world.entity(self_).speed == 0.0 {
        ctx.world.entity_mut(self_).speed = 10000.0;
    }

    ctx.world.entity_mut(self_).think = Some(EntThink::asteroid_field_think).into();
    ctx.world.entity_mut(self_).nextthink = ctx.world.level.time + 100;

    let self_ptr = ctx.world.entity_mut(self_) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_ptr.cast()));
}
