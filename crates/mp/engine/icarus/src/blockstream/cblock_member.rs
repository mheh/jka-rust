#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;

/// Raven `CBlockMember` — one ID/size/data record in an Icarus block stream.
///
/// Only the class's data members are ABI-relevant here; its methods
/// (`WriteMember`, `ReadMember`, `SetData`, `GetData`, `Duplicate`, ...) are
/// behavior, not layout, and are ported separately.
/// Type definition source: `oracle/oracle/codemp/game/../icarus/blockstream.h:38-105`
#[repr(C)]
pub struct CBlockMember {
	/// ID of the value contained in data
	pub m_id: i32,
	/// Size of the data member variable
	pub m_size: i32,
	/// Data for this member (Raven's own type is `void *`)
	pub m_data: *mut c_void,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<CBlockMember>() == 16);
const _: () = assert!(core::mem::offset_of!(CBlockMember, m_id) == 0);
const _: () = assert!(core::mem::offset_of!(CBlockMember, m_size) == 4);
const _: () = assert!(core::mem::offset_of!(CBlockMember, m_data) == 8);
