use super::super::SpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `UI_KEY_CLEARSTATES` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:191`
pub struct UiKeyClearstates;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiKeyClearstatesArgs;

impl UiKeyClearstatesArgs {
    pub const fn new() -> Self {
        Self
    }
}

impl OutboundSysCall for UiKeyClearstates {
    type Import = SpUiImport;
    /// Raven wrapper: `syscall( UI_KEY_CLEARSTATES );`
    /// Raven transport: `Key_ClearStates(); return 0;`
    ///
    /// Args source: `oracle/code/ui/ui_syscalls.cpp:128-130`
    /// Output source: `oracle/code/ui/ui_syscalls.cpp:128-130`
    /// Transport/switch source: `oracle/code/client/cl_ui.cpp:415-417`
    type Args = UiKeyClearstatesArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_CLEARSTATES;
}

impl EncodeSysCall for UiKeyClearstates {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiKeyClearstates {
    fn decode_return(_word: isize) -> Self::Output {}
}
