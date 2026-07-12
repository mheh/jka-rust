//! SP `pmove_t` player-move in/out struct copied from Raven `code/game/bg_public.h`.
//!
//! Type definition source: `oracle/code/game/bg_public.h:130-163`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::common::sp::ghoul2::eg2_collision::EG2_Collision;
use sp_qshared::common::sp::qcommon::{playerState_t, usercmd_t};
use sp_qshared::common::sp::trace_t::trace_t;
use sp_qshared::shared::{qboolean, vec3_t};

/// Raven `MAXTOUCH` — max number of entities `touchents` can hold.
///
/// Source: `oracle/code/game/bg_public.h:129`
pub const MAXTOUCH: usize = 32;

/// Raven `pmove_t` — player-move in/out parameter block passed to `Pmove`.
///
/// Type definition source: `oracle/code/game/bg_public.h:130-163`
#[repr(C)]
#[derive(Debug)]
pub struct pmove_t {
    /// state (in / out)
    /// Raven field source: `oracle/code/game/bg_public.h:132`
    pub ps: *mut playerState_t,

    /// command (in)
    /// Raven field source: `oracle/code/game/bg_public.h:135`
    pub cmd: usercmd_t,
    /// collide against these types of surfaces
    /// Raven field source: `oracle/code/game/bg_public.h:136`
    pub tracemask: c_int,
    /// if set, diagnostic output will be printed
    /// Raven field source: `oracle/code/game/bg_public.h:137`
    pub debugLevel: c_int,
    /// if the game is setup for no footsteps by the server
    /// Raven field source: `oracle/code/game/bg_public.h:138`
    pub noFootsteps: qboolean,

    /// results (out)
    /// Raven field source: `oracle/code/game/bg_public.h:141`
    pub numtouch: c_int,
    /// Raven field source: `oracle/code/game/bg_public.h:142`
    pub touchents: [c_int; MAXTOUCH],

    /// Raven field source: `oracle/code/game/bg_public.h:144`
    pub useEvent: c_int,

    /// bounding box size
    /// Raven field source: `oracle/code/game/bg_public.h:146`
    pub mins: vec3_t,
    /// Raven field source: `oracle/code/game/bg_public.h:146`
    pub maxs: vec3_t,

    /// Raven field source: `oracle/code/game/bg_public.h:148`
    pub watertype: c_int,
    /// Raven field source: `oracle/code/game/bg_public.h:149`
    pub waterlevel: c_int,

    /// Raven field source: `oracle/code/game/bg_public.h:151`
    pub xyspeed: f32,
    /// Pointer to entity in g_entities[]
    /// Raven field source: `oracle/code/game/bg_public.h:152`
    pub gent: *mut gentity_t,

    /// Raven: callbacks to test the world; these will be different functions
    /// during game and cgame.
    /// Raven field source: `oracle/code/game/bg_public.h:159-160`
    pub trace: Option<
        unsafe extern "C" fn(
            results: *mut trace_t,
            start: *const vec3_t,
            mins: *const vec3_t,
            maxs: *const vec3_t,
            end: *const vec3_t,
            passEntityNum: c_int,
            contentMask: c_int,
            eG2TraceType: EG2_Collision,
            useLod: c_int,
        ),
    >,
    /// Raven field source: `oracle/code/game/bg_public.h:162`
    pub pointcontents:
        Option<unsafe extern "C" fn(point: *const vec3_t, passEntityNum: c_int) -> c_int>,
}

const _: () = assert!(core::mem::size_of::<pmove_t>() == 248);
const _: () = assert!(core::mem::offset_of!(pmove_t, ps) == 0);
const _: () = assert!(core::mem::offset_of!(pmove_t, cmd) == 8);
const _: () = assert!(core::mem::offset_of!(pmove_t, tracemask) == 36);
const _: () = assert!(core::mem::offset_of!(pmove_t, debugLevel) == 40);
const _: () = assert!(core::mem::offset_of!(pmove_t, noFootsteps) == 44);
const _: () = assert!(core::mem::offset_of!(pmove_t, numtouch) == 48);
const _: () = assert!(core::mem::offset_of!(pmove_t, touchents) == 52);
const _: () = assert!(core::mem::offset_of!(pmove_t, useEvent) == 180);
const _: () = assert!(core::mem::offset_of!(pmove_t, mins) == 184);
const _: () = assert!(core::mem::offset_of!(pmove_t, maxs) == 196);
const _: () = assert!(core::mem::offset_of!(pmove_t, watertype) == 208);
const _: () = assert!(core::mem::offset_of!(pmove_t, waterlevel) == 212);
const _: () = assert!(core::mem::offset_of!(pmove_t, xyspeed) == 216);
const _: () = assert!(core::mem::offset_of!(pmove_t, gent) == 224);
const _: () = assert!(core::mem::offset_of!(pmove_t, trace) == 232);
const _: () = assert!(core::mem::offset_of!(pmove_t, pointcontents) == 240);
