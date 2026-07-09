use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;
use core::ffi::c_int;
use core::ffi::c_void;

/// `UI_G2_REMOVEGHOUL2MODEL` outbound game-to-engine syscall.
///
/// Remove the model at `model_index` from the ghoul2 instance pointed to by `ghl_info`.
#[derive(Debug)]
pub struct UiG2Removeghoul2ModelArgs {
    ghl_info: *mut c_void,
    model_index: c_int,
}

impl UiG2Removeghoul2ModelArgs {
    pub fn new(ghl_info: *mut c_void, model_index: c_int) -> Self {
        Self {
            ghl_info,
            model_index,
        }
    }

    pub fn ghl_info(&self) -> *mut c_void {
        self.ghl_info
    }

    pub fn model_index(&self) -> c_int {
        self.model_index
    }
}

/// `UI_G2_REMOVEGHOUL2MODEL` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:527`
pub struct UiG2Removeghoul2Model;

impl OutboundSysCall for UiG2Removeghoul2Model {
    type Import = MpUiImport;
    type Args = UiG2Removeghoul2ModelArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_REMOVEGHOUL2MODEL;
}

impl EncodeSysCall for UiG2Removeghoul2Model {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ghl_info), a.model_index as isize])
    }
}

impl DecodeSysCallReturn for UiG2Removeghoul2Model {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
