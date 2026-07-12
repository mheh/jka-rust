use core::ffi::c_void;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_G2_LISTSURFACES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct UiG2ListsurfacesArgs {
    /// Ghoul2 instance pointer whose surface list is dumped to the console.
    ghl_info: *mut c_void,
}

impl UiG2ListsurfacesArgs {
    pub fn new(ghl_info: *mut c_void) -> Self {
        Self { ghl_info }
    }

    pub fn ghl_info(&self) -> *mut c_void {
        self.ghl_info
    }
}

/// `UI_G2_LISTSURFACES` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:508`
pub struct UiG2Listsurfaces;

impl OutboundSysCall for UiG2Listsurfaces {
    type Import = MpUiImport;
    type Args = UiG2ListsurfacesArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_G2_LISTSURFACES;
}

impl EncodeSysCall for UiG2Listsurfaces {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghl_info)])
    }
}

impl DecodeSysCallReturn for UiG2Listsurfaces {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
