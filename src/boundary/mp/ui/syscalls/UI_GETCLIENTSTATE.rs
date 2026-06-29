use core::ffi::c_void;

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_GETCLIENTSTATE`.
///
/// Raven wrapper: `syscall( UI_GETCLIENTSTATE, state );`
/// Raven transport: `GetClientState( (uiClientState_t *)VMA(1) );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:258-259`
#[derive(Debug)]
pub struct UiGetclientstateArgs {
    state: *mut c_void,
}

impl UiGetclientstateArgs {
    pub fn new(state: *mut c_void) -> Self {
        Self { state }
    }

    pub fn state(&self) -> *mut c_void {
        self.state
    }
}

/// `UI_GETCLIENTSTATE` MP UI imports syscall boundary token.
///
/// Raven wrapper: `syscall( UI_GETCLIENTSTATE, state );`
/// Raven transport: `GetClientState( (uiClientState_t *)VMA(1) );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:63`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:258-259`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:961`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1044-1046`
pub struct UiGetclientstate;

impl OutboundSysCall for UiGetclientstate {
    type Import = MpUiImport;
    type Args = UiGetclientstateArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_GETCLIENTSTATE;
}

impl EncodeSysCall for UiGetclientstate {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.state())])
    }
}

impl DecodeSysCallReturn for UiGetclientstate {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
