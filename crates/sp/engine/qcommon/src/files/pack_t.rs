#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::{c_char, c_int};

use super::file_in_pack_s::fileInPack_t;
use super::unz_file::unzFile;

/// Raven `pack_t` — an open `.pk3` archive tracked by the filesystem.
///
/// Type definition source: `oracle/code/qcommon/files.h:33-43`
#[repr(C)]
pub struct pack_t {
    /// c:\quake3\base\asset0.pk3
    pub pakFilename: [c_char; 260],
    pub handle: unzFile,
    pub checksum: c_int,
    pub numfiles: c_int,
    /// hash table size (power of 2)
    pub hashSize: c_int,
    /// hash table
    pub hashTable: *mut *mut fileInPack_t,
    /// buffer with the filenames etc.
    pub buildBuffer: *mut fileInPack_t,
}

const _: () = assert!(core::mem::size_of::<pack_t>() == 304);
const _: () = assert!(core::mem::offset_of!(pack_t, pakFilename) == 0);
const _: () = assert!(core::mem::offset_of!(pack_t, handle) == 264);
const _: () = assert!(core::mem::offset_of!(pack_t, checksum) == 272);
const _: () = assert!(core::mem::offset_of!(pack_t, numfiles) == 276);
const _: () = assert!(core::mem::offset_of!(pack_t, hashSize) == 280);
const _: () = assert!(core::mem::offset_of!(pack_t, hashTable) == 288);
const _: () = assert!(core::mem::offset_of!(pack_t, buildBuffer) == 296);
