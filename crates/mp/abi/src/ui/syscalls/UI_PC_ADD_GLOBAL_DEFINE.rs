use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_PC_ADD_GLOBAL_DEFINE`.
///
/// Raven wrapper: `syscall( UI_PC_ADD_GLOBAL_DEFINE, define );`
/// Raven transport: `return botlib_export->PC_AddGlobalDefine( (char *)VMA(1) );`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:362-363`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1157-1158`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiPcAddGlobalDefineArgs {
    define: *mut c_char,
}

impl UiPcAddGlobalDefineArgs {
    pub const fn new(define: *mut c_char) -> Self {
        Self { define }
    }

    pub const fn define(&self) -> *mut c_char {
        self.define
    }
}

/// `UI_PC_ADD_GLOBAL_DEFINE` MP UI imports syscall ABI token.
///
/// Raven wrapper: `int trap_PC_AddGlobalDefine( char *define ) { return syscall( UI_PC_ADD_GLOBAL_DEFINE, define ); }`
/// Raven transport: `return botlib_export->PC_AddGlobalDefine( (char *)VMA(1) );`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:84`
/// Enum comment source: `oracle/codemp/ui/ui_public.h:82-90`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:362-363`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1157-1158`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1157-1158`
pub struct UiPcAddGlobalDefine;

impl OutboundSysCall for UiPcAddGlobalDefine {
    type Import = MpUiImport;
    type Args = UiPcAddGlobalDefineArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_PC_ADD_GLOBAL_DEFINE;
}

impl EncodeSysCall for UiPcAddGlobalDefine {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.define())])
    }
}

impl DecodeSysCallReturn for UiPcAddGlobalDefine {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
