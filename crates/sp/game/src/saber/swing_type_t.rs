#![allow(non_camel_case_types, non_snake_case)]

/// Raven `swingType_t` — swing type.
///
/// Type definition source: `oracle/code/game/wp_saber.h:206-211`
#[repr(i32)]
pub enum swingType_t {
	SWING_FAST,
	SWING_MEDIUM,
	SWING_STRONG,
}
