#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::botlib::projectileinfo_s::projectileinfo_t;
use mp_qshared::common::mp::botlib::weaponinfo_s::weaponinfo_t;

/// Raven `weaponconfig_t` — the loaded weapon configuration.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_weap.cpp:96-102`
#[repr(C)]
pub struct weaponconfig_t {
	pub numweapons: i32,
	pub numprojectiles: i32,
	pub projectileinfo: *mut projectileinfo_t,
	pub weaponinfo: *mut weaponinfo_t,
}

pub type weaponconfig_s = weaponconfig_t;

const _: () = assert!(core::mem::size_of::<weaponconfig_t>() == 24);
const _: () = assert!(core::mem::offset_of!(weaponconfig_t, numweapons) == 0);
const _: () = assert!(core::mem::offset_of!(weaponconfig_t, numprojectiles) == 4);
const _: () = assert!(core::mem::offset_of!(weaponconfig_t, projectileinfo) == 8);
const _: () = assert!(core::mem::offset_of!(weaponconfig_t, weaponinfo) == 16);
