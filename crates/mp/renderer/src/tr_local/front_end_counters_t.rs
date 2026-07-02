#![allow(non_camel_case_types, non_snake_case)]

/// Raven `frontEndCounters_t` — front-end (CPU) rendering stat counters.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:1235-1244`
#[repr(C)]
pub struct frontEndCounters_t {
	pub c_sphere_cull_patch_in: i32,
	pub c_sphere_cull_patch_clip: i32,
	pub c_sphere_cull_patch_out: i32,
	pub c_box_cull_patch_in: i32,
	pub c_box_cull_patch_clip: i32,
	pub c_box_cull_patch_out: i32,
	pub c_sphere_cull_md3_in: i32,
	pub c_sphere_cull_md3_clip: i32,
	pub c_sphere_cull_md3_out: i32,
	pub c_box_cull_md3_in: i32,
	pub c_box_cull_md3_clip: i32,
	pub c_box_cull_md3_out: i32,

	pub c_leafs: i32,
	pub c_dlightSurfaces: i32,
	pub c_dlightSurfacesCulled: i32,
}

const _: () = assert!(core::mem::size_of::<frontEndCounters_t>() == 60);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_sphere_cull_patch_in) == 0);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_sphere_cull_patch_clip) == 4);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_sphere_cull_patch_out) == 8);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_box_cull_patch_in) == 12);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_box_cull_patch_clip) == 16);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_box_cull_patch_out) == 20);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_sphere_cull_md3_in) == 24);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_sphere_cull_md3_clip) == 28);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_sphere_cull_md3_out) == 32);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_box_cull_md3_in) == 36);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_box_cull_md3_clip) == 40);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_box_cull_md3_out) == 44);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_leafs) == 48);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_dlightSurfaces) == 52);
const _: () = assert!(core::mem::offset_of!(frontEndCounters_t, c_dlightSurfacesCulled) == 56);
