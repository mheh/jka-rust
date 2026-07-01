use super::super::MpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `UI_KEY_CLEARSTATES`.
///
/// Raven wrapper: `syscall( UI_KEY_CLEARSTATES );`
/// Raven transport: `Key_ClearStates(); return 0;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:242-243`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:957`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1029-1031`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiKeyClearstatesArgs;

impl UiKeyClearstatesArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_KEY_CLEARSTATES` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:58`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:242-243`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:242-243`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1029-1031`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1029-1031`
pub struct UiKeyClearstates;

impl OutboundSysCall for UiKeyClearstates {
    type Import = MpUiImport;
    type Args = UiKeyClearstatesArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_KEY_CLEARSTATES;
}

impl EncodeSysCall for UiKeyClearstates {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiKeyClearstates {
    fn decode_return(_word: isize) -> Self::Output {}
}
