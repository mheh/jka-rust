use super::super::types::uiClientState_t;
use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_GETCLIENTSTATE` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/code/ui/ui_public.h:196`
/// Type definition source: `oracle/codemp/ui/ui_public.h:7-15`
/// Args source: `oracle/code/ui/ui_syscalls.cpp:151-153`
/// Args source: `oracle/codemp/ui/ui_local.h:961`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1044-1046`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1044-1046`
/// SP caveat: `UI_GETCLIENTSTATE` is commented out in
/// `oracle/code/ui/ui_syscalls.cpp:148-153` and not active in the current
/// `oracle/code/client/cl_ui.cpp` branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiGetclientstateArgs {
    state: *mut uiClientState_t,
}

impl UiGetclientstateArgs {
    pub const fn new(state: *mut uiClientState_t) -> Self {
        Self { state }
    }

    pub const fn state(&self) -> *mut uiClientState_t {
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
