use super::super::SpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use core::ffi::c_int;

/// `UI_KEY_GETOVERSTRIKEMODE` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:189`
pub struct UiKeyGetoverstrikemode;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiKeyGetoverstrikemodeArgs;

impl UiKeyGetoverstrikemodeArgs {
    pub const fn new() -> Self {
        Self
    }
}

impl OutboundSysCall for UiKeyGetoverstrikemode {
    type Import = SpUiImport;
    /// Raven wrapper: `syscall( UI_KEY_GETOVERSTRIKEMODE );`
    ///
    /// Args source: `oracle/code/ui/ui_syscalls.cpp:118-121`
    /// Output source: `oracle/code/ui/ui_syscalls.cpp:118-121`
    /// Transport/switch source: `oracle/code/ui/ui_syscalls.cpp:439` (commented path in `client/cl_ui.cpp`)
    type Args = UiKeyGetoverstrikemodeArgs;
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_GETOVERSTRIKEMODE;
}

impl EncodeSysCall for UiKeyGetoverstrikemode {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiKeyGetoverstrikemode {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
