#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Raven `ammoData_t` — per-ammo-type data.
///
/// Type definition source: `oracle/codemp/game/bg_weapons.h:87-91`
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct ammoData_t {
	//	char	icon[32];	// Name of ammo icon file
	pub max: i32, // Max amount player can hold of ammo
}

const _: () = assert!(core::mem::size_of::<ammoData_t>() == 4);
const _: () = assert!(core::mem::offset_of!(ammoData_t, max) == 0);
