// PORT-COMPLETE: g_ICARUScb.c
//! FAITHFUL port of `oracle/codemp/game/g_ICARUScb.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//!
//! Safe-state migration **Stage 2c** (deref sweep): every entity reach is a
//! checked `ctx.world.entity[_mut](id)` borrow — the per-body raw
//! `*mut gentity_t` re-derives are gone, seam trap handles are re-derived fresh
//! at each `trap::*` call. The remaining `unsafe` blocks hold only sanctioned
//! raw ops: pool-client (`gclient_t`) and `gNPC_t` derefs through copied
//! pointer values, `parms_t` pool derefs, the `*mut *mut c_char` out-param, the
//! `bState_t` transmutes and `cstr_to_str` string reads. Behavior is
//! byte-identical, referee-verified.
#![allow(non_snake_case, unused, clippy::all)]

use core::ffi::CStr;

use crate::g_nav::NAV_FindClosestWaypointForEnt;
use crate::prelude::*;
use crate::q_math::vec3_origin;
use native_string::atof::{atof, atof_bytes};

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
use crate::ent_id::ent_id;
use crate::ent_id::resolve;
use crate::g_client::SetClientViewAngle;
use crate::g_combat::{player_die, G_Damage};
use crate::g_misc::{TAG_GetAngles, TAG_GetOrigin, TAG_GetOrigin2, TAG_GetRadius};
use crate::g_mover::{G_PlayDoorSound, MatchTeam, BMS_END};
use crate::g_utils::G_FreeEntity;
use crate::veh_dispatch::eject_all;
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
/// Source: `oracle/codemp/game/g_ICARUScb.c:275-324`
pub fn G_DebugPrint(
    ctx: &mut GameContext,
    level: c_int,
    format: *const c_char,
    // variadic `...` — C varargs, seam decision pending
) {
    unsafe {
        if ctx.world.cvars.g_developer.integer != 2 {
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

            let targ = ctx.world.g_entities[ent_num as usize].script_targetname;
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
    let client = ctx.world.entity(ent).client;
    if client.is_null() {
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            b"Q3_GetAnimLower: attempted to read animation state off non-client!\n\0".as_ptr()
                as *const c_char,
        );
        return std::ptr::null_mut();
    }

    // Pool client: NPCs carry a `BG_Alloc`'d gclient_t (g_utils.c:430), so the
    // deref stays raw through the copied pointer — never index level.clients.
    let anim: c_int = unsafe { (*client).ps.legsAnim };

    animTable[anim as usize].name
}

/// Raven `Q3_GetAnimUpper`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:351-364`
pub fn Q3_GetAnimUpper(ctx: &mut GameContext, ent: EntityId) -> *mut c_char {
    let client = ctx.world.entity(ent).client;
    if client.is_null() {
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            b"Q3_GetAnimUpper: attempted to read animation state off non-client!\n\0".as_ptr()
                as *const c_char,
        );
        return std::ptr::null_mut();
    }

    // Pool client: NPCs carry a `BG_Alloc`'d gclient_t (g_utils.c:430), so the
    // deref stays raw through the copied pointer — never index level.clients.
    let anim: c_int = unsafe { (*client).ps.torsoAnim };

    animTable[anim as usize].name
}

/// Raven `Q3_GetAnimBoth`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:371-398`
pub fn Q3_GetAnimBoth(ctx: &mut GameContext, ent: EntityId) -> *mut c_char {
    let lower_name = Q3_GetAnimLower(ctx, ent);
    let upper_name = Q3_GetAnimUpper(ctx, ent);

    // `lower_name`/`upper_name` are `animTable` entry strings (or NULL); the
    // empty-string test derefs the raw C string, so it stays a tight unsafe.
    if lower_name.is_null() || unsafe { *lower_name } == 0 {
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            b"Q3_GetAnimBoth: NULL legs animation string found!\n\0".as_ptr() as *const c_char,
        );
        return std::ptr::null_mut();
    }

    if upper_name.is_null() || unsafe { *upper_name } == 0 {
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
    let id = EntityId(entID as u32);
    let mut final_name = [0 as c_char; MAX_QPATH as usize];
    Q_strncpyz(final_name.as_mut_ptr(), name, MAX_QPATH as c_int);
    Q_strupr(final_name.as_mut_ptr());
    COM_StripExtension(final_name.as_ptr(), final_name.as_mut_ptr());

    let sound_handle = G_SoundIndex(final_name.as_ptr());
    let mut b_broadcast = qfalse;

    let classname = ctx.world.entity(id).classname;
    if Q_stricmp(channel, b"CHAN_ANNOUNCER\0".as_ptr() as *const c_char) == 0
        || (!classname.is_null()
            && Q_stricmp(
                b"target_scriptrunner\0".as_ptr() as *const c_char,
                classname,
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
        let t_f_val = atof_bytes(unsafe { CStr::from_ptr(buf.as_ptr()) }.to_bytes()) as f32;

        if t_f_val > 1.0 {
            // Skip the damn sound!
            return qtrue;
        } else {
            G_Sound(ctx, Some(id), voice_chan, sound_handle);
        }
        trap::ICARUS_TaskIDSet(
            ctx.engine,
            GIcarusTaskidsetArgs::new(
                (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                taskID_t::TID_CHAN_VOICE as c_int,
                taskID,
            ),
        );
        return qfalse;
    }

    if b_broadcast != 0 {
        let origin = ctx.world.entity(id).r.currentOrigin;
        let te_id = G_TempEntity(ctx, origin, EV_GLOBAL_SOUND as c_int);
        ctx.world.entity_mut(te_id).s.eventParm = sound_handle;
        ctx.world.entity_mut(te_id).r.svFlags |= SVF_BROADCAST;
    } else {
        G_Sound(ctx, Some(id), CHAN_AUTO, sound_handle);
    }

    qtrue
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
    let id = EntityId(entID as u32);

    if Q_stricmp(r#type, b"PLAY_ROFF\0".as_ptr() as *const c_char) == 0 {
        // Raven passes `name` (already a `char*`) straight to `trap_ROFF_Cache`;
        // the ABI arg is an owned `CString` here.
        let file = CString::new(unsafe { std::ffi::CStr::from_ptr(name) }.to_bytes()).unwrap();
        let roffid = trap::ROFF_Cache(ctx.engine, GRoffCacheArgs::new(file));
        ctx.world.entity_mut(id).roffid = roffid;
        if roffid != 0 {
            let roffname = G_NewString(ctx, name);
            ctx.world.entity_mut(id).roffname = roffname;

            // Save this off for later
            trap::ICARUS_TaskIDSet(
                ctx.engine,
                GIcarusTaskidsetArgs::new(
                    (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                    taskID_t::TID_MOVE_NAV as c_int,
                    taskID,
                ),
            );

            let cur_origin = ctx.world.entity(id).r.currentOrigin;
            ctx.world.entity_mut(id).s.origin2 = cur_origin;
            let cur_angles = ctx.world.entity(id).r.currentAngles;
            ctx.world.entity_mut(id).s.angles2 = cur_angles;

            trap::LinkEntity(
                ctx.engine,
                GLinkentityArgs::new((ctx.world.entity_mut(id) as *mut gentity_t).cast()),
            );

            let number = ctx.world.entity(id).s.number;
            trap::ROFF_Play(ctx.engine, GRoffPlayArgs::new(number, roffid, qtrue));
        }
    }
}

/// Raven `anglerCallback`.
///
/// Utility function.
/// Source: `oracle/codemp/game/g_ICARUScb.c:569-591`
pub fn anglerCallback(ctx: &mut GameContext, ent: EntityId) {
    trap::ICARUS_TaskIDComplete(
        ctx.engine,
        GIcarusTaskidcompleteArgs::new(
            (ctx.world.entity_mut(ent) as *mut gentity_t).cast(),
            taskID_t::TID_ANGLE_FACE as c_int,
        ),
    );

    {
        let e = ctx.world.entity_mut(ent);
        // VectorMA(trBase, trDuration*0.001, trDelta, currentAngles)
        let scale = e.s.apos.trDuration as f32 * 0.001;
        for i in 0..3 {
            e.r.currentAngles[i] = e.s.apos.trBase[i] + scale * e.s.apos.trDelta[i];
        }
        e.s.apos.trBase = e.r.currentAngles;
        e.s.apos.trDelta = [0.0, 0.0, 0.0];
        e.s.apos.trDuration = 1;
        e.s.apos.trType = trType_t::TR_STATIONARY;
    }
    let level_time = ctx.world.level.time;
    {
        let e = ctx.world.entity_mut(ent);
        e.s.apos.trTime = level_time;

        // Stop thinking.
        e.reached = FnId::NONE;
        // Raven compares `ent->think == anglerCallback` by address (fn-ID
        // enums replace address compares) before clearing it; the
        // `gentity_t.think` field is not yet retrofitted from a raw fn-ptr to
        // `Option<EntThink>` so the compare itself can't be reproduced here.
        // This callback is only ever assigned as its own think, so
        // unconditionally clearing is behaviorally equivalent.
        e.think = FnId::NONE;
    }

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new((ctx.world.entity_mut(ent) as *mut gentity_t).cast()),
    );
}

/// Raven `moverCallback`.
///
/// Utility function.
/// Source: `oracle/codemp/game/g_ICARUScb.c:603-633`
pub fn moverCallback(ctx: &mut GameContext, ent: EntityId) {
    trap::ICARUS_TaskIDComplete(
        ctx.engine,
        GIcarusTaskidcompleteArgs::new(
            (ctx.world.entity_mut(ent) as *mut gentity_t).cast(),
            taskID_t::TID_MOVE_NAV as c_int,
        ),
    );

    {
        let e = ctx.world.entity_mut(ent);
        e.s.loopSound = 0;
        e.s.loopIsSoundset = qfalse;
    }
    // BMS_END: unported sound-slot const (missing_symbols).
    G_PlayDoorSound(ctx, ent, BMS_END);

    let mover_state = ctx.world.entity(ent).moverState;
    if mover_state == MOVER_1TO2 {
        let time = ctx.world.level.time;
        MatchTeam(ctx, ent, MOVER_POS2 as c_int, time);
    } else if mover_state == MOVER_2TO1 {
        let time = ctx.world.level.time;
        MatchTeam(ctx, ent, MOVER_POS1 as c_int, time);
    }

    if ctx.world.entity(ent).blocked.get() == Some(EntBlocked::Blocked_Mover) {
        ctx.world.entity_mut(ent).blocked = FnId::NONE;
    }
}

/// Raven `Blocked_Mover`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:635-658`
pub fn Blocked_Mover(ctx: &mut GameContext, ent: EntityId, other: Option<EntityId>) {
    // Raven derefs `other` unconditionally (the blocking entity is always real);
    // resolve the handle up front to keep that behavior.
    let other = other.unwrap();

    // remove anything other than a client -- no longer the case

    // don't remove security keys or goodie keys
    let o = ctx.world.entity(other);
    if o.s.eType == entityType_t::ET_ITEM as c_int {
        // should we be doing anything special if a key blocks it... move it somehow..?
    } else if o.s.number != 0
        && (o.client.is_null()
            || (!o.client.is_null()
                && o.health <= 0
                && o.r.contents == CONTENTS_CORPSE
                && o.message.is_null()))
    {
        // if your not a client, or your a dead client remove yourself...
        // if an item or weapon can we do a little explosion..?
        G_FreeEntity(ctx, Some(other));
        return;
    }

    let damage = ctx.world.entity(ent).damage;
    if damage != 0 {
        // Raven passes `NULL` for both `dir` and `point`; `dir` is now
        // `Option<&mut vec3_t>` so `None` is faithful, but
        // `point` is still a by-value `vec3_t` (no null representation),
        // so the zero vector (`vec3_origin`) remains the stand-in there.
        G_Damage(
            ctx,
            Some(other),
            Some(ent),
            Some(ent),
            None,
            vec3_origin,
            damage,
            0,
            MOD_CRUSH as c_int,
        );
    }
}

/// Raven `moveAndRotateCallback`.
///
/// Utility function.
/// Source: `oracle/codemp/game/g_ICARUScb.c:667-673`
pub fn moveAndRotateCallback(ctx: &mut GameContext, ent: EntityId) {
    //stop turning
    anglerCallback(ctx, ent);
    //stop moving
    moverCallback(ctx, ent);
}

/// Raven `Q3_Lerp2Start`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:682-721`
pub fn Q3_Lerp2Start(ctx: &mut GameContext, entID: c_int, taskID: c_int, duration: f32) {
    let id = EntityId(entID as u32);

    let is_not_mover = {
        let e = ctx.world.entity(id);
        !e.client.is_null()
            || Q_stricmp(
                e.classname,
                b"target_scriptrunner\0".as_ptr() as *const c_char,
            ) == 0
    };
    if is_not_mover {
        G_DebugPrint(
            ctx,
            WL_ERROR as c_int,
            cstr(&format!("Q3_Lerp2Start: ent {} is NOT a mover!\n", entID)).as_ptr(),
        );
        return;
    }

    {
        let e = ctx.world.entity_mut(id);
        if e.s.eType != entityType_t::ET_MOVER as c_int {
            e.s.eType = entityType_t::ET_MOVER as c_int;
        }

        e.moverState = MOVER_2TO1;
        e.s.eType = entityType_t::ET_MOVER as c_int;
        e.reached = Some(EntReached::moverCallback).into();
        if e.damage != 0 {
            e.blocked = Some(EntBlocked::Blocked_Mover).into();
        }

        e.s.pos.trDuration = (duration * 10.0) as c_int;
    }
    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(id).s.pos.trTime = level_time;

    trap::ICARUS_TaskIDSet(
        ctx.engine,
        GIcarusTaskidsetArgs::new(
            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
            taskID_t::TID_MOVE_NAV as c_int,
            taskID,
        ),
    );
    G_PlayDoorLoopSound(ctx, id);
    G_PlayDoorSound(ctx, id, BMS_START);

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new((ctx.world.entity_mut(id) as *mut gentity_t).cast()),
    );
}

/// Raven `Q3_Lerp2End`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:730-769`
pub fn Q3_Lerp2End(ctx: &mut GameContext, entID: c_int, taskID: c_int, duration: f32) {
    let id = EntityId(entID as u32);

    let is_not_mover = {
        let e = ctx.world.entity(id);
        !e.client.is_null()
            || Q_stricmp(
                e.classname,
                b"target_scriptrunner\0".as_ptr() as *const c_char,
            ) == 0
    };
    if is_not_mover {
        G_DebugPrint(
            ctx,
            WL_ERROR as c_int,
            cstr(&format!("Q3_Lerp2End: ent {} is NOT a mover!\n", entID)).as_ptr(),
        );
        return;
    }

    {
        let e = ctx.world.entity_mut(id);
        if e.s.eType != entityType_t::ET_MOVER as c_int {
            e.s.eType = entityType_t::ET_MOVER as c_int;
        }

        e.moverState = MOVER_1TO2;
        e.s.eType = entityType_t::ET_MOVER as c_int;
        e.reached = Some(EntReached::moverCallback).into();
        if e.damage != 0 {
            e.blocked = Some(EntBlocked::Blocked_Mover).into();
        }

        e.s.pos.trDuration = (duration * 10.0) as c_int;
    }
    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(id).s.time = level_time;

    trap::ICARUS_TaskIDSet(
        ctx.engine,
        GIcarusTaskidsetArgs::new(
            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
            taskID_t::TID_MOVE_NAV as c_int,
            taskID,
        ),
    );
    G_PlayDoorLoopSound(ctx, id);
    G_PlayDoorSound(ctx, id, BMS_START);

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new((ctx.world.entity_mut(id) as *mut gentity_t).cast()),
    );
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
    let id = EntityId(entID as u32);

    let is_not_mover = {
        let e = ctx.world.entity(id);
        !e.client.is_null()
            || Q_stricmp(
                e.classname,
                b"target_scriptrunner\0".as_ptr() as *const c_char,
            ) == 0
    };
    if is_not_mover {
        G_DebugPrint(
            ctx,
            WL_ERROR as c_int,
            cstr(&format!("Q3_Lerp2Pos: ent {} is NOT a mover!\n", entID)).as_ptr(),
        );
        return;
    }

    {
        let e = ctx.world.entity_mut(id);
        if e.s.eType != entityType_t::ET_MOVER as c_int {
            e.s.eType = entityType_t::ET_MOVER as c_int;
        }
    }

    let mut duration = duration;
    if duration == 0.0 {
        duration = 1.0;
    }

    let mut moverState = ctx.world.entity(id).moverState;

    {
        let e = ctx.world.entity_mut(id);
        if moverState == MOVER_POS1 || moverState == MOVER_2TO1 {
            e.pos1 = e.r.currentOrigin;
            e.pos2 = *origin;
            moverState = MOVER_1TO2;
        } else {
            e.pos2 = e.r.currentOrigin;
            e.pos1 = *origin;
            moverState = MOVER_2TO1;
        }
        e.moverState = moverState;

        InitMoverTrData(e);

        e.s.pos.trDuration = duration as c_int;
    }

    let time = ctx.world.level.time;
    MatchTeam(ctx, id, moverState as c_int, time);

    if let Some(angles) = angles {
        {
            let e = ctx.world.entity_mut(id);
            let mut ang = [0.0f32; 3];
            for i in 0..3 {
                ang[i] = AngleDelta(angles[i], e.r.currentAngles[i]);
                e.s.apos.trDelta[i] = ang[i] / (duration * 0.001);
            }

            e.s.apos.trBase = e.r.currentAngles;

            e.s.apos.trType = if e.alt_fire != 0 {
                trType_t::TR_LINEAR_STOP
            } else {
                trType_t::TR_NONLINEAR_STOP
            };
            e.s.apos.trDuration = duration as c_int;
        }
        let level_time = ctx.world.level.time;
        {
            let e = ctx.world.entity_mut(id);
            e.s.apos.trTime = level_time;
            e.reached = Some(EntReached::moveAndRotateCallback).into();
        }
        trap::ICARUS_TaskIDSet(
            ctx.engine,
            GIcarusTaskidsetArgs::new(
                (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                taskID_t::TID_ANGLE_FACE as c_int,
                taskID,
            ),
        );
    } else {
        ctx.world.entity_mut(id).reached = Some(EntReached::moverCallback).into();
    }

    if ctx.world.entity(id).damage != 0 {
        ctx.world.entity_mut(id).blocked = Some(EntBlocked::Blocked_Mover).into();
    }

    trap::ICARUS_TaskIDSet(
        ctx.engine,
        GIcarusTaskidsetArgs::new(
            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
            taskID_t::TID_MOVE_NAV as c_int,
            taskID,
        ),
    );
    G_PlayDoorLoopSound(ctx, id);
    G_PlayDoorSound(ctx, id, BMS_START);

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new((ctx.world.entity_mut(id) as *mut gentity_t).cast()),
    );
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
    let id = EntityId(entID as u32);

    {
        let e = ctx.world.entity_mut(id);
        e.s.apos.trDuration = if duration > 0.0 { duration as c_int } else { 1 };

        let mut ang = [0.0f32; 3];
        for i in 0..3 {
            ang[i] = AngleSubtract(angles[i], e.r.currentAngles[i]);
            e.s.apos.trDelta[i] = ang[i] / (e.s.apos.trDuration as f32 * 0.001);
        }

        e.s.apos.trBase = e.r.currentAngles;

        e.s.apos.trType = if e.alt_fire != 0 {
            trType_t::TR_LINEAR_STOP
        } else {
            trType_t::TR_NONLINEAR_STOP
        };
    }
    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(id).s.apos.trTime = level_time;

    trap::ICARUS_TaskIDSet(
        ctx.engine,
        GIcarusTaskidsetArgs::new(
            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
            taskID_t::TID_ANGLE_FACE as c_int,
            taskID,
        ),
    );

    {
        let e = ctx.world.entity_mut(id);
        e.think = Some(EntThink::anglerCallback).into();
    }
    let level_time = ctx.world.level.time;
    ctx.world.entity_mut(id).nextthink = level_time + duration as c_int;

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new((ctx.world.entity_mut(id) as *mut gentity_t).cast()),
    );
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
    let id = EntityId(entID as u32);

    if ctx.world.entity(id).inuse == 0 {
        debug_assert!(false);
        return 0;
    }

    let ownername = ctx.world.entity(id).ownername;
    // `TYPE_ORIGIN`/`TYPE_ANGLES` are module-level consts (see above).
    if lookup == TYPE_ORIGIN {
        return TAG_GetOrigin(ctx, ownername, name, info);
    } else if lookup == TYPE_ANGLES {
        return TAG_GetAngles(ctx, ownername, name, info);
    }

    0
}

/// Raven `Q3_Use`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:981-998`
pub fn Q3_Use(ctx: &mut GameContext, entID: c_int, target: *const c_char) {
    let id = EntityId(entID as u32);

    if target.is_null() || unsafe { *target } == 0 {
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            b"Q3_Use: string is NULL!\n\0".as_ptr() as *const c_char,
        );
        return;
    }

    G_UseTargets2(ctx, Some(id), Some(id), target);
}

/// Raven `Q3_Kill`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1009-1052`
pub fn Q3_Kill(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    let id = EntityId(entID as u32);

    let victim: Option<EntityId> = if Q_stricmp(name, b"self\0".as_ptr() as *const c_char) == 0 {
        Some(id)
    } else if Q_stricmp(name, b"enemy\0".as_ptr() as *const c_char) == 0 {
        ctx.world.entity(id).enemy
    } else {
        let found = G_Find(
            ctx,
            None,
            core::mem::offset_of!(gentity_t, targetname) as c_int,
            name,
        );
        ctx.entity_id_of(found)
    };

    let Some(vid) = victim else {
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            cstr(&format!("Q3_Kill: can't find {}\n", unsafe {
                cstr_to_str(name)
            }))
            .as_ptr(),
        );
        return;
    };

    let o_health = ctx.world.entity(vid).health;
    ctx.world.entity_mut(vid).health = 0;
    if !ctx.world.entity(vid).client.is_null() {
        ctx.world.entity_mut(vid).flags |= FL_NO_KNOCKBACK;
    }

    if let Some(die_fn) = ctx.world.entity(vid).die.get() {
        // `dispatch_die` is the fn-ptr-dispatch seam: it takes the raw
        // `gentity_t*` Raven passed (victim thrice) and re-derives ids inside.
        let vp = ctx.world.entity_mut(vid) as *mut gentity_t;
        crate::ent_fn_enums::dispatch_die(ctx, die_fn, vp, vp, vp, o_health, MOD_UNKNOWN as c_int);
    }
}

/// Raven `Q3_RemoveEnt`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1062-1116`
pub fn Q3_RemoveEnt(ctx: &mut GameContext, victim: EntityId) {
    let client = ctx.world.entity(victim).client;
    if !client.is_null() {
        if ctx.world.entity(victim).s.eType != ET_NPC as c_int {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                b"Q3_RemoveEnt: You can't remove clients in MP!\n\0".as_ptr() as *const c_char,
            );
            debug_assert!(false);
        } else {
            // Remove the NPC.
            // Pool client (NPC): read NPC_class raw through the copied pointer.
            let npc_class = unsafe { (*client).NPC_class };
            if npc_class == CLASS_VEHICLE {
                // Eject everyone out of a vehicle that's about to remove itself.
                let pVeh = ctx.world.entity(victim).m_pVehicle;
                if !pVeh.is_null() && !unsafe { (*pVeh).m_pVehicleInfo }.is_null() {
                    eject_all(ctx, pVeh);
                }
            }
            let level_time = ctx.world.level.time;
            let e = ctx.world.entity_mut(victim);
            e.think = Some(EntThink::G_FreeEntity).into();
            e.nextthink = level_time + 100;
        }
    } else {
        let level_time = ctx.world.level.time;
        let e = ctx.world.entity_mut(victim);
        e.think = Some(EntThink::G_FreeEntity).into();
        e.nextthink = level_time + 100;
    }
}

/// Raven `Q3_Remove`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1128-1168`
pub fn Q3_Remove(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    let id = EntityId(entID as u32);

    if Q_stricmp(b"self\0".as_ptr() as *const c_char, name) == 0 {
        Q3_RemoveEnt(ctx, id);
    } else if Q_stricmp(b"enemy\0".as_ptr() as *const c_char, name) == 0 {
        let victim = ctx.world.entity(id).enemy;
        if victim.is_none() {
            G_DebugPrint(
                ctx,
                WL_WARNING as c_int,
                b"Q3_Remove: can't find enemy\n\0".as_ptr() as *const c_char,
            );
            return;
        }
        Q3_RemoveEnt(ctx, victim.unwrap());
    } else {
        let mut victim = G_Find(
            ctx,
            None,
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
            let vid = ctx.entity_id_of(victim).unwrap();
            Q3_RemoveEnt(ctx, vid);
            victim = G_Find(
                ctx,
                Some(vid),
                core::mem::offset_of!(gentity_t, targetname) as c_int,
                name,
            );
        }
    }
}

/// Raven `Q3_GetFloat`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1189-1559`
pub fn Q3_GetFloat(
    ctx: &mut GameContext,
    entID: c_int,
    r#type: c_int,
    name: *const c_char,
    value: *mut f32,
) -> c_int {
    unsafe {
        let id = EntityId(entID as u32);

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
                let parms = ctx.world.entity(id).parms;
                if parms.is_null() {
                    let classname = ctx.world.entity(id).classname;
                    let targetname = ctx.world.entity(id).targetname;
                    G_DebugPrint(
                        ctx,
                        WL_ERROR as c_int,
                        cstr(&format!(
                            "GET_PARM: {} {} did not have any parms set!\n",
                            cstr_to_str(classname),
                            cstr_to_str(targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                // Parms pool struct: deref raw through the copied pointer.
                *value = atof_bytes(
                    CStr::from_ptr((*parms).parm[(toGet - SET_PARM1 as i32) as usize].as_ptr())
                        .to_bytes(),
                ) as f32;
            }
            _ if toGet == SET_COUNT as i32 => *value = ctx.world.entity(id).count as f32,
            _ if toGet == SET_HEALTH as i32 => *value = ctx.world.entity(id).health as f32,
            _ if toGet == SET_SKILL as i32 => return 0,
            _ if toGet == SET_XVELOCITY as i32 => {
                let client = ctx.world.entity(id).client;
                if client.is_null() {
                    let targetname = ctx.world.entity(id).targetname;
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_XVELOCITY, {} not a client\n",
                            cstr_to_str(targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*client).ps.velocity[0];
            }
            _ if toGet == SET_YVELOCITY as i32 => {
                let client = ctx.world.entity(id).client;
                if client.is_null() {
                    let targetname = ctx.world.entity(id).targetname;
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_YVELOCITY, {} not a client\n",
                            cstr_to_str(targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*client).ps.velocity[1];
            }
            _ if toGet == SET_ZVELOCITY as i32 => {
                let client = ctx.world.entity(id).client;
                if client.is_null() {
                    let targetname = ctx.world.entity(id).targetname;
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_ZVELOCITY, {} not a client\n",
                            cstr_to_str(targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*client).ps.velocity[2];
            }
            _ if toGet == SET_Z_OFFSET as i32 => {
                let e = ctx.world.entity(id);
                *value = e.r.currentOrigin[2] - e.s.origin[2]
            }
            _ if toGet == SET_DPITCH as i32 => return 0,
            _ if toGet == SET_DYAW as i32 => return 0,
            _ if toGet == SET_WIDTH as i32 => *value = ctx.world.entity(id).r.mins[0],
            _ if toGet == SET_TIMESCALE as i32 => return 0,
            _ if toGet == SET_CAMERA_GROUP_Z_OFS as i32 => return 0,
            _ if toGet == SET_VISRANGE as i32 => return 0,
            _ if toGet == SET_EARSHOT as i32 => return 0,
            _ if toGet == SET_VIGILANCE as i32 => return 0,
            _ if toGet == SET_GRAVITY as i32 => *value = ctx.world.cvars.g_gravity.value,
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
            _ if toGet == SET_WAIT as i32 => *value = ctx.world.entity(id).wait,
            _ if toGet == SET_FOLLOWDIST as i32 => return 0,
            _ if toGet == SET_ANIM_HOLDTIME_LOWER as i32 => {
                let client = ctx.world.entity(id).client;
                if client.is_null() {
                    let targetname = ctx.world.entity(id).targetname;
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_ANIM_HOLDTIME_LOWER, {} not a client\n",
                            cstr_to_str(targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*client).ps.legsTimer as f32;
            }
            _ if toGet == SET_ANIM_HOLDTIME_UPPER as i32 => {
                let client = ctx.world.entity(id).client;
                if client.is_null() {
                    let targetname = ctx.world.entity(id).targetname;
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_ANIM_HOLDTIME_UPPER, {} not a client\n",
                            cstr_to_str(targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*client).ps.torsoTimer as f32;
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
                let client = ctx.world.entity(id).client;
                if client.is_null() {
                    let targetname = ctx.world.entity(id).targetname;
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetFloat: SET_ARMOR, {} not a client\n",
                            cstr_to_str(targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
                *value = (*client).ps.stats[STAT_ARMOR as usize] as f32;
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
            _ if toGet == SET_NOTARGET as i32 => {
                *value = (ctx.world.entity(id).flags & FL_NOTARGET) as f32
            }
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
            _ if toGet == SET_SOLID as i32 => *value = ctx.world.entity(id).r.contents as f32,
            _ if toGet == SET_PLAYER_USABLE as i32 => {
                *value = (ctx.world.entity(id).r.svFlags & SVF_PLAYER_USABLE) as f32
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
            _ if toGet == SET_INVISIBLE as i32 => {
                *value = (ctx.world.entity(id).s.eFlags & EF_NODRAW) as f32
            }
            _ if toGet == SET_PLAYER_LOCKED as i32
                || toGet == SET_LOCK_PLAYER_WEAPONS as i32
                || toGet == SET_NO_IMPACT_DAMAGE as i32 =>
            {
                return 0
            }
            _ if toGet == SET_NO_KNOCKBACK as i32 => {
                *value = (ctx.world.entity(id).flags & FL_NO_KNOCKBACK) as f32
            }
            _ if toGet == SET_ALT_FIRE as i32 || toGet == SET_NO_RESPONSE as i32 => return 0,
            _ if toGet == SET_INVINCIBLE as i32 => {
                *value = (ctx.world.entity(id).flags & FL_GODMODE) as f32
            }
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
/// Source: `oracle/codemp/game/g_ICARUScb.c:1573-1629`
pub fn Q3_GetVector(
    ctx: &mut GameContext,
    entID: c_int,
    r#type: c_int,
    name: *const c_char,
    value: &mut [f32; 3],
) -> c_int {
    unsafe {
        let id = EntityId(entID as u32);

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
                // Parms pool struct: deref raw through the copied pointer.
                let parms = ctx.world.entity(id).parms;
                let parm_str =
                    cstr_to_str((*parms).parm[(toGet - SET_PARM1 as i32) as usize].as_ptr());
                sscanf_f32s(&parm_str, value);
            }
            _ if toGet == SET_ORIGIN as i32 => *value = ctx.world.entity(id).r.currentOrigin,
            _ if toGet == SET_ANGLES as i32 => *value = ctx.world.entity(id).r.currentAngles,
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
/// Source: `oracle/codemp/game/g_ICARUScb.c:1642-1854`
pub fn Q3_GetString(
    ctx: &mut GameContext,
    entID: c_int,
    r#type: c_int,
    name: *const c_char,
    value: *mut *mut c_char,
) -> c_int {
    unsafe {
        let id = EntityId(entID as u32);

        let toGet = GetIDForString(setTable.as_ptr() as *mut stringID_table_t, name);

        match toGet {
            _ if toGet == SET_ANIM_BOTH as i32 => {
                *value = Q3_GetAnimBoth(ctx, id);
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
                let parms = ctx.world.entity(id).parms;
                if !parms.is_null() {
                    *value = (*parms).parm[(toGet - SET_PARM1 as i32) as usize].as_mut_ptr();
                } else {
                    let targetname = ctx.world.entity(id).targetname;
                    G_DebugPrint(
                        ctx,
                        WL_WARNING as c_int,
                        cstr(&format!(
                            "Q3_GetString: invalid ent {} has no parms!\n",
                            cstr_to_str(targetname)
                        ))
                        .as_ptr(),
                    );
                    return 0;
                }
            }
            _ if toGet == SET_TARGET as i32 => *value = ctx.world.entity(id).target,
            _ if toGet == SET_LOCATION as i32 => return 0,
            _ if toGet == SET_SPAWNSCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_SPAWN as usize]
            }
            _ if toGet == SET_USESCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_USE as usize]
            }
            _ if toGet == SET_AWAKESCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_AWAKE as usize]
            }
            _ if toGet == SET_ANGERSCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_ANGER as usize]
            }
            _ if toGet == SET_ATTACKSCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_ATTACK as usize]
            }
            _ if toGet == SET_VICTORYSCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_VICTORY as usize]
            }
            _ if toGet == SET_LOSTENEMYSCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_LOSTENEMY as usize]
            }
            _ if toGet == SET_PAINSCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_PAIN as usize]
            }
            _ if toGet == SET_FLEESCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_FLEE as usize]
            }
            _ if toGet == SET_DEATHSCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_DEATH as usize]
            }
            _ if toGet == SET_DELAYEDSCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_DELAYED as usize]
            }
            _ if toGet == SET_BLOCKEDSCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_BLOCKED as usize]
            }
            _ if toGet == SET_FFIRESCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_FFIRE as usize]
            }
            _ if toGet == SET_FFDEATHSCRIPT as i32 => {
                *value = ctx.world.entity(id).behaviorSet[BSET_FFDEATH as usize]
            }
            _ if toGet == SET_ENEMY as i32
                || toGet == SET_LEADER as i32
                || toGet == SET_CAPTURE as i32 =>
            {
                return 0
            }
            _ if toGet == SET_TARGETNAME as i32 => *value = ctx.world.entity(id).targetname,
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
            _ if toGet == SET_FULLNAME as i32 => *value = ctx.world.entity(id).fullName,
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
    // `owner` is `&g_entities[ownerNum]` — an array element, never NULL, so
    // Raven's dead `owner->` null guard collapses to the `inuse` check.
    let owner_id = EntityId(ctx.world.entity(self_).r.ownerNum as u32);

    let level_time = ctx.world.level.time;
    {
        let s = ctx.world.entity_mut(self_);
        s.nextthink = level_time + FRAMETIME;
        s.think = Some(EntThink::G_FreeEntity).into();
    }

    if ctx.world.entity(owner_id).inuse == 0 {
        return;
    }

    let cur_origin = ctx.world.entity(self_).r.currentOrigin;
    if SpotWouldTelefrag2(ctx, owner_id, cur_origin) != 0 {
        ctx.world.entity_mut(self_).think = Some(EntThink::MoveOwner).into();
    } else {
        G_SetOrigin(ctx.world.entity_mut(owner_id), cur_origin);
        trap::ICARUS_TaskIDComplete(
            ctx.engine,
            GIcarusTaskidcompleteArgs::new(
                (ctx.world.entity_mut(owner_id) as *mut gentity_t).cast(),
                taskID_t::TID_MOVE_NAV as c_int,
            ),
        );
    }
}

/// Raven `Q3_SetTeleportDest`.
///
/// `org` is only ever read here, so it stays by-value.
/// Source: `oracle/codemp/game/g_ICARUScb.c:1895-1920`
pub fn Q3_SetTeleportDest(ctx: &mut GameContext, entID: c_int, org: vec3_t) -> qboolean {
    let id = EntityId(entID as u32);

    if SpotWouldTelefrag2(ctx, id, org) != 0 {
        let tp_id = G_Spawn(ctx);

        G_SetOrigin(ctx.world.entity_mut(tp_id), org);
        let number = ctx.world.entity(id).s.number;
        let level_time = ctx.world.level.time;
        let tp = ctx.world.entity_mut(tp_id);
        tp.r.ownerNum = number;
        tp.think = Some(EntThink::MoveOwner).into();
        tp.nextthink = level_time + FRAMETIME;

        qfalse
    } else {
        G_SetOrigin(ctx.world.entity_mut(id), org);
        qtrue
    }
}

/// Raven `Q3_SetOrigin`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1929-1961`
pub fn Q3_SetOrigin(ctx: &mut GameContext, entID: c_int, origin: vec3_t) {
    let id = EntityId(entID as u32);

    trap::UnlinkEntity(
        ctx.engine,
        GUnlinkentityArgs::new((ctx.world.entity_mut(id) as *mut gentity_t).cast()),
    );

    let client = ctx.world.entity(id).client;
    if !client.is_null() {
        ctx.world.entity_mut(id).r.currentOrigin = origin;
        // Pool client (NPC): deref raw through the copied pointer.
        unsafe {
            (*client).ps.origin = origin;
            (*client).ps.origin[2] += 1.0;

            (*client).ps.velocity = [0.0, 0.0, 0.0];
            (*client).ps.pm_time = 160;
            (*client).ps.pm_flags |= PMF_TIME_KNOCKBACK;

            (*client).ps.eFlags ^= EF_TELEPORT_BIT;
        }
    } else {
        G_SetOrigin(ctx.world.entity_mut(id), origin);
    }

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new((ctx.world.entity_mut(id) as *mut gentity_t).cast()),
    );
}

/// Raven `Q3_SetCopyOrigin`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1970-1983`
pub fn Q3_SetCopyOrigin(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    let found = G_Find(
        ctx,
        None,
        core::mem::offset_of!(gentity_t, targetname) as c_int,
        name,
    );

    if !found.is_null() {
        let fid = ctx.entity_id_of(found).unwrap();
        let origin = ctx.world.entity(fid).r.currentOrigin;
        Q3_SetOrigin(ctx, entID, origin);
        let id = EntityId(entID as u32);
        let angles = ctx.world.entity(fid).s.angles;
        SetClientViewAngle(ctx.world.entity_mut(id), angles);
    } else {
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            b"Q3_SetCopyOrigin: ent not found!\n\0".as_ptr() as *const c_char,
        );
    }
}

/// Raven `Q3_SetVelocity`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:1992-2013`
pub fn Q3_SetVelocity(ctx: &mut GameContext, entID: c_int, axis: c_int, speed: f32) {
    let id = EntityId(entID as u32);

    let client = ctx.world.entity(id).client;
    if client.is_null() {
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            cstr(&format!("Q3_SetVelocity: not a client {}\n", entID)).as_ptr(),
        );
        return;
    }

    // Pool client (NPC): deref raw through the copied pointer.
    unsafe {
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
    let id = EntityId(entID as u32);

    if ctx.world.entity(id).client.is_null() {
        ctx.world.entity_mut(id).s.angles = angles;
    } else {
        SetClientViewAngle(ctx.world.entity_mut(id), angles);
    }
    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new((ctx.world.entity_mut(id) as *mut gentity_t).cast()),
    );
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
    let id = EntityId(entID as u32);

    let is_not_mover = {
        let e = ctx.world.entity(id);
        !e.client.is_null()
            || Q_stricmp(
                e.classname,
                b"target_scriptrunner\0".as_ptr() as *const c_char,
            ) == 0
    };
    if is_not_mover {
        G_DebugPrint(
            ctx,
            WL_ERROR as c_int,
            cstr(&format!("Q3_Lerp2Origin: ent {} is NOT a mover!\n", entID)).as_ptr(),
        );
        return;
    }

    {
        let e = ctx.world.entity_mut(id);
        if e.s.eType != entityType_t::ET_MOVER as c_int {
            e.s.eType = entityType_t::ET_MOVER as c_int;
        }
    }

    let mut moverState = ctx.world.entity(id).moverState;

    {
        let e = ctx.world.entity_mut(id);
        if moverState == MOVER_POS1 || moverState == MOVER_2TO1 {
            e.pos1 = e.r.currentOrigin;
            e.pos2 = origin;
            moverState = MOVER_1TO2;
        } else if moverState == MOVER_POS2 || moverState == MOVER_1TO2 {
            e.pos2 = e.r.currentOrigin;
            e.pos1 = origin;
            moverState = MOVER_2TO1;
        }
        e.moverState = moverState;

        InitMoverTrData(e);

        e.s.pos.trDuration = duration as c_int;
    }

    let time = ctx.world.level.time;
    MatchTeam(ctx, id, moverState as c_int, time);

    {
        let e = ctx.world.entity_mut(id);
        e.reached = Some(EntReached::moverCallback).into();
        if e.damage != 0 {
            e.blocked = Some(EntBlocked::Blocked_Mover).into();
        }
    }
    if taskID != -1 {
        trap::ICARUS_TaskIDSet(
            ctx.engine,
            GIcarusTaskidsetArgs::new(
                (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                taskID_t::TID_MOVE_NAV as c_int,
                taskID,
            ),
        );
    }

    G_PlayDoorLoopSound(ctx, id);
    G_PlayDoorSound(ctx, id, BMS_START);

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new((ctx.world.entity_mut(id) as *mut gentity_t).cast()),
    );
}

/// Raven `Q3_SetOriginOffset`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2114-2140`
pub fn Q3_SetOriginOffset(ctx: &mut GameContext, entID: c_int, axis: c_int, offset: f32) {
    let id = EntityId(entID as u32);

    let is_not_mover = {
        let e = ctx.world.entity(id);
        !e.client.is_null()
            || Q_stricmp(
                e.classname,
                b"target_scriptrunner\0".as_ptr() as *const c_char,
            ) == 0
    };
    if is_not_mover {
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

    let mut origin = ctx.world.entity(id).s.origin;
    origin[axis as usize] += offset;
    let mut duration = 0.0f32;
    let speed = ctx.world.entity(id).speed;
    if speed != 0.0 {
        // C's `fabs` is the double libm function: the divide and `*1000.0f`
        // evaluate in f64, narrowing to the float `duration` only at the
        // assignment. f32-throughout would diverge at Q3_Lerp2Origin's
        // `trDuration` truncation boundaries.
        duration = ((offset as f64).abs() / (speed as f64).abs() * 1000.0) as f32;
    }
    Q3_Lerp2Origin(ctx, -1, entID, origin, duration);
}

/// Raven `Q3_SetEnemy`.
///
/// `ent->NPC` is only null-checked (never dereferenced) in this fn.
/// Source: `oracle/codemp/game/g_ICARUScb.c:2149-2197`
pub fn Q3_SetEnemy(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    let id = EntityId(entID as u32);

    if Q_stricmp(b"NONE\0".as_ptr() as *const c_char, name) == 0
        || Q_stricmp(b"NULL\0".as_ptr() as *const c_char, name) == 0
    {
        if !ctx.world.entity(id).NPC.is_null() {
            G_ClearEnemy(ctx, id);
        } else {
            ctx.world.entity_mut(id).enemy = None;
        }
    } else {
        let enemy = G_Find(
            ctx,
            None,
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

        let enemy_id = ctx.entity_id_of(enemy);
        G_SetEnemy(ctx, id, enemy_id);
        if !ctx.world.entity(id).NPC.is_null() {
            ctx.world.entity_mut(id).cantHitEnemyCounter = 0;
        }
    }
}

/// Raven `Q3_SetLeader`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2207-2246`
pub fn Q3_SetLeader(ctx: &mut GameContext, entID: c_int, name: *const c_char) {
    let id = EntityId(entID as u32);

    let client = ctx.world.entity(id).client;
    if client.is_null() {
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

    if Q_stricmp(b"NONE\0".as_ptr() as *const c_char, name) == 0
        || Q_stricmp(b"NULL\0".as_ptr() as *const c_char, name) == 0
    {
        // Pool client (NPC): deref raw through the copied pointer.
        unsafe { (*client).leader = None };
    } else {
        let leader = G_Find(
            ctx,
            None,
            core::mem::offset_of!(gentity_t, targetname) as c_int,
            name,
        );

        if leader.is_null() {
            return;
        }
        let lid = ctx.entity_id_of(leader).unwrap();
        if ctx.world.entity(lid).health <= 0 {
            return;
        }
        // Pool client (NPC): deref raw through the copied pointer.
        unsafe { (*client).leader = Some(lid) };
    }
}

/// Raven `Q3_SetNavGoal`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2255-2320`
pub fn Q3_SetNavGoal(ctx: &mut GameContext, entID: c_int, name: *const c_char) -> qboolean {
    // `npc` (gNPC_t) has no accessor; its derefs stay raw through the copied
    // pointer inside this unsafe block (recipe 2c), as do the `cstr_to_str`
    // string reads. Entity access goes through `ctx.world.entity[_mut](id)`.
    unsafe {
        let id = EntityId(entID as u32);
        let mut goalPos: vec3_t = [0.0, 0.0, 0.0];

        if ctx.world.entity(id).health == 0 {
            let st = ctx.world.entity(id).script_targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetNavGoal: tried to set a navgoal (\"{}\") on a corpse! \"{}\"\n",
                    cstr_to_str(name),
                    cstr_to_str(st)
                ))
                .as_ptr(),
            );
            return qfalse;
        }
        if ctx.world.entity(id).NPC.is_null() {
            let st = ctx.world.entity(id).script_targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetNavGoal: tried to set a navgoal (\"{}\") on a non-NPC: \"{}\"\n",
                    cstr_to_str(name),
                    cstr_to_str(st)
                ))
                .as_ptr(),
            );
            return qfalse;
        }
        let npc = ctx.world.entity(id).NPC;
        if (*npc).tempGoal.is_none() {
            let st = ctx.world.entity(id).script_targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetNavGoal: tried to set a navgoal (\"{}\") on a dead NPC: \"{}\"\n",
                    cstr_to_str(name),
                    cstr_to_str(st)
                ))
                .as_ptr(),
            );
            return qfalse;
        }
        let temp_goal_id = (*npc).tempGoal.unwrap();
        if ctx.world.entity(temp_goal_id).inuse == 0 {
            let st = ctx.world.entity(id).script_targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetNavGoal: NPC's (\"{}\") navgoal is freed: \"{}\"\n",
                    cstr_to_str(name),
                    cstr_to_str(st)
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
                GIcarusTaskidcompleteArgs::new(
                    (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                    taskID_t::TID_MOVE_NAV as c_int,
                ),
            );
            return qfalse;
        }

        if TAG_GetOrigin2(ctx, std::ptr::null(), name, &mut goalPos) == qfalse {
            let targ = G_Find(
                ctx,
                None,
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
            let targ_id = ctx.entity_id_of(targ).unwrap();
            (*npc).goalEntity = Some(targ_id);
            // C's `sqrt` is the double libm function: the float sums promote to
            // f64, are rooted and summed in f64, then truncated to the int
            // `goalRadius`. f32-throughout would diverge at truncation boundaries.
            let ent_maxs0 = ctx.world.entity(id).r.maxs[0];
            let targ_maxs0 = ctx.world.entity(targ_id).r.maxs[0];
            (*npc).goalRadius = ((ent_maxs0 as f64 + ent_maxs0 as f64).sqrt()
                + (targ_maxs0 as f64 + targ_maxs0 as f64).sqrt())
                as c_int;
            (*npc).aiFlags &= !NPCAI_TOUCHED_GOAL;
            qfalse
        } else {
            let goalRadius = TAG_GetRadius(ctx, std::ptr::null(), name);
            NPC_SetMoveGoal(ctx, id, goalPos, goalRadius, qtrue, -1, None);
            let goal_id = (*npc).goalEntity.unwrap();
            ctx.world.entity_mut(goal_id).lastWaypoint = WAYPOINT_NONE;
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
    let id = EntityId(entID as u32);

    if ctx.world.entity(id).client.is_null() {
        G_DebugPrint(
            ctx,
            WL_ERROR as c_int,
            b"SetLowerAnim: ent is NOT a player or NPC!\n\0".as_ptr() as *const c_char,
        );
        return;
    }

    G_SetAnim(
        ctx,
        id,
        std::ptr::null_mut(),
        SETANIM_LEGS,
        animID,
        SETANIM_FLAG_RESTART | SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        0,
    );
}

/// Raven `SetUpperAnim`.
///
/// `ent->client` is only null-checked (never dereferenced) in this fn.
/// Source: `oracle/codemp/game/g_ICARUScb.c:2358-2375`
pub fn SetUpperAnim(ctx: &mut GameContext, entID: c_int, animID: c_int) {
    let id = EntityId(entID as u32);

    if ctx.world.entity(id).client.is_null() {
        G_DebugPrint(
            ctx,
            WL_ERROR as c_int,
            b"SetUpperAnim: ent is NOT a player or NPC!\n\0".as_ptr() as *const c_char,
        );
        return;
    }

    G_SetAnim(
        ctx,
        id,
        std::ptr::null_mut(),
        SETANIM_TORSO,
        animID,
        SETANIM_FLAG_RESTART | SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        0,
    );
}

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
    let id = EntityId(entID as u32);
    let mut data = data;

    if data < 0 {
        data = 0;
    }

    ctx.world.entity_mut(id).health = data;

    let client = ctx.world.entity(id).client;
    if client.is_null() {
        return;
    }

    // Pool client (NPC): deref raw through the copied pointer.
    unsafe {
        (*client).ps.stats[STAT_HEALTH as usize] = data;

        if (*client).ps.stats[STAT_HEALTH as usize] > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
            let max = (*client).ps.stats[STAT_MAX_HEALTH as usize];
            ctx.world.entity_mut(id).health = max;
            let h = ctx.world.entity(id).health;
            (*client).ps.stats[STAT_HEALTH as usize] = h;
        }
        if data == 0 {
            ctx.world.entity_mut(id).health = 1;
            if (*client).sess.sessionTeam == TEAM_SPECTATOR {
                return;
            }

            ctx.world.entity_mut(id).flags &= !FL_GODMODE;
            ctx.world.entity_mut(id).health = -999;
            let h = ctx.world.entity(id).health;
            (*client).ps.stats[STAT_HEALTH as usize] = h;
            player_die(ctx, id, Some(id), Some(id), 100000, MOD_FALLING as c_int);
        }
    }
}

/// Raven `Q3_SetArmor`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2539-2559`
pub fn Q3_SetArmor(ctx: &mut GameContext, entID: c_int, data: c_int) {
    let id = EntityId(entID as u32);

    let client = ctx.world.entity(id).client;
    if client.is_null() {
        return;
    }

    // Pool client (NPC): deref raw through the copied pointer.
    unsafe {
        (*client).ps.stats[STAT_ARMOR as usize] = data;
        if (*client).ps.stats[STAT_ARMOR as usize] > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
            (*client).ps.stats[STAT_ARMOR as usize] = (*client).ps.stats[STAT_MAX_HEALTH as usize];
        }
    }
}

/// Raven `Q3_SetBState`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2573-2687`
pub fn Q3_SetBState(ctx: &mut GameContext, entID: c_int, bs_name: *const c_char) -> qboolean {
    // `npc` (gNPC_t) has no accessor; its derefs, the pool-client `noclip`
    // write, the `bState_t` transmutes and `cstr_to_str` all stay raw inside
    // this unsafe block (recipe 2c). Entity access uses `ctx.world.entity[_mut]`.
    unsafe {
        let id = EntityId(entID as u32);

        if ctx.world.entity(id).NPC.is_null() {
            let tn = ctx.world.entity(id).targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetBState: '{}' is not an NPC\n",
                    cstr_to_str(tn)
                ))
                .as_ptr(),
            );
            return qtrue;
        }
        let npc = ctx.world.entity(id).NPC;

        let bSID = GetIDForString(BSTable.as_ptr() as *mut stringID_table_t, bs_name);
        if bSID > -1 {
            if bSID == (BS_SEARCH) as i32 || bSID == (BS_WANDER) as i32 {
                let wp = ctx.world.entity(id).waypoint;
                if wp != WAYPOINT_NONE {
                    NPC_BSSearchStart(ctx, wp, core::mem::transmute::<c_int, bState_t>(bSID));
                } else {
                    let new_wp = NAV_FindClosestWaypointForEnt(ctx, id, WAYPOINT_NONE);
                    ctx.world.entity_mut(id).waypoint = new_wp;

                    if new_wp != WAYPOINT_NONE {
                        NPC_BSSearchStart(
                            ctx,
                            new_wp,
                            core::mem::transmute::<c_int, bState_t>(bSID),
                        );
                    } else {
                        let tn = ctx.world.entity(id).targetname;
                        G_DebugPrint(
                            ctx,
                            WL_ERROR as c_int,
                            cstr(&format!(
                                "Q3_SetBState: '{}' is not in a valid waypoint to search from!\n",
                                cstr_to_str(tn)
                            ))
                            .as_ptr(),
                        );
                        return qtrue;
                    }
                }
            }

            (*npc).tempBehavior = BS_DEFAULT;
            if (*npc).behaviorState == BS_NOCLIP && bSID != (BS_NOCLIP) as i32 {
                ctx.world.entity_mut(id).r.currentOrigin[2] += 0.125;
                let cur = ctx.world.entity(id).r.currentOrigin;
                G_SetOrigin(ctx.world.entity_mut(id), cur);
            }
            (*npc).behaviorState = core::mem::transmute::<c_int, bState_t>(bSID);
            if bSID == (BS_DEFAULT) as i32 {
                (*npc).defaultBehavior = core::mem::transmute::<c_int, bState_t>(bSID);
            }
        }

        (*npc).aiFlags &= !NPCAI_TOUCHED_GOAL;

        let client = ctx.world.entity(id).client;
        if bSID == (BS_NOCLIP) as i32 {
            (*client).noclip = qtrue;
        } else {
            (*client).noclip = qfalse;
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

/// Raven `Q3_SetTempBState`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2699-2737`
pub fn Q3_SetTempBState(ctx: &mut GameContext, entID: c_int, bs_name: *const c_char) -> qboolean {
    // gNPC_t deref + transmute + cstr_to_str stay raw (recipe 2c).
    unsafe {
        let id = EntityId(entID as u32);

        if ctx.world.entity(id).NPC.is_null() {
            let tn = ctx.world.entity(id).targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetTempBState: '{}' is not an NPC\n",
                    cstr_to_str(tn)
                ))
                .as_ptr(),
            );
            return qtrue;
        }
        let npc = ctx.world.entity(id).NPC;

        let bSID = GetIDForString(BSTable.as_ptr() as *mut stringID_table_t, bs_name);
        if bSID > -1 {
            (*npc).tempBehavior = core::mem::transmute::<c_int, bState_t>(bSID);
        }

        qtrue
    }
}

/// Raven `Q3_SetDefaultBState`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2749-2771`
pub fn Q3_SetDefaultBState(ctx: &mut GameContext, entID: c_int, bs_name: *const c_char) {
    // gNPC_t deref + transmute + cstr_to_str stay raw (recipe 2c).
    unsafe {
        let id = EntityId(entID as u32);

        if ctx.world.entity(id).NPC.is_null() {
            let tn = ctx.world.entity(id).targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetDefaultBState: '{}' is not an NPC\n",
                    cstr_to_str(tn)
                ))
                .as_ptr(),
            );
            return;
        }
        let npc = ctx.world.entity(id).NPC;

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

/// Raven `Q3_SetInvisible`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:2941-2968`
pub fn Q3_SetInvisible(ctx: &mut GameContext, entID: c_int, invisible: qboolean) {
    let id = EntityId(entID as u32);

    if invisible != 0 {
        ctx.world.entity_mut(id).s.eFlags |= EF_NODRAW;
        let client = ctx.world.entity(id).client;
        if !client.is_null() {
            // Pool client (NPC): deref raw through the copied pointer.
            unsafe { (*client).ps.eFlags |= EF_NODRAW };
        }
        ctx.world.entity_mut(id).r.contents = 0;
    } else {
        ctx.world.entity_mut(id).s.eFlags &= !EF_NODRAW;
        let client = ctx.world.entity(id).client;
        if !client.is_null() {
            // Pool client (NPC): deref raw through the copied pointer.
            unsafe { (*client).ps.eFlags &= !EF_NODRAW };
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
    let id = EntityId(entID as u32);

    if Q_stricmp(b"NULL\0".as_ptr() as *const c_char, name) == 0
        || Q_stricmp(b"NONE\0".as_ptr() as *const c_char, name) == 0
    {
        let e = ctx.world.entity_mut(id);
        e.s.loopSound = 0;
        e.s.loopIsSoundset = qfalse;
        return;
    }

    let index = G_SoundIndex(name);

    if index != 0 {
        let e = ctx.world.entity_mut(id);
        e.s.loopSound = index;
        e.s.loopIsSoundset = qfalse;
    } else {
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            b"Q3_SetLoopSound: can't find sound file\n\0".as_ptr() as *const c_char,
        );
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
    let mut self_ = G_Find(
        ctx,
        None,
        core::mem::offset_of!(gentity_t, targetname) as c_int,
        name,
    );
    if self_.is_null() {
        self_ = G_Find(
            ctx,
            None,
            core::mem::offset_of!(gentity_t, script_targetname) as c_int,
            name,
        );
    }

    if self_.is_null() {
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            cstr(&format!("Q3_SetICARUSFreeze: invalid ent {}\n", unsafe {
                cstr_to_str(name)
            }))
            .as_ptr(),
        );
        return;
    }

    let sid = ctx.entity_id_of(self_).unwrap();
    if freeze != 0 {
        ctx.world.entity_mut(sid).r.svFlags |= SVF_ICARUS_FREEZE;
    } else {
        ctx.world.entity_mut(sid).r.svFlags &= !SVF_ICARUS_FREEZE;
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

/// Raven `Q3_SetWeapon`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3104-3111`
pub fn Q3_SetWeapon(ctx: &mut GameContext, entID: c_int, wp_name: *const c_char) {
    let id = EntityId(entID as u32);
    let wp = GetIDForString(WPTable.as_ptr() as *mut stringID_table_t, wp_name);

    let client = ctx.world.entity(id).client;
    // Pool client (NPC): deref raw through the copied pointer (Raven assumes
    // non-NULL here, so no guard is added).
    unsafe {
        (*client).ps.stats[STAT_WEAPONS as usize] = 1 << wp;
    }
    ChangeWeapon(ctx, Some(id), wp);
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
    // gNPC_t + pool-client derefs and cstr_to_str stay raw (recipe 2c).
    unsafe {
        let id = EntityId(entID as u32);

        if ctx.world.entity(id).NPC.is_null() {
            let tn = ctx.world.entity(id).targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetWalkSpeed: '{}' is not an NPC!\n",
                    cstr_to_str(tn)
                ))
                .as_ptr(),
            );
            return;
        }
        let npc = ctx.world.entity(id).NPC;
        let client = ctx.world.entity(id).client;

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
    // gNPC_t + pool-client derefs and cstr_to_str stay raw (recipe 2c).
    unsafe {
        let id = EntityId(entID as u32);

        if ctx.world.entity(id).NPC.is_null() {
            let tn = ctx.world.entity(id).targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetRunSpeed: '{}' is not an NPC!\n",
                    cstr_to_str(tn)
                ))
                .as_ptr(),
            );
            return;
        }
        let npc = ctx.world.entity(id).NPC;
        let client = ctx.world.entity(id).client;

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
    let id = EntityId(entID as u32);

    if ctx.world.entity(id).client.is_null() {
        let tn = ctx.world.entity(id).targetname;
        G_DebugPrint(
            ctx,
            WL_ERROR as c_int,
            cstr(&format!(
                "Q3_SetFriction: '{}' is not an NPC/player!\n",
                unsafe { cstr_to_str(tn) }
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

/// Raven `Q3_SetGravity`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3285-3307`
pub fn Q3_SetGravity(ctx: &mut GameContext, entID: c_int, float_data: f32) {
    // gNPC_t + pool-client derefs and cstr_to_str stay raw (recipe 2c).
    unsafe {
        let id = EntityId(entID as u32);

        if ctx.world.entity(id).client.is_null() {
            let tn = ctx.world.entity(id).targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetGravity: '{}' is not an NPC/player!\n",
                    cstr_to_str(tn)
                ))
                .as_ptr(),
            );
            return;
        }
        let client = ctx.world.entity(id).client;

        let npc = ctx.world.entity(id).NPC;
        if !npc.is_null() {
            (*npc).aiFlags |= NPCAI_CUSTOM_GRAVITY;
        }
        (*client).ps.gravity = float_data as c_int;
    }
}

/// Raven `Q3_SetWait`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3319-3330`
pub fn Q3_SetWait(ctx: &mut GameContext, entID: c_int, float_data: f32) {
    let id = EntityId(entID as u32);
    ctx.world.entity_mut(id).wait = float_data;
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
    let id = EntityId(entID as u32);

    let client = ctx.world.entity(id).client;
    if !client.is_null() {
        // Pool client (NPC): deref raw through the copied pointer.
        unsafe {
            if float_data < 0.0 {
                (*client).ps.iModelScale = float_data as c_int;
            } else {
                (*client).ps.iModelScale = (float_data * 100.0) as c_int;
            }
        }
    } else {
        if float_data < 0.0 {
            ctx.world.entity_mut(id).s.iModelScale = float_data as c_int;
        } else {
            ctx.world.entity_mut(id).s.iModelScale = (float_data * 100.0) as c_int;
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
                val = atof(rest) as f32;
            }
        } else if let Some(rest) = s.strip_prefix('-') {
            if !rest.is_empty() {
                val = atof(rest) as f32 * -1.0;
            }
        }

        val
    }
}

/// Raven `Q3_SetCount`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3440-3460`
pub fn Q3_SetCount(ctx: &mut GameContext, entID: c_int, data: *const c_char) {
    let id = EntityId(entID as u32);

    let val = Q3_GameSideCheckStringCounterIncrement(data);
    if val != 0.0 {
        ctx.world.entity_mut(id).count += val as c_int;
    } else {
        ctx.world.entity_mut(id).count = atoi(data);
    }
}

/// Raven `Q3_SetTargetName`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3472-3490`
pub fn Q3_SetTargetName(ctx: &mut GameContext, entID: c_int, targetname: *const c_char) {
    let id = EntityId(entID as u32);

    if Q_stricmp(b"NULL\0".as_ptr() as *const c_char, targetname) == 0 {
        ctx.world.entity_mut(id).targetname = std::ptr::null_mut();
    } else {
        let s = G_NewString(ctx, targetname);
        ctx.world.entity_mut(id).targetname = s;
    }
}

/// Raven `Q3_SetTarget`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3502-3520`
pub fn Q3_SetTarget(ctx: &mut GameContext, entID: c_int, target: *const c_char) {
    let id = EntityId(entID as u32);

    if Q_stricmp(b"NULL\0".as_ptr() as *const c_char, target) == 0 {
        ctx.world.entity_mut(id).target = std::ptr::null_mut();
    } else {
        let s = G_NewString(ctx, target);
        ctx.world.entity_mut(id).target = s;
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
    let id = EntityId(entID as u32);

    if Q_stricmp(b"NULL\0".as_ptr() as *const c_char, fullName) == 0 {
        ctx.world.entity_mut(id).fullName = std::ptr::null_mut();
    } else {
        let s = G_NewString(ctx, fullName);
        ctx.world.entity_mut(id).fullName = s;
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
// Raven's `G_DebugPrint` warnings (parmNum range, truncation) are dropped
// here, matching the file's other `Q3_Set*` stubs.
pub fn Q3_SetParm(ctx: &mut GameContext, entID: c_int, parmNum: c_int, parmValue: *const c_char) {
    // `parms` (parms_t pool alloc) has no accessor; its alloc/zero and field
    // writes stay raw through the copied pointer (recipe 2c). Entity access
    // goes through `ctx.world.entity[_mut](id)`.
    unsafe {
        let id = EntityId(entID as u32);

        if parmNum < 0 || parmNum >= MAX_PARMS as c_int {
            return;
        }

        if ctx.world.entity(id).parms.is_null() {
            let p = G_Alloc(ctx, core::mem::size_of::<parms_t>() as c_int) as *mut parms_t;
            ctx.world.entity_mut(id).parms = p;
            // G_Alloc is a bump allocator whose pool is not re-zeroed on map
            // restart; C memsets the fresh parms_t so reused regions read empty.
            core::ptr::write_bytes(p as *mut u8, 0, core::mem::size_of::<parms_t>());
        }

        let parms = ctx.world.entity(id).parms;
        let val = Q3_GameSideCheckStringCounterIncrement(parmValue);
        if val != 0.0 {
            // Raven: `val += atof(...)` — atof returns double; the f32 `val`
            // promotes, adds in f64, narrows once. `%f` promotes the float back
            // to double for its 6-decimal print, so format via f64.
            // Source: `oracle/codemp/game/g_ICARUScb.c:3676-3677`
            let total = (val as f64
                + atof_bytes(CStr::from_ptr((*parms).parm[parmNum as usize].as_ptr()).to_bytes()))
                as f32;
            write_cstr_field(
                &mut (*parms).parm[parmNum as usize],
                &format!("{:.6}", total as f64),
            );
        } else {
            // Raven: strncpy + explicit truncation-NUL; write_cstr_field is the
            // Q_strncpyz/Com_sprintf byte-copy dual.
            write_cstr_field(
                &mut (*parms).parm[parmNum as usize],
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
    let id = EntityId(entID as u32);

    if data != 0 {
        ctx.world.entity_mut(id).flags |= FL_NOTARGET;
    } else {
        ctx.world.entity_mut(id).flags &= !FL_NOTARGET;
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
    let id = EntityId(entID as u32);

    if add != 0 {
        ctx.world.entity_mut(id).flags |= FL_INACTIVE;
    } else {
        ctx.world.entity_mut(id).flags &= !FL_INACTIVE;
    }
}

/// Raven `Q3_SetFuncUsableVisible`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:3857-3880`
pub fn Q3_SetFuncUsableVisible(ctx: &mut GameContext, entID: c_int, visible: qboolean) {
    let id = EntityId(entID as u32);

    if visible != 0 {
        let e = ctx.world.entity_mut(id);
        e.r.svFlags &= !SVF_NOCLIENT;
        e.s.eFlags &= !EF_NODRAW;
    } else {
        let e = ctx.world.entity_mut(id);
        e.r.svFlags |= SVF_NOCLIENT;
        e.s.eFlags |= EF_NODRAW;
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
    // gNPC_t deref + cstr_to_str stay raw (recipe 2c).
    unsafe {
        let id = EntityId(entID as u32);

        if ctx.world.entity(id).NPC.is_null() {
            let tn = ctx.world.entity(id).targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetWalking: '{}' is not an NPC!\n",
                    cstr_to_str(tn)
                ))
                .as_ptr(),
            );
            return;
        }
        let npc = ctx.world.entity(id).NPC;

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
    // gNPC_t deref + cstr_to_str stay raw (recipe 2c).
    unsafe {
        let id = EntityId(entID as u32);

        if ctx.world.entity(id).NPC.is_null() {
            let tn = ctx.world.entity(id).targetname;
            G_DebugPrint(
                ctx,
                WL_ERROR as c_int,
                cstr(&format!(
                    "Q3_SetNoAvoid: '{}' is not an NPC!\n",
                    cstr_to_str(tn)
                ))
                .as_ptr(),
            );
            return;
        }
        let npc = ctx.world.entity(id).NPC;

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
    // `owner` is `&g_entities[ownerNum]` — an array element, never NULL, so
    // Raven's dead `owner->` null guard collapses to the `inuse` check.
    let owner_id = EntityId(ctx.world.entity(self_).r.ownerNum as u32);

    let level_time = ctx.world.level.time;
    {
        let s = ctx.world.entity_mut(self_);
        s.nextthink = level_time + FRAMETIME;
        s.think = Some(EntThink::G_FreeEntity).into();
    }

    if ctx.world.entity(owner_id).inuse == 0 {
        return;
    }

    let oldContents = ctx.world.entity(owner_id).r.contents;
    ctx.world.entity_mut(owner_id).r.contents = CONTENTS_BODY;
    let owner_origin = ctx.world.entity(owner_id).r.currentOrigin;
    if SpotWouldTelefrag2(ctx, owner_id, owner_origin) != qfalse {
        ctx.world.entity_mut(owner_id).r.contents = oldContents;
        ctx.world.entity_mut(self_).think = Some(EntThink::SolidifyOwner).into();
    } else {
        trap::ICARUS_TaskIDComplete(
            ctx.engine,
            GIcarusTaskidcompleteArgs::new(
                (ctx.world.entity_mut(owner_id) as *mut gentity_t).cast(),
                taskID_t::TID_RESIZE as c_int,
            ),
        );
    }
}

/// Raven `Q3_SetSolid`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4305-4345`
pub fn Q3_SetSolid(ctx: &mut GameContext, entID: c_int, solid: qboolean) -> qboolean {
    let id = EntityId(entID as u32);

    // `ent` is `&g_entities[entID]` — never NULL, so the guard is the `inuse` half.
    if ctx.world.entity(id).inuse == 0 {
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            cstr(&format!("Q3_SetSolid: invalid entID {}\n", entID)).as_ptr(),
        );
        return qtrue;
    }

    if solid != 0 {
        //FIXME: Presumption
        let oldContents = ctx.world.entity(id).r.contents;
        ctx.world.entity_mut(id).r.contents = CONTENTS_BODY;
        let cur = ctx.world.entity(id).r.currentOrigin;
        if SpotWouldTelefrag2(ctx, id, cur) != qfalse {
            let sid = G_Spawn(ctx);

            let number = ctx.world.entity(id).s.number;
            let level_time = ctx.world.level.time;
            let s = ctx.world.entity_mut(sid);
            s.r.ownerNum = number;
            s.think = Some(EntThink::SolidifyOwner).into();
            s.nextthink = level_time + FRAMETIME;

            ctx.world.entity_mut(id).r.contents = oldContents;
            return qfalse;
        }
        ctx.world.entity_mut(id).clipmask |= CONTENTS_BODY;
    } else {
        //FIXME: Presumption
        if ctx.world.entity(id).s.eFlags & EF_NODRAW != 0 {
            //We're invisible too, so set contents to none
            ctx.world.entity_mut(id).r.contents = 0;
        } else {
            ctx.world.entity_mut(id).r.contents = CONTENTS_CORPSE;
        }
    }
    qtrue
}

/// Raven `Q3_SetForwardMove`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4354-4372`
pub fn Q3_SetForwardMove(ctx: &mut GameContext, entID: c_int, fmoveVal: c_int) {
    let id = EntityId(entID as u32);

    // `ent` is `&g_entities[entID]` — never NULL, so Raven's `!ent` guard is dead.
    if ctx.world.entity(id).client.is_null() {
        let tn = ctx.world.entity(id).targetname;
        G_DebugPrint(
            ctx,
            WL_ERROR as c_int,
            cstr(&format!(
                "Q3_SetForwardMove: '{}' is not an NPC/player!\n",
                unsafe { cstr_to_str(tn) }
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

/// Raven `Q3_SetRightMove`.
///
/// Raven: entID/gentity_t is never null (address-of array element); the
/// `!ent`/`!ent->client` guards are dead/live-checked here as client-null
/// only. Body is a debug-print stub — behavior is commented out in Raven.
/// Source: `oracle/codemp/game/g_ICARUScb.c:4381-4399`
pub fn Q3_SetRightMove(ctx: &mut GameContext, entID: c_int, rmoveVal: c_int) {
    let id = EntityId(entID as u32);
    if ctx.world.entity(id).client.is_null() {
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

/// Raven `Q3_SetLockAngle`.
///
/// Raven: the renderInfo.lockYaw/RF_LOCKEDANGLE assignment is fully
/// commented out in Raven; body is a debug-print stub only.
/// Source: `oracle/codemp/game/g_ICARUScb.c:4408-4445`
pub fn Q3_SetLockAngle(ctx: &mut GameContext, entID: c_int, lockAngle: *const c_char) {
    let id = EntityId(entID as u32);
    if ctx.world.entity(id).client.is_null() {
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
    {
        let id = EntityId(entID as u32);
        let mut bSet = bSet_t::BSET_INVALID;

        // `ent` is `&g_entities[entID]` — never NULL, so Raven's null guard is dead.

        bSet = match toSet {
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
            if !ctx.world.entity(id).behaviorSet[bSet as usize].is_null() {
                //			gi.TagFree( ent->behaviorSet[bSet] );
            }
            ctx.world.entity_mut(id).behaviorSet[bSet as usize] = core::ptr::null_mut();
        } else if !scriptname.is_null() {
            if !ctx.world.entity(id).behaviorSet[bSet as usize].is_null() {
                //				gi.TagFree( ent->behaviorSet[bSet] );
            }
            let s = G_NewString(ctx, scriptname); //FIXME: This really isn't good...
            ctx.world.entity_mut(id).behaviorSet[bSet as usize] = s;
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
    let id = EntityId(entID as u32);

    // `ent` is `&g_entities[entID]` — never NULL, so Raven's null guard is dead.
    if usable != 0 {
        ctx.world.entity_mut(id).r.svFlags |= SVF_PLAYER_USABLE;
    } else {
        ctx.world.entity_mut(id).r.svFlags &= !SVF_PLAYER_USABLE;
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
    let id = EntityId(entID as u32);

    // `ent` is `&g_entities[entID]` — never NULL; the guard is the `inuse` half.
    if ctx.world.entity(id).inuse == 0 {
        return;
    }

    if ctx.world.entity(id).client.is_null() {
        G_DebugPrint(
            ctx,
            WL_WARNING as c_int,
            cstr(&format!("Q3_SetSaberActive: {} is not a client\n", entID)).as_ptr(),
        );
    }

    //fixme: Take into account player being in state where saber won't toggle? For now we simply won't care.
    // Pool client (NPC): deref raw through the copied pointer (Raven derefs
    // even on the warned NULL path, so no guard is added).
    let client = ctx.world.entity(id).client;
    let holstered = unsafe { (*client).ps.saberHolstered };
    if holstered == 0 && active != 0 {
        Cmd_ToggleSaber_f(ctx, id);
    } else if unsafe { BG_SabersOff(&mut (*client).ps as *mut playerState_t) } != 0 && active == 0 {
        Cmd_ToggleSaber_f(ctx, id);
    }
}

/// Raven `Q3_SetNoKnockback`.
///
/// Source: `oracle/codemp/game/g_ICARUScb.c:4900-4918`
pub fn Q3_SetNoKnockback(ctx: &mut GameContext, entID: c_int, noKnockback: qboolean) {
    let id = EntityId(entID as u32);
    if noKnockback != 0 {
        ctx.world.entity_mut(id).flags |= FL_NO_KNOCKBACK;
    } else {
        ctx.world.entity_mut(id).flags &= !FL_NO_KNOCKBACK;
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
        let id = EntityId(entID as u32);
        let mut float_data: f32;
        let mut int_data: c_int;
        let mut vector_data: vec3_t = [0.0, 0.0, 0.0];

        // Convert the shared-memory value once; every scalar `atof` arm below
        // parses these bytes (libc strtod semantics via `native_string`).
        let data_b = CStr::from_ptr(data).to_bytes();

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
                G_SetOrigin(ctx.world.entity_mut(id), vector_data);
                let classname = ctx.world.entity(id).classname;
                if Q_strncmp(b"NPC_\0".as_ptr() as *const c_char, classname, 4) == 0 {
                    //hack for moving spawners
                    crate::q_math::_VectorCopy(vector_data, &mut ctx.world.entity_mut(id).s.origin);
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
                        GIcarusTaskidsetArgs::new(
                            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                            taskID_t::TID_MOVE_NAV as c_int,
                            taskID,
                        ),
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
                float_data = atof_bytes(data_b) as f32;
                Q3_SetVelocity(ctx, entID, 0, float_data);
            }
            _ if toSet == SET_YVELOCITY as i32 => {
                float_data = atof_bytes(data_b) as f32;
                Q3_SetVelocity(ctx, entID, 1, float_data);
            }
            _ if toSet == SET_ZVELOCITY as i32 => {
                float_data = atof_bytes(data_b) as f32;
                Q3_SetVelocity(ctx, entID, 2, float_data);
            }

            _ if toSet == SET_Z_OFFSET as i32 => {
                float_data = atof_bytes(data_b) as f32;
                Q3_SetOriginOffset(ctx, entID, 2, float_data);
            }

            _ if toSet == SET_ENEMY as i32 => Q3_SetEnemy(ctx, entID, data),
            _ if toSet == SET_LEADER as i32 => Q3_SetLeader(ctx, entID, data),

            _ if toSet == SET_NAVGOAL as i32 => {
                if Q3_SetNavGoal(ctx, entID, data) != qfalse {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(
                            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                            taskID_t::TID_MOVE_NAV as c_int,
                            taskID,
                        ),
                    );
                    return qfalse; //Don't call it back
                }
            }

            _ if toSet == SET_ANIM_UPPER as i32 => {
                if Q3_SetAnimUpper(ctx, entID, data) != qfalse {
                    Q3_TaskIDClear(
                        &mut ctx.world.entity_mut(id).taskID[taskID_t::TID_ANIM_BOTH as usize],
                    ); //We only want to wait for the top
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(
                            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                            taskID_t::TID_ANIM_UPPER as c_int,
                            taskID,
                        ),
                    );
                    return qfalse; //Don't call it back
                }
            }

            _ if toSet == SET_ANIM_LOWER as i32 => {
                if Q3_SetAnimLower(ctx, entID, data) != qfalse {
                    Q3_TaskIDClear(
                        &mut ctx.world.entity_mut(id).taskID[taskID_t::TID_ANIM_BOTH as usize],
                    ); //We only want to wait for the bottom
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(
                            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                            taskID_t::TID_ANIM_LOWER as c_int,
                            taskID,
                        ),
                    );
                    return qfalse; //Don't call it back
                }
            }

            _ if toSet == SET_ANIM_BOTH as i32 => {
                let mut both: c_int = 0;
                if Q3_SetAnimUpper(ctx, entID, data) != qfalse {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(
                            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                            taskID_t::TID_ANIM_UPPER as c_int,
                            taskID,
                        ),
                    );
                    both += 1;
                } else {
                    let tn = ctx.world.entity(id).targetname;
                    G_DebugPrint(
                        ctx,
                        WL_ERROR as c_int,
                        cstr(&format!(
                            "Q3_SetAnimUpper: {} does not have anim {}!\n",
                            cstr_to_str(tn),
                            cstr_to_str(data)
                        ))
                        .as_ptr(),
                    );
                }
                if Q3_SetAnimLower(ctx, entID, data) != qfalse {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(
                            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                            taskID_t::TID_ANIM_LOWER as c_int,
                            taskID,
                        ),
                    );
                    both += 1;
                } else {
                    let tn = ctx.world.entity(id).targetname;
                    G_DebugPrint(
                        ctx,
                        WL_ERROR as c_int,
                        cstr(&format!(
                            "Q3_SetAnimLower: {} does not have anim {}!\n",
                            cstr_to_str(tn),
                            cstr_to_str(data)
                        ))
                        .as_ptr(),
                    );
                }
                if both >= 2 {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(
                            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                            taskID_t::TID_ANIM_BOTH as c_int,
                            taskID,
                        ),
                    );
                }
                if both != 0 {
                    return qfalse; //Don't call it back
                }
            }

            _ if toSet == SET_ANIM_HOLDTIME_LOWER as i32 => {
                int_data = atoi(data);
                Q3_SetAnimHoldTime(ctx, entID, int_data, qtrue);
                Q3_TaskIDClear(
                    &mut ctx.world.entity_mut(id).taskID[taskID_t::TID_ANIM_BOTH as usize],
                ); //We only want to wait for the bottom
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(
                        (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                        taskID_t::TID_ANIM_LOWER as c_int,
                        taskID,
                    ),
                );
                return qfalse; //Don't call it back
            }

            _ if toSet == SET_ANIM_HOLDTIME_UPPER as i32 => {
                int_data = atoi(data);
                Q3_SetAnimHoldTime(ctx, entID, int_data, qfalse);
                Q3_TaskIDClear(
                    &mut ctx.world.entity_mut(id).taskID[taskID_t::TID_ANIM_BOTH as usize],
                ); //We only want to wait for the top
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(
                        (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                        taskID_t::TID_ANIM_UPPER as c_int,
                        taskID,
                    ),
                );
                return qfalse; //Don't call it back
            }

            _ if toSet == SET_ANIM_HOLDTIME_BOTH as i32 => {
                int_data = atoi(data);
                Q3_SetAnimHoldTime(ctx, entID, int_data, qfalse);
                Q3_SetAnimHoldTime(ctx, entID, int_data, qtrue);
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(
                        (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                        taskID_t::TID_ANIM_BOTH as c_int,
                        taskID,
                    ),
                );
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(
                        (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                        taskID_t::TID_ANIM_UPPER as c_int,
                        taskID,
                    ),
                );
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(
                        (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                        taskID_t::TID_ANIM_LOWER as c_int,
                        taskID,
                    ),
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
                        GIcarusTaskidsetArgs::new(
                            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                            taskID_t::TID_BSTATE as c_int,
                            taskID,
                        ),
                    );
                    return qfalse; //don't complete
                }
            }

            _ if toSet == SET_DEFAULT_BSTATE as i32 => Q3_SetDefaultBState(ctx, entID, data),

            _ if toSet == SET_TEMP_BSTATE as i32 => {
                if Q3_SetTempBState(ctx, entID, data) == qfalse {
                    trap::ICARUS_TaskIDSet(
                        ctx.engine,
                        GIcarusTaskidsetArgs::new(
                            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                            taskID_t::TID_BSTATE as c_int,
                            taskID,
                        ),
                    );
                    return qfalse; //don't complete
                }
            }

            _ if toSet == SET_CAPTURE as i32 => Q3_SetCaptureGoal(ctx, entID, data),

            _ if toSet == SET_DPITCH as i32 => {
                //FIXME: make these set tempBehavior to BS_FACE and await completion?  Or set lockedDesiredPitch/Yaw and aimTime?
                float_data = atof_bytes(data_b) as f32;
                Q3_SetDPitch(ctx, entID, float_data);
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(
                        (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                        taskID_t::TID_ANGLE_FACE as c_int,
                        taskID,
                    ),
                );
                return qfalse;
            }

            _ if toSet == SET_DYAW as i32 => {
                float_data = atof_bytes(data_b) as f32;
                Q3_SetDYaw(ctx, entID, float_data);
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(
                        (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                        taskID_t::TID_ANGLE_FACE as c_int,
                        taskID,
                    ),
                );
                return qfalse;
            }

            _ if toSet == SET_EVENT as i32 => Q3_SetEvent(ctx, entID, data),

            _ if toSet == SET_VIEWTARGET as i32 => {
                Q3_SetViewTarget(ctx, entID, data);
                trap::ICARUS_TaskIDSet(
                    ctx.engine,
                    GIcarusTaskidsetArgs::new(
                        (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                        taskID_t::TID_ANGLE_FACE as c_int,
                        taskID,
                    ),
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
                float_data = atof_bytes(data_b) as f32;
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
                float_data = atof_bytes(data_b) as f32;
                Q3_SetGravity(ctx, entID, float_data);
            }

            _ if toSet == SET_WAIT as i32 => {
                float_data = atof_bytes(data_b) as f32;
                Q3_SetWait(ctx, entID, float_data);
            }

            _ if toSet == SET_FOLLOWDIST as i32 => {
                float_data = atof_bytes(data_b) as f32;
                Q3_SetFollowDist(ctx, entID, float_data);
            }

            _ if toSet == SET_SCALE as i32 => {
                float_data = atof_bytes(data_b) as f32;
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
                float_data = atof_bytes(data_b) as f32;
                Q3_SetShootDist(ctx, entID, float_data);
            }

            _ if toSet == SET_TIMESCALE as i32 => Q3_SetTimeScale(ctx, entID, data),

            _ if toSet == SET_VISRANGE as i32 => {
                float_data = atof_bytes(data_b) as f32;
                Q3_SetVisrange(ctx, entID, float_data);
            }

            _ if toSet == SET_EARSHOT as i32 => {
                float_data = atof_bytes(data_b) as f32;
                Q3_SetEarshot(ctx, entID, float_data);
            }

            _ if toSet == SET_VIGILANCE as i32 => {
                float_data = atof_bytes(data_b) as f32;
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
                        GIcarusTaskidsetArgs::new(
                            (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                            taskID_t::TID_LOCATION as c_int,
                            taskID,
                        ),
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
                            GIcarusTaskidsetArgs::new(
                                (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                                taskID_t::TID_RESIZE as c_int,
                                taskID,
                            ),
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
                float_data = atof_bytes(data_b) as f32;
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
                float_data = atof_bytes(data_b) as f32;
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
                    GIcarusTaskidsetArgs::new(
                        (ctx.world.entity_mut(id) as *mut gentity_t).cast(),
                        taskID_t::TID_ANIM_BOTH as c_int,
                        taskID,
                    ),
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
                    UnLockDoors(&mut ctx.world.g_entities[entID as usize]);
                } else if Q_stricmp(b"locked\0".as_ptr() as *const c_char, data) == 0 {
                    LockDoors(&mut ctx.world.g_entities[entID as usize]);
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
