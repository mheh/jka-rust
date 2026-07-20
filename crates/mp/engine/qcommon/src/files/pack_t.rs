#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_int;

use super::file_in_pack_s::fileInPack_t;
use super::unz_file::unzFile;

/// Raven `pack_t` — an open `.pk3` archive tracked by the filesystem.
///
/// Engine-internal only (reached through `searchpath_t.pack`, never crosses
/// the module ABI), so the C layout + asserts are dropped (string-data
/// migration, DEC-32): Raven's single `Z_Malloc(sizeof(pack_t) +
/// hashSize*ptr)` block becomes a `Box`ed struct whose hash buckets and file
/// entries are owned `Vec`s (bucket values index `buildBuffer`, `None` =
/// Raven's null). The name buffers' `MAX_OSPATH` size survives as the
/// write-site truncation bound (`cap_ospath` in `files_common`).
/// Type definition source: `oracle/codemp/qcommon/files.h:42-56`
pub struct pack_t {
    /// c:\quake3\base\pak0.pk3
    pub pakFilename: String,
    /// pak0
    pub pakBasename: String,
    /// base
    pub pakGamename: String,
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
    pub hashTable: Vec<Option<u32>>,
    /// buffer with the filenames etc.
    pub buildBuffer: Vec<fileInPack_t>,
}
