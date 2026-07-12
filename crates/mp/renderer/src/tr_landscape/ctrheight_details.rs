#![allow(non_camel_case_types, non_snake_case)]
use mp_qshared::shared::qhandle_t;

/// Raven `CTRHeightDetails` — per-landscape-tier shader handle for height-based detail texturing.
///
/// Type definition source: `oracle/codemp/renderer/tr_landscape.h:37-47`
#[repr(C)]
pub struct CTRHeightDetails {
    mShader: qhandle_t,
}

impl CTRHeightDetails {
    /// Raven `CTRHeightDetails::GetShader`.
    pub fn GetShader(&self) -> qhandle_t {
        self.mShader
    }

    /// Raven `CTRHeightDetails::SetShader`.
    pub fn SetShader(&mut self, shader: qhandle_t) {
        self.mShader = shader;
    }
}

const _: () = assert!(core::mem::size_of::<CTRHeightDetails>() == 4);
const _: () = assert!(core::mem::offset_of!(CTRHeightDetails, mShader) == 0);
