#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::{c_char, c_int};

use super::file_in_pack_s::fileInPack_t;
use super::unz_file::unzFile;

/// Raven `pack_t` — an open `.pk3` archive tracked by the filesystem.
///
/// Type definition source: `oracle/codemp/qcommon/files.h:42-56`
#[repr(C)]
pub struct pack_t {
    /// c:\quake3\base\pak0.pk3
    pub pakFilename: [c_char; 1024],
    /// pak0
    pub pakBasename: [c_char; 1024],
    /// base
    pub pakGamename: [c_char; 1024],
    /// handle to zip file
    pub handle: unzFile,
    /// regular checksum
    pub checksum: c_int,
    /// checksum for pure
    pub pure_checksum: c_int,
    /// number of files in pk3
    pub numfiles: c_int,
    /// referenced file flags
    pub referenced: c_int,
    /// hash table size (power of 2)
    pub hashSize: c_int,
    /// hash table
    pub hashTable: *mut *mut fileInPack_t,
    /// buffer with the filenames etc.
    pub buildBuffer: *mut fileInPack_t,
}

const _: () = assert!(core::mem::offset_of!(pack_t, pakFilename) == 0);
const _: () = assert!(core::mem::offset_of!(pack_t, pakBasename) == 1024);
const _: () = assert!(core::mem::offset_of!(pack_t, pakGamename) == 2048);
const _: () = assert!(core::mem::offset_of!(pack_t, handle) == 3072);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<pack_t>() == 3120);
    assert!(core::mem::offset_of!(pack_t, checksum) == 3080);
    assert!(core::mem::offset_of!(pack_t, pure_checksum) == 3084);
    assert!(core::mem::offset_of!(pack_t, numfiles) == 3088);
    assert!(core::mem::offset_of!(pack_t, referenced) == 3092);
    assert!(core::mem::offset_of!(pack_t, hashSize) == 3096);
    assert!(core::mem::offset_of!(pack_t, hashTable) == 3104);
    assert!(core::mem::offset_of!(pack_t, buildBuffer) == 3112);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<pack_t>() == 3104);
    assert!(core::mem::offset_of!(pack_t, checksum) == 3076);
    assert!(core::mem::offset_of!(pack_t, pure_checksum) == 3080);
    assert!(core::mem::offset_of!(pack_t, numfiles) == 3084);
    assert!(core::mem::offset_of!(pack_t, referenced) == 3088);
    assert!(core::mem::offset_of!(pack_t, hashSize) == 3092);
    assert!(core::mem::offset_of!(pack_t, hashTable) == 3096);
    assert!(core::mem::offset_of!(pack_t, buildBuffer) == 3100);
};
