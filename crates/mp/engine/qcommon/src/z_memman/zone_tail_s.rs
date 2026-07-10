#![allow(non_camel_case_types, non_snake_case)]

/// Raven `zoneTail_t` — the trailing magic sentinel of a zone allocation.
///
/// Type definition source: `oracle/codemp/qcommon/z_memman_pc.cpp:39-43`
#[repr(C)]
pub struct zoneTail_t {
	pub iMagic: i32,
}

const _: () = assert!(core::mem::size_of::<zoneTail_t>() == 4);
const _: () = assert!(core::mem::offset_of!(zoneTail_t, iMagic) == 0);
