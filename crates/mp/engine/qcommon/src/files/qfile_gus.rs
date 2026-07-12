#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_void;

/// Raven `qfile_gut` — the raw file/archive handle backing a `qfile_ut`.
///
/// Raven: `FILE*` (`o`) is a raw libc stream; `unzFile` (`z`) is a minizip
/// archive handle. Both are opaque pointer-sized handles at this seam.
/// Type definition source: `oracle/codemp/qcommon/files.h:71-76`
#[repr(C)]
#[derive(Clone, Copy)]
pub union qfile_gut {
    pub o: *mut c_void,
    pub z: *mut c_void,
}

/// Raven's C tag name for `qfile_gut`.
pub type qfile_gus = qfile_gut;

const _: () = assert!(core::mem::size_of::<qfile_gut>() == 8);
const _: () = assert!(core::mem::offset_of!(qfile_gut, o) == 0);
const _: () = assert!(core::mem::offset_of!(qfile_gut, z) == 0);
