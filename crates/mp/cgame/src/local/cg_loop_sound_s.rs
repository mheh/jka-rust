#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::{sfxHandle_t, vec3_t};

/// Raven `cgLoopSound_t` — a looping sound attached to an entity or a fixed point.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:324-329`
#[repr(C)]
pub struct cgLoopSound_t {
	pub entityNum: i32,
	pub origin: vec3_t,
	pub velocity: vec3_t,
	pub sfx: sfxHandle_t,
}

const _: () = assert!(core::mem::size_of::<cgLoopSound_t>() == 32);
const _: () = assert!(core::mem::offset_of!(cgLoopSound_t, entityNum) == 0);
const _: () = assert!(core::mem::offset_of!(cgLoopSound_t, origin) == 4);
const _: () = assert!(core::mem::offset_of!(cgLoopSound_t, velocity) == 16);
const _: () = assert!(core::mem::offset_of!(cgLoopSound_t, sfx) == 28);
