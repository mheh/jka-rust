use super::super::SpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use core::ffi::c_int;

/// `UI_KEY_GETCATCHER` SP UI imports syscall ABI token.
///
/// Raven: 40
/// Source: `oracle/code/ui/ui_public.h:192`
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
    /// Args source: `oracle/code/ui/ui_syscalls.cpp:133-137`
    /// Output source: `oracle/code/ui/ui_syscalls.cpp:133-137`
    /// Transport/switch source: `oracle/code/client/cl_ui.cpp:449-450`
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
