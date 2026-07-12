use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use core::ffi::c_int;
use mp_qshared::shared::qboolean;

/// `UI_G2_HASGHOUL2MODELONINDEX` outbound game-to-engine syscall.
///
/// Mirrors C ABI: `qboolean trap_G2API_HasGhoul2ModelOnIndex(*void ghl_info, int model_index)`
#[derive(Debug)]
pub struct UiG2Hasghoul2ModelonindexArgs {
    /// Opaque ghoul2 instance pointer (`void *ghl_info` in C).
    ghl_info: *mut core::ffi::c_void,
    /// Slot index to query.
    model_index: c_int,
}

impl UiG2Hasghoul2ModelonindexArgs {
    pub fn new(ghl_info: *mut core::ffi::c_void, model_index: c_int) -> Self {
        Self {
            ghl_info,
            model_index,
        }
    }

    pub fn ghl_info(&self) -> *mut core::ffi::c_void {
        self.ghl_info
    }

    pub fn model_index(&self) -> c_int {
        self.model_index
    }
}

/// `UI_G2_HASGHOUL2MODELONINDEX` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:526`
pub struct UiG2Hasghoul2Modelonindex;

impl OutboundSysCall for UiG2Hasghoul2Modelonindex {
    type Import = MpUiImport;
    type Args = UiG2Hasghoul2ModelonindexArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_HASGHOUL2MODELONINDEX;
}

impl EncodeSysCall for UiG2Hasghoul2Modelonindex {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghl_info()), a.model_index() as isize])
    }
}

impl DecodeSysCallReturn for UiG2Hasghoul2Modelonindex {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
