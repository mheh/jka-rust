#![allow(non_camel_case_types, non_snake_case)]

/// Raven `lump_t` — BSP file directory entry (offset + length into the file).
///
/// Type definition source: `oracle/code/qcommon/qfiles.h:420-422`
#[repr(C)]
pub struct lump_t {
	pub fileofs: i32,
	pub filelen: i32,
}

const _: () = assert!(core::mem::size_of::<lump_t>() == 8);
const _: () = assert!(core::mem::offset_of!(lump_t, fileofs) == 0);
const _: () = assert!(core::mem::offset_of!(lump_t, filelen) == 4);
