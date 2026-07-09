use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;
use core::ffi::c_int;
use std::ffi::CString;

/// `UI_G2_SETROOTSURFACE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct UiG2SetrootsurfaceArgs {
    pub ghoul2: *mut core::ffi::c_void,
    pub model_index: c_int,
    pub surface_name: CString,
}

impl UiG2SetrootsurfaceArgs {
    pub fn new(ghoul2: *mut core::ffi::c_void, model_index: c_int, surface_name: CString) -> Self {
        Self {
            ghoul2,
            model_index,
            surface_name,
        }
    }

    pub fn ghoul2(&self) -> *mut core::ffi::c_void {
        self.ghoul2
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
    pub fn surface_name(&self) -> &CString {
        &self.surface_name
    }
}

/// `UI_G2_SETROOTSURFACE` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:533`
pub struct UiG2Setrootsurface;

impl OutboundSysCall for UiG2Setrootsurface {
    type Import = MpUiImport;
    type Args = UiG2SetrootsurfaceArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_SETROOTSURFACE;
}

impl EncodeSysCall for UiG2Setrootsurface {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.model_index as isize,
            ptr_to_word(a.surface_name.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for UiG2Setrootsurface {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
