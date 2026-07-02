#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use crate::shared::vec3_t;

use super::projectileinfo_s::projectileinfo_t;

/// `MAX_STRINGFIELD`.
///
/// Source: `oracle/oracle/codemp/botlib/l_struct.h:16`
const MAX_STRINGFIELD: usize = 80;

/// Raven `weaponinfo_t` — bot AI weapon info.
///
/// Type definition source: `oracle/oracle/codemp/game/be_ai_weap.h:45-71`
#[repr(C)]
pub struct weaponinfo_t {
	/// true if the weapon info is valid
	pub valid: i32,
	/// number of the weapon
	pub number: i32,
	pub name: [c_char; MAX_STRINGFIELD],
	pub model: [c_char; MAX_STRINGFIELD],
	pub level: i32,
	pub weaponindex: i32,
	pub flags: i32,
	pub projectile: [c_char; MAX_STRINGFIELD],
	pub numprojectiles: i32,
	pub hspread: f32,
	pub vspread: f32,
	pub speed: f32,
	pub acceleration: f32,
	pub recoil: vec3_t,
	pub offset: vec3_t,
	pub angleoffset: vec3_t,
	pub extrazvelocity: f32,
	pub ammoamount: i32,
	pub ammoindex: i32,
	pub activate: f32,
	pub reload: f32,
	pub spinup: f32,
	pub spindown: f32,
	/// pointer to the used projectile
	pub proj: projectileinfo_t,
}

pub type weaponinfo_s = weaponinfo_t;

const _: () = assert!(core::mem::size_of::<weaponinfo_t>() == 552);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, valid) == 0);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, number) == 4);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, name) == 8);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, model) == 88);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, level) == 168);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, weaponindex) == 172);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, flags) == 176);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, projectile) == 180);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, numprojectiles) == 260);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, hspread) == 264);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, vspread) == 268);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, speed) == 272);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, acceleration) == 276);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, recoil) == 280);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, offset) == 292);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, angleoffset) == 304);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, extrazvelocity) == 316);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, ammoamount) == 320);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, ammoindex) == 324);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, activate) == 328);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, reload) == 332);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, spinup) == 336);
const _: () = assert!(core::mem::offset_of!(weaponinfo_t, proj) == 344);
