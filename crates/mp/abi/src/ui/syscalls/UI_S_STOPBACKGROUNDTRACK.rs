use super::super::MpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_S_STOPBACKGROUNDTRACK`.
///
/// Raven wrapper: `syscall( UI_S_STOPBACKGROUNDTRACK );`
/// Raven transport: `S_StopBackgroundTrack(); return 0;`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:392-393`
/// Args source: `oracle/codemp/ui/ui_local.h:1000`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1173-1175`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiSStopbackgroundtrackArgs;

impl UiSStopbackgroundtrackArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_S_STOPBACKGROUNDTRACK` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:92`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:392-393`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1173-1175`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1173-1175`
pub struct UiSStopbackgroundtrack;

impl OutboundSysCall for UiSStopbackgroundtrack {
    type Import = MpUiImport;
    type Args = UiSStopbackgroundtrackArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_S_STOPBACKGROUNDTRACK;
}

impl EncodeSysCall for UiSStopbackgroundtrack {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiSStopbackgroundtrack {
    fn decode_return(_word: isize) -> Self::Output {}
}
