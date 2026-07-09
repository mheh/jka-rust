#![allow(non_camel_case_types, non_snake_case)]

/// Raven `saberQuadrant_t` — saber quadrant.
///
/// Type definition source: `oracle/code/game/wp_saber.h:416-426`
#[repr(i32)]
pub enum saberQuadrant_t {
	Q_BR,
	Q_R,
	Q_TR,
	Q_T,
	Q_TL,
	Q_L,
	Q_BL,
	Q_B,
	Q_NUM_QUADS,
}
