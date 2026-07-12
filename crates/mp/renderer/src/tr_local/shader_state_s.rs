#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

use mp_qshared::shared::MAX_QPATH;

use super::shader_s::shader_s;

// Source: oracle/codemp/renderer/tr_local.h:51
const MAX_STATE_NAME: usize = 32;

/// Raven `shaderState_s` (typedef `shaderState_t`) — a named shader-remap
/// state used for shader animation via `RE_SetActiveShaderName`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:532-538`
#[repr(C)]
pub struct shaderState_t {
    /// name of shader this state belongs to
    pub shaderName: [c_char; MAX_QPATH as usize],
    /// name of this state
    pub name: [c_char; MAX_STATE_NAME],
    /// shader this name invokes
    pub stateShader: [c_char; MAX_QPATH as usize],
    /// time this cycle lasts, <= 0 is forever
    pub cycleTime: i32,
    pub shader: *mut shader_s,
}

/// Raven manifest tag name; the typedef is `shaderState_t`.
pub type shaderState_s = shaderState_t;

const _: () = assert!(core::mem::offset_of!(shaderState_t, shaderName) == 0);
const _: () = assert!(core::mem::offset_of!(shaderState_t, name) == 64);
const _: () = assert!(core::mem::offset_of!(shaderState_t, stateShader) == 96);
const _: () = assert!(core::mem::offset_of!(shaderState_t, cycleTime) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<shaderState_t>() == 176);
    assert!(core::mem::offset_of!(shaderState_t, shader) == 168);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<shaderState_t>() == 168);
    assert!(core::mem::offset_of!(shaderState_t, shader) == 164);
};
