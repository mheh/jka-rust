#![allow(non_camel_case_types)]
use core::ffi::c_void;

/// Raven `unzFile` — opaque handle to an open `.pk3` zip archive (vendored minizip).
///
/// The non-STRICT build resolves the typedef to `void*`.
/// Type definition source: `oracle/oracle/code/qcommon/unzip.h:8`
pub type unzFile = *mut c_void;
