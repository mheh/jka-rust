#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_char;

use mp_qshared::shared::qboolean;

use super::qfile_us::qfile_ut;

/// Raven's `MAX_ZPATH` — max length of a file path inside a pak/zip.
/// Source: `oracle/codemp/qcommon/files.h:32`
const MAX_ZPATH: usize = 256;

/// Raven `fileHandleData_t` — per-handle state for an open engine file.
///
/// Type definition source: `oracle/codemp/qcommon/files.h:84-100`
#[repr(C)]
pub struct fileHandleData_t {
    pub handleFiles: qfile_ut,
    pub handleSync: qboolean,
    pub baseOffset: i32,
    pub fileSize: i32,
    pub zipFilePos: i32,
    pub zipFile: qboolean,
    pub streamed: qboolean,
    pub name: [c_char; MAX_ZPATH],
}

const _: () = assert!(core::mem::size_of::<fileHandleData_t>() == 296);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, handleFiles) == 0);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, handleSync) == 16);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, baseOffset) == 20);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, fileSize) == 24);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, zipFilePos) == 28);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, zipFile) == 32);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, streamed) == 36);
const _: () = assert!(core::mem::offset_of!(fileHandleData_t, name) == 40);
