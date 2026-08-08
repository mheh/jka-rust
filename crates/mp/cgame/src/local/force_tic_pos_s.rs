#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use mp_qshared::shared::qhandle_t;

/// Raven `forceTicPos_t` — screen position/size of a force-power icon, plus its shader.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:1018-1026`
#[repr(C)]
pub struct forceTicPos_t {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub file: *mut c_char,
    pub tic: qhandle_t,
}

// The head of the struct holds only 4-byte members, so these offsets hold on both pointer widths.
const _: () = assert!(core::mem::offset_of!(forceTicPos_t, x) == 0);
const _: () = assert!(core::mem::offset_of!(forceTicPos_t, y) == 4);
const _: () = assert!(core::mem::offset_of!(forceTicPos_t, width) == 8);
const _: () = assert!(core::mem::offset_of!(forceTicPos_t, height) == 12);
// `file` is a pointer, so the size and the tail from `file` onward go in the width-gated blocks.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<forceTicPos_t>() == 32);
    assert!(core::mem::offset_of!(forceTicPos_t, file) == 16);
    assert!(core::mem::offset_of!(forceTicPos_t, tic) == 24);
};
// ILP32 twin: clang i386 ground truth, where msvc and linux-gnu agree.
// These numbers are the retail 32-bit module ABI.
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<forceTicPos_t>() == 24);
    assert!(core::mem::offset_of!(forceTicPos_t, file) == 16);
    assert!(core::mem::offset_of!(forceTicPos_t, tic) == 20);
};
