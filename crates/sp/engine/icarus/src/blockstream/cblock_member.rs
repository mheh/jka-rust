#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;

/// Raven `CBlockMember` — a single named/typed member of a block (id + size +
/// data blob) written/read to Icarus block-format save files.
///
/// Type definition source: `oracle/oracle/code/icarus/blockstream.h:15-89`
#[repr(C)]
pub struct CBlockMember {
	/// ID of the value contained in data
	pub m_id: i32,
	/// Size of the data member variable
	pub m_size: i32,
	/// Data for this member
	pub m_data: *mut c_void,
}

const _: () = assert!(core::mem::size_of::<CBlockMember>() == 16);
const _: () = assert!(core::mem::offset_of!(CBlockMember, m_id) == 0);
const _: () = assert!(core::mem::offset_of!(CBlockMember, m_size) == 4);
const _: () = assert!(core::mem::offset_of!(CBlockMember, m_data) == 8);
