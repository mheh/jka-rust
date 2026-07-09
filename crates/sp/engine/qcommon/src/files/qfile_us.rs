#![allow(non_camel_case_types, non_snake_case)]
use sp_qshared::shared::qboolean;

use super::qfile_gus::qfile_gut;

/// Raven `qfile_ut` — a file handle paired with its "unique" (not part of a
/// pak) flag.
///
/// Type definition source: `oracle/code/qcommon/files.h:67-70`
#[repr(C)]
pub struct qfile_ut {
    pub file: qfile_gut,
    pub unique: qboolean,
}

/// Raven's C tag name for `qfile_ut`.
pub type qfile_us = qfile_ut;

const _: () = assert!(core::mem::size_of::<qfile_ut>() == 16);
const _: () = assert!(core::mem::offset_of!(qfile_ut, file) == 0);
const _: () = assert!(core::mem::offset_of!(qfile_ut, unique) == 8);
