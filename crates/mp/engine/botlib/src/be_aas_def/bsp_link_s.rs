#![allow(non_camel_case_types, non_snake_case)]

/// Raven `bsp_link_t` — linked-list node tying a BSP entity into a leaf.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_def.h:59-65`
#[repr(C)]
pub struct bsp_link_t {
	pub entnum: i32,
	pub leafnum: i32,
	pub next_ent: *mut bsp_link_t,
	pub prev_ent: *mut bsp_link_t,
	pub next_leaf: *mut bsp_link_t,
	pub prev_leaf: *mut bsp_link_t,
}

pub type bsp_link_s = bsp_link_t;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<bsp_link_t>() == 40);
const _: () = assert!(core::mem::offset_of!(bsp_link_t, entnum) == 0);
const _: () = assert!(core::mem::offset_of!(bsp_link_t, leafnum) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bsp_link_t, next_ent) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bsp_link_t, prev_ent) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bsp_link_t, next_leaf) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bsp_link_t, prev_leaf) == 32);
