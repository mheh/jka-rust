#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

/// Raven `bsp_surface_t` — surface info reported by a BSP trace.
///
/// Type definition source: `oracle/codemp/game/botlib.h:108-113`
#[repr(C)]
pub struct bsp_surface_t {
	pub name: [c_char; 16],
	pub flags: c_int,
	pub value: c_int,
}

pub type bsp_surface_s = bsp_surface_t;

const _: () = assert!(core::mem::size_of::<bsp_surface_t>() == 24);
const _: () = assert!(core::mem::offset_of!(bsp_surface_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(bsp_surface_t, flags) == 16);
const _: () = assert!(core::mem::offset_of!(bsp_surface_t, value) == 20);
