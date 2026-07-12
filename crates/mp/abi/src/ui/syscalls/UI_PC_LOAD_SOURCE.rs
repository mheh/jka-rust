use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_PC_LOAD_SOURCE`.
///
/// Raven wrapper: `syscall( UI_PC_LOAD_SOURCE, filename );`
/// Raven transport: `return botlib_export->PC_LoadSourceHandle( (const char *)VMA(1) );`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:366-367`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1159-1160`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiPcLoadSourceArgs {
    filename: *const c_char,
}

impl UiPcLoadSourceArgs {
    pub const fn new(filename: *const c_char) -> Self {
        Self { filename }
    }

    pub const fn filename(&self) -> *const c_char {
        self.filename
    }
}

/// `UI_PC_LOAD_SOURCE` MP UI imports syscall ABI token.
///
/// Raven wrapper: `int trap_PC_LoadSource( const char *filename ) { return syscall( UI_PC_LOAD_SOURCE, filename ); }`
/// Raven transport: `return botlib_export->PC_LoadSourceHandle( (const char *)VMA(1) );`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:85`
/// Enum comment source: `oracle/codemp/ui/ui_public.h:82-90`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:366-367`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1159-1160`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1159-1160`
pub struct UiPcLoadSource;

impl OutboundSysCall for UiPcLoadSource {
    type Import = MpUiImport;
    type Args = UiPcLoadSourceArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_PC_LOAD_SOURCE;
}

impl EncodeSysCall for UiPcLoadSource {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.filename())])
    }
}

impl DecodeSysCallReturn for UiPcLoadSource {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
