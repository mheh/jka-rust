#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_long};

use super::file::FILE;

/// Raven `CBlockStream` — buffered reader/writer for an Icarus `.ibi` block stream file.
///
/// Only the class's data members are ABI-relevant here; its methods (`Init`,
/// `Create`, `Free`, `BlockAvailable`, `WriteBlock`, `ReadBlock`, `Open`,
/// overloaded `new`/`delete`) are behavior, not layout, and are ported separately.
/// Type definition source: `oracle/oracle/code/icarus/blockstream.h:163-211`
#[repr(C)]
pub struct CBlockStream {
	/// Size of the file
	pub m_fileSize: c_long,
	/// Global file handle of current I/O source
	pub m_fileHandle: *mut FILE,
	/// Name of the current file
	pub m_fileName: [c_char; 1024],
	/// Stream of data to be parsed
	pub m_stream: *mut c_char,
	pub m_streamPos: c_long,
}

const _: () = assert!(core::mem::size_of::<CBlockStream>() == 1056);
const _: () = assert!(core::mem::offset_of!(CBlockStream, m_fileSize) == 0);
const _: () = assert!(core::mem::offset_of!(CBlockStream, m_fileHandle) == 8);
const _: () = assert!(core::mem::offset_of!(CBlockStream, m_fileName) == 16);
const _: () = assert!(core::mem::offset_of!(CBlockStream, m_stream) == 1040);
const _: () = assert!(core::mem::offset_of!(CBlockStream, m_streamPos) == 1048);
