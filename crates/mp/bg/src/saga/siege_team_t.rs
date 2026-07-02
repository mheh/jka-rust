#![allow(non_camel_case_types, non_snake_case)]

use super::siege_class_t::siegeClass_t;

/// Raven `MAX_SIEGE_CLASSES_PER_TEAM`.
///
/// Source: `oracle/oracle/codemp/game/bg_saga.h:13`
pub const MAX_SIEGE_CLASSES_PER_TEAM: usize = 16;

/// Raven `siegeTeam_t` — one team's siege class roster.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_saga.h:82-88`
#[repr(C)]
pub struct siegeTeam_t {
	pub name: [core::ffi::c_char; 512],
	pub classes: [*mut siegeClass_t; MAX_SIEGE_CLASSES_PER_TEAM],
	pub numClasses: i32,
	pub friendlyShader: i32,
}

const _: () = assert!(core::mem::size_of::<siegeTeam_t>() == 648);
const _: () = assert!(core::mem::offset_of!(siegeTeam_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(siegeTeam_t, classes) == 512);
const _: () = assert!(core::mem::offset_of!(siegeTeam_t, numClasses) == 640);
const _: () = assert!(core::mem::offset_of!(siegeTeam_t, friendlyShader) == 644);
