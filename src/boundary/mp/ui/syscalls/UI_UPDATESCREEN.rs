use super::super::MpUiImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_UPDATESCREEN`.
///
/// Raven wrapper: `syscall( UI_UPDATESCREEN );`
/// Raven transport: `SCR_UpdateScreen(); return 0;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:202-203`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:947`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:992-994`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiUpdatescreenArgs;

impl UiUpdatescreenArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_UPDATESCREEN` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:47`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:202-203`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:202-203`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:992-994`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:992-994`
pub struct UiUpdatescreen;

impl OutboundSysCall for UiUpdatescreen {
    type Import = MpUiImport;
    type Args = UiUpdatescreenArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_UPDATESCREEN;
}

impl EncodeSysCall for UiUpdatescreen {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiUpdatescreen {
    fn decode_return(_word: isize) -> Self::Output {}
}
