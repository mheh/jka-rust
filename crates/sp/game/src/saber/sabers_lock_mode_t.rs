#![allow(non_camel_case_types, non_snake_case)]

/// Raven `sabersLockMode_t` — sabers lock mode.
///
/// Type definition source: `oracle/oracle/code/game/wp_saber.h:44-59`
#[repr(i32)]
pub enum sabersLockMode_t {
	LOCK_FIRST = 0,
	LOCK_TOP, // = LOCK_FIRST
	LOCK_DIAG_TR,
	LOCK_DIAG_TL,
	LOCK_DIAG_BR,
	LOCK_DIAG_BL,
	LOCK_R,
	LOCK_L,
	LOCK_RANDOM,
	LOCK_KYLE_GRAB1,
	LOCK_KYLE_GRAB2,
	LOCK_KYLE_GRAB3,
	LOCK_FORCE_DRAIN,
}
