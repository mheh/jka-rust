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

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<structdef_t>() == 16);
    assert!(core::mem::offset_of!(structdef_t, size) == 0);
    assert!(core::mem::offset_of!(structdef_t, fields) == 8);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<structdef_t>() == 8);
    assert!(core::mem::offset_of!(structdef_t, size) == 0);
    assert!(core::mem::offset_of!(structdef_t, fields) == 4);
};
