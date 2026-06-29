use super::super::MpUiImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `UI_LANGUAGE_ISASIAN`.
///
/// Raven wrapper: `return syscall( UI_LANGUAGE_ISASIAN );`
/// Raven transport: `return re.Language_IsAsian();`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:136-138`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:997`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1148-1149`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiLanguageIsasianArgs;

impl UiLanguageIsasianArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_LANGUAGE_ISASIAN` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:80`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:136-138`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:997`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1148-1149`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1148-1149`
pub struct UiLanguageIsasian;

impl OutboundSysCall for UiLanguageIsasian {
    type Import = MpUiImport;
    type Args = UiLanguageIsasianArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_LANGUAGE_ISASIAN;
}

impl EncodeSysCall for UiLanguageIsasian {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiLanguageIsasian {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
