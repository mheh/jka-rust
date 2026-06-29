use core::ffi::c_void;

use super::super::SpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport, ptr_to_word,
};

/// `UI_GETCLIENTSTATE` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:196`
/// Args source: `oracle/oracle/code/ui/ui_syscalls.cpp:151-153`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1044-1046`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1044-1046`
/// SP caveat: `UI_GETCLIENTSTATE` is not active in the current `oracle/oracle/code/client/cl_ui.cpp` branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiGetclientstateArgs {
    // FIXME: create type `uiClientState_t` in Rust when UI client state payload is modeled.
    state: *mut c_void,
}

impl UiGetclientstateArgs {
    pub const fn new(state: *mut c_void) -> Self {
        Self { state }
    }

    pub const fn state(&self) -> *mut c_void {
        self.state
    }
}
pub struct UiGetclientstate;

impl OutboundSysCall for UiGetclientstate {
    type Import = SpUiImport;
    type Args = UiGetclientstateArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_GETCLIENTSTATE;
}

impl EncodeSysCall for UiGetclientstate {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.state())])
    }
}

impl DecodeSysCallReturn for UiGetclientstate {
    fn decode_return(_word: isize) -> Self::Output {}
}
