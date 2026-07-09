use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `UI_SP_GETNUMLANGUAGES`.
///
/// Raven wrapper: `return syscall( UI_SP_GETNUMLANGUAGES );`
/// Raven transport: `return SE_GetNumLanguages();`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:438-440`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1210-1211`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiSpGetnumlanguagesArgs;

impl UiSpGetnumlanguagesArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_SP_GETNUMLANGUAGES` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:135`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:438-440`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1210-1211`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1210-1211`
pub struct UiSpGetnumlanguages;

impl OutboundSysCall for UiSpGetnumlanguages {
    type Import = MpUiImport;
    type Args = UiSpGetnumlanguagesArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_SP_GETNUMLANGUAGES;
}

impl EncodeSysCall for UiSpGetnumlanguages {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for UiSpGetnumlanguages {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
