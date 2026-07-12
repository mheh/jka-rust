#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;

/// Raven `variable_t` — an Icarus interpreter variable slot.
///
/// Type definition source: `oracle/codemp/icarus/interpreter.h:115-120`
#[repr(C)]
pub struct variable_t {
    pub name: [u8; 64],
    pub r#type: i32,
    pub data: *mut c_void,
}

/// Raven tag name for `variable_t`.
pub type variable_s = variable_t;

const _: () = assert!(core::mem::offset_of!(variable_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(variable_t, r#type) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<variable_t>() == 80);
    assert!(core::mem::offset_of!(variable_t, data) == 72);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<variable_t>() == 72);
    assert!(core::mem::offset_of!(variable_t, data) == 68);
};
