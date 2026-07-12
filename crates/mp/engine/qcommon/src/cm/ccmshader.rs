#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_qshared::shared::MAX_QPATH;

/// Raven `CCMShader` — a collision-model shader record (name + surface/content flags),
/// intrusive-linked into a per-name hash chain via `mNext`.
///
/// Type definition source: `oracle/codemp/qcommon/cm_local.h:77-89`
#[repr(C)]
pub struct CCMShader {
    pub shader: [c_char; MAX_QPATH],
    pub mNext: *mut CCMShader,
    pub surfaceFlags: c_int,
    pub contentFlags: c_int,
}

impl CCMShader {
    /// Raven `CCMShader::GetName`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_local.h:85`
    pub fn GetName(&self) -> *const c_char {
        self.shader.as_ptr()
    }

    /// Raven `CCMShader::GetNext`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_local.h:86`
    pub fn GetNext(&self) -> *mut CCMShader {
        self.mNext
    }

    /// Raven `CCMShader::SetNext`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_local.h:87`
    pub fn SetNext(&mut self, next: *mut CCMShader) {
        self.mNext = next;
    }

    /// Raven `CCMShader::Destroy`.
    ///
    /// Raven: no-op.
    /// Source: `oracle/codemp/qcommon/cm_local.h:88`
    pub fn Destroy(&mut self) {}
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<CCMShader>() == 80);
const _: () = assert!(core::mem::offset_of!(CCMShader, shader) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(CCMShader, mNext) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(CCMShader, surfaceFlags) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(CCMShader, contentFlags) == 76);
