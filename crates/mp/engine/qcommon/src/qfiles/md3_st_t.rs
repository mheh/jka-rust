#![allow(non_camel_case_types, non_snake_case)]

/// Raven `md3St_t` — MD3 texture coordinate.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/../qcommon/qfiles.h:160-162`
#[repr(C)]
pub struct md3St_t {
	pub st: [f32; 2],
}

const _: () = assert!(core::mem::size_of::<md3St_t>() == 8);
const _: () = assert!(core::mem::offset_of!(md3St_t, st) == 0);
