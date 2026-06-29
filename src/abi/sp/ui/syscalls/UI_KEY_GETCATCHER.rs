use super::super::SpUiImport;
use core::ffi::c_int;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_KEY_GETCATCHER` SP UI imports syscall ABI token.
///
/// Raven: 40
/// Source: `oracle/oracle/code/ui/ui_public.h:192`
pub struct UiKeyGetcatcher;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiKeyGetcatcherArgs;

impl UiKeyGetcatcherArgs {
    pub const fn new() -> Self {
        Self
    }
}

impl OutboundSysCall for UiKeyGetcatcher {
    type Import = SpUiImport;
    /// Raven's trap forwards only the syscall token.
    ///
    /// Args source: `oracle/oracle/code/ui/ui_syscalls.cpp:133-137`
    /// Output source: `oracle/oracle/code/ui/ui_syscalls.cpp:133-137`
    /// Transport/switch source: `oracle/oracle/code/client/cl_ui.cpp:449-450`
    type Args = UiKeyGetcatcherArgs;
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_GETCATCHER;
}

impl EncodeSysCall for UiKeyGetcatcher {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiKeyGetcatcher {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
