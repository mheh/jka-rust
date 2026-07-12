use super::super::MpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_R_CLEARSCENE`.
///
/// Raven wrapper: `syscall( UI_R_CLEARSCENE );`
/// Raven transport: `re.ClearScene(); return 0;`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:170-171`
/// Args source: `oracle/codemp/ui/ui_local.h:939`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:956-958`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiRClearsceneArgs;

impl UiRClearsceneArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_R_CLEARSCENE` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:40`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:170-171`
/// Output source: `oracle/codemp/client/cl_ui.cpp:956-958`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:956-958`
pub struct UiRClearscene;

impl OutboundSysCall for UiRClearscene {
    type Import = MpUiImport;
    type Args = UiRClearsceneArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_R_CLEARSCENE;
}

impl EncodeSysCall for UiRClearscene {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiRClearscene {
    fn decode_return(_word: isize) -> Self::Output {}
}
