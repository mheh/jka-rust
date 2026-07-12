#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use super::structdef_s::structdef_t;

/// Raven `fielddef_t` — a single field description in a botlib struct definition.
///
/// Type definition source: `oracle/codemp/botlib/l_struct.h:31-40`
#[repr(C)]
pub struct fielddef_t {
    pub name: *mut c_char, //name of the field
    pub offset: i32,       //offset in the structure
    pub r#type: i32,       //type of the field
    //type specific fields
    pub maxarray: i32,               //maximum array size
    pub floatmin: f32,               //float min and max
    pub floatmax: f32,               //float min and max
    pub substruct: *mut structdef_t, //sub structure
}

pub type fielddef_s = fielddef_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<fielddef_t>() == 40);
    assert!(core::mem::offset_of!(fielddef_t, name) == 0);
    assert!(core::mem::offset_of!(fielddef_t, offset) == 8);
    assert!(core::mem::offset_of!(fielddef_t, r#type) == 12);
    assert!(core::mem::offset_of!(fielddef_t, maxarray) == 16);
    assert!(core::mem::offset_of!(fielddef_t, floatmin) == 20);
    assert!(core::mem::offset_of!(fielddef_t, floatmax) == 24);
    assert!(core::mem::offset_of!(fielddef_t, substruct) == 32);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<fielddef_t>() == 28);
    assert!(core::mem::offset_of!(fielddef_t, name) == 0);
    assert!(core::mem::offset_of!(fielddef_t, offset) == 4);
    assert!(core::mem::offset_of!(fielddef_t, r#type) == 8);
    assert!(core::mem::offset_of!(fielddef_t, maxarray) == 12);
    assert!(core::mem::offset_of!(fielddef_t, floatmin) == 16);
    assert!(core::mem::offset_of!(fielddef_t, floatmax) == 20);
    assert!(core::mem::offset_of!(fielddef_t, substruct) == 24);
};
