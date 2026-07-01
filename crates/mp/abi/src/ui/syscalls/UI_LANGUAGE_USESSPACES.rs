use super::super::MpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use mp_qshared::shared::qboolean;

/// Arguments for `UI_LANGUAGE_USESSPACES`.
///
/// Raven wrapper: `return syscall( UI_LANGUAGE_USESSPACES );`
/// Raven transport: `return re.Language_UsesSpaces();`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:141-143`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:998`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1151-1152`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiLanguageUsesspacesArgs;

impl UiLanguageUsesspacesArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_LANGUAGE_USESSPACES` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:81`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:141-143`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:998`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1151-1152`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1151-1152`
pub struct UiLanguageUsesspaces;

impl OutboundSysCall for UiLanguageUsesspaces {
    type Import = MpUiImport;
    type Args = UiLanguageUsesspacesArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_LANGUAGE_USESSPACES;
}

impl EncodeSysCall for UiLanguageUsesspaces {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiLanguageUsesspaces {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
