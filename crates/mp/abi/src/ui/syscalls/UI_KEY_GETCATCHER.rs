use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_KEY_GETCATCHER`.
///
/// Raven's `trap_Key_GetCatcher` forwards only the syscall token, so this call
/// has no transport payload.
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:529-530`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:991-992`
#[derive(Debug, Default)]
pub struct UiKeyGetcatcherArgs;

impl UiKeyGetcatcherArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_KEY_GETCATCHER` MP UI imports syscall ABI token.
///
/// C signature: `int trap_Key_GetCatcher(void)`.
/// Raven transport: `return syscall( UI_KEY_GETCATCHER );`
/// Raven switch: `return Key_GetCatcher();`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:195`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:529-530`
/// Output source: `oracle/codemp/ui/ui_syscalls.c:529-530`
/// Output source: `oracle/codemp/client/cl_ui.cpp:991-992`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:991-992`
pub struct UiKeyGetcatcher;

impl OutboundSysCall for UiKeyGetcatcher {
    type Import = MpUiImport;
    type Args = UiKeyGetcatcherArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_KEY_GETCATCHER;
}

impl EncodeSysCall for UiKeyGetcatcher {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiKeyGetcatcher {
    // `Key_GetCatcher` returns an `int`; the engine's return word is that value.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
