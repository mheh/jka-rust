use core::ffi::c_void;

use super::super::MpUiImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_G2_CLEANMODELS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct UiG2CleanmodelsArgs {
    /// Pointer to the ghoul2 instance pointer; the engine frees the instance and NULLs the slot.
    ghoul2_ptr: *mut *mut c_void,
}

impl UiG2CleanmodelsArgs {
    pub fn new(ghoul2_ptr: *mut *mut c_void) -> Self {
        Self { ghoul2_ptr }
    }

    pub fn ghoul2_ptr(&self) -> *mut *mut c_void {
        self.ghoul2_ptr
    }
}

/// `UI_G2_CLEANMODELS` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:529`
pub struct UiG2Cleanmodels;

impl OutboundSysCall for UiG2Cleanmodels {
    type Import = MpUiImport;
    type Args = UiG2CleanmodelsArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_G2_CLEANMODELS;
}

impl EncodeSysCall for UiG2Cleanmodels {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghoul2_ptr as *const _)])
    }
}

impl DecodeSysCallReturn for UiG2Cleanmodels {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
