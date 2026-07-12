use super::super::MpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_PC_REMOVE_ALL_GLOBAL_DEFINES`.
///
/// Raven wrapper: `syscall ( UI_PC_REMOVE_ALL_GLOBAL_DEFINES );`
/// Raven transport: `botlib_export->PC_RemoveAllGlobalDefines ( ); return 0;`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:387-389`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1169-1171`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiPcRemoveAllGlobalDefinesArgs;

impl UiPcRemoveAllGlobalDefinesArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `UI_PC_REMOVE_ALL_GLOBAL_DEFINES` MP UI imports syscall ABI token.
///
/// Raven wrapper: `void trap_PC_RemoveAllGlobalDefines ( void ) { syscall ( UI_PC_REMOVE_ALL_GLOBAL_DEFINES ); }`
/// Raven transport: `botlib_export->PC_RemoveAllGlobalDefines ( ); return 0;`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:90`
/// Enum comment source: `oracle/codemp/ui/ui_public.h:82-90`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:387-389`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1169-1171`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1169-1171`
pub struct UiPcRemoveAllGlobalDefines;

impl OutboundSysCall for UiPcRemoveAllGlobalDefines {
    type Import = MpUiImport;
    type Args = UiPcRemoveAllGlobalDefinesArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_PC_REMOVE_ALL_GLOBAL_DEFINES;
}

impl EncodeSysCall for UiPcRemoveAllGlobalDefines {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([])
    }
}

impl DecodeSysCallReturn for UiPcRemoveAllGlobalDefines {
    fn decode_return(_word: isize) -> Self::Output {}
}
