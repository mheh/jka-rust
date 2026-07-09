#![allow(non_camel_case_types, non_snake_case)]

/// Raven `siegeExtended_t` — cached siege-mode HUD extras for an entity.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:1611-1618`
#[repr(C)]
pub struct siegeExtended_t {
	pub health: i32,
	pub maxhealth: i32,
	pub ammo: i32,
	pub weapon: i32,
	pub lastUpdated: i32,
}

const _: () = assert!(core::mem::size_of::<siegeExtended_t>() == 20);
const _: () = assert!(core::mem::offset_of!(siegeExtended_t, health) == 0);
const _: () = assert!(core::mem::offset_of!(siegeExtended_t, maxhealth) == 4);
const _: () = assert!(core::mem::offset_of!(siegeExtended_t, ammo) == 8);
const _: () = assert!(core::mem::offset_of!(siegeExtended_t, weapon) == 12);
const _: () = assert!(core::mem::offset_of!(siegeExtended_t, lastUpdated) == 16);
