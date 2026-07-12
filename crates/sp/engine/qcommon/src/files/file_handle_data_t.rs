#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_char;

use sp_qshared::shared::{qboolean, MAX_QPATH};

use super::qfile_us::qfile_ut;

/// Raven `fileHandleData_t` — per-handle state for an open engine file.
///
/// Type definition source: `oracle/code/qcommon/files.h:72-87`
#[repr(C)]
pub struct fileHandleData_t {
    pub handleFiles: qfile_ut,
    pub handleSync: qboolean,
    pub baseOffset: i32,
    pub fileSize: i32,
    pub zipFilePos: i32,
    pub zipFile: qboolean,
    pub name: [c_char; MAX_QPATH],
}

const _: () = assert!(core::mem::size_of::<fileHandleData_t>() == 104);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, handleFiles) == 0);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, handleSync) == 16);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, baseOffset) == 20);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, fileSize) == 24);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, zipFilePos) == 28);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, zipFile) == 32);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, name) == 36);
