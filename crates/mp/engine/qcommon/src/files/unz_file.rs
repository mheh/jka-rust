#![allow(non_camel_case_types)]
use core::ffi::c_void;

/// Raven `unzFile` — opaque handle to an open `.pk3` zip archive (vendored minizip).
///
/// The non-STRICT build resolves the typedef to `void*`.
/// Type definition source: `oracle/codemp/qcommon/unzip.h:11`
pub type unzFile = *mut c_void;

// The `unz*` reader implementation mirrors `oracle/codemp/qcommon/unzip.cpp` in
// `crate::unzip`; re-export the public API here so the FS layer reaches it through
// the `files::unz_file` seam module.
pub use crate::files::unzip_consts::UNZ_OK;
pub use crate::unzip::{
    unzClose, unzCloseCurrentFile, unzGetCurrentFileInfo, unzGetCurrentFileInfoPosition,
    unzGetGlobalInfo, unzGoToFirstFile, unzGoToNextFile, unzOpen, unzOpenCurrentFile, unzReOpen,
    unzReadCurrentFile, unzSetCurrentFileInfoPosition,
};
