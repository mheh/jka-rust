#![allow(non_camel_case_types, non_snake_case)]

/// Raven `solid_t` — object collision classification.
///
/// Type definition source: `oracle/codemp/game/be_aas.h:59-65`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum solid_t {
	/// No interaction with other objects.
	SOLID_NOT = 0,
	/// Only touch when inside, after moving.
	SOLID_TRIGGER = 1,
	/// Touch on edge.
	SOLID_BBOX = 2,
	/// BSP clip, touch on edge.
	SOLID_BSP = 3,
}
