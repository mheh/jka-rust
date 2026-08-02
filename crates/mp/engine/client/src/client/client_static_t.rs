#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use core::ffi::c_int;

use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::shared::limits::MAX_TOKEN_CHARS;
use mp_qshared::shared::{connstate_t, qboolean, qhandle_t, MAX_INFO_STRING};

use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;

use super::server_address_t::serverAddress_t;
use super::server_info_t::serverInfo_t;

/// Raven `MAX_OSPATH` — max length of an OS filesystem path.
///
/// Source: `oracle/codemp/qcommon/qcommon.h`
const MAX_OSPATH: usize = 1024;

/// Raven `MAX_OTHER_SERVERS` — max entries in the local/favorites/mplayer server
/// lists.
///
/// Source: `oracle/codemp/client/client.h`
pub const MAX_OTHER_SERVERS: usize = 128;

/// Raven `MAX_GLOBAL_SERVERS` — max entries in the global server list.
///
/// Source: `oracle/codemp/client/client.h`
pub const MAX_GLOBAL_SERVERS: usize = 2048;

// `MAX_INFO_STRING` (`q_shared.h`) imported from its canonical home in
// `mp_qshared::shared`.

/// Raven `clientStatic_t` — client state not wiped across level loads; the
/// only persistent client state.
///
/// Type definition source: `oracle/codemp/client/client.h:295-349`
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

    pub framecount: c_int,
    /// Raven: msec since last frame
    pub frametime: c_int,

    /// Raven: ignores pause
    pub realtime: c_int,
    /// Raven: ignoring pause, so console always works
    pub realFrametime: c_int,

    pub numlocalservers: c_int,
    pub localServers: [serverInfo_t; MAX_OTHER_SERVERS],

    pub numglobalservers: c_int,
    pub globalServers: [serverInfo_t; MAX_GLOBAL_SERVERS],
    // Raven: additional global servers
    pub numGlobalServerAddresses: c_int,
    pub globalServerAddresses: [serverAddress_t; MAX_GLOBAL_SERVERS],

    pub numfavoriteservers: c_int,
    pub favoriteServers: [serverInfo_t; MAX_OTHER_SERVERS],

    pub nummplayerservers: c_int,
    pub mplayerServers: [serverInfo_t; MAX_OTHER_SERVERS],

    /// Raven: source currently pinging or updating
    pub pingUpdateSource: c_int,

    pub masterNum: c_int,

    // Raven: update server info
    pub updateServer: netadr_t,
    pub updateChallenge: [c_char; MAX_TOKEN_CHARS],
    pub updateInfoString: [c_char; MAX_INFO_STRING],

    pub authorizeServer: netadr_t,

    // Raven: rendering info
    pub glconfig: glconfig_t,
    pub charSetShader: qhandle_t,
    pub whiteShader: qhandle_t,
    pub consoleShader: qhandle_t,
    // Raven: #ifdef _XBOX
    // 	short		mainGamepad;
    // #endif
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<clientStatic_t>() == 414432);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, state) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, keyCatchers) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, servername) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, rendererStarted) == 1032);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, soundStarted) == 1036);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, soundRegistered) == 1040);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, uiStarted) == 1044);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, cgameStarted) == 1048);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, framecount) == 1052);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, frametime) == 1056);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, realtime) == 1060);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, realFrametime) == 1064);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, numlocalservers) == 1068);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, localServers) == 1072);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, numglobalservers) == 22064);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, globalServers) == 22068);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, numGlobalServerAddresses) == 357940);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, globalServerAddresses) == 357944);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, numfavoriteservers) == 370232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, favoriteServers) == 370236);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, nummplayerservers) == 391228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, mplayerServers) == 391232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, pingUpdateSource) == 412224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, masterNum) == 412228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, updateServer) == 412232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, updateChallenge) == 412252);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, updateInfoString) == 413276);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, authorizeServer) == 414300);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, glconfig) == 414320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, charSetShader) == 414416);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, whiteShader) == 414420);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientStatic_t, consoleShader) == 414424);

// Every field is a C scalar, an array, a null-valid raw pointer, or a `#[repr(C)]` struct of those.
// Every embedded enum has a zero discriminant, for example `CA_UNINITIALIZED`, `NA_BOT`, and `TC_NONE`.
// The all-zero image is therefore a valid inhabitant, and Raven's `cls` global starts zeroed.
// The 405 KB mass builds heap-first through `zeroed_box` (STATE-D9).
unsafe impl native_platform::ZeroValid for clientStatic_t {}
