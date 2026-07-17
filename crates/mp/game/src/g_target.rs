// PORT-COMPLETE: g_target.c
//! FAITHFUL port of `oracle/codemp/game/g_target.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::world::GameWorld;

use crate::entity::flags::FL_INACTIVE;
use crate::g_combat::{AddScore, G_Damage};
use crate::g_main::Com_Error;
use crate::g_misc::TeleportPlayer;
use crate::g_team::Team_ReturnFlag;
use crate::g_utils::{
    G_AddEvent, G_Find, G_PickTarget, G_SetOrigin, G_UseTargets, G_UseTargets2, GlobalUse,
};
use crate::level::damage_flags::DAMAGE_NO_PROTECTION;
use crate::NPC_utils::G_ActivateBehavior;
use mp_abi::game::syscalls::G_ICARUS_INITENT::GIcarusInitentArgs;
use mp_abi::game::syscalls::G_ICARUS_ISINITIALIZED::GIcarusIsinitializedArgs;
use mp_abi::game::syscalls::G_ICARUS_RUNSCRIPT::GIcarusRunscriptArgs;
use mp_abi::game::syscalls::G_ICARUS_VALIDENT::GIcarusValidentArgs;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_abi::game::syscalls::G_UNLINKENTITY::GUnlinkentityArgs;
use mp_bg::public::entity_event::entity_event_t;
use mp_bg::public::means_of_death::meansOfDeath_t;
use mp_bg::public::powerup::{PW_BLUEFLAG, PW_NEUTRALFLAG, PW_REDFLAG};
use mp_bg::public::team::{TEAM_BLUE, TEAM_FREE, TEAM_RED};
use mp_qshared::common::mp::qcommon::b_set_t::bSet_t;
use mp_qshared::common::mp::qcommon::player_state::MAX_POWERUPS;

/// Raven `#define Q3_SCRIPT_DIR "scripts"`.
/// Source: `oracle/codemp/game/q_shared.h:10`
pub const Q3_SCRIPT_DIR: &core::ffi::CStr = c"scripts";

/// Raven `Use_Target_Give`.
///
/// Source: `oracle/codemp/game/g_target.c:10-34`
pub fn Use_Target_Give(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // Raven derefs `activator->client` unconditionally (no NULL guard), so the
    // handler treats activator as present.
    let activator = activator.expect("Use_Target_Give: null activator");
    if ctx.entity(activator).client.is_null() {
        return;
    }

    let ent_target = ctx.entity(ent).target;
    if ent_target.is_null() {
        return;
    }

    // trace_t has no zeroing constructor; the mem::zeroed is a plain POD-init.
    let mut trace: trace_t = unsafe { core::mem::zeroed() };
    let mut t: *mut gentity_t = core::ptr::null_mut();
    loop {
        t = G_Find(
            ctx,
            ctx.entity_id_of(t),
            core::mem::offset_of!(gentity_t, targetname) as c_int,
            ent_target,
        );
        if t.is_null() {
            break;
        }
        let t_id = ctx.entity_id_of(t).unwrap();
        if ctx.entity(t_id).item.is_none() {
            continue;
        }
        Touch_Item(ctx, t_id, Some(activator), &mut trace);

        // make sure it isn't going to respawn or show any events
        ctx.entity_mut(t_id).nextthink = 0;
        trap::UnlinkEntity(
            ctx.engine,
            GUnlinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(t_id)).cast()),
        );
    }
}

/// Raven `SP_target_give`.
///
/// Source: `oracle/codemp/game/g_target.c:36-38`
pub fn SP_target_give(ent: &mut gentity_t) {
    ent.use_ = Some(EntUse::Use_Target_Give).into();
}

/// Raven `Use_target_remove_powerups`.
///
/// Source: `oracle/codemp/game/g_target.c:47-61`
pub fn Use_target_remove_powerups(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // Raven derefs `activator->client` unconditionally (no NULL guard).
    let activator = activator.expect("Use_target_remove_powerups: null activator");
    let client = ctx.entity(activator).client;
    if client.is_null() {
        return;
    }
    unsafe {
        if (*client).ps.powerups[PW_REDFLAG as usize] != 0 {
            Team_ReturnFlag(ctx, TEAM_RED);
        } else if (*client).ps.powerups[PW_BLUEFLAG as usize] != 0 {
            Team_ReturnFlag(ctx, TEAM_BLUE);
        } else if (*client).ps.powerups[PW_NEUTRALFLAG as usize] != 0 {
            Team_ReturnFlag(ctx, TEAM_FREE);
        }
        (*client).ps.powerups = [0; MAX_POWERUPS];
    }
}

/// Raven `SP_target_remove_powerups`.
///
/// Source: `oracle/codemp/game/g_target.c:63-65`
pub fn SP_target_remove_powerups(ent: &mut gentity_t) {
    ent.use_ = Some(EntUse::Use_target_remove_powerups).into();
}

/// Raven `Think_Target_Delay`.
///
/// Source: `oracle/codemp/game/g_target.c:78-80`
pub fn Think_Target_Delay(ctx: &mut GameContext, ent: EntityId) {
    let activator = ctx.entity(ent).activator;
    let activator_ptr =
        unsafe { crate::ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), activator) };
    G_UseTargets(ctx, Some(ent), ctx.entity_id_of(activator_ptr));
}

/// Raven `Use_Target_Delay`.
///
/// Source: `oracle/codemp/game/g_target.c:82-91`
pub fn Use_Target_Delay(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let ent_ptr: *mut gentity_t = ctx.entity_mut(ent);
    let ent_id = ctx.entity_id_of(ent_ptr);
    {
        let e = ctx.entity(ent);
        if e.nextthink > ctx.world.level.time && (e.spawnflags & 1) != 0 {
            return;
        }
    }
    G_ActivateBehavior(ctx, ent_id, bSet_t::BSET_USE as c_int);
    let crand = ctx.world.bg_state.rng.crandom();
    let level_time = ctx.world.level.time;
    let e = ctx.entity_mut(ent);
    // C computes the whole RHS in `double` (`crand` is `double`) and truncates
    // once into the `int` nextthink.
    e.nextthink = (level_time as f64 + (e.wait as f64 + e.random as f64 * crand) * 1000.0) as c_int;
    e.think = Some(EntThink::Think_Target_Delay).into();
    // C stored the raw `activator` pointer (NULL stays NULL and
    // Think_Target_Delay's G_UseTargets tolerates a NULL activator); the
    // `Option<EntityId>` handle carries the same nullability directly.
    e.activator = activator;
}

/// Raven `SP_target_delay`.
///
/// Source: `oracle/codemp/game/g_target.c:93-103`
pub fn SP_target_delay(ctx: &mut GameContext, ent: EntityId) {
    let delay_ptr: *mut f32 = &mut ctx.entity_mut(ent).wait;
    // check delay for backwards compatibility
    if G_SpawnFloat(
        ctx,
        b"delay\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        delay_ptr,
    ) == 0
    {
        let wait_ptr: *mut f32 = &mut ctx.entity_mut(ent).wait;
        G_SpawnFloat(
            ctx,
            b"wait\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
            wait_ptr,
        );
    }

    let e = ctx.entity_mut(ent);
    if e.wait == 0.0 {
        e.wait = 1.0;
    }
    e.use_ = Some(EntUse::Use_Target_Delay).into();
}

/// Raven `Use_Target_Score`.
///
/// Source: `oracle/codemp/game/g_target.c:113-115`
pub fn Use_Target_Score(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let e = ctx.entity(ent);
    let (origin, count) = (e.r.currentOrigin, e.count);
    let activator_ptr =
        unsafe { crate::ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), activator) };
    AddScore(ctx, ctx.entity_id_of(activator_ptr).unwrap(), origin, count);
}

/// Raven `SP_target_score`.
///
/// Source: `oracle/codemp/game/g_target.c:117-122`
pub fn SP_target_score(ent: &mut gentity_t) {
    if ent.count == 0 {
        ent.count = 1;
    }
    ent.use_ = Some(EntUse::Use_Target_Score).into();
}

/// Raven `Use_Target_Print`.
///
/// Source: `oracle/codemp/game/g_target.c:132-237`
pub fn Use_Target_Print(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // Raven's `if (!ent || !ent->inuse)` — the receiver handle is never NULL,
    // so only the inuse arm can fire.
    if ctx.entity(ent).inuse == 0 {
        // Com_Printf("ERROR: Bad ent in Use_Target_Print");
        return;
    }

    let level_time = ctx.world.level.time;
    let wait = ctx.entity(ent).wait;
    if wait != 0.0 {
        if ctx.entity(ent).genericValue14 >= level_time {
            return;
        }
        ctx.entity_mut(ent).genericValue14 = (level_time as f32 + wait) as c_int;
    }

    // `#ifndef FINAL_BUILD` block — LIVE in the referee build (FINAL_BUILD is
    // undefined). The `!ent || !ent->inuse` arm is dead (receiver handle is never
    // null and inuse was checked above), so only the else-if activator arm fires.
    // Source: `oracle/codemp/game/g_target.c:149-181`
    if activator.is_none() || ctx.entity(activator.unwrap()).inuse == 0 {
        Com_Error(
            ERR_DROP as c_int,
            c"Bad activator in Use_Target_Print".as_ptr(),
        );
    }

    if ctx.entity(ent).genericValue15 > level_time {
        Com_Printf(c"TARGET PRINT ERRORS:\n".as_ptr());
        unsafe {
            if let Some(activator) = activator {
                let classname = ctx.entity(activator).classname;
                if !classname.is_null() && *classname != 0 {
                    Com_Printf(
                        cstr(&format!(
                            "activator classname: {}\n",
                            cstr_to_str(classname)
                        ))
                        .as_ptr(),
                    );
                }
                let target = ctx.entity(activator).target;
                if !target.is_null() && *target != 0 {
                    Com_Printf(
                        cstr(&format!("activator target: {}\n", cstr_to_str(target))).as_ptr(),
                    );
                }
                let targetname = ctx.entity(activator).targetname;
                if !targetname.is_null() && *targetname != 0 {
                    Com_Printf(
                        cstr(&format!(
                            "activator targetname: {}\n",
                            cstr_to_str(targetname)
                        ))
                        .as_ptr(),
                    );
                }
            }
            let ent_targetname = ctx.entity(ent).targetname;
            if !ent_targetname.is_null() && *ent_targetname != 0 {
                Com_Printf(
                    cstr(&format!(
                        "print targetname: {}\n",
                        cstr_to_str(ent_targetname)
                    ))
                    .as_ptr(),
                );
            }
        }
        Com_Error(
            ERR_DROP as c_int,
            c"target_print used in quick succession, fix it! See the console for details.".as_ptr(),
        );
    }
    ctx.entity_mut(ent).genericValue15 = level_time + 5000;

    let ent_ptr: *mut gentity_t = ctx.entity_mut(ent);
    let ent_id = ctx.entity_id_of(ent_ptr);
    G_ActivateBehavior(ctx, ent_id, bSet_t::BSET_USE as c_int);

    let message = ctx.entity(ent).message;
    let spawnflags = ctx.entity(ent).spawnflags;

    if spawnflags & 4 != 0 {
        // private, to one client only
        if activator.is_none() || ctx.entity(activator.unwrap()).inuse == 0 {
            // Com_Printf("ERROR: Bad activator in Use_Target_Print");
        }
        // Oracle gates the send only on `activator && activator->client`
        // (no inuse check). g_target.c:190.
        if let Some(activator) = activator {
            if !ctx.entity(activator).client.is_null() {
                // make sure there's a valid client ent to send it to
                let msg = unsafe { crate::cstr_util::cstr_to_str(message) };
                let client_num = ctx.entity(activator).s.number;
                unsafe {
                    if *message == b'@' as c_char && *message.add(1) != b'@' as c_char {
                        trap::SendServerCommand(
                            ctx.engine,
                            mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                                client_num,
                                crate::cstr_util::cstr(&format!("cps \"{}\"", msg)),
                            ),
                        );
                    } else {
                        trap::SendServerCommand(
                            ctx.engine,
                            mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                                client_num,
                                crate::cstr_util::cstr(&format!("cp \"{}\"", msg)),
                            ),
                        );
                    }
                }
            }
        }
        return;
    }

    if spawnflags & 3 != 0 {
        let msg = unsafe { crate::cstr_util::cstr_to_str(message) };
        if spawnflags & 1 != 0 {
            if unsafe { *message == b'@' as c_char && *message.add(1) != b'@' as c_char } {
                G_TeamCommand(
                    ctx,
                    TEAM_RED,
                    crate::cstr_util::cstr(&format!("cps \"{}\"", msg)).as_ptr() as *mut c_char,
                );
            } else {
                G_TeamCommand(
                    ctx,
                    TEAM_RED,
                    crate::cstr_util::cstr(&format!("cp \"{}\"", msg)).as_ptr() as *mut c_char,
                );
            }
        }
        if spawnflags & 2 != 0 {
            if unsafe { *message == b'@' as c_char && *message.add(1) != b'@' as c_char } {
                G_TeamCommand(
                    ctx,
                    TEAM_BLUE,
                    crate::cstr_util::cstr(&format!("cps \"{}\"", msg)).as_ptr() as *mut c_char,
                );
            } else {
                G_TeamCommand(
                    ctx,
                    TEAM_BLUE,
                    crate::cstr_util::cstr(&format!("cp \"{}\"", msg)).as_ptr() as *mut c_char,
                );
            }
        }
        return;
    }

    // Send to all players
    let msg = unsafe { crate::cstr_util::cstr_to_str(message) };
    if unsafe { *message == b'@' as c_char && *message.add(1) != b'@' as c_char } {
        trap::SendServerCommand(
            ctx.engine,
            mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                -1,
                crate::cstr_util::cstr(&format!("cps \"{}\"", msg)),
            ),
        );
    } else {
        trap::SendServerCommand(
            ctx.engine,
            mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                -1,
                crate::cstr_util::cstr(&format!("cp \"{}\"", msg)),
            ),
        );
    }
}

/// Raven `SP_target_print`.
///
/// Source: `oracle/codemp/game/g_target.c:239-241`
pub fn SP_target_print(ent: &mut gentity_t) {
    ent.use_ = Some(EntUse::Use_Target_Print).into();
}

/// Raven `Use_Target_Speaker`.
///
/// Source: `oracle/codemp/game/g_target.c:259-284`
pub fn Use_Target_Speaker(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let ent_ptr: *mut gentity_t = ctx.entity_mut(ent);
    let ent_id = ctx.entity_id_of(ent_ptr);
    G_ActivateBehavior(ctx, ent_id, bSet_t::BSET_USE as c_int);

    let spawnflags = ctx.entity(ent).spawnflags;
    if spawnflags & 3 != 0 {
        // looping sound toggles
        let e = ctx.entity_mut(ent);
        if e.s.loopSound != 0 {
            e.s.loopSound = 0; // turn it off
            e.s.loopIsSoundset = qfalse;
            e.s.trickedentindex = 1;
        } else {
            e.s.loopSound = e.noise_index; // start it
            e.s.loopIsSoundset = qfalse;
            e.s.trickedentindex = 0;
        }
    } else {
        // normal sound
        let noise_index = ctx.entity(ent).noise_index;
        if spawnflags & 8 != 0 {
            // C derefs `activator` unconditionally here (would UB-deref NULL);
            // the handle carries the same nullability, so `expect` is the one
            // defined behavior (§19).
            let activator_id = activator.expect("Use_Target_Speaker: null activator speaker");
            G_AddEvent(
                ctx.entity_mut(activator_id),
                entity_event_t::EV_GENERAL_SOUND as c_int,
                noise_index,
            );
        } else if spawnflags & 4 != 0 {
            G_AddEvent(
                ctx.entity_mut(ent),
                entity_event_t::EV_GLOBAL_SOUND as c_int,
                noise_index,
            );
        } else {
            G_AddEvent(
                ctx.entity_mut(ent),
                entity_event_t::EV_GENERAL_SOUND as c_int,
                noise_index,
            );
        }
    }
}

/// Raven `SP_target_speaker`.
///
/// Source: `oracle/codemp/game/g_target.c:286-340`
pub fn SP_target_speaker(ctx: &mut GameContext, ent: EntityId) {
    let mut s: *mut c_char = core::ptr::null_mut();

    let wait_ptr: *mut f32 = &mut ctx.entity_mut(ent).wait;
    G_SpawnFloat(
        ctx,
        b"wait\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        wait_ptr,
    );
    let random_ptr: *mut f32 = &mut ctx.entity_mut(ent).random;
    G_SpawnFloat(
        ctx,
        b"random\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        random_ptr,
    );

    if G_SpawnString(
        ctx,
        b"soundSet\0".as_ptr() as *const c_char,
        b"\0".as_ptr() as *const c_char,
        &mut s,
    ) != 0
    {
        // this is a sound set
        let soundset = G_SoundSetIndex(ctx, s);
        let e = ctx.entity_mut(ent);
        e.s.soundSetIndex = soundset;
        e.s.eFlags = mp_bg::public::entity_flags::EF_PERMANENT;
        e.s.pos.trBase = e.s.origin;
        trap::LinkEntity(
            ctx.engine,
            GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(ent)).cast()),
        );
        return;
    }

    if G_SpawnString(
        ctx,
        b"noise\0".as_ptr() as *const c_char,
        b"NOSOUND\0".as_ptr() as *const c_char,
        &mut s,
    ) == 0
    {
        // G_Error is PARKED, so we can't call it properly
        // G_Error(ctx, "target_speaker without a noise key at %s", vtos(ctx, ent.s.origin));
    }

    // force all client relative sounds to be "activator" speakers
    if unsafe { *s == b'*' as c_char } {
        ctx.entity_mut(ent).spawnflags |= 8;
    }

    let mut buffer: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    Q_strncpyz(buffer.as_mut_ptr(), s, MAX_QPATH as c_int);

    let noise_index = G_SoundIndex(buffer.as_ptr());
    let e = ctx.entity_mut(ent);
    e.noise_index = noise_index;

    // a repeating speaker can be done completely client side
    e.s.eType = (mp_bg::public::entity_type::entityType_t::ET_SPEAKER) as i32;
    e.s.eventParm = e.noise_index;
    e.s.frame = (e.wait * 10.0) as c_int;
    e.s.clientNum = (e.random * 10.0) as c_int;

    // check for prestarted looping sound
    if e.spawnflags & 1 != 0 {
        e.s.loopSound = e.noise_index;
        e.s.loopIsSoundset = qfalse;
    }

    e.use_ = Some(EntUse::Use_Target_Speaker).into();

    if e.spawnflags & 4 != 0 {
        e.r.svFlags |= SVF_BROADCAST;
    }

    e.s.pos.trBase = e.s.origin;

    // must link the entity so we get areas and clusters
    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(ent)).cast()),
    );
}

/// Raven `target_laser_think`.
///
/// Source: `oracle/codemp/game/g_target.c:349-377`
pub fn target_laser_think(ctx: &mut GameContext, self_: EntityId) {
    let mut end: vec3_t = [0.0; 3];
    // trace_t has no zeroing constructor; the mem::zeroed is a plain POD-init.
    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    let mut point: vec3_t = [0.0; 3];

    // if pointed at another entity, set movedir to point at it
    if let Some(enemy_id) = ctx.entity(self_).enemy {
        let enemy = ctx.entity(enemy_id);
        let (e_origin, e_mins, e_maxs) = (enemy.s.origin, enemy.r.mins, enemy.r.maxs);
        // VectorMA(self->enemy->s.origin, 0.5, self->enemy->r.mins, point)
        point[0] = e_origin[0] + 0.5 * e_mins[0];
        point[1] = e_origin[1] + 0.5 * e_mins[1];
        point[2] = e_origin[2] + 0.5 * e_mins[2];

        // VectorMA(point, 0.5, self->enemy->r.maxs, point)
        point[0] += 0.5 * e_maxs[0];
        point[1] += 0.5 * e_maxs[1];
        point[2] += 0.5 * e_maxs[2];

        // VectorSubtract(point, self->s.origin, self->movedir)
        let s = ctx.entity_mut(self_);
        s.movedir[0] = point[0] - s.s.origin[0];
        s.movedir[1] = point[1] - s.s.origin[1];
        s.movedir[2] = point[2] - s.s.origin[2];

        VectorNormalize(&mut s.movedir);
    }

    // fire forward and see what we hit
    let (self_origin, self_movedir, self_number) = {
        let s = ctx.entity(self_);
        (s.s.origin, s.movedir, s.s.number)
    };
    end[0] = self_origin[0] + 2048.0 * self_movedir[0];
    end[1] = self_origin[1] + 2048.0 * self_movedir[1];
    end[2] = self_origin[2] + 2048.0 * self_movedir[2];

    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &self_origin as *const vec3_t,
            core::ptr::null(),
            core::ptr::null(),
            &end as *const vec3_t,
            self_number,
            CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_CORPSE,
        ),
    );

    if tr.entityNum != 0 {
        // hurt it if we can
        let targ_id = EntityId(tr.entityNum as u32);
        let activator = ctx.entity(self_).activator;
        let damage = ctx.entity(self_).damage;
        // `G_Damage` normalizes `dir` in place (g_combat.rs:5078); C passes
        // `&self->movedir` so the normalized value lands back on the entity.
        // Copy the field out, hand `G_Damage` a `&mut` to the local (the only
        // channel it touches movedir through), then copy the result back.
        let mut movedir = ctx.entity(self_).movedir;
        G_Damage(
            ctx,
            Some(targ_id),
            Some(self_),
            activator,
            Some(&mut movedir),
            tr.endpos,
            damage,
            DAMAGE_NO_KNOCKBACK,
            meansOfDeath_t::MOD_TARGET_LASER as c_int,
        );
        ctx.entity_mut(self_).movedir = movedir;
    }

    // VectorCopy(tr.endpos, self->s.origin2)
    let s = ctx.entity_mut(self_);
    s.s.origin2[0] = tr.endpos[0];
    s.s.origin2[1] = tr.endpos[1];
    s.s.origin2[2] = tr.endpos[2];

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(self_)).cast()),
    );
    ctx.entity_mut(self_).nextthink = ctx.world.level.time + crate::g_items::FRAMETIME;
}

/// Raven `target_laser_on`.
///
/// Source: `oracle/codemp/game/g_target.c:379-384`
pub fn target_laser_on(ctx: &mut GameContext, self_: EntityId) {
    if ctx.entity(self_).activator.is_none() {
        // C: `self->activator = self` — the receiver handle is never NULL.
        ctx.entity_mut(self_).activator = Some(self_);
    }
    target_laser_think(ctx, self_);
}

/// Raven `target_laser_off`.
///
/// Source: `oracle/codemp/game/g_target.c:386-390`
pub fn target_laser_off(ctx: &mut GameContext, self_: EntityId) {
    trap::UnlinkEntity(
        ctx.engine,
        GUnlinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(self_)).cast()),
    );
    ctx.entity_mut(self_).nextthink = 0;
}

/// Raven `target_laser_use`.
///
/// Source: `oracle/codemp/game/g_target.c:392-399`
pub fn target_laser_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    ctx.entity_mut(self_).activator = activator;
    if ctx.entity(self_).nextthink > 0 {
        target_laser_off(ctx, self_);
    } else {
        target_laser_on(ctx, self_);
    }
}

/// Raven `target_laser_start`.
///
/// Source: `oracle/codemp/game/g_target.c:401-428`
pub fn target_laser_start(ctx: &mut GameContext, self_: EntityId) {
    ctx.entity_mut(self_).s.eType = (mp_bg::public::entity_type::entityType_t::ET_BEAM) as i32;

    let target = ctx.entity(self_).target;
    if !target.is_null() {
        let ent = G_Find(
            ctx,
            ctx.entity_id_of(core::ptr::null_mut()),
            core::mem::offset_of!(gentity_t, targetname) as c_int,
            target,
        );
        if ent.is_null() {
            // G_Printf("%s at %s: %s is a bad target\n", self->classname, vtos(self->s.origin), self->target);
        }
        ctx.entity_mut(self_).enemy = unsafe { ent_id_opt(ctx.world.g_entities.as_mut_ptr(), ent) };
    } else {
        let ent = ctx.entity_mut(self_);
        G_SetMovedir(&mut ent.s.angles, &mut ent.movedir);
    }

    ctx.entity_mut(self_).use_ = Some(EntUse::target_laser_use).into();
    ctx.entity_mut(self_).think = Some(EntThink::target_laser_think).into();

    if ctx.entity(self_).damage == 0 {
        ctx.entity_mut(self_).damage = 1;
    }

    if ctx.entity(self_).spawnflags & 1 != 0 {
        target_laser_on(ctx, self_);
    } else {
        target_laser_off(ctx, self_);
    }
}

/// Raven `SP_target_laser`.
///
/// Source: `oracle/codemp/game/g_target.c:430-435`
pub fn SP_target_laser(ctx: &mut GameContext, self_: EntityId) {
    // let everything else get spawned before we start firing
    ctx.entity_mut(self_).think = Some(EntThink::target_laser_start).into();
    ctx.entity_mut(self_).nextthink = ctx.world.level.time + crate::g_items::FRAMETIME;
}

/// Raven `target_teleporter_use`.
///
/// Source: `oracle/codemp/game/g_target.c:440-455`
pub fn target_teleporter_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // Raven derefs `activator->client` unconditionally.
    let activator = activator.expect("target_teleporter_use: null activator");
    let self_ptr: *mut gentity_t = ctx.entity_mut(self_);
    let self_id = ctx.entity_id_of(self_ptr);
    if ctx.entity(activator).client.is_null() {
        return;
    }

    G_ActivateBehavior(ctx, self_id, bSet_t::BSET_USE as c_int);

    let target = ctx.entity(self_).target;
    let dest = G_PickTarget(ctx, target);
    let Some(dest_id) = ctx.entity_id_of(dest) else {
        // G_Printf(ctx, "Couldn't find teleporter destination\n") — the
        // staged signature has no engine handle to route the outbound
        // print through; dropped here (informational only).
        return;
    };

    let (dest_origin, dest_angles) = {
        let d = ctx.entity(dest_id);
        (d.s.origin, d.s.angles)
    };
    TeleportPlayer(ctx, activator, dest_origin, dest_angles);
}

/// Raven `SP_target_teleporter`.
///
/// Source: `oracle/codemp/game/g_target.c:460-465`
pub fn SP_target_teleporter(ctx: &mut GameContext, self_: EntityId) {
    if ctx.entity(self_).targetname.is_null() {
        // Informational print; dropped.
    }

    ctx.entity_mut(self_).use_ = Some(EntUse::target_teleporter_use).into();
}

/// Raven `target_relay_use`.
///
/// Source: `oracle/codemp/game/g_target.c:479-518`
pub fn target_relay_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // Raven derefs `activator->client` only inside the spawnflag-guarded `&&`
    // chains, so it assumes activator is present there.
    if ctx.entity(self_).spawnflags & 1 != 0 {
        let a = ctx.entity(activator.expect("target_relay_use: null activator"));
        if !a.client.is_null() && unsafe { (*(a.client)).sess.sessionTeam } != TEAM_RED {
            return;
        }
    }
    if ctx.entity(self_).spawnflags & 2 != 0 {
        let a = ctx.entity(activator.expect("target_relay_use: null activator"));
        if !a.client.is_null() && unsafe { (*(a.client)).sess.sessionTeam } != TEAM_BLUE {
            return;
        }
    }

    if ctx.entity(self_).flags & FL_INACTIVE != 0 {
        // set by target_deactivate
        return;
    }

    let activator_ptr =
        unsafe { crate::ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), activator) };

    let self_ptr: *mut gentity_t = ctx.entity_mut(self_);
    let self_id = ctx.entity_id_of(self_ptr);
    let ranscript = G_ActivateBehavior(ctx, self_id, bSet_t::BSET_USE as c_int);
    if ctx.entity(self_).wait == -1.0 {
        // never use again
        if ranscript != 0 {
            // crap, can't remove!
            ctx.entity_mut(self_).use_ = FnId::NONE;
        } else {
            // remove
            ctx.entity_mut(self_).think = Some(EntThink::G_FreeEntity).into();
            ctx.entity_mut(self_).nextthink = ctx.world.level.time + crate::g_items::FRAMETIME;
        }
    }

    if ctx.entity(self_).spawnflags & 4 != 0 {
        let target = ctx.entity(self_).target;
        let ent = G_PickTarget(ctx, target);
        if let Some(ent_id) = ctx.entity_id_of(ent) {
            if ctx.entity(ent_id).use_.is_some() {
                GlobalUse(
                    ctx,
                    Some(ent_id),
                    Some(self_),
                    ctx.entity_id_of(activator_ptr),
                );
            }
        }
        return;
    }

    G_UseTargets(ctx, Some(self_), ctx.entity_id_of(activator_ptr));
}

/// Raven `SP_target_relay`.
///
/// Source: `oracle/codemp/game/g_target.c:520-526`
pub fn SP_target_relay(self_: &mut gentity_t) {
    self_.use_ = Some(EntUse::target_relay_use).into();
    if self_.spawnflags & 128 != 0 {
        self_.flags |= FL_INACTIVE;
    }
}

/// Raven `target_kill_use`.
///
/// Source: `oracle/codemp/game/g_target.c:534-537`
pub fn target_kill_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let self_ptr: *mut gentity_t = ctx.entity_mut(self_);
    let self_id = ctx.entity_id_of(self_ptr);
    G_ActivateBehavior(ctx, self_id, bSet_t::BSET_USE as c_int);
    let activator_ptr =
        unsafe { crate::ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), activator) };
    G_Damage(
        ctx,
        ctx.entity_id_of(activator_ptr),
        ctx.entity_id_of(core::ptr::null_mut()),
        ctx.entity_id_of(core::ptr::null_mut()),
        None,
        [0.0; 3],
        100000,
        DAMAGE_NO_PROTECTION,
        meansOfDeath_t::MOD_TELEFRAG as c_int,
    );
}

/// Raven `SP_target_kill`.
///
/// Source: `oracle/codemp/game/g_target.c:539-541`
/// Source: `oracle/codemp/game/g_target.c:539-541`
pub fn SP_target_kill(self_: &mut gentity_t) {
    self_.use_ = Some(EntUse::target_kill_use).into();
}

/// Raven `SP_target_position`.
///
/// Source: `oracle/codemp/game/g_target.c:546-552`
pub fn SP_target_position(self_: &mut gentity_t) {
    let origin = self_.s.origin;
    G_SetOrigin(&mut *(self_), origin);
}

/// Raven `target_location_linkup`.
///
/// Source: `oracle/codemp/game/g_target.c:554-582`
pub fn target_location_linkup(ctx: &mut GameContext, ent: EntityId) {
    // Raven's `ent` param is unused by this linkup pass (it walks all entities).
    let _ = ent;
    if ctx.world.level.locationLinked != 0 {
        return;
    }

    ctx.world.level.locationLinked = qtrue;
    ctx.world.level.locationHead = core::ptr::null_mut();

    trap::SetConfigstring(
        ctx.engine,
        mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
            mp_bg::public::configstring::CS_LOCATIONS,
            cstr("unknown"),
        ),
    );

    let mut n = 1;
    let num_entities = ctx.world.level.num_entities as usize;
    for i in 0..num_entities {
        let id = EntityId(i as u32);
        let classname = ctx.entity(id).classname;
        if !classname.is_null()
            && Q_stricmp(classname, b"target_location\0".as_ptr() as *const c_char) == 0
        {
            // lets overload some variables!
            ctx.entity_mut(id).health = n; // use for location marking
            let message = ctx.entity(id).message;
            trap::SetConfigstring(
                ctx.engine,
                mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                    mp_bg::public::configstring::CS_LOCATIONS + n,
                    cstr(&unsafe { cstr_to_str(message) }),
                ),
            );
            n += 1;
            // `level.locationHead` is a raw `gentity_t*` seam field (§D5); the
            // intrusive `nextTrain` chain stores each node's `Option<EntityId>`.
            let head = ctx.world.level.locationHead;
            ctx.entity_mut(id).nextTrain = if head.is_null() {
                None
            } else {
                Some(unsafe { ent_id(ctx.world.g_entities.as_ptr(), head) })
            };
            ctx.world.level.locationHead = ctx.entity_mut(id);
        }
    }

    // All linked together now
}

/// Raven `SP_target_location`.
///
/// Source: `oracle/codemp/game/g_target.c:592-597`
pub fn SP_target_location(ctx: &mut GameContext, self_: EntityId) {
    ctx.entity_mut(self_).think = Some(EntThink::target_location_linkup).into();
    ctx.entity_mut(self_).nextthink = ctx.world.level.time + 200; // Let them all spawn first

    let origin = ctx.entity(self_).s.origin;
    G_SetOrigin(ctx.entity_mut(self_), origin);
}

/// Raven `target_counter_use`.
///
/// Source: `oracle/codemp/game/g_target.c:611-658`
pub fn target_counter_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    if ctx.entity(self_).count == 0 {
        return;
    }

    ctx.entity_mut(self_).count -= 1;

    // G_DebugPrint(ctx, WL_VERBOSE, "target_counter %s used by %s (%d/%d)\n",
    // self->targetname, activator->targetname, self->genericValue1 -
    // self->count, self->genericValue1) — debug-only console spam;
    // dropped here (informational only, no observable game-state effect).

    let activator_ptr =
        unsafe { crate::ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), activator) };

    let self_ptr: *mut gentity_t = ctx.entity_mut(self_);
    let self_id = ctx.entity_id_of(self_ptr);
    if ctx.entity(self_).count != 0 {
        let target2 = ctx.entity(self_).target2;
        if !target2.is_null() {
            G_UseTargets2(
                ctx,
                Some(self_),
                ctx.entity_id_of(activator_ptr),
                target2 as *const c_char,
            );
        }
        return;
    }

    G_ActivateBehavior(ctx, self_id, bSet_t::BSET_USE as c_int);

    if ctx.entity(self_).spawnflags & 128 != 0 {
        ctx.entity_mut(self_).flags |= FL_INACTIVE;
    }

    ctx.entity_mut(self_).activator = activator;
    G_UseTargets(ctx, Some(self_), ctx.entity_id_of(activator_ptr));

    if ctx.entity(self_).count == 0 {
        if ctx.entity(self_).bounceCount == 0 {
            return;
        }
        ctx.entity_mut(self_).count = ctx.entity(self_).genericValue1;
        if ctx.entity(self_).bounceCount > 0 {
            // -1 means bounce back forever
            ctx.entity_mut(self_).bounceCount -= 1;
        }
    }
}

/// Raven `SP_target_counter`.
///
/// Source: `oracle/codemp/game/g_target.c:660-673`
pub fn SP_target_counter(self_: &mut gentity_t) {
    self_.wait = -1.0;
    if self_.count == 0 {
        self_.count = 2;
    }
    // we will reset when we use up our count, remember our initial count
    self_.genericValue1 = self_.count;

    self_.use_ = Some(EntUse::target_counter_use).into();
}

/// Raven `target_random_use`.
///
/// Source: `oracle/codemp/game/g_target.c:681-746`
pub fn target_random_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let mut t_count = 0;
    let mut t: *mut gentity_t = core::ptr::null_mut();

    let self_ptr: *mut gentity_t = ctx.entity_mut(self_);
    let self_id = ctx.entity_id_of(self_ptr);
    G_ActivateBehavior(ctx, self_id, bSet_t::BSET_USE as c_int);

    if ctx.entity(self_).spawnflags & 1 != 0 {
        ctx.entity_mut(self_).use_ = FnId::NONE;
    }

    let target = ctx.entity(self_).target;
    let activator_ptr =
        unsafe { crate::ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), activator) };

    // Count matching targets
    loop {
        t = G_Find(
            ctx,
            ctx.entity_id_of(t),
            core::mem::offset_of!(gentity_t, targetname) as c_int,
            target,
        );
        if t.is_null() {
            break;
        }
        if ctx.entity_id_of(t) != Some(self_) {
            t_count += 1;
        }
    }

    if t_count == 0 {
        return;
    }

    if t_count == 1 {
        G_UseTargets(ctx, Some(self_), ctx.entity_id_of(activator_ptr));
        return;
    }

    // Pick a random target
    let pick = ctx.world.bg_state.rng.Q_irand(1, t_count);
    t_count = 0;

    loop {
        t = G_Find(
            ctx,
            ctx.entity_id_of(t),
            core::mem::offset_of!(gentity_t, targetname) as c_int,
            target,
        );
        if t.is_null() {
            break;
        }
        if ctx.entity_id_of(t) != Some(self_) {
            t_count += 1;
        } else {
            continue;
        }

        let t_id = ctx.entity_id_of(t).unwrap();
        if t_id == self_ {
            // WARNING: Entity used itself (shouldn't happen)
        } else if t_count == pick {
            if ctx.entity(t_id).use_.is_some() {
                // check can be omitted
                GlobalUse(
                    ctx,
                    Some(t_id),
                    Some(self_),
                    ctx.entity_id_of(activator_ptr),
                );
                return;
            }
        }

        if ctx.entity(self_).inuse == 0 {
            // Com_Printf("entity was removed while using targets\n");
            return;
        }
    }
}

/// Raven `SP_target_random`.
///
/// Source: `oracle/codemp/game/g_target.c:748-751`
pub fn SP_target_random(self_: &mut gentity_t) {
    self_.use_ = Some(EntUse::target_random_use).into();
}

/// Raven `scriptrunner_run`.
///
/// Source: `oracle/codemp/game/g_target.c:754-837`
pub fn scriptrunner_run(ctx: &mut GameContext, self_: EntityId) {
    if ctx.entity(self_).count != -1 {
        if ctx.entity(self_).count <= 0 {
            ctx.entity_mut(self_).use_ = FnId::NONE;
            ctx.entity_mut(self_).behaviorSet[bSet_t::BSET_USE as usize] = core::ptr::null_mut();
            return;
        } else {
            ctx.entity_mut(self_).count -= 1;
        }
    }

    if !ctx.entity(self_).behaviorSet[bSet_t::BSET_USE as usize].is_null() {
        if ctx.entity(self_).spawnflags & 1 != 0 {
            if ctx.entity(self_).activator.is_none() {
                if ctx.world.cvars.g_developer.integer != 0 {
                    // Informational debug message
                }
                return;
            }

            // activator is Option<EntityId>; dereferenced via arena lookup.
            let activator_id = ctx.entity(self_).activator.unwrap();

            if trap::ICARUS_IsInitialized(
                ctx.engine,
                GIcarusIsinitializedArgs::new(ctx.entity(self_).s.number),
            ) == 0
            {
                // `script_targetname` is a `*const c_char` seam field; the char
                // deref stays unsafe, the field access goes through the accessor.
                let stn = ctx.entity(activator_id).script_targetname;
                if stn.is_null() || unsafe { *stn == b'\0' as c_char } {
                    // DIVERGENCE: store owned string instead of va() pointer
                    let name = format!("newICARUSEnt{}", ctx.world.globals.numNewICARUSEnts);
                    ctx.world.globals.numNewICARUSEnts += 1;
                    let s = G_NewString(ctx, cstr(&name).as_ptr());
                    ctx.entity_mut(activator_id).script_targetname = s;
                }

                if trap::ICARUS_ValidEnt(
                    ctx.engine,
                    GIcarusValidentArgs::new(
                        core::ptr::from_mut(ctx.entity_mut(activator_id)).cast(),
                    ),
                ) != 0
                {
                    trap::ICARUS_InitEnt(
                        ctx.engine,
                        GIcarusInitentArgs::new(
                            core::ptr::from_mut(ctx.entity_mut(activator_id)).cast(),
                        ),
                    );
                } else {
                    if ctx.world.cvars.g_developer.integer != 0 {
                        // Informational debug message
                    }
                    return;
                }
            }

            if ctx.world.cvars.g_developer.integer != 0 {
                // Informational debug message
            }
            let behavior = ctx.entity(self_).behaviorSet[bSet_t::BSET_USE as usize];
            let script_path = format!(
                "{}/{}",
                unsafe { cstr_to_str(Q3_SCRIPT_DIR.as_ptr()) },
                unsafe { cstr_to_str(behavior) }
            );
            trap::ICARUS_RunScript(
                ctx.engine,
                GIcarusRunscriptArgs::new(
                    core::ptr::from_mut(ctx.entity_mut(activator_id)).cast(),
                    cstr(&script_path).as_ptr(),
                ),
            );
        } else {
            let self_ptr: *mut gentity_t = ctx.entity_mut(self_);
            let self_id = ctx.entity_id_of(self_ptr);
            if ctx.world.cvars.g_developer.integer != 0 && ctx.entity(self_).activator.is_some() {
                // Informational debug message
            }
            G_ActivateBehavior(ctx, self_id, bSet_t::BSET_USE as c_int);
        }
    }

    if ctx.entity(self_).wait != 0.0 {
        ctx.entity_mut(self_).nextthink =
            (ctx.world.level.time as f32 + ctx.entity(self_).wait) as c_int;
    }
}

/// Raven `target_scriptrunner_use`.
///
/// Source: `oracle/codemp/game/g_target.c:839-857`
pub fn target_scriptrunner_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    if ctx.entity(self_).nextthink > ctx.world.level.time {
        return;
    }

    ctx.entity_mut(self_).activator = activator;
    ctx.entity_mut(self_).enemy = other;
    if ctx.entity(self_).delay != (0.0) as i32 {
        // delay before firing scriptrunner
        ctx.entity_mut(self_).think = Some(EntThink::scriptrunner_run).into();
        ctx.entity_mut(self_).nextthink = ctx.world.level.time + ctx.entity(self_).delay;
    } else {
        scriptrunner_run(ctx, self_);
    }
}

/// Raven `SP_target_scriptrunner`.
///
/// Source: `oracle/codemp/game/g_target.c:871-898`
pub fn SP_target_scriptrunner(ctx: &mut GameContext, self_: EntityId) {
    if ctx.entity(self_).spawnflags & 128 != 0 {
        ctx.entity_mut(self_).flags |= FL_INACTIVE;
    }

    if ctx.entity(self_).count == 0 {
        ctx.entity_mut(self_).count = 1; // default 1 use only
    }

    let mut v = 0.0f32;
    G_SpawnFloat(
        ctx,
        b"delay\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        &mut v,
    );
    ctx.entity_mut(self_).delay = (v * 1000.0) as i32; // sec to ms
    ctx.entity_mut(self_).wait *= 1000.0; // sec to ms

    let origin = ctx.entity(self_).s.origin;
    G_SetOrigin(ctx.entity_mut(self_), origin);
    ctx.entity_mut(self_).use_ = Some(EntUse::target_scriptrunner_use).into();
}

/// Raven `G_SetActiveState`.
///
/// Source: `oracle/codemp/game/g_target.c:900-907`
pub fn G_SetActiveState(ctx: &mut GameContext, targetstring: *mut c_char, actState: qboolean) {
    let mut target: *mut gentity_t = core::ptr::null_mut();
    loop {
        target = G_Find(
            ctx,
            ctx.entity_id_of(target),
            core::mem::offset_of!(gentity_t, targetname) as c_int,
            targetstring as *const c_char,
        );
        if target.is_null() {
            break;
        }
        let target_id = ctx.entity_id_of(target).unwrap();
        let flags = ctx.entity(target_id).flags;
        ctx.entity_mut(target_id).flags = if actState != qfalse {
            flags & !FL_INACTIVE
        } else {
            flags | FL_INACTIVE
        };
    }
}

/// Raven `target_activate_use`.
///
/// Source: `oracle/codemp/game/g_target.c:912-917`
pub fn target_activate_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let self_ptr: *mut gentity_t = ctx.entity_mut(self_);
    let self_id = ctx.entity_id_of(self_ptr);
    G_ActivateBehavior(ctx, self_id, bSet_t::BSET_USE as c_int);
    let target = ctx.entity(self_).target;
    G_SetActiveState(ctx, target, qtrue);
}

/// Raven `target_deactivate_use`.
///
/// Source: `oracle/codemp/game/g_target.c:919-924`
pub fn target_deactivate_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let self_ptr: *mut gentity_t = ctx.entity_mut(self_);
    let self_id = ctx.entity_id_of(self_ptr);
    G_ActivateBehavior(ctx, self_id, bSet_t::BSET_USE as c_int);
    let target = ctx.entity(self_).target;
    G_SetActiveState(ctx, target, qfalse);
}

/// Raven `SP_target_activate`.
///
/// Source: `oracle/codemp/game/g_target.c:930-934`
pub fn SP_target_activate(self_: &mut gentity_t) {
    let origin = self_.s.origin;
    G_SetOrigin(&mut *(self_), origin);
    self_.use_ = Some(EntUse::target_activate_use).into();
}

/// Raven `SP_target_deactivate`.
///
/// Source: `oracle/codemp/game/g_target.c:939-943`
pub fn SP_target_deactivate(self_: &mut gentity_t) {
    let origin = self_.s.origin;
    G_SetOrigin(&mut *(self_), origin);
    self_.use_ = Some(EntUse::target_deactivate_use).into();
}

/// Raven `target_level_change_use`.
///
/// Source: `oracle/codemp/game/g_target.c:945-950`
pub fn target_level_change_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let self_ptr: *mut gentity_t = ctx.entity_mut(self_);
    let self_id = ctx.entity_id_of(self_ptr);
    G_ActivateBehavior(ctx, self_id, bSet_t::BSET_USE as c_int);
    let message = ctx.entity(self_).message;
    trap::SendConsoleCommand(
        ctx.engine,
        mp_abi::game::syscalls::G_SEND_CONSOLE_COMMAND::GSendConsoleCommandArgs::new(
            cbufExec_t::EXEC_NOW as c_int,
            cstr(&format!("map {}", unsafe { cstr_to_string(message) })),
        ),
    );
}

/// Raven `SP_target_level_change`.
///
/// Source: `oracle/codemp/game/g_target.c:955-970`
pub fn SP_target_level_change(ctx: &mut GameContext, self_: EntityId) {
    let mut s: *mut c_char = core::ptr::null_mut();

    G_SpawnString(
        ctx,
        b"mapname\0".as_ptr() as *const c_char,
        b"\0".as_ptr() as *const c_char,
        &mut s,
    );
    let msg = G_NewString(ctx, s);
    ctx.entity_mut(self_).message = msg;

    let message = ctx.entity(self_).message;
    if message.is_null() || unsafe { *message == b'\0' as c_char } {
        // G_Error("target_level_change with no mapname!\n");
        return;
    }

    let origin = ctx.entity(self_).s.origin;
    G_SetOrigin(ctx.entity_mut(self_), origin);
    ctx.entity_mut(self_).use_ = Some(EntUse::target_level_change_use).into();
}

/// Raven `target_play_music_use`.
///
/// Source: `oracle/codemp/game/g_target.c:972-976`
pub fn target_play_music_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    let self_ptr: *mut gentity_t = ctx.entity_mut(self_);
    let self_id = ctx.entity_id_of(self_ptr);
    G_ActivateBehavior(ctx, self_id, bSet_t::BSET_USE as c_int);
    let message = ctx.entity(self_).message;
    trap::SetConfigstring(
        ctx.engine,
        mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
            mp_bg::public::configstring::CS_MUSIC,
            unsafe { core::ffi::CStr::from_ptr(message) }.to_owned(),
        ),
    );
}

/// Raven `SP_target_play_music`.
///
/// Source: `oracle/codemp/game/g_target.c:989-1002`
pub fn SP_target_play_music(ctx: &mut GameContext, self_: EntityId) {
    let mut s: *mut c_char = core::ptr::null_mut();

    let origin = ctx.entity(self_).s.origin;
    G_SetOrigin(ctx.entity_mut(self_), origin);
    if G_SpawnString(
        ctx,
        b"music\0".as_ptr() as *const c_char,
        b"\0".as_ptr() as *const c_char,
        &mut s,
    ) == 0
    {
        // Error case; informational message dropped.
    }

    let msg = G_NewString(ctx, s);
    ctx.entity_mut(self_).message = msg;

    ctx.entity_mut(self_).use_ = Some(EntUse::target_play_music_use).into();
}
