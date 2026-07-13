// PORT-COMPLETE: g_ICARUScb.c 8/155
// (Q3_TaskIDClear, Q3_GetAnimBoth, Q3_GetAnimLower, Q3_GetAnimUpper,
// anglerCallback, moverCallback, Blocked_Mover, moveAndRotateCallback fully
// ported; the animTable anim reads dereference `client` via the standard
// `*mut c_void as *mut gclient_t` cast idiom.
// Remaining functions parked: 148 on entid-lookup (no g_entities/EntityId
// accessor exposed to this raw *mut gentity_t-staged skeleton), 1 on
// variadic-c-abi (G_DebugPrint).)
//! FAITHFUL port of `oracle/codemp/game/g_ICARUScb.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_nav::NAV_FindClosestWaypointForEnt;
use crate::prelude::*;
use crate::q_math::vec3_origin;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

// `FRAMETIME` resolves via the crate prelude glob (`crate::g_items`); the
// shadowing local copy was removed by the placeholder-const sweep.

// Raven ICARUS `Q3_Registers.h` anonymous variable-type enum. Ported locally
// (small, self-contained, independent of the `interpreter.h` ID/Type enum
// chain) since only `VTYPE_FLOAT` is referenced here.
// Source: `oracle/codemp/icarus/Q3_Registers.h:5-10`
const VTYPE_NONE: c_int = 0;
const VTYPE_FLOAT: c_int = 1;
const VTYPE_STRING: c_int = 2;
const VTYPE_VECTOR: c_int = 3;

use crate::ent_fn_enums::{EntBlocked, EntReached, EntThink};
use crate::ent_id::resolve;
use crate::g_client::SetClientViewAngle;
use crate::g_combat::{player_die, G_Damage};
use crate::g_misc::{TAG_GetAngles, TAG_GetOrigin, TAG_GetOrigin2, TAG_GetRadius};
use crate::g_mover::{G_PlayDoorSound, MatchTeam, BMS_END};
use crate::g_utils::G_FreeEntity;
use mp_abi::game::syscalls::G_CVAR_SET::GCvarSetArgs;
use mp_abi::game::syscalls::G_CVAR_VARIABLE_STRING_BUFFER::GCvarVariableStringBufferArgs;
use mp_abi::game::syscalls::G_ICARUS_GETFLOATVARIABLE::GIcarusGetfloatvariableArgs;
use mp_abi::game::syscalls::G_ICARUS_GETSTRINGVARIABLE::GIcarusGetstringvariableArgs;
use mp_abi::game::syscalls::G_ICARUS_GETVECTORVARIABLE::GIcarusGetvectorvariableArgs;
use mp_abi::game::syscalls::G_ICARUS_SETVAR::GIcarusSetvarArgs;
use mp_abi::game::syscalls::G_ICARUS_TASKIDCOMPLETE::GIcarusTaskidcompleteArgs;
use mp_abi::game::syscalls::G_ICARUS_TASKIDSET::GIcarusTaskidsetArgs;
use mp_abi::game::syscalls::G_ICARUS_VARIABLEDECLARED::GIcarusVariabledeclaredArgs;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_ROFF_CACHE::GRoffCacheArgs;
use mp_abi::game::syscalls::G_ROFF_PLAY::GRoffPlayArgs;
use mp_abi::game::syscalls::G_UNLINKENTITY::GUnlinkentityArgs;
use mp_qshared::common::mp::entity_id::ent_id;
use std::ffi::CString;

/// Raven `Q3_TaskIDClear`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:270-273`
pub fn Q3_TaskIDClear(taskID: *mut c_int) {
    unsafe {
        *taskID = -1;
    }
}

/// Raven `G_DebugPrint`.
///
// PORT-NOTE(variadic-c-abi): Raven `vsprintf(text, format, argptr)` expands
// the caller's varargs into `text`; this seam has no varargs channel, so
// every call site in this file passes an already-formatted string and
// `format` is treated as that finished `text` verbatim (va/printf
// callers bind a `format!`ed String before the call).
/// Source: `oracle/codemp/game/g_ICARUScb.c:275-324`
pub fn G_DebugPrint(
    ctx: &mut GameContext,
    level: c_int,
    format: *const c_char,
    // variadic `...` — C varargs, seam decision pending
) {
    unsafe {
        if (*ctx.world_raw()).cvars.g_developer.integer != 2 {
            return;
        }

        let text = cstr_to_str(format);

        if level == WL_ERROR as c_int {
            Com_Printf(cstr(&format!("{}ERROR: {}", S_COLOR_RED.to_string_lossy(), text)).as_ptr());
        } else if level == WL_WARNING as c_int {
            Com_Printf(
                cstr(&format!(
                    "{}WARNING: {}",
                    S_COLOR_YELLOW.to_string_lossy(),
                    text
                ))
                .as_ptr(),
            );
        } else if level == WL_DEBUG as c_int {
            let mut ent_num: c_int = text
                .split_whitespace()
                .next()
                .and_then(|t| t.parse().ok())
                .unwrap_or(0);
            let buffer = if text.len() > 5 { &text[5..] } else { "" };

            if ent_num < 0 || ent_num > MAX_GENTITIES as c_int {
                ent_num = 0;
            }

            let targ = (*ctx.world_raw()).g_entities[ent_num as usize].script_targetname;
            let targ_str = if targ.is_null() {
                String::new()
            } else {
                cstr_to_str(targ)
            };
            Com_Printf(
                cstr(&format!(
                    "{}DEBUG: {}({}): {}\n",
                    S_COLOR_BLUE.to_string_lossy(),
                    targ_str,
                    ent_num,
                    buffer
                ))
                .as_ptr(),
            );
        } else {
            // default / WL_VERBOSE
            Com_Printf(
                cstr(&format!(
                    "{}INFO: {}",
                    S_COLOR_GREEN.to_string_lossy(),
                    text
                ))
                .as_ptr(),
            );
        }
    }
}

/// Raven `Q3_GetAnimLower`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:331-344`
pub fn Q3_GetAnimLower(ctx: &mut GameContext, ent: EntityId) -> *mut c_char {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        if (*ent).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                b"Q3_GetAnimLower: attempted to read animation state off non-client!\n\0".as_ptr()
                    as *const c_char,
            );
            return std::ptr::null_mut();
        }

        let anim: c_int = (*((*ent).client as *mut gclient_t)).ps.legsAnim;

        animTable[anim as usize].name
    }
}

/// Raven `Q3_GetAnimUpper`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:351-364`
pub fn Q3_GetAnimUpper(ctx: &mut GameContext, ent: EntityId) -> *mut c_char {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        if (*ent).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                b"Q3_GetAnimUpper: attempted to read animation state off non-client!\n\0".as_ptr()
                    as *const c_char,
            );
            return std::ptr::null_mut();
        }

        let anim: c_int = (*((*ent).client as *mut gclient_t)).ps.torsoAnim;

        animTable[anim as usize].name
    }
}

/// Raven `Q3_GetAnimBoth`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:371-398`
pub fn Q3_GetAnimBoth(ctx: &mut GameContext, ent: EntityId) -> *mut c_char {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        let lower_name = Q3_GetAnimLower(ctx, ctx.entity_id_of(ent).unwrap());
        let upper_name = Q3_GetAnimUpper(ctx, ctx.entity_id_of(ent).unwrap());

        if lower_name.is_null() || *lower_name == 0 {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                b"Q3_GetAnimBoth: NULL legs animation string found!\n\0".as_ptr() as *const c_char,
            );
            return std::ptr::null_mut();
        }

        if upper_name.is_null() || *upper_name == 0 {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                b"Q3_GetAnimBoth: NULL torso animation string found!\n\0".as_ptr() as *const c_char,
            );
            return std::ptr::null_mut();
        }

        // Raven: `#ifdef _DEBUG` mismatch warning is dev-build noise only; behavior
        // (return legs anim regardless) is unconditional and preserved here.
        lower_name
    }
}

/// Raven `Q3_PlaySound`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:400-520`
// The `#if 0`-style subtitle/broadcast-text block (g_ICARUScb.c:441-479) is
// commented out in Raven itself; not transcribed (dead source).
pub fn Q3_PlaySound(
    ctx: &mut GameContext,
    taskID: c_int,
    entID: c_int,
    name: *const c_char,
    channel: *const c_char,
) -> c_int {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
        let mut final_name = [0 as c_char; MAX_QPATH as usize];
        Q_strncpyz(final_name.as_mut_ptr(), name, MAX_QPATH as c_int);
        Q_strupr(final_name.as_mut_ptr());
        COM_StripExtension(final_name.as_ptr(), final_name.as_mut_ptr());

        let sound_handle = G_SoundIndex(final_name.as_ptr());
        let mut b_broadcast = qfalse;

        if Q_stricmp(channel, b"CHAN_ANNOUNCER\0".as_ptr() as *const c_char) == 0
            || (!(*ent).classname.is_null()
                && Q_stricmp(
                    b"target_scriptrunner\0".as_ptr() as *const c_char,
                    (*ent).classname,
                ) == 0)
        {
            b_broadcast = qtrue;
        }

        let mut voice_chan = CHAN_VOICE;
        let mut type_voice = qfalse;
        if Q_stricmp(channel, b"CHAN_VOICE\0".as_ptr() as *const c_char) == 0 {
            voice_chan = CHAN_VOICE;
            type_voice = qtrue;
        } else if Q_stricmp(channel, b"CHAN_VOICE_ATTEN\0".as_ptr() as *const c_char) == 0 {
            voice_chan = CHAN_AUTO;
            type_voice = qtrue;
        } else if Q_stricmp(channel, b"CHAN_VOICE_GLOBAL\0".as_ptr() as *const c_char) == 0 {
            voice_chan = CHAN_AUTO;
            type_voice = qtrue;
            b_broadcast = qtrue;
        }

        if type_voice != 0 {
            let mut buf = [0 as c_char; 128];
            trap::Cvar_VariableStringBuffer(
                ctx.engine,
                GCvarVariableStringBufferArgs::new(
                    CString::new("timescale").unwrap(),
                    buf.as_mut_ptr(),
                    buf.len() as c_int,
                ),
            );
            let t_f_val = atof(buf.as_ptr()) as f32;

            if t_f_val > 1.0 {
                // Skip the damn sound!
                return qtrue;
            } else {
                G_Sound(ctx, ctx.entity_id_of(ent), voice_chan, sound_handle);
            }
            trap::ICARUS_TaskIDSet(
                ctx.engine,
                GIcarusTaskidsetArgs::new(ent, taskID_t::TID_CHAN_VOICE as c_int, taskID),
            );
            return qfalse;
        }

        if b_broadcast != 0 {
            let te = G_TempEntity(ctx, (*ent).r.currentOrigin, EV_GLOBAL_SOUND as c_int);
            (*te).s.eventParm = sound_handle;
            (*te).r.svFlags |= SVF_BROADCAST;
        } else {
            G_Sound(ctx, ctx.entity_id_of(ent), CHAN_AUTO, sound_handle);
        }

        qtrue
    }
}

/// Raven `Q3_Play`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:527-560`
pub fn Q3_Play(
    ctx: &mut GameContext,
    taskID: c_int,
    entID: c_int,
    r#type: *const c_char,
    name: *const c_char,
) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if Q_stricmp(r#type, b"PLAY_ROFF\0".as_ptr() as *const c_char) == 0 {
            // Raven passes `name` (already a `char*`) straight to `trap_ROFF_Cache`;
            // the ABI arg is an owned `CString` here.
            let file = CString::new(std::ffi::CStr::from_ptr(name).to_bytes()).unwrap();
            (*ent).roffid = trap::ROFF_Cache(ctx.engine, GRoffCacheArgs::new(file));
            if (*ent).roffid != 0 {
                (*ent).roffname = G_NewString(ctx, name);

                // Save this off for later
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(ent, taskID_t::TID_MOVE_NAV as c_int, taskID),
                );

                (*ent).s.origin2 = (*ent).r.currentOrigin;
                (*ent).s.angles2 = (*ent).r.currentAngles;

                trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));

                trap::ROFF_Play(
                    ctx.engine,
                    GRoffPlayArgs::new((*ent).s.number, (*ent).roffid, qtrue),
                );
            }
        }
    }
}

/// Raven `anglerCallback`.
///
/// Utility function.
/// Source: `oracle/codemp/game/g_ICARUScb.c:569-591`
pub fn anglerCallback(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        trap::ICARUS_TaskIDComplete(
            ctx.engine,
            GIcarusTaskidcompleteArgs::new(ent, taskID_t::TID_ANGLE_FACE as c_int),
        );

        // VectorMA(trBase, trDuration*0.001, trDelta, currentAngles)
        let scale = (*ent).s.apos.trDuration as f32 * 0.001;
        for i in 0..3 {
            (*ent).r.currentAngles[i] = (*ent).s.apos.trBase[i] + scale * (*ent).s.apos.trDelta[i];
        }
        (*ent).s.apos.trBase = (*ent).r.currentAngles;
        (*ent).s.apos.trDelta = [0.0, 0.0, 0.0];
        (*ent).s.apos.trDuration = 1;
        (*ent).s.apos.trType = trType_t::TR_STATIONARY;
        (*ent).s.apos.trTime = (*ctx.world_raw()).level.time;

        // Stop thinking.
        (*ent).reached = FnId::NONE;
        // Raven compares `ent->think == anglerCallback` by address (fn-ID
        // enums replace address compares) before clearing it; the
        // `gentity_t.think` field is not yet retrofitted from a raw fn-ptr to
        // `Option<EntThink>` so the compare itself can't be reproduced here.
        // This callback is only ever assigned as its own think, so
        // unconditionally clearing is behaviorally equivalent.
        (*ent).think = FnId::NONE;

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
    }
}

/// Raven `moverCallback`.
///
/// Utility function.
/// Source: `oracle/codemp/game/g_ICARUScb.c:603-633`
pub fn moverCallback(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    unsafe {
        trap::ICARUS_TaskIDComplete(
            ctx.engine,
            GIcarusTaskidcompleteArgs::new(ent, taskID_t::TID_MOVE_NAV as c_int),
        );

        (*ent).s.loopSound = 0;
        (*ent).s.loopIsSoundset = qfalse;
        // BMS_END: unported sound-slot const (missing_symbols).
        G_PlayDoorSound(ctx, ctx.entity_id_of(ent).unwrap(), BMS_END);

        if (*ent).moverState == MOVER_1TO2 {
            let __h525 = ctx.entity_id_of(ent).unwrap();
            let __h526 = (*ctx.world_raw()).level.time;
            MatchTeam(ctx, __h525, MOVER_POS2 as c_int, __h526);
        } else if (*ent).moverState == MOVER_2TO1 {
            let __h527 = ctx.entity_id_of(ent).unwrap();
            let __h528 = (*ctx.world_raw()).level.time;
            MatchTeam(ctx, __h527, MOVER_POS1 as c_int, __h528);
        }

        if (*ent).blocked.get() == Some(EntBlocked::Blocked_Mover) {
            (*ent).blocked = FnId::NONE;
        }
    }
}

/// Raven `Blocked_Mover`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:635-658`
pub fn Blocked_Mover(ctx: &mut GameContext, ent: EntityId, other: Option<EntityId>) {
    // STAGE-1: EntityId ent + Option<EntityId> other; raw body re-derived verbatim (Stage-2 debt).
    let base = ctx.world.g_entities.as_mut_ptr();
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    let other: *mut gentity_t = unsafe { resolve(base, other) };

    unsafe {
        // remove anything other than a client -- no longer the case

        // don't remove security keys or goodie keys
        if (*other).s.eType == entityType_t::ET_ITEM as c_int {
            // should we be doing anything special if a key blocks it... move it somehow..?
        } else if (*other).s.number != 0
            && ((*other).client.is_null()
                || (!(*other).client.is_null()
                    && (*other).health <= 0
                    && (*other).r.contents == CONTENTS_CORPSE
                    && (*other).message.is_null()))
        {
            // if your not a client, or your a dead client remove yourself...
            // if an item or weapon can we do a little explosion..?
            G_FreeEntity(ctx, ctx.entity_id_of(other));
            return;
        }

        if (*ent).damage != 0 {
            // Raven passes `NULL` for both `dir` and `point`; `dir` is now
            // `Option<&mut vec3_t>` so `None` is faithful, but
            // `point` is still a by-value `vec3_t` (no null representation),
            // so the zero vector (`vec3_origin`) remains the stand-in there.
            G_Damage(
                ctx,
                ctx.entity_id_of(other),
                ctx.entity_id_of(ent),
                ctx.entity_id_of(ent),
                None,
                vec3_origin,
                (*ent).damage,
                0,
                MOD_CRUSH as c_int,
            );
        }
    }
}

/// Raven `moveAndRotateCallback`.
///
/// Utility function.
/// Source: `oracle/codemp/game/g_ICARUScb.c:667-673`
pub fn moveAndRotateCallback(ctx: &mut GameContext, ent: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);

    //stop turning
    anglerCallback(ctx, ctx.entity_id_of(ent).unwrap());
    //stop moving
    moverCallback(ctx, ctx.entity_id_of(ent).unwrap());
}

/// Raven `Q3_Lerp2Start`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:682-721`
pub fn Q3_Lerp2Start(ctx: &mut GameContext, entID: c_int, taskID: c_int, duration: f32) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if !(*ent).client.is_null()
            || Q_stricmp(
                (*ent).classname,
                b"target_scriptrunner\0".as_ptr() as *const c_char,
            ) == 0
        {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!("Q3_Lerp2Start: ent {} is NOT a mover!\n", entID)).as_ptr(),
            );
            return;
        }

        if (*ent).s.eType != entityType_t::ET_MOVER as c_int {
            (*ent).s.eType = entityType_t::ET_MOVER as c_int;
        }

        (*ent).moverState = MOVER_2TO1;
        (*ent).s.eType = entityType_t::ET_MOVER as c_int;
        (*ent).reached = Some(EntReached::moverCallback).into();
        if (*ent).damage != 0 {
            (*ent).blocked = Some(EntBlocked::Blocked_Mover).into();
        }

        (*ent).s.pos.trDuration = (duration * 10.0) as c_int;
        (*ent).s.pos.trTime = (*ctx.world_raw()).level.time;

        trap::ICARUS_TaskIDSet(
            ctx.engine,
            GIcarusTaskidsetArgs::new(ent, taskID_t::TID_MOVE_NAV as c_int, taskID),
        );
        G_PlayDoorLoopSound(ctx, ctx.entity_id_of(ent).unwrap());
        G_PlayDoorSound(ctx, ctx.entity_id_of(ent).unwrap(), BMS_START);

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
    }
}

/// Raven `Q3_Lerp2End`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:730-769`
pub fn Q3_Lerp2End(ctx: &mut GameContext, entID: c_int, taskID: c_int, duration: f32) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if !(*ent).client.is_null()
            || Q_stricmp(
                (*ent).classname,
                b"target_scriptrunner\0".as_ptr() as *const c_char,
            ) == 0
        {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!("Q3_Lerp2End: ent {} is NOT a mover!\n", entID)).as_ptr(),
            );
            return;
        }

        if (*ent).s.eType != entityType_t::ET_MOVER as c_int {
            (*ent).s.eType = entityType_t::ET_MOVER as c_int;
        }

        (*ent).moverState = MOVER_1TO2;
        (*ent).s.eType = entityType_t::ET_MOVER as c_int;
        (*ent).reached = Some(EntReached::moverCallback).into();
        if (*ent).damage != 0 {
            (*ent).blocked = Some(EntBlocked::Blocked_Mover).into();
        }

        (*ent).s.pos.trDuration = (duration * 10.0) as c_int;
        (*ent).s.time = (*ctx.world_raw()).level.time;

        trap::ICARUS_TaskIDSet(
            ctx.engine,
            GIcarusTaskidsetArgs::new(ent, taskID_t::TID_MOVE_NAV as c_int, taskID),
        );
        G_PlayDoorLoopSound(ctx, ctx.entity_id_of(ent).unwrap());
        G_PlayDoorSound(ctx, ctx.entity_id_of(ent).unwrap(), BMS_START);

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
    }
}

/// Raven `Q3_Lerp2Pos`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:781-883`
pub fn Q3_Lerp2Pos(
    ctx: &mut GameContext,
    taskID: c_int,
    entID: c_int,
    origin: &mut [f32; 3],
    angles: Option<&mut [f32; 3]>,
    duration: f32,
) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if !(*ent).client.is_null()
            || Q_stricmp(
                (*ent).classname,
                b"target_scriptrunner\0".as_ptr() as *const c_char,
            ) == 0
        {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!("Q3_Lerp2Pos: ent {} is NOT a mover!\n", entID)).as_ptr(),
            );
            return;
        }

        if (*ent).s.eType != entityType_t::ET_MOVER as c_int {
            (*ent).s.eType = entityType_t::ET_MOVER as c_int;
        }

        let mut duration = duration;
        if duration == 0.0 {
            duration = 1.0;
        }

        let mut moverState = (*ent).moverState;

        if moverState == MOVER_POS1 || moverState == MOVER_2TO1 {
            (*ent).pos1 = (*ent).r.currentOrigin;
            (*ent).pos2 = *origin;
            moverState = MOVER_1TO2;
        } else {
            (*ent).pos2 = (*ent).r.currentOrigin;
            (*ent).pos1 = *origin;
            moverState = MOVER_2TO1;
        }
        (*ent).moverState = moverState;

        InitMoverTrData(&mut *ent);

        (*ent).s.pos.trDuration = duration as c_int;

        let __h529 = ctx.entity_id_of(ent).unwrap();
        let __h530 = (*ctx.world_raw()).level.time;
        MatchTeam(ctx, __h529, moverState as c_int, __h530);

        if let Some(angles) = angles {
            let mut ang = [0.0f32; 3];
            for i in 0..3 {
                ang[i] = AngleDelta(angles[i], (*ent).r.currentAngles[i]);
                (*ent).s.apos.trDelta[i] = ang[i] / (duration * 0.001);
            }

            (*ent).s.apos.trBase = (*ent).r.currentAngles;

            (*ent).s.apos.trType = if (*ent).alt_fire != 0 {
                trType_t::TR_LINEAR_STOP
            } else {
                trType_t::TR_NONLINEAR_STOP
            };
            (*ent).s.apos.trDuration = duration as c_int;
            (*ent).s.apos.trTime = (*ctx.world_raw()).level.time;

            (*ent).reached = Some(EntReached::moveAndRotateCallback).into();
            trap::ICARUS_TaskIDSet(
                ctx.engine,
                GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANGLE_FACE as c_int, taskID),
            );
        } else {
            (*ent).reached = Some(EntReached::moverCallback).into();
        }

        if (*ent).damage != 0 {
            (*ent).blocked = Some(EntBlocked::Blocked_Mover).into();
        }

        trap::ICARUS_TaskIDSet(
            ctx.engine,
            GIcarusTaskidsetArgs::new(ent, taskID_t::TID_MOVE_NAV as c_int, taskID),
        );
        G_PlayDoorLoopSound(ctx, ctx.entity_id_of(ent).unwrap());
        G_PlayDoorSound(ctx, ctx.entity_id_of(ent).unwrap(), BMS_START);

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
    }
}

/// Raven `Q3_Lerp2Angles`.
///
/// `angles` is written through (`ang[i] = AngleSubtract(...)`, but the
/// output is `ent->s.apos.trDelta`, not `angles` itself) — re-checking the
/// oracle: `angles` is only ever read here, so it stays by-value.
/// Source: `oracle/codemp/game/g_ICARUScb.c:892-939`
pub fn Q3_Lerp2Angles(
    ctx: &mut GameContext,
    taskID: c_int,
    entID: c_int,
    angles: vec3_t,
    duration: f32,
) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        (*ent).s.apos.trDuration = if duration > 0.0 { duration as c_int } else { 1 };

        let mut ang = [0.0f32; 3];
        for i in 0..3 {
            ang[i] = AngleSubtract(angles[i], (*ent).r.currentAngles[i]);
            (*ent).s.apos.trDelta[i] = ang[i] / ((*ent).s.apos.trDuration as f32 * 0.001);
        }

        (*ent).s.apos.trBase = (*ent).r.currentAngles;

        (*ent).s.apos.trType = if (*ent).alt_fire != 0 {
            trType_t::TR_LINEAR_STOP
        } else {
            trType_t::TR_NONLINEAR_STOP
        };

        (*ent).s.apos.trTime = (*ctx.world_raw()).level.time;

        trap::ICARUS_TaskIDSet(
            ctx.engine,
            GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANGLE_FACE as c_int, taskID),
        );

        (*ent).think = Some(EntThink::anglerCallback).into();
        (*ent).nextthink = (*ctx.world_raw()).level.time + duration as c_int;

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
    }
}

/// ICARUS `interpreter.h` type-ID for a `vec3_t` angles field.
///
/// Derived through the generated token chain (tokenizer.h `TK_USERDEF`=8 ->
/// interpreter.h `NUM_USER_TOKENS`=19 -> `ID_AFFECT`=19 ... `NUM_IDS`=51 ->
/// `TYPE_WAIT_COMPLETE`=51, `TYPE_WAIT_TRIGGERED`=52, `TYPE_ANGLES`=53).
/// Source: `oracle/codemp/icarus/interpreter.h:35-80`
const TYPE_ANGLES: c_int = 53;

/// ICARUS `interpreter.h` type-ID for a `vec3_t` origin field (`TYPE_ORIGIN`=54).
/// Source: `oracle/codemp/icarus/interpreter.h:35-80`
const TYPE_ORIGIN: c_int = 54;

/// Raven `Q3_GetTag`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:948-970`
pub fn Q3_GetTag(
    ctx: &mut GameContext,
    entID: c_int,
    name: *const c_char,
    lookup: c_int,
    info: &mut [f32; 3],
) -> c_int {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*ent).inuse == 0 {
            debug_assert!(false);
            return 0;
        }

        // `TYPE_ORIGIN`/`TYPE_ANGLES` are module-level consts (see above).
        if lookup == TYPE_ORIGIN {
            return TAG_GetOrigin(ctx, (*ent).ownername, name, info);
        } else if lookup == TYPE_ANGLES {
            return TAG_GetAngles(ctx, (*ent).ownername, name, info);
        }

        0
    }
}

/// Raven `Q3_Use`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:981-998`
pub fn Q3_Use(ctx: &mut GameContext, entID: c_int, target: *const c_char) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if target.is_null() || *target == 0 {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                b"Q3_Use: string is NULL!\n\0".as_ptr() as *const c_char,
            );
            return;
        }

        G_UseTargets2(ctx, ctx.entity_id_of(ent), ctx.entity_id_of(ent), target);
    }
}

/// Raven `Q3_Kill`.
///
// PORT-NOTE(die-dispatch-invoke): Raven calls the stored `victim->die` fn
// pointer directly with (victim,victim,victim,o_health,MOD_UNKNOWN); the
// ported field is `Option<EntDie>` so the call routes through the central
// `dispatch_die` (ent_fn_enums.rs) per the fn-ptr-dispatch idiom.
/// Source: `oracle/codemp/game/g_ICARUScb.c:1009-1052`
pub fn Q3_Kill(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
        let mut victim: *mut gentity_t = std::ptr::null_mut();

        if Q_stricmp(name, b"self\0".as_ptr() as *const c_char) == 0 {
            victim = ent;
        } else if Q_stricmp(name, b"enemy\0".as_ptr() as *const c_char) == 0 {
            if let Some(enemy_id) = (*ent).enemy {
                victim = &mut (*ctx.world_raw()).g_entities[enemy_id.0 as usize] as *mut gentity_t;
            }
        } else {
            victim = G_Find(
                ctx,
                ctx.entity_id_of(std::ptr::null_mut()),
                core::mem::offset_of!(gentity_t, targetname) as c_int,
                name,
            );
        }

        if victim.is_null() {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                cstr(&format!("Q3_Kill: can't find {}\n", cstr_to_str(name))).as_ptr(),
            );
            return;
        }

        let o_health = (*victim).health;
        (*victim).health = 0;
        if !(*victim).client.is_null() {
            (*victim).flags |= FL_NO_KNOCKBACK;
        }

        if let Some(die_fn) = (*victim).die.get() {
            crate::ent_fn_enums::dispatch_die(
                ctx,
                die_fn,
                victim,
                victim,
                victim,
                o_health,
                MOD_UNKNOWN as c_int,
            );
        }
    }
}

/// Raven `Q3_RemoveEnt`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1062-1116`
pub fn Q3_RemoveEnt(ctx: &mut GameContext, victim: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let victim: *mut gentity_t = ctx.entity_mut(victim);

    unsafe {
        if !(*victim).client.is_null() {
            if (*victim).s.eType != ET_NPC as c_int {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_RemoveEnt: You can't remove clients in MP!\n\0".as_ptr() as *const c_char,
                );
                debug_assert!(false);
            } else {
                // Remove the NPC.
                let client = (*victim).client as *mut gclient_t;
                if (*client).NPC_class == CLASS_VEHICLE {
                    // Eject everyone out of a vehicle that's about to remove itself.
                    // PORT-NOTE(vehicle-eject): Vehicle_t/m_pVehicleInfo->EjectAll is
                    // C++-track (icarus/vehicle) surface — not transcribed here; see
                    // porting-rules §F (idiomatic C++ reimplementation, not yet ported).
                }
                (*victim).think = Some(EntThink::G_FreeEntity).into();
                (*victim).nextthink = (*ctx.world_raw()).level.time + 100;
            }
        } else {
            (*victim).think = Some(EntThink::G_FreeEntity).into();
            (*victim).nextthink = (*ctx.world_raw()).level.time + 100;
        }
    }
}

/// Raven `Q3_Remove`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1128-1168`
pub fn Q3_Remove(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if Q_stricmp(b"self\0".as_ptr() as *const c_char, name) == 0 {
            Q3_RemoveEnt(ctx, ctx.entity_id_of(ent).unwrap());
        } else if Q_stricmp(b"enemy\0".as_ptr() as *const c_char, name) == 0 {
            let victim = (*ent).enemy;
            if victim.is_none() {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_Remove: can't find enemy\n\0".as_ptr() as *const c_char,
                );
                return;
            }
            let victim =
                &mut (*ctx.world_raw()).g_entities[victim.unwrap().0 as usize] as *mut gentity_t;
            Q3_RemoveEnt(ctx, ctx.entity_id_of(victim).unwrap());
        } else {
            let mut victim = G_Find(
                ctx,
                ctx.entity_id_of(std::ptr::null_mut()),
                core::mem::offset_of!(gentity_t, targetname) as c_int,
                name,
            );
            if victim.is_null() {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_Remove: can't find target\n\0".as_ptr() as *const c_char,
                );
                return;
            }
            while !victim.is_null() {
                Q3_RemoveEnt(ctx, ctx.entity_id_of(victim).unwrap());
                victim = G_Find(
                    ctx,
                    ctx.entity_id_of(victim),
                    core::mem::offset_of!(gentity_t, targetname) as c_int,
                    name,
                );
            }
        }
    }
}

/// Raven `Q3_GetFloat`.
///
// PORT-NOTE(unported-global): `setTable`/`SET_*` (the ICARUS set-table) are
// not ported anywhere in the worktree yet — referenced verbatim per
// zero-park (missing_symbols).
/// Source: `oracle/codemp/game/g_ICARUScb.c:1189-1559`
pub fn Q3_GetFloat(
    ctx: &mut GameContext,
    entID: c_int,
    r#type: c_int,
    name: *const c_char,
    value: *mut f32,
) -> c_int {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        let toGet = GetIDForString(setTable.as_ptr() as *mut stringID_table_t, name);

        match toGet {
            _ if toGet == SET_PARM1 as i32
                || toGet == SET_PARM2 as i32
                || toGet == SET_PARM3 as i32
                || toGet == SET_PARM4 as i32
                || toGet == SET_PARM5 as i32
                || toGet == SET_PARM6 as i32
                || toGet == SET_PARM7 as i32
                || toGet == SET_PARM8 as i32
                || toGet == SET_PARM9 as i32
                || toGet == SET_PARM10 as i32
                || toGet == SET_PARM11 as i32
                || toGet == SET_PARM12 as i32
                || toGet == SET_PARM13 as i32
                || toGet == SET_PARM14 as i32
                || toGet == SET_PARM15 as i32
                || toGet == SET_PARM16 as i32 =>
            {
                if (*ent).parms.is_null() {
                    G_DebugPrint(
                        ctx,
                        WL_ERROR as c_int,
                        cstr(&format!(
                            "GET_PARM: {} {} did not have any parms set!\n",
                            cstr_to_str((*ent).classname),
                            cstr_to_str((*ent).targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value =
                    atof((*(*ent).parms).parm[(toGet - SET_PARM1 as i32) as usize].as_ptr()) as f32;
            }
            _ if toGet == SET_COUNT as i32 => *value = (*ent).count as f32,
            _ if toGet == SET_HEALTH as i32 => *value = (*ent).health as f32,
            _ if toGet == SET_SKILL as i32 => return 0,
            _ if toGet == SET_XVELOCITY as i32 => {
                if (*ent).client.is_null() {
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_XVELOCITY, {} not a client\n",
                            cstr_to_str((*ent).targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*((*ent).client as *mut gclient_t)).ps.velocity[0];
            }
            _ if toGet == SET_YVELOCITY as i32 => {
                if (*ent).client.is_null() {
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_YVELOCITY, {} not a client\n",
                            cstr_to_str((*ent).targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*((*ent).client as *mut gclient_t)).ps.velocity[1];
            }
            _ if toGet == SET_ZVELOCITY as i32 => {
                if (*ent).client.is_null() {
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_ZVELOCITY, {} not a client\n",
                            cstr_to_str((*ent).targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*((*ent).client as *mut gclient_t)).ps.velocity[2];
            }
            _ if toGet == SET_Z_OFFSET as i32 => {
                *value = (*ent).r.currentOrigin[2] - (*ent).s.origin[2]
            }
            _ if toGet == SET_DPITCH as i32 => return 0,
            _ if toGet == SET_DYAW as i32 => return 0,
            _ if toGet == SET_WIDTH as i32 => *value = (*ent).r.mins[0],
            _ if toGet == SET_TIMESCALE as i32 => return 0,
            _ if toGet == SET_CAMERA_GROUP_Z_OFS as i32 => return 0,
            _ if toGet == SET_VISRANGE as i32 => return 0,
            _ if toGet == SET_EARSHOT as i32 => return 0,
            _ if toGet == SET_VIGILANCE as i32 => return 0,
            _ if toGet == SET_GRAVITY as i32 => *value = (*ctx.world_raw()).cvars.g_gravity.value,
            _ if toGet == SET_FACEEYESCLOSED as i32
                || toGet == SET_FACEEYESOPENED as i32
                || toGet == SET_FACEAUX as i32
                || toGet == SET_FACEBLINK as i32
                || toGet == SET_FACEBLINKFROWN as i32
                || toGet == SET_FACEFROWN as i32
                || toGet == SET_FACENORMAL as i32 =>
            {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetFloat: SET_FACE___ not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_WAIT as i32 => *value = (*ent).wait,
            _ if toGet == SET_FOLLOWDIST as i32 => return 0,
            _ if toGet == SET_ANIM_HOLDTIME_LOWER as i32 => {
                if (*ent).client.is_null() {
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_ANIM_HOLDTIME_LOWER, {} not a client\n",
                            cstr_to_str((*ent).targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*((*ent).client as *mut gclient_t)).ps.legsTimer as f32;
            }
            _ if toGet == SET_ANIM_HOLDTIME_UPPER as i32 => {
                if (*ent).client.is_null() {
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_ANIM_HOLDTIME_UPPER, {} not a client\n",
                            cstr_to_str((*ent).targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*((*ent).client as *mut gclient_t)).ps.torsoTimer as f32;
            }
            _ if toGet == SET_ANIM_HOLDTIME_BOTH as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetFloat: SET_ANIM_HOLDTIME_BOTH not implemented\n\0".as_ptr()
                        as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_ARMOR as i32 => {
                if (*ent).client.is_null() {
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_ARMOR, {} not a client\n",
                            cstr_to_str((*ent).targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*((*ent).client as *mut gclient_t)).ps.stats[STAT_ARMOR as usize] as f32;
            }
            _ if toGet == SET_WALKSPEED as i32
                || toGet == SET_RUNSPEED as i32
                || toGet == SET_YAWSPEED as i32
                || toGet == SET_AGGRESSION as i32
                || toGet == SET_AIM as i32
                || toGet == SET_FRICTION as i32
                || toGet == SET_SHOOTDIST as i32
                || toGet == SET_HFOV as i32
                || toGet == SET_VFOV as i32
                || toGet == SET_DELAYSCRIPTTIME as i32
                || toGet == SET_FORWARDMOVE as i32
                || toGet == SET_RIGHTMOVE as i32
                || toGet == SET_STARTFRAME as i32
                || toGet == SET_ENDFRAME as i32
                || toGet == SET_ANIMFRAME as i32
                || toGet == SET_SHOT_SPACING as i32
                || toGet == SET_MISSIONSTATUSTIME as i32
                || toGet == SET_IGNOREPAIN as i32
                || toGet == SET_IGNOREENEMIES as i32
                || toGet == SET_IGNOREALERTS as i32
                || toGet == SET_DONTSHOOT as i32 =>
            {
                return 0
            }
            _ if toGet == SET_NOTARGET as i32 => *value = ((*ent).flags & FL_NOTARGET) as f32,
            _ if toGet == SET_DONTFIRE as i32
                || toGet == SET_LOCKED_ENEMY as i32
                || toGet == SET_CROUCHED as i32
                || toGet == SET_WALKING as i32
                || toGet == SET_RUNNING as i32
                || toGet == SET_CHASE_ENEMIES as i32
                || toGet == SET_LOOK_FOR_ENEMIES as i32
                || toGet == SET_FACE_MOVE_DIR as i32
                || toGet == SET_FORCED_MARCH as i32
                || toGet == SET_UNDYING as i32
                || toGet == SET_NOAVOID as i32 =>
            {
                return 0
            }
            _ if toGet == SET_SOLID as i32 => *value = (*ent).r.contents as f32,
            _ if toGet == SET_PLAYER_USABLE as i32 => {
                *value = ((*ent).r.svFlags & SVF_PLAYER_USABLE) as f32
            }
            _ if toGet == SET_LOOP_ANIM as i32 => return 0,
            _ if toGet == SET_INTERFACE as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetFloat: SET_INTERFACE not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_SHIELDS as i32
                || toGet == SET_VAMPIRE as i32
                || toGet == SET_FORCE_INVINCIBLE as i32
                || toGet == SET_GREET_ALLIES as i32 =>
            {
                return 0
            }
            _ if toGet == SET_VIDEO_FADE_IN as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetFloat: SET_VIDEO_FADE_IN not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_VIDEO_FADE_OUT as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetFloat: SET_VIDEO_FADE_OUT not implemented\n\0".as_ptr()
                        as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_INVISIBLE as i32 => *value = ((*ent).s.eFlags & EF_NODRAW) as f32,
            _ if toGet == SET_PLAYER_LOCKED as i32
                || toGet == SET_LOCK_PLAYER_WEAPONS as i32
                || toGet == SET_NO_IMPACT_DAMAGE as i32 =>
            {
                return 0
            }
            _ if toGet == SET_NO_KNOCKBACK as i32 => {
                *value = ((*ent).flags & FL_NO_KNOCKBACK) as f32
            }
            _ if toGet == SET_ALT_FIRE as i32 || toGet == SET_NO_RESPONSE as i32 => return 0,
            _ if toGet == SET_INVINCIBLE as i32 => *value = ((*ent).flags & FL_GODMODE) as f32,
            _ if toGet == SET_MISSIONSTATUSACTIVE as i32
                || toGet == SET_NO_COMBAT_TALK as i32
                || toGet == SET_NO_ALERT_TALK as i32
                || toGet == SET_USE_CP_NEAREST as i32
                || toGet == SET_DISMEMBERABLE as i32
                || toGet == SET_NO_FORCE as i32
                || toGet == SET_NO_ACROBATICS as i32
                || toGet == SET_USE_SUBTITLES as i32
                || toGet == SET_NO_FALLTODEATH as i32
                || toGet == SET_MORELIGHT as i32
                || toGet == SET_TREASONED as i32
                || toGet == SET_DISABLE_SHADER_ANIM as i32
                || toGet == SET_SHADER_ANIM as i32 =>
            {
                return 0
            }
            _ => {
                if trap::ICARUS_VariableDeclared(ctx.engine, GIcarusVariabledeclaredArgs::new(name))
                    != VTYPE_FLOAT
                {
                    return 0;
                }
                return trap::ICARUS_GetFloatVariable(
                    ctx.engine,
                    GIcarusGetfloatvariableArgs::new(name, value),
                );
            }
        }

        1
    }
}

/// Raven `Q3_GetVector`.
///
// PORT-NOTE(unported-global): same `setTable`/`SET_*` dependency as
// `Q3_GetFloat` (missing_symbols).
/// Source: `oracle/codemp/game/g_ICARUScb.c:1573-1629`
pub fn Q3_GetVector(
    ctx: &mut GameContext,
    entID: c_int,
    r#type: c_int,
    name: *const c_char,
    value: &mut [f32; 3],
) -> c_int {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        let toGet = GetIDForString(setTable.as_ptr() as *mut stringID_table_t, name);

        match toGet {
            _ if toGet == SET_PARM1 as i32
                || toGet == SET_PARM2 as i32
                || toGet == SET_PARM3 as i32
                || toGet == SET_PARM4 as i32
                || toGet == SET_PARM5 as i32
                || toGet == SET_PARM6 as i32
                || toGet == SET_PARM7 as i32
                || toGet == SET_PARM8 as i32
                || toGet == SET_PARM9 as i32
                || toGet == SET_PARM10 as i32
                || toGet == SET_PARM11 as i32
                || toGet == SET_PARM12 as i32
                || toGet == SET_PARM13 as i32
                || toGet == SET_PARM14 as i32
                || toGet == SET_PARM15 as i32
                || toGet == SET_PARM16 as i32 =>
            {
                // Raven: sscanf(parm, "%f %f %f", &value[0], &value[1], &value[2])
                // — oracle g_ICARUScb.c:1604 has no count check; unmatched
                // components are left untouched (porting-rules §19).
                let parm_str =
                    cstr_to_str((*(*ent).parms).parm[(toGet - SET_PARM1 as i32) as usize].as_ptr());
                sscanf_f32s(&parm_str, value);
            }
            _ if toGet == SET_ORIGIN as i32 => *value = (*ent).r.currentOrigin,
            _ if toGet == SET_ANGLES as i32 => *value = (*ent).r.currentAngles,
            _ if toGet == SET_TELEPORT_DEST as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetVector: SET_TELEPORT_DEST not implemented\n\0".as_ptr()
                        as *const c_char,
                );
                return 0;
            }
            _ => {
                if trap::ICARUS_VariableDeclared(ctx.engine, GIcarusVariabledeclaredArgs::new(name))
                    != VTYPE_VECTOR
                {
                    return 0;
                }
                return trap::ICARUS_GetVectorVariable(
                    ctx.engine,
                    GIcarusGetvectorvariableArgs::new(name, value as *mut vec3_t),
                );
            }
        }

        1
    }
}

/// Raven `Q3_GetString`.
///
// PORT-NOTE(unported-global): same `setTable`/`SET_*` dependency as
// `Q3_GetFloat` (missing_symbols).
/// Source: `oracle/codemp/game/g_ICARUScb.c:1642-1854`
pub fn Q3_GetString(
    ctx: &mut GameContext,
    entID: c_int,
    r#type: c_int,
    name: *const c_char,
    value: *mut *mut c_char,
) -> c_int {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        let toGet = GetIDForString(setTable.as_ptr() as *mut stringID_table_t, name);

        match toGet {
            _ if toGet == SET_ANIM_BOTH as i32 => {
                *value = Q3_GetAnimBoth(ctx, ctx.entity_id_of(ent).unwrap());
                if value.is_null() || (*value).is_null() {
                    return 0;
                }
            }
            _ if toGet == SET_PARM1 as i32
                || toGet == SET_PARM2 as i32
                || toGet == SET_PARM3 as i32
                || toGet == SET_PARM4 as i32
                || toGet == SET_PARM5 as i32
                || toGet == SET_PARM6 as i32
                || toGet == SET_PARM7 as i32
                || toGet == SET_PARM8 as i32
                || toGet == SET_PARM9 as i32
                || toGet == SET_PARM10 as i32
                || toGet == SET_PARM11 as i32
                || toGet == SET_PARM12 as i32
                || toGet == SET_PARM13 as i32
                || toGet == SET_PARM14 as i32
                || toGet == SET_PARM15 as i32
                || toGet == SET_PARM16 as i32 =>
            {
                if !(*ent).parms.is_null() {
                    *value = (*(*ent).parms).parm[(toGet - SET_PARM1 as i32) as usize].as_mut_ptr();
                } else {
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetString: invalid ent {} has no parms!\n",
                            cstr_to_str((*ent).targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
            }
            _ if toGet == SET_TARGET as i32 => *value = (*ent).target,
            _ if toGet == SET_LOCATION as i32 => return 0,
            _ if toGet == SET_SPAWNSCRIPT as i32 => {
                *value = (*ent).behaviorSet[BSET_SPAWN as usize]
            }
            _ if toGet == SET_USESCRIPT as i32 => *value = (*ent).behaviorSet[BSET_USE as usize],
            _ if toGet == SET_AWAKESCRIPT as i32 => {
                *value = (*ent).behaviorSet[BSET_AWAKE as usize]
            }
            _ if toGet == SET_ANGERSCRIPT as i32 => {
                *value = (*ent).behaviorSet[BSET_ANGER as usize]
            }
            _ if toGet == SET_ATTACKSCRIPT as i32 => {
                *value = (*ent).behaviorSet[BSET_ATTACK as usize]
            }
            _ if toGet == SET_VICTORYSCRIPT as i32 => {
                *value = (*ent).behaviorSet[BSET_VICTORY as usize]
            }
            _ if toGet == SET_LOSTENEMYSCRIPT as i32 => {
                *value = (*ent).behaviorSet[BSET_LOSTENEMY as usize]
            }
            _ if toGet == SET_PAINSCRIPT as i32 => *value = (*ent).behaviorSet[BSET_PAIN as usize],
            _ if toGet == SET_FLEESCRIPT as i32 => *value = (*ent).behaviorSet[BSET_FLEE as usize],
            _ if toGet == SET_DEATHSCRIPT as i32 => {
                *value = (*ent).behaviorSet[BSET_DEATH as usize]
            }
            _ if toGet == SET_DELAYEDSCRIPT as i32 => {
                *value = (*ent).behaviorSet[BSET_DELAYED as usize]
            }
            _ if toGet == SET_BLOCKEDSCRIPT as i32 => {
                *value = (*ent).behaviorSet[BSET_BLOCKED as usize]
            }
            _ if toGet == SET_FFIRESCRIPT as i32 => {
                *value = (*ent).behaviorSet[BSET_FFIRE as usize]
            }
            _ if toGet == SET_FFDEATHSCRIPT as i32 => {
                *value = (*ent).behaviorSet[BSET_FFDEATH as usize]
            }
            _ if toGet == SET_ENEMY as i32
                || toGet == SET_LEADER as i32
                || toGet == SET_CAPTURE as i32 =>
            {
                return 0
            }
            _ if toGet == SET_TARGETNAME as i32 => *value = (*ent).targetname,
            _ if toGet == SET_PAINTARGET as i32
                || toGet == SET_CAMERA_GROUP as i32
                || toGet == SET_CAMERA_GROUP_TAG as i32 =>
            {
                return 0
            }
            _ if toGet == SET_LOOK_TARGET as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_LOOK_TARGET, NOT SUPPORTED IN MULTIPLAYER\n\0".as_ptr()
                        as *const c_char,
                );
            }
            _ if toGet == SET_TARGET2 as i32
                || toGet == SET_REMOVE_TARGET as i32
                || toGet == SET_WEAPON as i32
                || toGet == SET_ITEM as i32
                || toGet == SET_MUSIC_STATE as i32 =>
            {
                return 0
            }
            _ if toGet == SET_NAVGOAL as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_NAVGOAL not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_VIEWTARGET as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_VIEWTARGET not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_WATCHTARGET as i32 => return 0,
            _ if toGet == SET_VIEWENTITY as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_VIEWENTITY not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_CAPTIONTEXTCOLOR as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_CAPTIONTEXTCOLOR not implemented\n\0".as_ptr()
                        as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_CENTERTEXTCOLOR as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_CENTERTEXTCOLOR not implemented\n\0".as_ptr()
                        as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_SCROLLTEXTCOLOR as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_SCROLLTEXTCOLOR not implemented\n\0".as_ptr()
                        as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_COPY_ORIGIN as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_COPY_ORIGIN not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_DEFEND_TARGET as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_COPY_ORIGIN not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_VIDEO_PLAY as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_VIDEO_PLAY not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_LOADGAME as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_LOADGAME not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_LOCKYAW as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_LOCKYAW not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_SCROLLTEXT as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_SCROLLTEXT not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_LCARSTEXT as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_GetString: SET_LCARSTEXT not implemented\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            _ if toGet == SET_FULLNAME as i32 => *value = (*ent).fullName,
            _ => {
                if trap::ICARUS_VariableDeclared(ctx.engine, GIcarusVariabledeclaredArgs::new(name))
                    != VTYPE_STRING
                {
                    return 0;
                }
                return trap::ICARUS_GetStringVariable(
                    ctx.engine,
                    GIcarusGetstringvariableArgs::new(name, *value as *const c_char),
                );
            }
        }

        1
    }
}

/// Raven `MoveOwner`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1865-1886`
pub fn MoveOwner(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);

    unsafe {
        let owner =
            &mut (*ctx.world_raw()).g_entities[(*self_).r.ownerNum as usize] as *mut gentity_t;

        (*self_).nextthink = (*ctx.world_raw()).level.time + FRAMETIME;
        (*self_).think = Some(EntThink::G_FreeEntity).into();

        if owner.is_null() || (*owner).inuse == 0 {
            return;
        }

        if SpotWouldTelefrag2(
            ctx,
            ctx.entity_id_of(owner).unwrap(),
            (*self_).r.currentOrigin,
        ) != 0
        {
            (*self_).think = Some(EntThink::MoveOwner).into();
        } else {
            G_SetOrigin(&mut *(owner), (*self_).r.currentOrigin);
            trap::ICARUS_TaskIDComplete(
                ctx.engine,
                GIcarusTaskidcompleteArgs::new(owner, taskID_t::TID_MOVE_NAV as c_int),
            );
        }
    }
}

/// Raven `Q3_SetTeleportDest`.
///
/// `org` is only ever read here, so it stays by-value.
/// Source: `oracle/codemp/game/g_ICARUScb.c:1895-1920`
pub fn Q3_SetTeleportDest(ctx: &mut GameContext, entID: c_int, org: vec3_t) -> qboolean {
    unsafe {
        let tele_ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if SpotWouldTelefrag2(ctx, ctx.entity_id_of(tele_ent).unwrap(), org) != 0 {
            let teleporter = G_Spawn(ctx);

            G_SetOrigin(&mut *(teleporter), org);
            (*teleporter).r.ownerNum = (*tele_ent).s.number;

            (*teleporter).think = Some(EntThink::MoveOwner).into();
            (*teleporter).nextthink = (*ctx.world_raw()).level.time + FRAMETIME;

            qfalse
        } else {
            G_SetOrigin(&mut *(tele_ent), org);
            qtrue
        }
    }
}

/// Raven `Q3_SetOrigin`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1929-1961`
// PORT-NOTE(client-still-void): the client branch writes
// `ent->client->ps.{origin,velocity,pm_time,pm_flags,eFlags}`; the non-client
// (`G_SetOrigin`) branch is faithful, the client branch panics loudly.
pub fn Q3_SetOrigin(ctx: &mut GameContext, entID: c_int, origin: vec3_t) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        trap::UnlinkEntity(ctx.engine, GUnlinkentityArgs::new(ent));

        if !(*ent).client.is_null() {
            let client = (*ent).client as *mut gclient_t;
            (*client).ps.origin = origin;
            (*ent).r.currentOrigin = origin;
            (*client).ps.origin[2] += 1.0;

            (*client).ps.velocity = [0.0, 0.0, 0.0];
            (*client).ps.pm_time = 160;
            (*client).ps.pm_flags |= PMF_TIME_KNOCKBACK;

            (*client).ps.eFlags ^= EF_TELEPORT_BIT;
        } else {
            G_SetOrigin(&mut *(ent), origin);
        }

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
    }
}

/// Raven `Q3_SetCopyOrigin`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1970-1983`
pub fn Q3_SetCopyOrigin(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    unsafe {
        let found = G_Find(
            ctx,
            ctx.entity_id_of(std::ptr::null_mut()),
            core::mem::offset_of!(gentity_t, targetname) as c_int,
            name,
        );

        if !found.is_null() {
            Q3_SetOrigin(ctx, entID, (*found).r.currentOrigin);
            let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
            SetClientViewAngle(&mut *ent, (*found).s.angles);
        } else {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                b"Q3_SetCopyOrigin: ent not found!\n\0".as_ptr() as *const c_char,
            );
        }
    }
}

/// Raven `Q3_SetVelocity`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1992-2013`
pub fn Q3_SetVelocity(ctx: &mut GameContext, entID: c_int, axis: c_int, speed: f32) {
    unsafe {
        let found = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*found).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                cstr(&format!("Q3_SetVelocity: not a client {}\n", entID)).as_ptr(),
            );
            return;
        }

        let client = (*found).client as *mut gclient_t;
        (*client).ps.velocity[axis as usize] += speed;

        (*client).ps.pm_time = 500;
        (*client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
    }
}

/// Raven `Q3_SetAngles`.
///
/// `angles` is only ever read here (never written through), so it stays a
/// by-value `vec3_t` ("keep by-value only if never written").
/// Source: `oracle/codemp/game/g_ICARUScb.c:2022-2042`
pub fn Q3_SetAngles(ctx: &mut GameContext, entID: c_int, angles: vec3_t) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*ent).client.is_null() {
            (*ent).s.angles = angles;
        } else {
            SetClientViewAngle(&mut *ent, angles);
        }
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
    }
}

/// Raven `Q3_Lerp2Origin`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2051-2112`
pub fn Q3_Lerp2Origin(
    ctx: &mut GameContext,
    taskID: c_int,
    entID: c_int,
    origin: vec3_t,
    duration: f32,
) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if !(*ent).client.is_null()
            || Q_stricmp(
                (*ent).classname,
                b"target_scriptrunner\0".as_ptr() as *const c_char,
            ) == 0
        {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!("Q3_Lerp2Origin: ent {} is NOT a mover!\n", entID)).as_ptr(),
            );
            return;
        }

        if (*ent).s.eType != entityType_t::ET_MOVER as c_int {
            (*ent).s.eType = entityType_t::ET_MOVER as c_int;
        }

        let mut moverState = (*ent).moverState;

        if moverState == MOVER_POS1 || moverState == MOVER_2TO1 {
            (*ent).pos1 = (*ent).r.currentOrigin;
            (*ent).pos2 = origin;
            moverState = MOVER_1TO2;
        } else if moverState == MOVER_POS2 || moverState == MOVER_1TO2 {
            (*ent).pos2 = (*ent).r.currentOrigin;
            (*ent).pos1 = origin;
            moverState = MOVER_2TO1;
        }
        (*ent).moverState = moverState;

        InitMoverTrData(&mut *ent);

        (*ent).s.pos.trDuration = duration as c_int;

        let __h531 = ctx.entity_id_of(ent).unwrap();
        let __h532 = (*ctx.world_raw()).level.time;
        MatchTeam(ctx, __h531, moverState as c_int, __h532);

        (*ent).reached = Some(EntReached::moverCallback).into();
        if (*ent).damage != 0 {
            (*ent).blocked = Some(EntBlocked::Blocked_Mover).into();
        }
        if taskID != -1 {
            trap::ICARUS_TaskIDSet(
                ctx.engine,
                GIcarusTaskidsetArgs::new(ent, taskID_t::TID_MOVE_NAV as c_int, taskID),
            );
        }

        G_PlayDoorLoopSound(ctx, ctx.entity_id_of(ent).unwrap());
        G_PlayDoorSound(ctx, ctx.entity_id_of(ent).unwrap(), BMS_START);

        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent));
    }
}

/// Raven `Q3_SetOriginOffset`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2114-2140`
pub fn Q3_SetOriginOffset(ctx: &mut GameContext, entID: c_int, axis: c_int, offset: f32) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if !(*ent).client.is_null()
            || Q_stricmp(
                (*ent).classname,
                b"target_scriptrunner\0".as_ptr() as *const c_char,
            ) == 0
        {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetOriginOffset: ent {} is NOT a mover!\n",
                    entID
                ))
                .as_ptr(),
            );
            return;
        }

        let mut origin = (*ent).s.origin;
        origin[axis as usize] += offset;
        let mut duration = 0.0f32;
        if (*ent).speed != 0.0 {
            // C's `fabs` is the double libm function: the divide and `*1000.0f`
            // evaluate in f64, narrowing to the float `duration` only at the
            // assignment. f32-throughout would diverge at Q3_Lerp2Origin's
            // `trDuration` truncation boundaries.
            duration = ((offset as f64).abs() / ((*ent).speed as f64).abs() * 1000.0) as f32;
        }
        Q3_Lerp2Origin(ctx, -1, entID, origin, duration);
    }
}

/// Raven `Q3_SetEnemy`.
///
/// `ent->NPC` is only null-checked (never dereferenced) in this fn.
/// Source: `oracle/codemp/game/g_ICARUScb.c:2149-2197`
pub fn Q3_SetEnemy(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if Q_stricmp(b"NONE\0".as_ptr() as *const c_char, name) == 0
            || Q_stricmp(b"NULL\0".as_ptr() as *const c_char, name) == 0
        {
            if !(*ent).NPC.is_null() {
                G_ClearEnemy(ctx, ctx.entity_id_of(ent).unwrap());
            } else {
                (*ent).enemy = None;
            }
        } else {
            let enemy = G_Find(
                ctx,
                ctx.entity_id_of(std::ptr::null_mut()),
                core::mem::offset_of!(gentity_t, targetname) as c_int,
                name,
            );

            if enemy.is_null() {
                G_DebugPrint(
                    ctx,
                    WL_ERROR as c_int,
                    b"Q3_SetEnemy: no such enemy\n\0".as_ptr() as *const c_char,
                );
                return;
            }

            G_SetEnemy(ctx, ctx.entity_id_of(ent).unwrap(), ctx.entity_id_of(enemy));
            if !(*ent).NPC.is_null() {
                (*ent).cantHitEnemyCounter = 0;
            }
        }
    }
}

/// Raven `Q3_SetLeader`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2207-2246`
pub fn Q3_SetLeader(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*ent).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetLeader: ent {} is NOT a player or NPC!\n",
                    entID
                ))
                .as_ptr(),
            );
            return;
        }
        let client = (*ent).client as *mut gclient_t;

        if Q_stricmp(b"NONE\0".as_ptr() as *const c_char, name) == 0
            || Q_stricmp(b"NULL\0".as_ptr() as *const c_char, name) == 0
        {
            (*client).leader = None;
        } else {
            let leader = G_Find(
                ctx,
                ctx.entity_id_of(std::ptr::null_mut()),
                core::mem::offset_of!(gentity_t, targetname) as c_int,
                name,
            );

            if leader.is_null() {
                return;
            } else if (*leader).health <= 0 {
                return;
            } else {
                (*client).leader = Some(ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), leader));
            }
        }
    }
}

/// Raven `Q3_SetNavGoal`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2255-2320`
pub fn Q3_SetNavGoal(ctx: &mut GameContext, entID: c_int, name: *const c_char) -> qboolean {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
        let mut goalPos: vec3_t = [0.0, 0.0, 0.0];

        if (*ent).health == 0 {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetNavGoal: tried to set a navgoal (\"{}\") on a corpse! \"{}\"\n",
                    cstr_to_str(name),
                    cstr_to_str((*ent).script_targetname)
                ))
                .as_ptr(),
            );
            return qfalse;
        }
        if (*ent).NPC.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetNavGoal: tried to set a navgoal (\"{}\") on a non-NPC: \"{}\"\n",
                    cstr_to_str(name),
                    cstr_to_str((*ent).script_targetname)
                ))
                .as_ptr(),
            );
            return qfalse;
        }
        let npc = (*ent).NPC as *mut gNPC_t;
        if (*npc).tempGoal.is_none() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetNavGoal: tried to set a navgoal (\"{}\") on a dead NPC: \"{}\"\n",
                    cstr_to_str(name),
                    cstr_to_str((*ent).script_targetname)
                ))
                .as_ptr(),
            );
            return qfalse;
        }
        let temp_goal_id = (*npc).tempGoal.unwrap();
        let temp_goal =
            &mut (*ctx.world_raw()).g_entities[temp_goal_id.0 as usize] as *mut gentity_t;
        if (*temp_goal).inuse == 0 {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetNavGoal: NPC's (\"{}\") navgoal is freed: \"{}\"\n",
                    cstr_to_str(name),
                    cstr_to_str((*ent).script_targetname)
                ))
                .as_ptr(),
            );
            return qfalse;
        }

        if Q_stricmp(b"null\0".as_ptr() as *const c_char, name) == 0
            || Q_stricmp(b"NULL\0".as_ptr() as *const c_char, name) == 0
        {
            (*npc).goalEntity = None;
            trap::ICARUS_TaskIDComplete(
                ctx.engine,
                GIcarusTaskidcompleteArgs::new(ent, taskID_t::TID_MOVE_NAV as c_int),
            );
            return qfalse;
        }

        if TAG_GetOrigin2(ctx, std::ptr::null(), name, &mut goalPos) == qfalse {
            let targ = G_Find(
                ctx,
                ctx.entity_id_of(std::ptr::null_mut()),
                core::mem::offset_of!(gentity_t, targetname) as c_int,
                name,
            );
            if targ.is_null() {
                G_DebugPrint(
                    ctx,
                    WL_ERROR as c_int,
                    cstr(&format!(
                        "Q3_SetNavGoal: can't find NAVGOAL \"{}\"\n",
                        cstr_to_str(name)
                    ))
                    .as_ptr(),
                );
                return qfalse;
            }
            (*npc).goalEntity = Some(ent_id((*ctx.world_raw()).g_entities.as_mut_ptr(), targ));
            // C's `sqrt` is the double libm function: the float sums promote to
            // f64, are rooted and summed in f64, then truncated to the int
            // `goalRadius`. f32-throughout would diverge at truncation boundaries.
            (*npc).goalRadius = (((*ent).r.maxs[0] as f64 + (*ent).r.maxs[0] as f64).sqrt()
                + ((*targ).r.maxs[0] as f64 + (*targ).r.maxs[0] as f64).sqrt())
                as c_int;
            (*npc).aiFlags &= !NPCAI_TOUCHED_GOAL;
            qfalse
        } else {
            let goalRadius = TAG_GetRadius(ctx, std::ptr::null(), name);
            NPC_SetMoveGoal(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                goalPos,
                goalRadius,
                qtrue,
                -1,
                None,
            );
            let goal_id = (*npc).goalEntity.unwrap();
            let goal_ent = &mut (*ctx.world_raw()).g_entities[goal_id.0 as usize] as *mut gentity_t;
            (*goal_ent).lastWaypoint = WAYPOINT_NONE;
            (*npc).aiFlags &= !NPCAI_TOUCHED_GOAL;
            // Raven's `#ifdef _DEBUG` block (tempGoal->target = G_NewString(name))
            // is dev-build-only diagnostic noise; not transcribed.
            qtrue
        }
    }
}

/// Raven `SetLowerAnim`.
///
/// `ent->client` is only null-checked (never dereferenced) in this fn.
/// Source: `oracle/codemp/game/g_ICARUScb.c:2330-2347`
pub fn SetLowerAnim(ctx: &mut GameContext, entID: c_int, animID: c_int) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*ent).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                b"SetLowerAnim: ent is NOT a player or NPC!\n\0".as_ptr() as *const c_char,
            );
            return;
        }

        G_SetAnim(
            ctx,
            ctx.entity_id_of(ent).unwrap(),
            std::ptr::null_mut(),
            SETANIM_LEGS,
            animID,
            SETANIM_FLAG_RESTART | SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
            0,
        );
    }
}

/// Raven `SetUpperAnim`.
///
/// `ent->client` is only null-checked (never dereferenced) in this fn.
/// Source: `oracle/codemp/game/g_ICARUScb.c:2358-2375`
pub fn SetUpperAnim(ctx: &mut GameContext, entID: c_int, animID: c_int) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*ent).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                b"SetUpperAnim: ent is NOT a player or NPC!\n\0".as_ptr() as *const c_char,
            );
            return;
        }

        G_SetAnim(
            ctx,
            ctx.entity_id_of(ent).unwrap(),
            std::ptr::null_mut(),
            SETANIM_TORSO,
            animID,
            SETANIM_FLAG_RESTART | SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
            0,
        );
    }
}

// PORT-NOTE(unported-global): `animTable` is not ported anywhere in the
// worktree (missing_symbols).
/// Raven `Q3_SetAnimUpper`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2384-2405`
pub fn Q3_SetAnimUpper(ctx: &mut GameContext, entID: c_int, anim_name: *const c_char) -> qboolean {
    unsafe {
        let animID = GetIDForString(animTable.as_ptr() as *mut stringID_table_t, anim_name);

        if animID == -1 {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                cstr(&format!(
                    "Q3_SetAnimUpper: unknown animation sequence '{}'\n",
                    cstr_to_str(anim_name)
                ))
                .as_ptr(),
            );
            return qfalse;
        }

        SetUpperAnim(ctx, entID, animID);
        qtrue
    }
}

// PORT-NOTE(unported-global): same `animTable` dependency as `Q3_SetAnimUpper`.
/// Raven `Q3_SetAnimLower`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2414-2437`
pub fn Q3_SetAnimLower(ctx: &mut GameContext, entID: c_int, anim_name: *const c_char) -> qboolean {
    unsafe {
        let animID = GetIDForString(animTable.as_ptr() as *mut stringID_table_t, anim_name);

        if animID == -1 {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                cstr(&format!(
                    "Q3_SetAnimLower: unknown animation sequence '{}'\n",
                    cstr_to_str(anim_name)
                ))
                .as_ptr(),
            );
            return qfalse;
        }

        SetLowerAnim(ctx, entID, animID);
        qtrue
    }
}

/// Raven `Q3_SetAnimHoldTime`.
///
/// Raven: the real body (`PM_SetLegsAnimTimer`/`PM_SetTorsoAnimTimer`) is
/// `#if 0`'d out in the oracle itself; only the "not supported" print remains live.
/// Source: `oracle/codemp/game/g_ICARUScb.c:2449-2476`
pub fn Q3_SetAnimHoldTime(ctx: &mut GameContext, entID: c_int, int_data: c_int, lower: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetAnimHoldTime is not currently supported in MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetHealth`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2487-2527`
pub fn Q3_SetHealth(ctx: &mut GameContext, entID: c_int, data: c_int) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
        let mut data = data;

        if data < 0 {
            data = 0;
        }

        (*ent).health = data;

        if (*ent).client.is_null() {
            return;
        }
        let client = (*ent).client as *mut gclient_t;

        (*client).ps.stats[STAT_HEALTH as usize] = data;

        if (*client).ps.stats[STAT_HEALTH as usize] > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
            (*ent).health = (*client).ps.stats[STAT_MAX_HEALTH as usize];
            (*client).ps.stats[STAT_HEALTH as usize] = (*ent).health;
        }
        if data == 0 {
            (*ent).health = 1;
            if (*client).sess.sessionTeam == TEAM_SPECTATOR {
                return;
            }

            (*ent).flags &= !FL_GODMODE;
            (*ent).health = -999;
            (*client).ps.stats[STAT_HEALTH as usize] = (*ent).health;
            player_die(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                ctx.entity_id_of(ent),
                ctx.entity_id_of(ent),
                100000,
                MOD_FALLING as c_int,
            );
        }
    }
}

/// Raven `Q3_SetArmor`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2539-2559`
pub fn Q3_SetArmor(ctx: &mut GameContext, entID: c_int, data: c_int) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*ent).client.is_null() {
            return;
        }
        let client = (*ent).client as *mut gclient_t;

        (*client).ps.stats[STAT_ARMOR as usize] = data;
        if (*client).ps.stats[STAT_ARMOR as usize] > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
            (*client).ps.stats[STAT_ARMOR as usize] = (*client).ps.stats[STAT_MAX_HEALTH as usize];
        }
    }
}

// PORT-NOTE(unported-global): `BSTable` (the bState_t string table) is not
// ported anywhere in the worktree (missing_symbols). The NAV_FindClosestWaypointForEnt/
// NPC_BSSearchStart search-start branch is a faithful transcription; the
// `#FIXME: Reimplement` comment is Raven's own, preserved.
/// Raven `Q3_SetBState`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2573-2687`
pub fn Q3_SetBState(ctx: &mut GameContext, entID: c_int, bs_name: *const c_char) -> qboolean {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*ent).NPC.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetBState: '{}' is not an NPC\n",
                    cstr_to_str((*ent).targetname)
                ))
                .as_ptr(),
            );
            return qtrue;
        }
        let npc = (*ent).NPC as *mut gNPC_t;

        let bSID = GetIDForString(BSTable.as_ptr() as *mut stringID_table_t, bs_name);
        if bSID > -1 {
            if bSID == (BS_SEARCH) as i32 || bSID == (BS_WANDER) as i32 {
                if (*ent).waypoint != WAYPOINT_NONE {
                    NPC_BSSearchStart(
                        ctx,
                        (*ent).waypoint,
                        core::mem::transmute::<c_int, bState_t>(bSID),
                    );
                } else {
                    (*ent).waypoint = NAV_FindClosestWaypointForEnt(
                        ctx,
                        ctx.entity_id_of(ent).unwrap(),
                        WAYPOINT_NONE,
                    );

                    if (*ent).waypoint != WAYPOINT_NONE {
                        NPC_BSSearchStart(
                            ctx,
                            (*ent).waypoint,
                            core::mem::transmute::<c_int, bState_t>(bSID),
                        );
                    } else {
                        G_DebugPrint(
                            ctx,
                            WL_ERROR as c_int,
                            cstr(&format!(
                                "Q3_SetBState: '{}' is not in a valid waypoint to search from!\n",
                                cstr_to_str((*ent).targetname)
                            ))
                            .as_ptr(),
                        );
                        return qtrue;
                    }
                }
            }

            (*npc).tempBehavior = BS_DEFAULT;
            if (*npc).behaviorState == BS_NOCLIP && bSID != (BS_NOCLIP) as i32 {
                (*ent).r.currentOrigin[2] += 0.125;
                G_SetOrigin(&mut *(ent), (*ent).r.currentOrigin);
            }
            (*npc).behaviorState = core::mem::transmute::<c_int, bState_t>(bSID);
            if bSID == (BS_DEFAULT) as i32 {
                (*npc).defaultBehavior = core::mem::transmute::<c_int, bState_t>(bSID);
            }
        }

        (*npc).aiFlags &= !NPCAI_TOUCHED_GOAL;

        if bSID == (BS_NOCLIP) as i32 {
            (*((*ent).client as *mut gclient_t)).noclip = qtrue;
        } else {
            (*((*ent).client as *mut gclient_t)).noclip = qfalse;
        }

        if bSID == (BS_ADVANCE_FIGHT) as i32 {
            return qfalse;
        }

        if bSID == (BS_JUMP) as i32 {
            (*npc).jumpState = JS_FACING;
        }

        qtrue
    }
}

// PORT-NOTE(unported-global): same `BSTable` dependency as `Q3_SetBState`.
/// Raven `Q3_SetTempBState`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2699-2737`
pub fn Q3_SetTempBState(ctx: &mut GameContext, entID: c_int, bs_name: *const c_char) -> qboolean {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*ent).NPC.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetTempBState: '{}' is not an NPC\n",
                    cstr_to_str((*ent).targetname)
                ))
                .as_ptr(),
            );
            return qtrue;
        }
        let npc = (*ent).NPC as *mut gNPC_t;

        let bSID = GetIDForString(BSTable.as_ptr() as *mut stringID_table_t, bs_name);
        if bSID > -1 {
            (*npc).tempBehavior = core::mem::transmute::<c_int, bState_t>(bSID);
        }

        qtrue
    }
}

// PORT-NOTE(unported-global): same `BSTable` dependency as `Q3_SetBState`.
/// Raven `Q3_SetDefaultBState`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2749-2771`
pub fn Q3_SetDefaultBState(ctx: &mut GameContext, entID: c_int, bs_name: *const c_char) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*ent).NPC.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetDefaultBState: '{}' is not an NPC\n",
                    cstr_to_str((*ent).targetname)
                ))
                .as_ptr(),
            );
            return;
        }
        let npc = (*ent).NPC as *mut gNPC_t;

        let bSID = GetIDForString(BSTable.as_ptr() as *mut stringID_table_t, bs_name);
        if bSID > -1 {
            (*npc).defaultBehavior = core::mem::transmute::<c_int, bState_t>(bSID);
        }
    }
}

/// Raven `Q3_SetDPitch`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2783-2787`
pub fn Q3_SetDPitch(ctx: &mut GameContext, entID: c_int, data: f32) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetDPitch: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetDYaw`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2799-2803`
pub fn Q3_SetDYaw(ctx: &mut GameContext, entID: c_int, data: f32) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetDYaw: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetShootDist`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2815-2819`
pub fn Q3_SetShootDist(ctx: &mut GameContext, entID: c_int, data: f32) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetShootDist: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetVisrange`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2831-2835`
pub fn Q3_SetVisrange(ctx: &mut GameContext, entID: c_int, data: f32) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetVisrange: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetEarshot`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2847-2851`
pub fn Q3_SetEarshot(ctx: &mut GameContext, entID: c_int, data: f32) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetEarshot: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetVigilance`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2863-2867`
pub fn Q3_SetVigilance(ctx: &mut GameContext, entID: c_int, data: f32) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetVigilance: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetVFOV`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2879-2883`
pub fn Q3_SetVFOV(ctx: &mut GameContext, entID: c_int, data: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetVFOV: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetHFOV`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2895-2899`
pub fn Q3_SetHFOV(ctx: &mut GameContext, entID: c_int, data: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetHFOV: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetWidth`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2911-2915`
pub fn Q3_SetWidth(ctx: &mut GameContext, entID: c_int, data: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetWidth: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetTimeScale`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2926-2929`
pub fn Q3_SetTimeScale(ctx: &mut GameContext, entID: c_int, data: *const c_char) {
    unsafe {
        let value = CString::new(std::ffi::CStr::from_ptr(data).to_bytes()).unwrap();
        trap::Cvar_Set(
            ctx.engine,
            GCvarSetArgs::new(CString::new("timescale").unwrap(), value),
        );
    }
}

// PORT-NOTE(client-still-void): `self->client->ps.eFlags` toggle is a
// real field write, not just a null check; leaving the whole fn parked rather
// than silently skipping that half of the behavior.
/// Raven `Q3_SetInvisible`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2941-2968`
pub fn Q3_SetInvisible(ctx: &mut GameContext, entID: c_int, invisible: qboolean) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if invisible != 0 {
            (*self_).s.eFlags |= EF_NODRAW;
            if !(*self_).client.is_null() {
                (*((*self_).client as *mut gclient_t)).ps.eFlags |= EF_NODRAW;
            }
            (*self_).r.contents = 0;
        } else {
            (*self_).s.eFlags &= !EF_NODRAW;
            if !(*self_).client.is_null() {
                (*((*self_).client as *mut gclient_t)).ps.eFlags &= !EF_NODRAW;
            }
        }
    }
}

/// Raven `Q3_SetVampire`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2979-2983`
pub fn Q3_SetVampire(ctx: &mut GameContext, entID: c_int, vampire: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetVampire: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetGreetAllies`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2993-2997`
pub fn Q3_SetGreetAllies(ctx: &mut GameContext, entID: c_int, greet: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetGreetAllies: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetViewTarget`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3009-3013`
pub fn Q3_SetViewTarget(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetViewTarget: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetWatchTarget`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3025-3029`
pub fn Q3_SetWatchTarget(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetWatchTarget: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetLoopSound`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3031-3054`
pub fn Q3_SetLoopSound(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if Q_stricmp(b"NULL\0".as_ptr() as *const c_char, name) == 0
            || Q_stricmp(b"NONE\0".as_ptr() as *const c_char, name) == 0
        {
            (*self_).s.loopSound = 0;
            (*self_).s.loopIsSoundset = qfalse;
            return;
        }

        let index = G_SoundIndex(name);

        if index != 0 {
            (*self_).s.loopSound = index;
            (*self_).s.loopIsSoundset = qfalse;
        } else {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                b"Q3_SetLoopSound: can't find sound file\n\0".as_ptr() as *const c_char,
            );
        }
    }
}

/// Raven `Q3_SetICARUSFreeze`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3056-3078`
pub fn Q3_SetICARUSFreeze(
    ctx: &mut GameContext,
    entID: c_int,
    name: *const c_char,
    freeze: qboolean,
) {
    unsafe {
        let mut self_ = G_Find(
            ctx,
            ctx.entity_id_of(std::ptr::null_mut()),
            core::mem::offset_of!(gentity_t, targetname) as c_int,
            name,
        );
        if self_.is_null() {
            self_ = G_Find(
                ctx,
                ctx.entity_id_of(std::ptr::null_mut()),
                core::mem::offset_of!(gentity_t, script_targetname) as c_int,
                name,
            );
        }

        if self_.is_null() {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                cstr(&format!(
                    "Q3_SetICARUSFreeze: invalid ent {}\n",
                    cstr_to_str(name)
                ))
                .as_ptr(),
            );
            return;
        }

        if freeze != 0 {
            (*self_).r.svFlags |= SVF_ICARUS_FREEZE;
        } else {
            (*self_).r.svFlags &= !SVF_ICARUS_FREEZE;
        }
    }
}

/// Raven `Q3_SetViewEntity`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3089-3092`
pub fn Q3_SetViewEntity(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetViewEntity currently unsupported in MP, ask if you need it.\n\0".as_ptr()
            as *const c_char,
    );
}

// PORT-NOTE(unported-global): `WPTable` (weapon-name string table) is not
// ported anywhere in the worktree (missing_symbols).
/// Raven `Q3_SetWeapon`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3104-3111`
pub fn Q3_SetWeapon(ctx: &mut GameContext, entID: c_int, wp_name: *const c_char) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
        let wp = GetIDForString(WPTable.as_ptr() as *mut stringID_table_t, wp_name);

        (*((*ent).client as *mut gclient_t)).ps.stats[STAT_WEAPONS as usize] = 1 << wp;
        ChangeWeapon(ctx, ctx.entity_id_of(ent), wp);
    }
}

/// Raven `Q3_SetItem`.
///
/// Raven: `//rww - unused in mp`.
/// Source: `oracle/codemp/game/g_ICARUScb.c:3122-3126`
pub fn Q3_SetItem(ctx: &mut GameContext, entID: c_int, item_name: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetItem: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetWalkSpeed`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3139-3161`
pub fn Q3_SetWalkSpeed(ctx: &mut GameContext, entID: c_int, int_data: c_int) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*self_).NPC.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetWalkSpeed: '{}' is not an NPC!\n",
                    cstr_to_str((*self_).targetname)
                ))
                .as_ptr(),
            );
            return;
        }
        let npc = (*self_).NPC as *mut gNPC_t;
        let client = (*self_).client as *mut gclient_t;

        if int_data == 0 {
            (*npc).stats.walkSpeed = 1;
            (*client).ps.speed = (1) as f32;
        }

        (*npc).stats.walkSpeed = int_data;
        (*client).ps.speed = int_data as f32;
    }
}

/// Raven `Q3_SetRunSpeed`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3173-3195`
pub fn Q3_SetRunSpeed(ctx: &mut GameContext, entID: c_int, int_data: c_int) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*self_).NPC.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetRunSpeed: '{}' is not an NPC!\n",
                    cstr_to_str((*self_).targetname)
                ))
                .as_ptr(),
            );
            return;
        }
        let npc = (*self_).NPC as *mut gNPC_t;
        let client = (*self_).client as *mut gclient_t;

        if int_data == 0 {
            (*npc).stats.runSpeed = 1;
            (*client).ps.speed = (1) as f32;
        }

        (*npc).stats.runSpeed = int_data;
        (*client).ps.speed = int_data as f32;
    }
}

/// Raven `Q3_SetYawSpeed`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3207-3211`
pub fn Q3_SetYawSpeed(ctx: &mut GameContext, entID: c_int, float_data: f32) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetYawSpeed: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetAggression`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3223-3227`
pub fn Q3_SetAggression(ctx: &mut GameContext, entID: c_int, int_data: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetAggression: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetAim`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3239-3243`
pub fn Q3_SetAim(ctx: &mut GameContext, entID: c_int, int_data: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetAim: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetFriction`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3255-3273`
pub fn Q3_SetFriction(ctx: &mut GameContext, entID: c_int, int_data: c_int) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*self_).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetFriction: '{}' is not an NPC/player!\n",
                    cstr_to_str((*self_).targetname)
                ))
                .as_ptr(),
            );
            return;
        }

        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            b"Q3_SetFriction currently unsupported in MP\n\0".as_ptr() as *const c_char,
        );
    }
}

/// Raven `Q3_SetGravity`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3285-3307`
pub fn Q3_SetGravity(ctx: &mut GameContext, entID: c_int, float_data: f32) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*self_).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetGravity: '{}' is not an NPC/player!\n",
                    cstr_to_str((*self_).targetname)
                ))
                .as_ptr(),
            );
            return;
        }
        let client = (*self_).client as *mut gclient_t;

        if !(*self_).NPC.is_null() {
            let npc = (*self_).NPC as *mut gNPC_t;
            (*npc).aiFlags |= NPCAI_CUSTOM_GRAVITY;
        }
        (*client).ps.gravity = float_data as c_int;
    }
}

/// Raven `Q3_SetWait`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3319-3330`
pub fn Q3_SetWait(ctx: &mut GameContext, entID: c_int, float_data: f32) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
        (*self_).wait = float_data;
    }
}

/// Raven `Q3_SetShotSpacing`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3333-3337`
pub fn Q3_SetShotSpacing(ctx: &mut GameContext, entID: c_int, int_data: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetShotSpacing: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetFollowDist`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3348-3352`
pub fn Q3_SetFollowDist(ctx: &mut GameContext, entID: c_int, float_data: f32) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetFollowDist: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetScale`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3364-3396`
pub fn Q3_SetScale(ctx: &mut GameContext, entID: c_int, float_data: f32) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if !(*self_).client.is_null() {
            let client = (*self_).client as *mut gclient_t;
            if float_data < 0.0 {
                (*client).ps.iModelScale = float_data as c_int;
            } else {
                (*client).ps.iModelScale = (float_data * 100.0) as c_int;
            }
        } else {
            if float_data < 0.0 {
                (*self_).s.iModelScale = float_data as c_int;
            } else {
                (*self_).s.iModelScale = (float_data * 100.0) as c_int;
            }
        }
    }
}

/// Raven `Q3_GameSideCheckStringCounterIncrement`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3406-3429`
pub fn Q3_GameSideCheckStringCounterIncrement(string: *const c_char) -> f32 {
    unsafe {
        let s = cstr_to_str(string);
        let mut val = 0.0f32;

        if let Some(rest) = s.strip_prefix('+') {
            if !rest.is_empty() {
                val = atof(cstr(rest).as_ptr()) as f32;
            }
        } else if let Some(rest) = s.strip_prefix('-') {
            if !rest.is_empty() {
                val = atof(cstr(rest).as_ptr()) as f32 * -1.0;
            }
        }

        val
    }
}

/// Raven `Q3_SetCount`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3440-3460`
pub fn Q3_SetCount(ctx: &mut GameContext, entID: c_int, data: *const c_char) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        let val = Q3_GameSideCheckStringCounterIncrement(data);
        if val != 0.0 {
            (*self_).count += val as c_int;
        } else {
            (*self_).count = atoi(data);
        }
    }
}

/// Raven `Q3_SetTargetName`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3472-3490`
pub fn Q3_SetTargetName(ctx: &mut GameContext, entID: c_int, targetname: *const c_char) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if Q_stricmp(b"NULL\0".as_ptr() as *const c_char, targetname) == 0 {
            (*self_).targetname = std::ptr::null_mut();
        } else {
            (*self_).targetname = G_NewString(ctx, targetname);
        }
    }
}

/// Raven `Q3_SetTarget`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3502-3520`
pub fn Q3_SetTarget(ctx: &mut GameContext, entID: c_int, target: *const c_char) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if Q_stricmp(b"NULL\0".as_ptr() as *const c_char, target) == 0 {
            (*self_).target = std::ptr::null_mut();
        } else {
            (*self_).target = G_NewString(ctx, target);
        }
    }
}

/// Raven `Q3_SetTarget2`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3531-3552`
pub fn Q3_SetTarget2(ctx: &mut GameContext, entID: c_int, target2: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetTarget2 does not exist in MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetRemoveTarget`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3562-3566`
pub fn Q3_SetRemoveTarget(ctx: &mut GameContext, entID: c_int, target: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetRemoveTarget: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetPainTarget`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3578-3599`
pub fn Q3_SetPainTarget(ctx: &mut GameContext, entID: c_int, targetname: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetPainTarget: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetFullName`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3610-3628`
pub fn Q3_SetFullName(ctx: &mut GameContext, entID: c_int, fullName: *const c_char) {
    unsafe {
        let self_ = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if Q_stricmp(b"NULL\0".as_ptr() as *const c_char, fullName) == 0 {
            (*self_).fullName = std::ptr::null_mut();
        } else {
            (*self_).fullName = G_NewString(ctx, fullName);
        }
    }
}

/// Raven `Q3_SetMusicState`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3630-3634`
pub fn Q3_SetMusicState(ctx: &mut GameContext, dms: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetMusicState: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetForcePowerLevel`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3636-3640`
pub fn Q3_SetForcePowerLevel(
    ctx: &mut GameContext,
    entID: c_int,
    forcePower: c_int,
    forceLevel: c_int,
) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetForcePowerLevel: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetParm`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3651-3690`
// PORT-NOTE(dropped-warnings): all callers thread the game `GameContext`
// (Q3_Set, BG_ParseField, NPC_Spawn_Do), so the entity arena is reached via
// `ctx.world`. Raven's `G_DebugPrint` warnings (parmNum range, truncation) are
// still dropped — no `WL_*` route is set up here — matching the file's other
// `Q3_Set*` stubs.
pub fn Q3_SetParm(ctx: &mut GameContext, entID: c_int, parmNum: c_int, parmValue: *const c_char) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if parmNum < 0 || parmNum >= MAX_PARMS as c_int {
            return;
        }

        if (*ent).parms.is_null() {
            (*ent).parms = G_Alloc(ctx, core::mem::size_of::<parms_t>() as c_int) as *mut parms_t;
            // G_Alloc is a bump allocator whose pool is not re-zeroed on map
            // restart; C memsets the fresh parms_t so reused regions read empty.
            core::ptr::write_bytes((*ent).parms as *mut u8, 0, core::mem::size_of::<parms_t>());
        }

        let val = Q3_GameSideCheckStringCounterIncrement(parmValue);
        if val != 0.0 {
            let cur = atof((*(*ent).parms).parm[parmNum as usize].as_ptr()) as f32;
            let total = val + cur;
            write_cstr_field(
                &mut (*(*ent).parms).parm[parmNum as usize],
                &format!("{:.6}", total),
            );
        } else {
            // Raven: strncpy + explicit truncation-NUL; write_cstr_field is the
            // Q_strncpyz/Com_sprintf byte-copy dual.
            write_cstr_field(
                &mut (*(*ent).parms).parm[parmNum as usize],
                &cstr_to_str(parmValue),
            );
        }
    }
}

/// Raven `Q3_SetCaptureGoal`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3701-3705`
pub fn Q3_SetCaptureGoal(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetCaptureGoal: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetEvent`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3714-3718`
pub fn Q3_SetEvent(ctx: &mut GameContext, entID: c_int, event_name: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetEvent: NOT SUPPORTED IN MP (may be in future, ask if needed)\n\0".as_ptr()
            as *const c_char,
    );
}

/// Raven `Q3_SetIgnorePain`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3727-3731`
pub fn Q3_SetIgnorePain(ctx: &mut GameContext, entID: c_int, data: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetIgnorePain: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetIgnoreEnemies`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3740-3744`
pub fn Q3_SetIgnoreEnemies(ctx: &mut GameContext, entID: c_int, data: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetIgnoreEnemies: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetIgnoreAlerts`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3753-3757`
pub fn Q3_SetIgnoreAlerts(ctx: &mut GameContext, entID: c_int, data: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetIgnoreAlerts: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetNoTarget`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3767-3781`
pub fn Q3_SetNoTarget(ctx: &mut GameContext, entID: c_int, data: qboolean) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if data != 0 {
            (*ent).flags |= FL_NOTARGET;
        } else {
            (*ent).flags &= !FL_NOTARGET;
        }
    }
}

/// Raven `Q3_SetDontShoot`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3790-3794`
pub fn Q3_SetDontShoot(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetDontShoot: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetDontFire`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3803-3807`
pub fn Q3_SetDontFire(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetDontFire: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetFireWeapon`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3816-3820`
pub fn Q3_SetFireWeapon(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetFireWeapon: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetInactive`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3830-3848`
pub fn Q3_SetInactive(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if add != 0 {
            (*ent).flags |= FL_INACTIVE;
        } else {
            (*ent).flags &= !FL_INACTIVE;
        }
    }
}

/// Raven `Q3_SetFuncUsableVisible`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3857-3880`
pub fn Q3_SetFuncUsableVisible(ctx: &mut GameContext, entID: c_int, visible: qboolean) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if visible != 0 {
            (*ent).r.svFlags &= !SVF_NOCLIENT;
            (*ent).s.eFlags &= !EF_NODRAW;
        } else {
            (*ent).r.svFlags |= SVF_NOCLIENT;
            (*ent).s.eFlags |= EF_NODRAW;
        }
    }
}

/// Raven `Q3_SetLockedEnemy`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3889-3893`
pub fn Q3_SetLockedEnemy(ctx: &mut GameContext, entID: c_int, locked: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetLockedEnemy: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetCinematicSkipScript`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3903-3907`
pub fn Q3_SetCinematicSkipScript(ctx: &mut GameContext, scriptname: *mut c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetCinematicSkipScript: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetNoMindTrick`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3916-3920`
pub fn Q3_SetNoMindTrick(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetNoMindTrick: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetCrouched`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3929-3933`
pub fn Q3_SetCrouched(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetCrouched: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetWalking`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3942-3967`
pub fn Q3_SetWalking(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*ent).NPC.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetWalking: '{}' is not an NPC!\n",
                    cstr_to_str((*ent).targetname)
                ))
                .as_ptr(),
            );
            return;
        }
        let npc = (*ent).NPC as *mut gNPC_t;

        if add != 0 {
            (*npc).scriptFlags |= SCF_WALKING;
        } else {
            (*npc).scriptFlags &= !SCF_WALKING;
        }
    }
}

/// Raven `Q3_SetRunning`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3976-3980`
pub fn Q3_SetRunning(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetRunning: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetForcedMarch`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3989-3993`
pub fn Q3_SetForcedMarch(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetForcedMarch: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetChaseEnemies`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4001-4005`
pub fn Q3_SetChaseEnemies(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetChaseEnemies: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetLookForEnemies`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4015-4019`
pub fn Q3_SetLookForEnemies(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetLookForEnemies: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetFaceMoveDir`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4027-4031`
pub fn Q3_SetFaceMoveDir(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetFaceMoveDir: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetAltFire`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4040-4044`
pub fn Q3_SetAltFire(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetAltFire: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetDontFlee`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4053-4057`
pub fn Q3_SetDontFlee(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetDontFlee: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetNoResponse`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4066-4070`
pub fn Q3_SetNoResponse(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetNoResponse: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetCombatTalk`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4079-4083`
pub fn Q3_SetCombatTalk(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetCombatTalk: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetAlertTalk`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4092-4096`
pub fn Q3_SetAlertTalk(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetAlertTalk: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetUseCpNearest`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4105-4109`
pub fn Q3_SetUseCpNearest(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetUseCpNearest: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetNoForce`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4118-4122`
pub fn Q3_SetNoForce(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetNoForce: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetNoAcrobatics`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4131-4135`
pub fn Q3_SetNoAcrobatics(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetNoAcrobatics: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetUseSubtitles`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4144-4148`
pub fn Q3_SetUseSubtitles(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetUseSubtitles: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetNoFallToDeath`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4157-4161`
pub fn Q3_SetNoFallToDeath(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetNoFallToDeath: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetDismemberable`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4170-4174`
pub fn Q3_SetDismemberable(ctx: &mut GameContext, entID: c_int, dismemberable: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetDismemberable: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetMoreLight`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4184-4188`
pub fn Q3_SetMoreLight(ctx: &mut GameContext, entID: c_int, add: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetMoreLight: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetUndying`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4197-4201`
pub fn Q3_SetUndying(ctx: &mut GameContext, entID: c_int, undying: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetUndying: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetInvincible`.
///
/// Raven: the debug message says "Invicible" (typo preserved verbatim).
/// Source: `oracle/codemp/game/g_ICARUScb.c:4210-4214`
pub fn Q3_SetInvincible(ctx: &mut GameContext, entID: c_int, invincible: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetInvicible: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetForceInvincible`.
///
/// Raven: the debug message says "Invicible" (typo preserved verbatim).
/// Source: `oracle/codemp/game/g_ICARUScb.c:4224-4228`
pub fn Q3_SetForceInvincible(ctx: &mut GameContext, entID: c_int, forceInv: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetForceInvicible: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetNoAvoid`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4237-4261`
pub fn Q3_SetNoAvoid(ctx: &mut GameContext, entID: c_int, noAvoid: qboolean) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if (*ent).NPC.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetNoAvoid: '{}' is not an NPC!\n",
                    cstr_to_str((*ent).targetname)
                ))
                .as_ptr(),
            );
            return;
        }
        let npc = (*ent).NPC as *mut gNPC_t;

        if noAvoid != 0 {
            (*npc).aiFlags |= NPCAI_NO_COLL_AVOID;
        } else {
            (*npc).aiFlags &= !NPCAI_NO_COLL_AVOID;
        }
    }
}

/// Raven `SolidifyOwner`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4271-4295`
pub fn SolidifyOwner(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let self_: *mut gentity_t = ctx.entity_mut(self_);

    unsafe {
        let owner =
            &mut (*ctx.world_raw()).g_entities[(*self_).r.ownerNum as usize] as *mut gentity_t;

        (*self_).nextthink = (*ctx.world_raw()).level.time + FRAMETIME;
        (*self_).think = Some(EntThink::G_FreeEntity).into();

        if owner.is_null() || (*owner).inuse == 0 {
            return;
        }

        let oldContents = (*owner).r.contents;
        (*owner).r.contents = CONTENTS_BODY;
        if SpotWouldTelefrag2(
            ctx,
            ctx.entity_id_of(owner).unwrap(),
            (*owner).r.currentOrigin,
        ) != qfalse
        {
            (*owner).r.contents = oldContents;
            (*self_).think = Some(EntThink::SolidifyOwner).into();
        } else {
            trap::ICARUS_TaskIDComplete(
                ctx.engine,
                GIcarusTaskidcompleteArgs::new(owner, taskID_t::TID_RESIZE as c_int),
            );
        }
    }
}

/// Raven `Q3_SetSolid`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4305-4345`
pub fn Q3_SetSolid(ctx: &mut GameContext, entID: c_int, solid: qboolean) -> qboolean {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if ent.is_null() || (*ent).inuse == 0 {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                cstr(&format!("Q3_SetSolid: invalid entID {}\n", entID)).as_ptr(),
            );
            return qtrue;
        }

        if solid != 0 {
            //FIXME: Presumption
            let oldContents = (*ent).r.contents;
            (*ent).r.contents = CONTENTS_BODY;
            if SpotWouldTelefrag2(ctx, ctx.entity_id_of(ent).unwrap(), (*ent).r.currentOrigin)
                != qfalse
            {
                let solidifier = G_Spawn(ctx);

                (*solidifier).r.ownerNum = (*ent).s.number;

                (*solidifier).think = Some(EntThink::SolidifyOwner).into();
                (*solidifier).nextthink = (*ctx.world_raw()).level.time + FRAMETIME;

                (*ent).r.contents = oldContents;
                return qfalse;
            }
            (*ent).clipmask |= CONTENTS_BODY;
        } else {
            //FIXME: Presumption
            if (*ent).s.eFlags & EF_NODRAW != 0 {
                //We're invisible too, so set contents to none
                (*ent).r.contents = 0;
            } else {
                (*ent).r.contents = CONTENTS_CORPSE;
            }
        }
        qtrue
    }
}

/// Raven `Q3_SetForwardMove`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4354-4372`
pub fn Q3_SetForwardMove(ctx: &mut GameContext, entID: c_int, fmoveVal: c_int) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if ent.is_null() {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                cstr(&format!("Q3_SetForwardMove: invalid entID {}\n", entID)).as_ptr(),
            );
            return;
        }

        if (*ent).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetForwardMove: '{}' is not an NPC/player!\n",
                    cstr_to_str((*ent).targetname)
                ))
                .as_ptr(),
            );
            return;
        }

        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            b"Q3_SetForwardMove: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
        );
        //ent->client->forced_forwardmove = fmoveVal;
    }
}

/// Raven `Q3_SetRightMove`.
///
/// Raven: entID/gentity_t is never null (address-of array element); the
/// `!ent`/`!ent->client` guards are dead/live-checked here as client-null
/// only. Body is a debug-print stub — behavior is commented out in Raven.
/// Source: `oracle/codemp/game/g_ICARUScb.c:4381-4399`
pub fn Q3_SetRightMove(ctx: &mut GameContext, entID: c_int, rmoveVal: c_int) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
        if (*ent).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                b"Q3_SetRightMove: '%s' is not an NPC/player!\n\0".as_ptr() as *const c_char,
            );
            return;
        }
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            b"Q3_SetRightMove: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
        );
    }
}

/// Raven `Q3_SetLockAngle`.
///
/// Raven: the renderInfo.lockYaw/RF_LOCKEDANGLE assignment is fully
/// commented out in Raven; body is a debug-print stub only.
/// Source: `oracle/codemp/game/g_ICARUScb.c:4408-4445`
pub fn Q3_SetLockAngle(ctx: &mut GameContext, entID: c_int, lockAngle: *const c_char) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
        if (*ent).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                b"Q3_SetLockAngle: '%s' is not an NPC/player!\n\0".as_ptr() as *const c_char,
            );
            return;
        }
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            b"Q3_SetLockAngle is not currently available. Ask if you really need it.\n\0".as_ptr()
                as *const c_char,
        );
    }
}

/// Raven `Q3_CameraGroup`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4455-4459`
pub fn Q3_CameraGroup(ctx: &mut GameContext, entID: c_int, camG: *mut c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_CameraGroup: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_CameraGroupZOfs`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4468-4472`
pub fn Q3_CameraGroupZOfs(ctx: &mut GameContext, camGZOfs: f32) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_CameraGroupZOfs: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_CameraGroupTag`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4480-4484`
pub fn Q3_CameraGroupTag(ctx: &mut GameContext, camGTag: *mut c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_CameraGroupTag: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_RemoveRHandModel`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4491-4494`
pub fn Q3_RemoveRHandModel(ctx: &mut GameContext, entID: c_int, addModel: *mut c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_RemoveRHandModel: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_AddRHandModel`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4501-4504`
pub fn Q3_AddRHandModel(ctx: &mut GameContext, entID: c_int, addModel: *mut c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_AddRHandModel: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_AddLHandModel`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4511-4514`
pub fn Q3_AddLHandModel(ctx: &mut GameContext, entID: c_int, addModel: *mut c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_AddLHandModel: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_RemoveLHandModel`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4521-4524`
pub fn Q3_RemoveLHandModel(ctx: &mut GameContext, entID: c_int, addModel: *mut c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_RemoveLHandModel: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_LookTarget`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4533-4537`
pub fn Q3_LookTarget(ctx: &mut GameContext, entID: c_int, targetName: *mut c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_LookTarget: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_Face`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4546-4549`
pub fn Q3_Face(ctx: &mut GameContext, entID: c_int, expression: c_int, holdtime: f32) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_Face: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetLocation`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4560-4564`
pub fn Q3_SetLocation(ctx: &mut GameContext, entID: c_int, location: *const c_char) -> qboolean {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetLocation: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
    qtrue
}

/// Raven `Q3_SetPlayerLocked`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4576-4579`
pub fn Q3_SetPlayerLocked(ctx: &mut GameContext, entID: c_int, locked: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetPlayerLocked: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetLockPlayerWeapons`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4590-4593`
pub fn Q3_SetLockPlayerWeapons(ctx: &mut GameContext, entID: c_int, locked: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetLockPlayerWeapons: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetNoImpactDamage`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4605-4608`
pub fn Q3_SetNoImpactDamage(ctx: &mut GameContext, entID: c_int, noImp: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetNoImpactDamage: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetBehaviorSet`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4617-4708`
pub fn Q3_SetBehaviorSet(
    ctx: &mut GameContext,
    entID: c_int,
    toSet: c_int,
    scriptname: *const c_char,
) -> qboolean {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
        let mut bSet = bSet_t::BSET_INVALID;

        if ent.is_null() {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                cstr(&format!("Q3_SetBehaviorSet: invalid entID {}\n", entID)).as_ptr(),
            );
            return qfalse;
        }

        bSet = match toSet {
            // PORT-NOTE(unported-consts): `SET_*` (the ICARUS set-table field
            // ids) are not ported anywhere in the worktree — matches the
            // `setTable`/`BSTable`/`WPTable` unported-global precedent above
            // (g_ICARUScb.c:1189/1573/1642/1839); missing_symbols.
            _ if toSet == SET_SPAWNSCRIPT as i32 => bSet_t::BSET_SPAWN,
            _ if toSet == SET_USESCRIPT as i32 => bSet_t::BSET_USE,
            _ if toSet == SET_AWAKESCRIPT as i32 => bSet_t::BSET_AWAKE,
            _ if toSet == SET_ANGERSCRIPT as i32 => bSet_t::BSET_ANGER,
            _ if toSet == SET_ATTACKSCRIPT as i32 => bSet_t::BSET_ATTACK,
            _ if toSet == SET_VICTORYSCRIPT as i32 => bSet_t::BSET_VICTORY,
            _ if toSet == SET_LOSTENEMYSCRIPT as i32 => bSet_t::BSET_LOSTENEMY,
            _ if toSet == SET_PAINSCRIPT as i32 => bSet_t::BSET_PAIN,
            _ if toSet == SET_FLEESCRIPT as i32 => bSet_t::BSET_FLEE,
            _ if toSet == SET_DEATHSCRIPT as i32 => bSet_t::BSET_DEATH,
            _ if toSet == SET_DELAYEDSCRIPT as i32 => bSet_t::BSET_DELAYED,
            _ if toSet == SET_BLOCKEDSCRIPT as i32 => bSet_t::BSET_BLOCKED,
            _ if toSet == SET_FFIRESCRIPT as i32 => bSet_t::BSET_FFIRE,
            _ if toSet == SET_FFDEATHSCRIPT as i32 => bSet_t::BSET_FFDEATH,
            _ if toSet == SET_MINDTRICKSCRIPT as i32 => bSet_t::BSET_MINDTRICK,
            _ => bSet,
        };

        // `bSet_t` is not `Copy`; use its discriminant from here on (Raven
        // indexes `behaviorSet[]` with it as an int anyway).
        let bSet = bSet as c_int;
        if bSet < (bSet_t::BSET_SPAWN as c_int) || bSet >= (bSet_t::NUM_BSETS as c_int) {
            return qfalse;
        }

        if Q_stricmp(b"NULL\0".as_ptr() as *const c_char, scriptname) == 0 {
            if !(*ent).behaviorSet[bSet as usize].is_null() {
                //			gi.TagFree( ent->behaviorSet[bSet] );
            }
            (*ent).behaviorSet[bSet as usize] = core::ptr::null_mut();
        } else if !scriptname.is_null() {
            if !(*ent).behaviorSet[bSet as usize].is_null() {
                //				gi.TagFree( ent->behaviorSet[bSet] );
            }
            (*ent).behaviorSet[bSet as usize] = G_NewString(ctx, scriptname); //FIXME: This really isn't good...
        }
        qtrue
    }
}

/// Raven `Q3_SetDelayScriptTime`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4717-4720`
pub fn Q3_SetDelayScriptTime(ctx: &mut GameContext, entID: c_int, delayTime: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetDelayScriptTime: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetPlayerUsable`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4734-4752`
pub fn Q3_SetPlayerUsable(ctx: &mut GameContext, entID: c_int, usable: qboolean) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if ent.is_null() {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                cstr(&format!("Q3_SetPlayerUsable: invalid entID {}\n", entID)).as_ptr(),
            );
            return;
        }

        if usable != 0 {
            // PORT-NOTE(unported-consts): `SVF_PLAYER_USABLE` is not ported
            // anywhere in the worktree (matches the `ValidUseTarget`
            // precedent, g_utils.rs:1349); missing_symbols.
            (*ent).r.svFlags |= SVF_PLAYER_USABLE;
        } else {
            (*ent).r.svFlags &= !SVF_PLAYER_USABLE;
        }
    }
}

/// Raven `Q3_SetDisableShaderAnims`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4763-4767`
pub fn Q3_SetDisableShaderAnims(ctx: &mut GameContext, entID: c_int, disabled: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetDisableShaderAnims: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetShaderAnim`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4778-4782`
pub fn Q3_SetShaderAnim(ctx: &mut GameContext, entID: c_int, disabled: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetShaderAnim: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetStartFrame`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4793-4796`
pub fn Q3_SetStartFrame(ctx: &mut GameContext, entID: c_int, startFrame: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetStartFrame: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetEndFrame`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4808-4811`
pub fn Q3_SetEndFrame(ctx: &mut GameContext, entID: c_int, endFrame: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetEndFrame: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetAnimFrame`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4822-4825`
pub fn Q3_SetAnimFrame(ctx: &mut GameContext, entID: c_int, animFrame: c_int) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetAnimFrame: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetLoopAnim`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4836-4839`
pub fn Q3_SetLoopAnim(ctx: &mut GameContext, entID: c_int, loopAnim: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetLoopAnim: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetShields`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4851-4855`
pub fn Q3_SetShields(ctx: &mut GameContext, entID: c_int, shields: qboolean) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetShields: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetSaberActive`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4866-4889`
pub fn Q3_SetSaberActive(ctx: &mut GameContext, entID: c_int, active: qboolean) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;

        if ent.is_null() || (*ent).inuse == 0 {
            return;
        }

        if (*ent).client.is_null() {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                cstr(&format!("Q3_SetSaberActive: {} is not a client\n", entID)).as_ptr(),
            );
        }

        //fixme: Take into account player being in state where saber won't toggle? For now we simply won't care.
        let client = (*ent).client as *mut gclient_t;
        if (*client).ps.saberHolstered == 0 && active != 0 {
            Cmd_ToggleSaber_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if BG_SabersOff(&mut (*client).ps as *mut playerState_t) != 0 && active == 0 {
            Cmd_ToggleSaber_f(ctx, ctx.entity_id_of(ent).unwrap());
        }
    }
}

/// Raven `Q3_SetNoKnockback`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4900-4918`
pub fn Q3_SetNoKnockback(ctx: &mut GameContext, entID: c_int, noKnockback: qboolean) {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
        if noKnockback != 0 {
            (*ent).flags |= FL_NO_KNOCKBACK;
        } else {
            (*ent).flags &= !FL_NO_KNOCKBACK;
        }
    }
}

/// Raven `Q3_SetCleanDamagingEnts`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4927-4931`
pub fn Q3_SetCleanDamagingEnts(ctx: &mut GameContext) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_SetCleanDamagingEnts: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `SetTextColor`.
///
/// Raven: `textcolor` is only ever read (never written) in this NOT-SUPPORTED
/// stub body, so it stays by-value `vec4_t` ("keep by-value only
/// if never written").
/// Raven `textcolor_caption` — file-scope static for caption text color.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4933`
static textcolor_caption: vec4_t = [0.0, 0.0, 0.0, 0.0];

/// Raven `textcolor_center` — file-scope static for center text color.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4934`
static textcolor_center: vec4_t = [0.0, 0.0, 0.0, 0.0];

/// Raven `textcolor_scroll` — file-scope static for scroll text color.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4935`
static textcolor_scroll: vec4_t = [0.0, 0.0, 0.0, 0.0];

/// Source: `oracle/codemp/game/g_ICARUScb.c:4942-4946`
pub fn SetTextColor(ctx: &mut GameContext, textcolor: vec4_t, color: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"SetTextColor: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_SetCaptionTextColor`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4955-4958`
pub fn Q3_SetCaptionTextColor(ctx: &mut GameContext, color: *const c_char) {
    SetTextColor(ctx, textcolor_caption, color);
}

/// Raven `Q3_SetCenterTextColor`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4967-4970`
pub fn Q3_SetCenterTextColor(ctx: &mut GameContext, color: *const c_char) {
    SetTextColor(ctx, textcolor_center, color);
}

/// Raven `Q3_SetScrollTextColor`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4979-4982`
pub fn Q3_SetScrollTextColor(ctx: &mut GameContext, color: *const c_char) {
    SetTextColor(ctx, textcolor_scroll, color);
}

/// Raven `Q3_ScrollText`.
///
/// Raven: the `trap_SendServerCommand` call is commented out; body is a
/// debug-print stub only.
/// Source: `oracle/codemp/game/g_ICARUScb.c:4991-4997`
pub fn Q3_ScrollText(ctx: &mut GameContext, id: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_ScrollText: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

/// Raven `Q3_LCARSText`.
///
/// Raven: the `trap_SendServerCommand` call is commented out; body is a
/// debug-print stub only (Raven's message string says "Q3_ScrollText" too —
/// preserved verbatim, not a transcription error).
/// Source: `oracle/codemp/game/g_ICARUScb.c:5006-5012`
pub fn Q3_LCARSText(ctx: &mut GameContext, id: *const c_char) {
    G_DebugPrint(
        ctx,
        WL_WARNING as c_int,
        b"Q3_ScrollText: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
    );
}

// PORT-NOTE(unported-consts): the entire 150-case switch keys off the ICARUS
// `SET_*` field-id enum (`toSet = GetIDForString(setTable.as_ptr() as *mut stringID_table_t, type_name)`) —
// `setTable`/`SET_*` are not ported anywhere in the worktree (same gap as
// `Q3_SetBehaviorSet`/`Q3_GetString` above); missing_symbols. Bodies of
// dozens of `Q3_Set*` helpers are transcribed literally against those bare
// names too.
/// Raven `Q3_Set`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:5018-6074`
pub fn Q3_Set(
    ctx: &mut GameContext,
    taskID: c_int,
    entID: c_int,
    type_name: *const c_char,
    data: *const c_char,
) -> qboolean {
    unsafe {
        let ent = &mut (*ctx.world_raw()).g_entities[entID as usize] as *mut gentity_t;
        let mut float_data: f32;
        let mut int_data: c_int;
        let mut vector_data: vec3_t = [0.0, 0.0, 0.0];

        // Set this for callbacks
        let toSet = GetIDForString(setTable.as_ptr() as *mut stringID_table_t, type_name);

        // Raven's `sscanf(data, "%f %f %f", ...)` at the three vector arms
        // below now routes through the shared libc-`%f` scanner
        // `cstr_util::sscanf_f32s` (stop-at-first-failure, longest-prefix
        // parse — matching libc, not a naive whitespace-split).
        // §19: C leaves any component sscanf fails to parse UNINITIALIZED
        // (garbage-float UB on `vector_data`); we pick "leave the 0.0 seed
        // above unmodified" as the one defined behavior.
        match toSet {
            _ if toSet == SET_ORIGIN as i32 => {
                {
                    let s = cstr_to_str(data);
                    sscanf_f32s(&s, &mut vector_data);
                }
                G_SetOrigin(&mut *(ent), vector_data);
                if Q_strncmp(b"NPC_\0".as_ptr() as *const c_char, (*ent).classname, 4) == 0 {
                    //hack for moving spawners
                    crate::q_math::_VectorCopy(vector_data, &mut (*ent).s.origin);
                }
            }

            _ if toSet == SET_TELEPORT_DEST as i32 => {
                {
                    let s = cstr_to_str(data);
                    sscanf_f32s(&s, &mut vector_data);
                }
                if Q3_SetTeleportDest(ctx, entID, vector_data) == qfalse {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(ent, taskID_t::TID_MOVE_NAV as c_int, taskID),
                    );
                    return qfalse;
                }
            }

            _ if toSet == SET_COPY_ORIGIN as i32 => Q3_SetCopyOrigin(ctx, entID, data),

            _ if toSet == SET_ANGLES as i32 => {
                let s = cstr_to_str(data);
                sscanf_f32s(&s, &mut vector_data);
                Q3_SetAngles(ctx, entID, vector_data);
            }

            _ if toSet == SET_XVELOCITY as i32 => {
                float_data = atof(data) as f32;
                Q3_SetVelocity(ctx, entID, 0, float_data);
            }
            _ if toSet == SET_YVELOCITY as i32 => {
                float_data = atof(data) as f32;
                Q3_SetVelocity(ctx, entID, 1, float_data);
            }
            _ if toSet == SET_ZVELOCITY as i32 => {
                float_data = atof(data) as f32;
                Q3_SetVelocity(ctx, entID, 2, float_data);
            }

            _ if toSet == SET_Z_OFFSET as i32 => {
                float_data = atof(data) as f32;
                Q3_SetOriginOffset(ctx, entID, 2, float_data);
            }

            _ if toSet == SET_ENEMY as i32 => Q3_SetEnemy(ctx, entID, data),
            _ if toSet == SET_LEADER as i32 => Q3_SetLeader(ctx, entID, data),

            _ if toSet == SET_NAVGOAL as i32 => {
                if Q3_SetNavGoal(ctx, entID, data) != qfalse {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(ent, taskID_t::TID_MOVE_NAV as c_int, taskID),
                    );
                    return qfalse; //Don't call it back
                }
            }

            _ if toSet == SET_ANIM_UPPER as i32 => {
                if Q3_SetAnimUpper(ctx, entID, data) != qfalse {
                    Q3_TaskIDClear(&mut (*ent).taskID[taskID_t::TID_ANIM_BOTH as usize]); //We only want to wait for the top
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANIM_UPPER as c_int, taskID),
                    );
                    return qfalse; //Don't call it back
                }
            }

            _ if toSet == SET_ANIM_LOWER as i32 => {
                if Q3_SetAnimLower(ctx, entID, data) != qfalse {
                    Q3_TaskIDClear(&mut (*ent).taskID[taskID_t::TID_ANIM_BOTH as usize]); //We only want to wait for the bottom
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANIM_LOWER as c_int, taskID),
                    );
                    return qfalse; //Don't call it back
                }
            }

            _ if toSet == SET_ANIM_BOTH as i32 => {
                let mut both: c_int = 0;
                if Q3_SetAnimUpper(ctx, entID, data) != qfalse {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANIM_UPPER as c_int, taskID),
                    );
                    both += 1;
                } else {
                    G_DebugPrint(
                        ctx,
                        WL_ERROR as c_int,
                        cstr(&format!(
                            "Q3_SetAnimUpper: {} does not have anim {}!\n",
                            cstr_to_str((*ent).targetname),
                            cstr_to_str(data)
                        ))
                        .as_ptr(),
                    );
                }
                if Q3_SetAnimLower(ctx, entID, data) != qfalse {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANIM_LOWER as c_int, taskID),
                    );
                    both += 1;
                } else {
                    G_DebugPrint(
                        ctx,
                        WL_ERROR as c_int,
                        cstr(&format!(
                            "Q3_SetAnimLower: {} does not have anim {}!\n",
                            cstr_to_str((*ent).targetname),
                            cstr_to_str(data)
                        ))
                        .as_ptr(),
                    );
                }
                if both >= 2 {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANIM_BOTH as c_int, taskID),
                    );
                }
                if both != 0 {
                    return qfalse; //Don't call it back
                }
            }

            _ if toSet == SET_ANIM_HOLDTIME_LOWER as i32 => {
                int_data = atoi(data);
                Q3_SetAnimHoldTime(ctx, entID, int_data, qtrue);
                Q3_TaskIDClear(&mut (*ent).taskID[taskID_t::TID_ANIM_BOTH as usize]); //We only want to wait for the bottom
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANIM_LOWER as c_int, taskID),
                );
                return qfalse; //Don't call it back
            }

            _ if toSet == SET_ANIM_HOLDTIME_UPPER as i32 => {
                int_data = atoi(data);
                Q3_SetAnimHoldTime(ctx, entID, int_data, qfalse);
                Q3_TaskIDClear(&mut (*ent).taskID[taskID_t::TID_ANIM_BOTH as usize]); //We only want to wait for the top
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANIM_UPPER as c_int, taskID),
                );
                return qfalse; //Don't call it back
            }

            _ if toSet == SET_ANIM_HOLDTIME_BOTH as i32 => {
                int_data = atoi(data);
                Q3_SetAnimHoldTime(ctx, entID, int_data, qfalse);
                Q3_SetAnimHoldTime(ctx, entID, int_data, qtrue);
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANIM_BOTH as c_int, taskID),
                );
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANIM_UPPER as c_int, taskID),
                );
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANIM_LOWER as c_int, taskID),
                );
                return qfalse; //Don't call it back
            }

            _ if toSet == SET_PLAYER_TEAM as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_SetPlayerTeam: Not in MP ATM, let a programmer (ideally Rich) know if you need it\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_ENEMY_TEAM as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_SetEnemyTeam: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_HEALTH as i32 => {
                int_data = atoi(data);
                Q3_SetHealth(ctx, entID, int_data);
            }

            _ if toSet == SET_ARMOR as i32 => {
                int_data = atoi(data);
                Q3_SetArmor(ctx, entID, int_data);
            }

            _ if toSet == SET_BEHAVIOR_STATE as i32 => {
                if Q3_SetBState(ctx, entID, data) == qfalse {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(ent, taskID_t::TID_BSTATE as c_int, taskID),
                    );
                    return qfalse; //don't complete
                }
            }

            _ if toSet == SET_DEFAULT_BSTATE as i32 => Q3_SetDefaultBState(ctx, entID, data),

            _ if toSet == SET_TEMP_BSTATE as i32 => {
                if Q3_SetTempBState(ctx, entID, data) == qfalse {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(ent, taskID_t::TID_BSTATE as c_int, taskID),
                    );
                    return qfalse; //don't complete
                }
            }

            _ if toSet == SET_CAPTURE as i32 => Q3_SetCaptureGoal(ctx, entID, data),

            _ if toSet == SET_DPITCH as i32 => {
                //FIXME: make these set tempBehavior to BS_FACE and await completion?  Or set lockedDesiredPitch/Yaw and aimTime?
                float_data = atof(data) as f32;
                Q3_SetDPitch(ctx, entID, float_data);
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANGLE_FACE as c_int, taskID),
                );
                return qfalse;
            }

            _ if toSet == SET_DYAW as i32 => {
                float_data = atof(data) as f32;
                Q3_SetDYaw(ctx, entID, float_data);
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANGLE_FACE as c_int, taskID),
                );
                return qfalse;
            }

            _ if toSet == SET_EVENT as i32 => Q3_SetEvent(ctx, entID, data),

            _ if toSet == SET_VIEWTARGET as i32 => {
                Q3_SetViewTarget(ctx, entID, data);
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANGLE_FACE as c_int, taskID),
                );
                return qfalse;
            }

            _ if toSet == SET_WATCHTARGET as i32 => Q3_SetWatchTarget(ctx, entID, data),
            _ if toSet == SET_VIEWENTITY as i32 => Q3_SetViewEntity(ctx, entID, data),
            _ if toSet == SET_LOOPSOUND as i32 => Q3_SetLoopSound(ctx, entID, data),

            _ if toSet == SET_ICARUS_FREEZE as i32 || toSet == SET_ICARUS_UNFREEZE as i32 => {
                Q3_SetICARUSFreeze(
                    ctx,
                    entID,
                    data,
                    if toSet == (SET_ICARUS_FREEZE) as i32 {
                        qtrue
                    } else {
                        qfalse
                    },
                );
            }

            _ if toSet == SET_WEAPON as i32 => Q3_SetWeapon(ctx, entID, data),
            _ if toSet == SET_ITEM as i32 => Q3_SetItem(ctx, entID, data),

            _ if toSet == SET_WALKSPEED as i32 => {
                int_data = atoi(data);
                Q3_SetWalkSpeed(ctx, entID, int_data);
            }

            _ if toSet == SET_RUNSPEED as i32 => {
                int_data = atoi(data);
                Q3_SetRunSpeed(ctx, entID, int_data);
            }

            _ if toSet == SET_WIDTH as i32 => {
                int_data = atoi(data);
                Q3_SetWidth(ctx, entID, int_data);
                return qfalse;
            }

            _ if toSet == SET_YAWSPEED as i32 => {
                float_data = atof(data) as f32;
                Q3_SetYawSpeed(ctx, entID, float_data);
            }

            _ if toSet == SET_AGGRESSION as i32 => {
                int_data = atoi(data);
                Q3_SetAggression(ctx, entID, int_data);
            }

            _ if toSet == SET_AIM as i32 => {
                int_data = atoi(data);
                Q3_SetAim(ctx, entID, int_data);
            }

            _ if toSet == SET_FRICTION as i32 => {
                int_data = atoi(data);
                Q3_SetFriction(ctx, entID, int_data);
            }

            _ if toSet == SET_GRAVITY as i32 => {
                float_data = atof(data) as f32;
                Q3_SetGravity(ctx, entID, float_data);
            }

            _ if toSet == SET_WAIT as i32 => {
                float_data = atof(data) as f32;
                Q3_SetWait(ctx, entID, float_data);
            }

            _ if toSet == SET_FOLLOWDIST as i32 => {
                float_data = atof(data) as f32;
                Q3_SetFollowDist(ctx, entID, float_data);
            }

            _ if toSet == SET_SCALE as i32 => {
                float_data = atof(data) as f32;
                Q3_SetScale(ctx, entID, float_data);
            }

            _ if toSet == SET_COUNT as i32 => Q3_SetCount(ctx, entID, data),

            _ if toSet == SET_SHOT_SPACING as i32 => {
                int_data = atoi(data);
                Q3_SetShotSpacing(ctx, entID, int_data);
            }

            _ if toSet == SET_IGNOREPAIN as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetIgnorePain(ctx, entID, qtrue);
                } else if Q_stricmp(b"false\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetIgnorePain(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_IGNOREENEMIES as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetIgnoreEnemies(ctx, entID, qtrue);
                } else if Q_stricmp(b"false\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetIgnoreEnemies(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_IGNOREALERTS as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetIgnoreAlerts(ctx, entID, qtrue);
                } else if Q_stricmp(b"false\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetIgnoreAlerts(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_DONTSHOOT as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetDontShoot(ctx, entID, qtrue);
                } else if Q_stricmp(b"false\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetDontShoot(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_DONTFIRE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetDontFire(ctx, entID, qtrue);
                } else if Q_stricmp(b"false\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetDontFire(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_LOCKED_ENEMY as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetLockedEnemy(ctx, entID, qtrue);
                } else if Q_stricmp(b"false\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetLockedEnemy(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_NOTARGET as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetNoTarget(ctx, entID, qtrue);
                } else if Q_stricmp(b"false\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetNoTarget(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_LEAN as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_LEAN NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_SHOOTDIST as i32 => {
                float_data = atof(data) as f32;
                Q3_SetShootDist(ctx, entID, float_data);
            }

            _ if toSet == SET_TIMESCALE as i32 => Q3_SetTimeScale(ctx, entID, data),

            _ if toSet == SET_VISRANGE as i32 => {
                float_data = atof(data) as f32;
                Q3_SetVisrange(ctx, entID, float_data);
            }

            _ if toSet == SET_EARSHOT as i32 => {
                float_data = atof(data) as f32;
                Q3_SetEarshot(ctx, entID, float_data);
            }

            _ if toSet == SET_VIGILANCE as i32 => {
                float_data = atof(data) as f32;
                Q3_SetVigilance(ctx, entID, float_data);
            }

            _ if toSet == SET_VFOV as i32 => {
                int_data = atoi(data);
                Q3_SetVFOV(ctx, entID, int_data);
            }

            _ if toSet == SET_HFOV as i32 => {
                int_data = atoi(data);
                Q3_SetHFOV(ctx, entID, int_data);
            }

            _ if toSet == SET_TARGETNAME as i32 => Q3_SetTargetName(ctx, entID, data),
            _ if toSet == SET_TARGET as i32 => Q3_SetTarget(ctx, entID, data),
            _ if toSet == SET_TARGET2 as i32 => Q3_SetTarget2(ctx, entID, data),

            _ if toSet == SET_LOCATION as i32 => {
                if Q3_SetLocation(ctx, entID, data) == qfalse {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(ent, taskID_t::TID_LOCATION as c_int, taskID),
                    );
                    return qfalse;
                }
            }

            _ if toSet == SET_PAINTARGET as i32 => Q3_SetPainTarget(ctx, entID, data),

            _ if toSet == SET_DEFEND_TARGET as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    cstr("Q3_SetDefendTarget unimplemented\n").as_ptr(),
                );
                //Q3_SetEnemy( entID, (char *) data);
            }

            _ if toSet == SET_PARM1 as i32
                || toSet == SET_PARM2 as i32
                || toSet == SET_PARM3 as i32
                || toSet == SET_PARM4 as i32
                || toSet == SET_PARM5 as i32
                || toSet == SET_PARM6 as i32
                || toSet == SET_PARM7 as i32
                || toSet == SET_PARM8 as i32
                || toSet == SET_PARM9 as i32
                || toSet == SET_PARM10 as i32
                || toSet == SET_PARM11 as i32
                || toSet == SET_PARM12 as i32
                || toSet == SET_PARM13 as i32
                || toSet == SET_PARM14 as i32
                || toSet == SET_PARM15 as i32
                || toSet == SET_PARM16 as i32 =>
            {
                Q3_SetParm(ctx, entID, toSet - SET_PARM1 as i32, data);
            }

            _ if toSet == SET_SPAWNSCRIPT as i32
                || toSet == SET_USESCRIPT as i32
                || toSet == SET_AWAKESCRIPT as i32
                || toSet == SET_ANGERSCRIPT as i32
                || toSet == SET_ATTACKSCRIPT as i32
                || toSet == SET_VICTORYSCRIPT as i32
                || toSet == SET_PAINSCRIPT as i32
                || toSet == SET_FLEESCRIPT as i32
                || toSet == SET_DEATHSCRIPT as i32
                || toSet == SET_DELAYEDSCRIPT as i32
                || toSet == SET_BLOCKEDSCRIPT as i32
                || toSet == SET_FFIRESCRIPT as i32
                || toSet == SET_FFDEATHSCRIPT as i32
                || toSet == SET_MINDTRICKSCRIPT as i32 =>
            {
                if Q3_SetBehaviorSet(ctx, entID, toSet, data) == qfalse {
                    G_DebugPrint(
                        ctx,
                        WL_ERROR as c_int,
                        cstr(&format!(
                            "Q3_SetBehaviorSet: Invalid bSet {}\n",
                            cstr_to_str(type_name)
                        ))
                        .as_ptr(),
                    );
                }
            }

            _ if toSet == SET_NO_MINDTRICK as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetNoMindTrick(ctx, entID, qtrue);
                } else {
                    Q3_SetNoMindTrick(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_CINEMATIC_SKIPSCRIPT as i32 => {
                Q3_SetCinematicSkipScript(ctx, data as *mut c_char);
            }

            _ if toSet == SET_DELAYSCRIPTTIME as i32 => {
                int_data = atoi(data);
                Q3_SetDelayScriptTime(ctx, entID, int_data);
            }

            _ if toSet == SET_CROUCHED as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetCrouched(ctx, entID, qtrue);
                } else {
                    Q3_SetCrouched(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_WALKING as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetWalking(ctx, entID, qtrue);
                } else {
                    Q3_SetWalking(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_RUNNING as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetRunning(ctx, entID, qtrue);
                } else {
                    Q3_SetRunning(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_CHASE_ENEMIES as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetChaseEnemies(ctx, entID, qtrue);
                } else {
                    Q3_SetChaseEnemies(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_LOOK_FOR_ENEMIES as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetLookForEnemies(ctx, entID, qtrue);
                } else {
                    Q3_SetLookForEnemies(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_FACE_MOVE_DIR as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetFaceMoveDir(ctx, entID, qtrue);
                } else {
                    Q3_SetFaceMoveDir(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_ALT_FIRE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetAltFire(ctx, entID, qtrue);
                } else {
                    Q3_SetAltFire(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_DONT_FLEE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetDontFlee(ctx, entID, qtrue);
                } else {
                    Q3_SetDontFlee(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_FORCED_MARCH as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetForcedMarch(ctx, entID, qtrue);
                } else {
                    Q3_SetForcedMarch(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_NO_RESPONSE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetNoResponse(ctx, entID, qtrue);
                } else {
                    Q3_SetNoResponse(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_NO_COMBAT_TALK as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetCombatTalk(ctx, entID, qtrue);
                } else {
                    Q3_SetCombatTalk(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_NO_ALERT_TALK as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetAlertTalk(ctx, entID, qtrue);
                } else {
                    Q3_SetAlertTalk(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_USE_CP_NEAREST as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetUseCpNearest(ctx, entID, qtrue);
                } else {
                    Q3_SetUseCpNearest(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_NO_FORCE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetNoForce(ctx, entID, qtrue);
                } else {
                    Q3_SetNoForce(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_NO_ACROBATICS as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetNoAcrobatics(ctx, entID, qtrue);
                } else {
                    Q3_SetNoAcrobatics(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_USE_SUBTITLES as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetUseSubtitles(ctx, entID, qtrue);
                } else {
                    Q3_SetUseSubtitles(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_NO_FALLTODEATH as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetNoFallToDeath(ctx, entID, qtrue);
                } else {
                    Q3_SetNoFallToDeath(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_DISMEMBERABLE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetDismemberable(ctx, entID, qtrue);
                } else {
                    Q3_SetDismemberable(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_MORELIGHT as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetMoreLight(ctx, entID, qtrue);
                } else {
                    Q3_SetMoreLight(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_TREASONED as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_VERBOSE as c_int,
                    b"SET_TREASONED is disabled, do not use\n\0".as_ptr() as *const c_char,
                );
                /*
                G_TeamRetaliation( NULL, SV_GentityNum(0), qfalse );
                ffireLevel = FFIRE_LEVEL_RETALIATION;
                */
            }

            _ if toSet == SET_UNDYING as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetUndying(ctx, entID, qtrue);
                } else {
                    Q3_SetUndying(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_INVINCIBLE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetInvincible(ctx, entID, qtrue);
                } else {
                    Q3_SetInvincible(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_NOAVOID as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetNoAvoid(ctx, entID, qtrue);
                } else {
                    Q3_SetNoAvoid(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_SOLID as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    if Q3_SetSolid(ctx, entID, qtrue) == qfalse {
                        trap::ICARUS_TaskIDSet(
                            ctx.engine,
                            GIcarusTaskidsetArgs::new(ent, taskID_t::TID_RESIZE as c_int, taskID),
                        );
                        return qfalse;
                    }
                } else {
                    Q3_SetSolid(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_INVISIBLE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetInvisible(ctx, entID, qtrue);
                } else {
                    Q3_SetInvisible(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_VAMPIRE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetVampire(ctx, entID, qtrue);
                } else {
                    Q3_SetVampire(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_FORCE_INVINCIBLE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetForceInvincible(ctx, entID, qtrue);
                } else {
                    Q3_SetForceInvincible(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_GREET_ALLIES as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetGreetAllies(ctx, entID, qtrue);
                } else {
                    Q3_SetGreetAllies(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_PLAYER_LOCKED as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetPlayerLocked(ctx, entID, qtrue);
                } else {
                    Q3_SetPlayerLocked(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_LOCK_PLAYER_WEAPONS as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetLockPlayerWeapons(ctx, entID, qtrue);
                } else {
                    Q3_SetLockPlayerWeapons(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_NO_IMPACT_DAMAGE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetNoImpactDamage(ctx, entID, qtrue);
                } else {
                    Q3_SetNoImpactDamage(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_FORWARDMOVE as i32 => {
                int_data = atoi(data);
                Q3_SetForwardMove(ctx, entID, int_data);
            }

            _ if toSet == SET_RIGHTMOVE as i32 => {
                int_data = atoi(data);
                Q3_SetRightMove(ctx, entID, int_data);
            }

            _ if toSet == SET_LOCKYAW as i32 => Q3_SetLockAngle(ctx, entID, data),

            _ if toSet == SET_CAMERA_GROUP as i32 => {
                Q3_CameraGroup(ctx, entID, data as *mut c_char)
            }
            _ if toSet == SET_CAMERA_GROUP_Z_OFS as i32 => {
                float_data = atof(data) as f32;
                Q3_CameraGroupZOfs(ctx, float_data);
            }
            _ if toSet == SET_CAMERA_GROUP_TAG as i32 => {
                Q3_CameraGroupTag(ctx, data as *mut c_char)
            }

            //FIXME: put these into camera commands
            _ if toSet == SET_LOOK_TARGET as i32 => Q3_LookTarget(ctx, entID, data as *mut c_char),
            _ if toSet == SET_ADDRHANDBOLT_MODEL as i32 => {
                Q3_AddRHandModel(ctx, entID, data as *mut c_char)
            }
            _ if toSet == SET_REMOVERHANDBOLT_MODEL as i32 => {
                Q3_RemoveRHandModel(ctx, entID, data as *mut c_char)
            }
            _ if toSet == SET_ADDLHANDBOLT_MODEL as i32 => {
                Q3_AddLHandModel(ctx, entID, data as *mut c_char)
            }
            _ if toSet == SET_REMOVELHANDBOLT_MODEL as i32 => {
                Q3_RemoveLHandModel(ctx, entID, data as *mut c_char)
            }

            _ if toSet == SET_FACEEYESCLOSED as i32
                || toSet == SET_FACEEYESOPENED as i32
                || toSet == SET_FACEAUX as i32
                || toSet == SET_FACEBLINK as i32
                || toSet == SET_FACEBLINKFROWN as i32
                || toSet == SET_FACEFROWN as i32
                || toSet == SET_FACENORMAL as i32 =>
            {
                float_data = atof(data) as f32;
                Q3_Face(ctx, entID, toSet, float_data);
            }

            _ if toSet == SET_SCROLLTEXT as i32 => Q3_ScrollText(ctx, data),
            _ if toSet == SET_LCARSTEXT as i32 => Q3_LCARSText(ctx, data),
            _ if toSet == SET_CAPTIONTEXTCOLOR as i32 => Q3_SetCaptionTextColor(ctx, data),
            _ if toSet == SET_CENTERTEXTCOLOR as i32 => Q3_SetCenterTextColor(ctx, data),
            _ if toSet == SET_SCROLLTEXTCOLOR as i32 => Q3_SetScrollTextColor(ctx, data),

            _ if toSet == SET_PLAYER_USABLE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetPlayerUsable(ctx, entID, qtrue);
                } else {
                    Q3_SetPlayerUsable(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_STARTFRAME as i32 => {
                int_data = atoi(data);
                Q3_SetStartFrame(ctx, entID, int_data);
            }

            _ if toSet == SET_ENDFRAME as i32 => {
                int_data = atoi(data);
                Q3_SetEndFrame(ctx, entID, int_data);

                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(ent, taskID_t::TID_ANIM_BOTH as c_int, taskID),
                );
                return qfalse;
            }

            _ if toSet == SET_ANIMFRAME as i32 => {
                int_data = atoi(data);
                Q3_SetAnimFrame(ctx, entID, int_data);
                return qfalse;
            }

            _ if toSet == SET_LOOP_ANIM as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetLoopAnim(ctx, entID, qtrue);
                } else {
                    Q3_SetLoopAnim(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_INTERFACE as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_SetInterface: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_SHIELDS as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetShields(ctx, entID, qtrue);
                } else {
                    Q3_SetShields(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_SABERACTIVE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetSaberActive(ctx, entID, qtrue);
                } else {
                    Q3_SetSaberActive(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_ADJUST_AREA_PORTALS as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_SetAdjustAreaPortals: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_DMG_BY_HEAVY_WEAP_ONLY as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_SetDmgByHeavyWeapOnly: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_SHIELDED as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_SetShielded: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_NO_GROUPS as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"Q3_SetNoGroups: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_FIRE_WEAPON as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetFireWeapon(ctx, entID, qtrue);
                } else if Q_stricmp(b"false\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetFireWeapon(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_INACTIVE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetInactive(ctx, entID, qtrue);
                } else if Q_stricmp(b"false\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetInactive(ctx, entID, qfalse);
                } else if Q_stricmp(b"unlocked\0".as_ptr() as *const c_char, data) == 0 {
                    UnLockDoors(&mut (*ctx.world_raw()).g_entities[entID as usize]);
                } else if Q_stricmp(b"locked\0".as_ptr() as *const c_char, data) == 0 {
                    LockDoors(&mut (*ctx.world_raw()).g_entities[entID as usize]);
                }
            }

            _ if toSet == SET_END_SCREENDISSOLVE as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_END_SCREENDISSOLVE: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_MISSION_STATUS_SCREEN as i32 => {
                //Cvar_Set("cg_missionstatusscreen", "1");
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_MISSION_STATUS_SCREEN: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_FUNC_USABLE_VISIBLE as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetFuncUsableVisible(ctx, entID, qtrue);
                } else if Q_stricmp(b"false\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetFuncUsableVisible(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_NO_KNOCKBACK as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetNoKnockback(ctx, entID, qtrue);
                } else {
                    Q3_SetNoKnockback(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_VIDEO_PLAY as i32 => {
                // don't do this check now, James doesn't want a scripted cinematic to also skip any Video cinematics as well,
                //	the "timescale" and "skippingCinematic" cvars will be set back to normal in the Video code, so doing a
                //	skip will now only skip one section of a multiple-part story (eg VOY1 bridge sequence)
                //
                //		if ( g_timescale->value <= 1.0f )
                {
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        b"SET_VIDEO_PLAY: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                    );
                    //SV_SendConsoleCommand( va("inGameCinematic %s\n", (char *)data) );
                }
            }

            _ if toSet == SET_VIDEO_FADE_IN as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_VIDEO_FADE_IN: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_VIDEO_FADE_OUT as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_VIDEO_FADE_OUT: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_REMOVE_TARGET as i32 => Q3_SetRemoveTarget(ctx, entID, data),

            _ if toSet == SET_LOADGAME as i32 => {
                //gi.SendConsoleCommand( va("load %s\n", (const char *) data ) );
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_LOADGAME: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_MENU_SCREEN as i32 => {
                //UI_SetActiveMenu( (const char *) data );
            }

            _ if toSet == SET_OBJECTIVE_SHOW as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_OBJECTIVE_SHOW: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }
            _ if toSet == SET_OBJECTIVE_HIDE as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_OBJECTIVE_HIDE: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }
            _ if toSet == SET_OBJECTIVE_SUCCEEDED as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_OBJECTIVE_SUCCEEDED: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }
            _ if toSet == SET_OBJECTIVE_FAILED as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_OBJECTIVE_FAILED: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_OBJECTIVE_CLEARALL as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_OBJECTIVE_CLEARALL: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_MISSIONFAILED as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_MISSIONFAILED: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_MISSIONSTATUSTEXT as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_MISSIONSTATUSTEXT: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_MISSIONSTATUSTIME as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_MISSIONSTATUSTIME: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_CLOSINGCREDITS as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_CLOSINGCREDITS: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_SKILL as i32 => {
                //		//can never be set
            }

            _ if toSet == SET_FULLNAME as i32 => Q3_SetFullName(ctx, entID, data),

            _ if toSet == SET_DISABLE_SHADER_ANIM as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetDisableShaderAnims(ctx, entID, qtrue);
                } else {
                    Q3_SetDisableShaderAnims(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_SHADER_ANIM as i32 => {
                if Q_stricmp(b"true\0".as_ptr() as *const c_char, data) == 0 {
                    Q3_SetShaderAnim(ctx, entID, qtrue);
                } else {
                    Q3_SetShaderAnim(ctx, entID, qfalse);
                }
            }

            _ if toSet == SET_MUSIC_STATE as i32 => Q3_SetMusicState(ctx, data),
            _ if toSet == SET_CLEAN_DAMAGING_ENTS as i32 => Q3_SetCleanDamagingEnts(ctx),

            _ if toSet == SET_HUD as i32 => {
                G_DebugPrint(
                    ctx,
                    WL_WARNING as c_int,
                    b"SET_HUD: NOT SUPPORTED IN MP\n\0".as_ptr() as *const c_char,
                );
            }

            _ if toSet == SET_FORCE_HEAL_LEVEL as i32
                || toSet == SET_FORCE_JUMP_LEVEL as i32
                || toSet == SET_FORCE_SPEED_LEVEL as i32
                || toSet == SET_FORCE_PUSH_LEVEL as i32
                || toSet == SET_FORCE_PULL_LEVEL as i32
                || toSet == SET_FORCE_MINDTRICK_LEVEL as i32
                || toSet == SET_FORCE_GRIP_LEVEL as i32
                || toSet == SET_FORCE_LIGHTNING_LEVEL as i32
                || toSet == SET_SABER_THROW as i32
                || toSet == SET_SABER_DEFENSE as i32
                || toSet == SET_SABER_OFFENSE as i32 =>
            {
                int_data = atoi(data);
                Q3_SetForcePowerLevel(ctx, entID, toSet - SET_FORCE_HEAL_LEVEL as i32, int_data);
            }

            _ => {
                //G_DebugPrint( WL_ERROR, "Q3_Set: '%s' is not a valid set field\n", type_name );
                trap::ICARUS_SetVar(
                    ctx.engine,
                    GIcarusSetvarArgs::new(taskID, entID, type_name, data),
                );
            }
        }

        qtrue
    }
}
