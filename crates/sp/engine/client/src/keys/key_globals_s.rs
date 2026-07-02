#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use sp_qshared::shared::qboolean;

use super::field_t::field_t;
use super::qkey_t::qkey_t;

// Raven's `#define COMMAND_HISTORY 32` (oracle/oracle/code/client/keys.h:10).
const COMMAND_HISTORY: usize = 32;
// Raven's `MAX_KEYS` enumerator (oracle/oracle/code/client/keycodes.h).
const MAX_KEYS: usize = 320;

/// Raven `keyGlobals_t` — global key/console-field input state.
///
/// Type definition source: `oracle/oracle/code/client/keys.h:19-33`
#[repr(C)]
pub struct keyGlobals_s {
	pub historyEditLines: [field_t; COMMAND_HISTORY],

	/// the last line in the history buffer, not masked
	pub nextHistoryLine: c_int,
	/// the line being displayed from history buffer
	/// will be <= nextHistoryLine
	pub historyLine: c_int,
	pub g_consoleField: field_t,

	pub anykeydown: qboolean,
	pub key_overstrikeMode: qboolean,
	pub keyDownCount: c_int,

	pub keys: [qkey_t; MAX_KEYS],
}

/// Raven `keyGlobals_t`.
pub type keyGlobals_t = keyGlobals_s;

const _: () = assert!(core::mem::size_of::<keyGlobals_t>() == 13984);
const _: () = assert!(core::mem::offset_of!(keyGlobals_t, historyEditLines) == 0);
const _: () = assert!(core::mem::offset_of!(keyGlobals_t, nextHistoryLine) == 8576);
const _: () = assert!(core::mem::offset_of!(keyGlobals_t, historyLine) == 8580);
const _: () = assert!(core::mem::offset_of!(keyGlobals_t, g_consoleField) == 8584);
const _: () = assert!(core::mem::offset_of!(keyGlobals_t, anykeydown) == 8852);
const _: () = assert!(core::mem::offset_of!(keyGlobals_t, key_overstrikeMode) == 8856);
const _: () = assert!(core::mem::offset_of!(keyGlobals_t, keyDownCount) == 8860);
const _: () = assert!(core::mem::offset_of!(keyGlobals_t, keys) == 8864);
