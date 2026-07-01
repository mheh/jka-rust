#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `columnInfo_s` — a single column layout within a list box.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_shared.h:166-170`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct columnInfo_s {
    pub pos: c_int,
    pub width: c_int,
    pub maxChars: c_int,
}

/// Raven `columnInfo_t` — `typedef struct columnInfo_s columnInfo_t`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_shared.h:166-170`
pub type columnInfo_t = columnInfo_s;

const _: () = assert!(core::mem::size_of::<columnInfo_t>() == 12);
const _: () = assert!(core::mem::offset_of!(columnInfo_t, pos) == 0);
const _: () = assert!(core::mem::offset_of!(columnInfo_t, width) == 4);
const _: () = assert!(core::mem::offset_of!(columnInfo_t, maxChars) == 8);
