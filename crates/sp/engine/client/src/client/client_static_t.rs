#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use core::ffi::c_int;

use sp_qshared::common::sp::renderer::glconfig_t::glconfig_t;
use sp_qshared::shared::{connstate_t, qboolean, qhandle_t};

/// Raven `MAX_OSPATH` — max length of a filesystem pathname.
///
/// Source: `oracle/code/game/q_shared.h:216`
const MAX_OSPATH: usize = 260;

/// Raven `MAX_INFO_STRING`.
///
/// Source: `oracle/code/game/q_shared.h:210`
const MAX_INFO_STRING: usize = 1024;

/// Raven `clientStatic_t` — client state not wiped across level loads; the
/// only persistent client state.
///
/// Type definition source: `oracle/code/client/client.h:193-229`
#[repr(C)]
pub struct clientStatic_t {
    /// Raven: connection status
    pub state: connstate_t,
    /// Raven: bit flags
    pub keyCatchers: c_int,

    /// Raven: name of server from original connect (used by reconnect)
    pub servername: [c_char; MAX_OSPATH],

    // Raven: when the server clears the hunk, all of these must be restarted
    pub rendererStarted: qboolean,
    pub soundStarted: qboolean,
    pub soundRegistered: qboolean,
    pub uiStarted: qboolean,
    pub cgameStarted: qboolean,
    // Raven: #ifdef _IMMERSION
    pub forceStarted: qboolean,
    // Raven: #endif // _IMMERSION
    pub framecount: c_int,
    /// Raven: msec since last frame
    pub frametime: c_int,
    /// Raven: fraction of a msec since last frame
    pub frametimeFraction: f32,

    /// Raven: ignores pause
    pub realtime: c_int,
    /// Raven: fraction of a msec accumulated
    pub realtimeFraction: f32,
    /// Raven: ignoring pause, so console always works
    pub realFrametime: c_int,

    // Raven: update server info
    pub updateInfoString: [c_char; MAX_INFO_STRING],

    // Raven: rendering info
    pub glconfig: glconfig_t,
    pub charSetShader: qhandle_t,
    pub whiteShader: qhandle_t,
    pub consoleShader: qhandle_t,
    // Raven: #ifdef _XBOX
    // 	short		mainGamepad;
    // #endif
}

const _: () = assert!(core::mem::size_of::<clientStatic_t>() == 1456);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, state) == 0);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, keyCatchers) == 4);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, servername) == 8);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, rendererStarted) == 268);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, soundStarted) == 272);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, soundRegistered) == 276);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, uiStarted) == 280);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, cgameStarted) == 284);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, forceStarted) == 288);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, framecount) == 292);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, frametime) == 296);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, frametimeFraction) == 300);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, realtime) == 304);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, realtimeFraction) == 308);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, realFrametime) == 312);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, updateInfoString) == 316);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, glconfig) == 1344);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, charSetShader) == 1440);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, whiteShader) == 1444);
const _: () = assert!(core::mem::offset_of!(clientStatic_t, consoleShader) == 1448);
