#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::qcommon::tags::memtag_t;

/// Raven `zoneHeader_t` — the per-block header of a zone allocation (magic +
/// tag + size + doubly-linked list links).
///
/// Type definition source: `oracle/codemp/qcommon/z_memman_pc.cpp:31-37`
#[repr(C)]
pub struct zoneHeader_t {
	pub iMagic: i32,
	pub eTag: memtag_t,
	pub iSize: i32,
	pub pNext: *mut zoneHeader_t,
	pub pPrev: *mut zoneHeader_t,
}

pub type zoneHeader_s = zoneHeader_t;

const _: () = assert!(core::mem::size_of::<zoneHeader_t>() == 32);
const _: () = assert!(core::mem::offset_of!(zoneHeader_t, iMagic) == 0);
const _: () = assert!(core::mem::offset_of!(zoneHeader_t, eTag) == 4);
const _: () = assert!(core::mem::offset_of!(zoneHeader_t, iSize) == 8);
const _: () = assert!(core::mem::offset_of!(zoneHeader_t, pNext) == 16);
const _: () = assert!(core::mem::offset_of!(zoneHeader_t, pPrev) == 24);
