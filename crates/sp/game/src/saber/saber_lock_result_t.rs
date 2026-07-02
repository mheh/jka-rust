#![allow(non_camel_case_types, non_snake_case)]

/// Raven `saberLockResult_t` — saber lock result.
///
/// Type definition source: `oracle/oracle/code/game/wp_saber.h:37-42`
#[repr(i32)]
pub enum saberLockResult_t {
	LOCK_VICTORY = 0, // one side won
	LOCK_STALEMATE,    // neither side won
	LOCK_DRAW,         // both people fall back
}
