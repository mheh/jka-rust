//! Port of `oracle/codemp/game/g_timer.c`.
//!
//! The threaded `GameContext`/`GameWorld` handle carries the file-scope timer pool and lists
//! (`g_timerPool`, `g_timers`, `g_timerFreeList`) and `level`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::q_shared::Q_stricmp;
use core::ffi::c_char;
use mp_qshared::shared::MAX_GENTITIES;

// Raven `qboolean` is `c_int`.
// Keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

/// Raven `gtimer_t` - a single timer with a name, an expiration time, and a link to the next timer.
///
/// Type definition source: `oracle/codemp/game/g_timer.c:10-15`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gtimer_t {
    pub name: *const c_char,
    pub time: c_int,
    pub next: *mut gtimer_t,
}

// This is `#[repr(C)]` POD: raw pointers are null-valid, and `c_int` is POD too.
// The all-zero image is a valid inhabitant, because Raven's `g_timerPool` starts zeroed.
// The pool is about 384 KB, so it builds heap-first through `zeroed_box`.
unsafe impl native_platform::ZeroValid for gtimer_t {}

/// Raven `MAX_GTIMERS`, the maximum number of timers in the pool.
/// Source: `oracle/codemp/game/g_timer.c:8`
pub const MAX_GTIMERS: usize = 16384;

// Raven's file-scope globals `g_timerPool`, `g_timers`, and `g_timerFreeList` are now
// `GameGlobals` fields, reached through `ctx.world.globals` at every call site below.
// Source: `oracle/codemp/game/g_timer.c:17-19`

/// Raven `TIMER_Clear`.
///
/// Source: `oracle/codemp/game/g_timer.c:27-41`
pub fn TIMER_Clear(ctx: &mut GameContext) {
    for i in 0..MAX_GENTITIES {
        ctx.world.globals.g_timers.0[i as usize] = core::ptr::null_mut();
    }

    for i in 0..MAX_GTIMERS - 1 {
        let next = &mut ctx.world.globals.g_timerPool.0[i + 1] as *mut gtimer_t;
        ctx.world.globals.g_timerPool.0[i].next = next;
    }
    ctx.world.globals.g_timerPool.0[MAX_GTIMERS - 1].next = core::ptr::null_mut();
    ctx.world.globals.g_timerFreeList = &mut ctx.world.globals.g_timerPool.0[0];
}

/// Raven `TIMER_Clear2`.
///
/// Source: `oracle/codemp/game/g_timer.c:49-74`
pub fn TIMER_Clear2(ctx: &mut GameContext, ent: Option<EntityId>) {
    // rudimentary safety checks, might be other things to check?
    // Raven checks `ent`, `ent->s.number > 0`, and the upper bound together in one `if`.
    // The port splits this into an early return on `None`, then a separate bounds check.
    // The entity read is an accessor borrow.
    // The `gtimer_t` free-list walk below stays raw pointers, because it walks the intrusive pool,
    // not an entity or client.
    let Some(ent) = ent else {
        return;
    };
    let entity_num = ctx.entity(ent).s.number as usize;

    if entity_num == 0 || entity_num >= MAX_GENTITIES as usize {
        return;
    }

    unsafe {
        let mut p = ctx.world.globals.g_timers.0[entity_num];

        // No timers at all -> do nothing
        if p.is_null() {
            return;
        }

        // Find the end of this ents timer list
        while !(*p).next.is_null() {
            p = (*p).next;
        }

        // Splice the lists
        (*p).next = ctx.world.globals.g_timerFreeList;
        ctx.world.globals.g_timerFreeList = ctx.world.globals.g_timers.0[entity_num];
        ctx.world.globals.g_timers.0[entity_num] = core::ptr::null_mut();
    }
}

/// Raven `TIMER_GetNew`.
///
/// Source: `oracle/codemp/game/g_timer.c:79-103`
// This returns Raven's `gtimer_t*` type directly, instead of an erased `*mut c_void` handle.
pub fn TIMER_GetNew(ctx: &mut GameContext, num: c_int, identifier: *const c_char) -> *mut gtimer_t {
    unsafe {
        let num_usize = num as usize;
        let mut p = ctx.world.globals.g_timers.0[num_usize];

        // Search for an existing timer with this name
        while !p.is_null() {
            if Q_stricmp((*p).name, identifier) == 0 {
                // Found it
                return p;
            }

            p = (*p).next;
        }

        // No existing timer with this name was found, so grab one from the free list
        if ctx.world.globals.g_timerFreeList.is_null() {
            return core::ptr::null_mut();
        }

        p = ctx.world.globals.g_timerFreeList;
        ctx.world.globals.g_timerFreeList = (*ctx.world.globals.g_timerFreeList).next;
        (*p).next = ctx.world.globals.g_timers.0[num_usize];
        ctx.world.globals.g_timers.0[num_usize] = p;
        p
    }
}

/// Raven `TIMER_GetExisting`.
///
/// Source: `oracle/codemp/game/g_timer.c:106-121`
// This returns Raven's `gtimer_t*` type directly, instead of an erased `*mut c_void` handle.
pub fn TIMER_GetExisting(
    ctx: &mut GameContext,
    num: c_int,
    identifier: *const c_char,
) -> *mut gtimer_t {
    unsafe {
        let num_usize = num as usize;
        let mut p = ctx.world.globals.g_timers.0[num_usize];

        while !p.is_null() {
            if Q_stricmp((*p).name, identifier) == 0 {
                // Found it
                return p;
            }

            p = (*p).next;
        }

        core::ptr::null_mut()
    }
}

/// Raven `TIMER_Set`.
///
/// Source: `oracle/codemp/game/g_timer.c:129-139`
pub fn TIMER_Set(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    identifier: *const c_char,
    duration: c_int,
) {
    // §19: oracle derefs `ent->s.number` unconditionally, which is UB when `ent` is null.
    // The port guards and returns early instead. Source: `g_timer.c:131`.
    // The entity read is an accessor borrow, and the `gtimer_t` pool write stays raw pointers.
    let Some(ent) = ent else {
        return;
    };
    let number = ctx.entity(ent).s.number;
    let timer = TIMER_GetNew(ctx, number, identifier);

    if timer.is_null() {
        return;
    }

    unsafe {
        (*timer).name = identifier;
        (*timer).time = ctx.world.level.time + duration;
    }
}

/// Raven `TIMER_Get`.
///
/// Source: `oracle/codemp/game/g_timer.c:147-157`
pub fn TIMER_Get(ctx: &mut GameContext, ent: Option<EntityId>, identifier: *const c_char) -> c_int {
    // §19: oracle derefs `ent->s.number` unconditionally, which is UB when `ent` is null.
    // The port guards and returns early instead. Source: `g_timer.c:149`.
    // The entity read is an accessor borrow, and the `gtimer_t` pool read stays raw pointers.
    let Some(ent) = ent else {
        return -1;
    };
    let number = ctx.entity(ent).s.number;
    let timer = TIMER_GetExisting(ctx, number, identifier);

    if timer.is_null() {
        return -1;
    }

    unsafe { (*timer).time }
}

/// Raven `TIMER_Done`.
///
/// Source: `oracle/codemp/game/g_timer.c:165-175`
pub fn TIMER_Done(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    identifier: *const c_char,
) -> qboolean {
    // §19: oracle derefs `ent->s.number` unconditionally, which is UB when `ent` is null.
    // The port guards and returns early instead. Source: `g_timer.c:167`.
    // The entity read is an accessor borrow, and the `gtimer_t` pool read stays raw pointers.
    let Some(ent) = ent else {
        return qtrue;
    };
    let number = ctx.entity(ent).s.number;
    let timer = TIMER_GetExisting(ctx, number, identifier);

    if timer.is_null() {
        return qtrue;
    }

    if unsafe { (*timer).time } < ctx.world.level.time {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `TIMER_RemoveHelper`.
///
/// Scans an entities timer list to remove a given
/// timer from the list and put it on the free list
///
/// Source: `oracle/codemp/game/g_timer.c:187-211`
// This types `timer` as Raven's `gtimer_t*`, instead of an erased `*mut c_void` handle.
pub fn TIMER_RemoveHelper(ctx: &mut GameContext, num: c_int, timer: *mut gtimer_t) {
    unsafe {
        let num_usize = num as usize;

        let mut p = ctx.world.globals.g_timers.0[num_usize];

        // Special case: first timer in list
        if p == timer {
            ctx.world.globals.g_timers.0[num_usize] =
                (*ctx.world.globals.g_timers.0[num_usize]).next;
            (*timer).next = ctx.world.globals.g_timerFreeList;
            ctx.world.globals.g_timerFreeList = timer;
            return;
        }

        // Find the predecessor
        while !p.is_null() && (*p).next != timer {
            p = (*p).next;
        }

        // Raven's predecessor walk has no bounds check and dereferences null if `timer` is not linked
        // under this entity. The port returns instead.
        if p.is_null() {
            return; // Timer not found, shouldn't happen
        }

        // Rewire
        (*p).next = (*timer).next;
        (*timer).next = ctx.world.globals.g_timerFreeList;
        ctx.world.globals.g_timerFreeList = timer;
    }
}

/// Raven `TIMER_Done2`.
///
/// Returns false if timer has been started but is not done, or if timer was never started.
/// Returns true if timer is done (optionally removing it from the list).
///
/// Source: `oracle/codemp/game/g_timer.c:223-242`
pub fn TIMER_Done2(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    identifier: *const c_char,
    remove: qboolean,
) -> qboolean {
    // §19: oracle derefs `ent->s.number` unconditionally, which is UB when `ent` is null.
    // The port guards and returns early instead. Source: `g_timer.c:225`.
    // The entity read is an accessor borrow, and the `gtimer_t` pool read stays raw pointers.
    let Some(ent) = ent else {
        return qfalse;
    };
    let number = ctx.entity(ent).s.number;
    let timer = TIMER_GetExisting(ctx, number, identifier);

    if timer.is_null() {
        return qfalse;
    }

    let res = unsafe { (*timer).time } < ctx.world.level.time;

    if res && remove == qtrue {
        // Put it back on the free list
        TIMER_RemoveHelper(ctx, number, timer);
    }

    if res {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `TIMER_Exists`.
///
/// Source: `oracle/codemp/game/g_timer.c:249-259`
pub fn TIMER_Exists(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    identifier: *const c_char,
) -> qboolean {
    // §19: oracle derefs `ent->s.number` unconditionally, which is UB when `ent` is null.
    // The port guards and returns early instead. Source: `g_timer.c:251`.
    // The entity read is an accessor borrow.
    let Some(ent) = ent else {
        return qfalse;
    };
    let number = ctx.entity(ent).s.number;
    let timer = TIMER_GetExisting(ctx, number, identifier);

    if timer.is_null() {
        qfalse
    } else {
        qtrue
    }
}

/// Raven `TIMER_Remove`.
///
/// Utility to get rid of any timer.
///
/// Source: `oracle/codemp/game/g_timer.c:267-278`
pub fn TIMER_Remove(ctx: &mut GameContext, ent: Option<EntityId>, identifier: *const c_char) {
    // §19: oracle derefs `ent->s.number` unconditionally, which is UB when `ent` is null.
    // The port guards and returns early instead. Source: `g_timer.c:269`.
    // The entity read is an accessor borrow.
    let Some(ent) = ent else {
        return;
    };
    let number = ctx.entity(ent).s.number;
    let timer = TIMER_GetExisting(ctx, number, identifier);

    if timer.is_null() {
        return;
    }

    // Put it back on the free list
    TIMER_RemoveHelper(ctx, number, timer);
}

/// Raven `TIMER_Start`.
///
/// Source: `oracle/codemp/game/g_timer.c:286-294`
pub fn TIMER_Start(
    ctx: &mut GameContext,
    self_: Option<EntityId>,
    identifier: *const c_char,
    duration: c_int,
) -> qboolean {
    if TIMER_Done(ctx, self_, identifier) == qtrue {
        TIMER_Set(ctx, self_, identifier, duration);
        qtrue
    } else {
        qfalse
    }
}
