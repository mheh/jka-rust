#![allow(non_camel_case_types, non_snake_case)]

/// Raven `serverState_t` — the state of the server.
///
/// Raven: enumerates the server's operational state from dead to actively running.
/// Type definition source: `oracle/oracle/code/server/server.h:42-46`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum serverState_t {
	/// no map loaded
	SS_DEAD = 0,
	/// spawning level entities
	SS_LOADING = 1,
	/// actively running
	SS_GAME = 2,
}
