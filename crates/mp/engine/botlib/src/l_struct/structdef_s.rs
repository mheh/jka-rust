#![allow(non_camel_case_types, non_snake_case)]

use super::fielddef_s::fielddef_t;

/// Raven `structdef_t` — a botlib struct definition (size + field list).
///
/// Type definition source: `oracle/codemp/botlib/l_struct.h:43-47`
#[repr(C)]
pub struct structdef_t {
    pub size: i32,
    pub fields: *mut fielddef_t,
}

pub type structdef_s = structdef_t;

const _: () = assert!(core::mem::size_of::<structdef_t>() == 16);
const _: () = assert!(core::mem::offset_of!(structdef_t, size) == 0);
const _: () = assert!(core::mem::offset_of!(structdef_t, fields) == 8);
