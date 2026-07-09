#![allow(non_camel_case_types, non_snake_case)]

/// Raven `serverState_t` — server operation state enumeration.
///
/// Type definition source: `oracle/codemp/qcommon/../server/server.h:47-51`
#[repr(i32)]
pub enum serverState_t {
	/// no map loaded
	SS_DEAD = 0,
	/// spawning level entities
	SS_LOADING = 1,
	/// actively running
	SS_GAME = 2,
}
