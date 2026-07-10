#![allow(non_camel_case_types, non_snake_case)]

/// Raven `areaParms_t` — parameters threaded through `SV_AreaEntities` recursion
/// (query bounds plus the output entity list).
///
/// Type definition source: `oracle/codemp/server/sv_world.cpp:359-364`
#[repr(C)]
pub struct areaParms_t {
	pub mins: *const f32,
	pub maxs: *const f32,
	pub list: *mut i32,
	pub count: i32,
	pub maxcount: i32,
}

const _: () = assert!(core::mem::size_of::<areaParms_t>() == 32);
const _: () = assert!(core::mem::offset_of!(areaParms_t, mins) == 0);
const _: () = assert!(core::mem::offset_of!(areaParms_t, maxs) == 8);
const _: () = assert!(core::mem::offset_of!(areaParms_t, list) == 16);
const _: () = assert!(core::mem::offset_of!(areaParms_t, count) == 24);
const _: () = assert!(core::mem::offset_of!(areaParms_t, maxcount) == 28);
