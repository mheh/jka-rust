use std::ffi::CString;

use super::super::MpUiImport;
use mp_qshared::shared::qhandle_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_R_REGISTERSKIN` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct UiRRegisterskinArgs {
    name: CString,
}

impl UiRRegisterskinArgs {
    pub fn new(name: CString) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &CString {
        &self.name
    }
}

/// `UI_R_REGISTERSKIN` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:506`
pub struct UiRRegisterskin;

impl OutboundSysCall for UiRRegisterskin {
    type Import = MpUiImport;
    type Args = UiRRegisterskinArgs;
    type Output = qhandle_t;

    const IMPORT: MpUiImport = MpUiImport::UI_R_REGISTERSKIN;
}

impl EncodeSysCall for UiRRegisterskin {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.name.as_ptr())])
    }
}

impl DecodeSysCallReturn for UiRRegisterskin {
    fn decode_return(word: isize) -> Self::Output {
        word as qhandle_t
    }
}
