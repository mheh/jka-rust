#![allow(non_camel_case_types, non_snake_case)]

/// Raven `vm_t` — SP's minimal handle to a statically-linked cgame/ui module.
///
/// Type definition source: `oracle/code/client/vmachine.h:48-50`
#[repr(C)]
pub struct vm_t {
    pub entryPoint: Option<unsafe extern "C" fn(callNum: i32, ...) -> i32>,
}

/// Raven C tag `vm_s` for the same type.
pub type vm_s = vm_t;

const _: () = assert!(core::mem::size_of::<vm_t>() == 8);
const _: () = assert!(core::mem::offset_of!(vm_t, entryPoint) == 0);
