use super::super::MpUiImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `UI_KEY_GETOVERSTRIKEMODE`.
///
/// Raven wrapper: `syscall( UI_KEY_GETOVERSTRIKEMODE );`
/// Raven transport: `return Key_GetOverstrikeMode();`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:234-235`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1022-1023`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiKeyGetoverstrikemodeArgs;

impl UiKeyGetoverstrikemodeArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_KEY_GETOVERSTRIKEMODE` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:56`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:52-62`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:234-235`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:234-235`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1022-1023`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1022-1023`
pub struct UiKeyGetoverstrikemode;

impl OutboundSysCall for UiKeyGetoverstrikemode {
    type Import = MpUiImport;
    type Args = UiKeyGetoverstrikemodeArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_KEY_GETOVERSTRIKEMODE;
}

impl EncodeSysCall for UiKeyGetoverstrikemode {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiKeyGetoverstrikemode {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
