#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_ulong;

use mp_qshared::shared::qboolean;

/// Raven `glstate_t` — cached OpenGL bind/state to avoid redundant GL calls.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1253-1260`
#[repr(C)]
pub struct glstate_t {
    pub currenttextures: [i32; 2],
    pub currenttmu: i32,
    pub finishCalled: qboolean,
    pub texEnv: [i32; 2],
    pub faceCulling: i32,
    // Raven `unsigned long` — platform-width, 4 bytes on ILP32.
    pub glStateBits: c_ulong,
}

const _: () = assert!(core::mem::offset_of!(glstate_t, currenttextures) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<glstate_t>() == 40);
    assert!(core::mem::offset_of!(glstate_t, currenttmu) == 8);
    assert!(core::mem::offset_of!(glstate_t, finishCalled) == 12);
    assert!(core::mem::offset_of!(glstate_t, texEnv) == 16);
    assert!(core::mem::offset_of!(glstate_t, faceCulling) == 24);
    assert!(core::mem::offset_of!(glstate_t, glStateBits) == 32);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<glstate_t>() == 32);
    assert!(core::mem::offset_of!(glstate_t, currenttmu) == 8);
    assert!(core::mem::offset_of!(glstate_t, finishCalled) == 12);
    assert!(core::mem::offset_of!(glstate_t, texEnv) == 16);
    assert!(core::mem::offset_of!(glstate_t, faceCulling) == 24);
    assert!(core::mem::offset_of!(glstate_t, glStateBits) == 28);
};
