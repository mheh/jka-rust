use core::ffi::c_void;

use super::super::MpUiImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_G2_LISTBONES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct UiG2ListbonesArgs {
    ghl_info: *mut c_void,
    frame: i32,
}

impl UiG2ListbonesArgs {
    pub fn new(ghl_info: *mut c_void, frame: i32) -> Self {
        Self { ghl_info, frame }
    }

    pub fn ghl_info(&self) -> *mut c_void {
        self.ghl_info
    }

    pub fn frame(&self) -> i32 {
        self.frame
    }
}

/// `UI_G2_LISTBONES` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:507`
pub struct UiG2Listbones;

impl OutboundSysCall for UiG2Listbones {
    type Import = MpUiImport;
    type Args = UiG2ListbonesArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_G2_LISTBONES;
}

impl EncodeSysCall for UiG2Listbones {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghl_info), a.frame as isize])
    }
}

impl DecodeSysCallReturn for UiG2Listbones {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
