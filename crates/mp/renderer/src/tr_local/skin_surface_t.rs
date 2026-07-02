#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

use mp_qshared::shared::MAX_QPATH;

use super::shader_s::shader_s;

/// Raven `skinSurface_t` — per-surface shader mapping within a `skin_s`.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:604-607`
#[repr(C)]
pub struct skinSurface_t {
    pub name: [c_char; MAX_QPATH as usize],
    pub shader: *mut shader_s,
}

const _: () = assert!(core::mem::size_of::<skinSurface_t>() == 72);
const _: () = assert!(core::mem::offset_of!(skinSurface_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(skinSurface_t, shader) == 64);
