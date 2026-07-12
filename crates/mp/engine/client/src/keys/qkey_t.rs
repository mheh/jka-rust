#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use mp_qshared::shared::qboolean;

/// Raven `qkey_t` — per-key input state (down, autorepeat count, bound command).
///
/// Type definition source: `oracle/codemp/client/keys.h:3-7`
#[repr(C)]
pub struct qkey_t {
    pub down: qboolean,
    /// if > 1, it is autorepeating
    pub repeats: c_int,
    pub binding: *mut core::ffi::c_char,
}

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<qkey_t>() == 16);
    assert!(core::mem::offset_of!(qkey_t, down) == 0);
    assert!(core::mem::offset_of!(qkey_t, repeats) == 4);
    assert!(core::mem::offset_of!(qkey_t, binding) == 8);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<qkey_t>() == 12);
    assert!(core::mem::offset_of!(qkey_t, down) == 0);
    assert!(core::mem::offset_of!(qkey_t, repeats) == 4);
    assert!(core::mem::offset_of!(qkey_t, binding) == 8);
};
