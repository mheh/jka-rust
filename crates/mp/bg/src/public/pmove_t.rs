//! MP `pmove_t` player-move in/out struct copied from Raven `codemp/game/bg_public.h`.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:435-492`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int, c_void};

use mp_qshared::common::mp::qcommon::{playerState_t, usercmd_t};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::{qboolean, vec3_t};

use crate::public::animation::animation_t;
use crate::public::bg_entity::bgEntity_t;

/// Raven `MAXTOUCH` — max number of entities `touchents` can hold.
///
/// Source: `oracle/codemp/game/bg_public.h:421`
pub const MAXTOUCH: usize = 32;

/// Raven `pmove_t` — player-move in/out parameter block passed to `Pmove`.
///
/// Type definition source: `oracle/codemp/game/bg_public.h:435-492`
#[repr(C)]
#[derive(Debug)]
pub struct pmove_t {
    /// state (in / out)
    /// Raven field source: `oracle/codemp/game/bg_public.h:437`
    pub ps: *mut playerState_t,

    /// Raven: rww - shared ghoul2 stuff (not actually the same data, but hey)
    /// Raven field source: `oracle/codemp/game/bg_public.h:440`
    pub ghoul2: *mut c_void,
    /// Raven field source: `oracle/codemp/game/bg_public.h:441`
    pub g2Bolts_LFoot: c_int,
    /// Raven field source: `oracle/codemp/game/bg_public.h:442`
    pub g2Bolts_RFoot: c_int,
    /// Raven field source: `oracle/codemp/game/bg_public.h:443`
    pub modelScale: vec3_t,

    /// Raven: hacky bool so we know if we're dealing with a nonhumanoid (which
    /// is probably a rockettrooper).
    /// Raven field source: `oracle/codemp/game/bg_public.h:446`
    pub nonHumanoid: qboolean,

    /// command (in)
    /// Raven field source: `oracle/codemp/game/bg_public.h:449`
    pub cmd: usercmd_t,
    /// collide against these types of surfaces
    /// Raven field source: `oracle/codemp/game/bg_public.h:450`
    pub tracemask: c_int,
    /// if set, diagnostic output will be printed
    /// Raven field source: `oracle/codemp/game/bg_public.h:451`
    pub debugLevel: c_int,
    /// if the game is setup for no footsteps by the server
    /// Raven field source: `oracle/codemp/game/bg_public.h:452`
    pub noFootsteps: qboolean,
    /// true if a gauntlet attack would actually hit something
    /// Raven field source: `oracle/codemp/game/bg_public.h:453`
    pub gauntletHit: qboolean,

    /// Raven field source: `oracle/codemp/game/bg_public.h:455`
    pub framecount: c_int,

    /// results (out)
    /// Raven field source: `oracle/codemp/game/bg_public.h:458`
    pub numtouch: c_int,
    /// Raven field source: `oracle/codemp/game/bg_public.h:459`
    pub touchents: [c_int; MAXTOUCH],

    /// Raven field source: `oracle/codemp/game/bg_public.h:461`
    pub useEvent: c_int,

    /// bounding box size
    /// Raven field source: `oracle/codemp/game/bg_public.h:463`
    pub mins: vec3_t,
    /// Raven field source: `oracle/codemp/game/bg_public.h:463`
    pub maxs: vec3_t,

    /// Raven field source: `oracle/codemp/game/bg_public.h:465`
    pub watertype: c_int,
    /// Raven field source: `oracle/codemp/game/bg_public.h:466`
    pub waterlevel: c_int,

    /// Raven field source: `oracle/codemp/game/bg_public.h:468`
    pub gametype: c_int,

    /// Raven field source: `oracle/codemp/game/bg_public.h:470`
    pub debugMelee: c_int,
    /// Raven field source: `oracle/codemp/game/bg_public.h:471`
    pub stepSlideFix: c_int,
    /// Raven field source: `oracle/codemp/game/bg_public.h:472`
    pub noSpecMove: c_int,

    /// Raven field source: `oracle/codemp/game/bg_public.h:474`
    pub animations: *mut animation_t,

    /// Raven field source: `oracle/codemp/game/bg_public.h:476`
    pub xyspeed: c_float,

    /// for fixed msec Pmove
    /// Raven field source: `oracle/codemp/game/bg_public.h:479`
    pub pmove_fixed: c_int,
    /// Raven field source: `oracle/codemp/game/bg_public.h:480`
    pub pmove_msec: c_int,

    /// Raven: callbacks to test the world; these will be different functions
    /// during game and cgame.
    /// Raven field source: `oracle/codemp/game/bg_public.h:484`
    pub trace: Option<
        unsafe extern "C" fn(
            results: *mut trace_t,
            start: *const vec3_t,
            mins: *const vec3_t,
            maxs: *const vec3_t,
            end: *const vec3_t,
            passEntityNum: c_int,
            contentMask: c_int,
        ),
    >,
    /// Raven field source: `oracle/codemp/game/bg_public.h:485`
    pub pointcontents:
        Option<unsafe extern "C" fn(point: *const vec3_t, passEntityNum: c_int) -> c_int>,

    /// Raven field source: `oracle/codemp/game/bg_public.h:487`
    pub checkDuelLoss: c_int,

    /// Raven: rww - bg entitystate access method; base address of the entity
    /// array (g_entities or cg_entities).
    /// Raven field source: `oracle/codemp/game/bg_public.h:490`
    pub baseEnt: *mut bgEntity_t,
    /// size of the struct (gentity_t or centity_t) so things can be dynamic
    /// Raven field source: `oracle/codemp/game/bg_public.h:491`
    pub entSize: c_int,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<pmove_t>() == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, ps) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, ghoul2) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, g2Bolts_LFoot) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, g2Bolts_RFoot) == 20);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, modelScale) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, nonHumanoid) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, cmd) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, tracemask) == 68);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, debugLevel) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, noFootsteps) == 76);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, gauntletHit) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, framecount) == 84);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, numtouch) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, touchents) == 92);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, useEvent) == 220);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, mins) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, maxs) == 236);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, watertype) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, waterlevel) == 252);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, gametype) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, debugMelee) == 260);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, stepSlideFix) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, noSpecMove) == 268);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, animations) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, xyspeed) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, pmove_fixed) == 284);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, pmove_msec) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, trace) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, pointcontents) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, checkDuelLoss) == 312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, baseEnt) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(pmove_t, entSize) == 328);
