use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_PC_LOAD_GLOBAL_DEFINES`.
///
/// Raven wrapper: `syscall ( UI_PC_LOAD_GLOBAL_DEFINES, filename );`
/// Raven transport: `return botlib_export->PC_LoadGlobalDefines ( (char *)VMA(1) );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:382-384`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1167-1168`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiPcLoadGlobalDefinesArgs {
    filename: *const c_char,
}

impl UiPcLoadGlobalDefinesArgs {
    pub const fn new(filename: *const c_char) -> Self {
        Self { filename }
    }

    pub const fn filename(&self) -> *const c_char {
        self.filename
    }
}

/// `UI_PC_LOAD_GLOBAL_DEFINES` MP UI imports syscall ABI token.
///
/// Raven wrapper: `int trap_PC_LoadGlobalDefines ( const char* filename ) { return syscall ( UI_PC_LOAD_GLOBAL_DEFINES, filename ); }`
/// Raven transport: `return botlib_export->PC_LoadGlobalDefines ( (char *)VMA(1) );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:89`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:82-90`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:382-384`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1167-1168`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1167-1168`
pub struct UiPcLoadGlobalDefines;

impl OutboundSysCall for UiPcLoadGlobalDefines {
    type Import = MpUiImport;
    type Args = UiPcLoadGlobalDefinesArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_PC_LOAD_GLOBAL_DEFINES;
}

impl EncodeSysCall for UiPcLoadGlobalDefines {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.filename())])
    }
}

impl DecodeSysCallReturn for UiPcLoadGlobalDefines {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
