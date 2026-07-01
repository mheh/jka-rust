use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_PC_SOURCE_FILE_AND_LINE`.
///
/// Raven wrapper: `syscall( UI_PC_SOURCE_FILE_AND_LINE, handle, filename, line );`
/// Raven transport: `return botlib_export->PC_SourceFileAndLine( args[1], (char *)VMA(2), (int *)VMA(3) );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:378-379`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1165-1166`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiPcSourceFileAndLineArgs {
    handle: c_int,
    filename: *mut c_char,
    line: *mut c_int,
}

impl UiPcSourceFileAndLineArgs {
    pub const fn new(handle: c_int, filename: *mut c_char, line: *mut c_int) -> Self {
        Self {
            handle,
            filename,
            line,
        }
    }

    pub const fn handle(&self) -> c_int {
        self.handle
    }

    pub const fn filename(&self) -> *mut c_char {
        self.filename
    }

    pub const fn line(&self) -> *mut c_int {
        self.line
    }
}

/// `UI_PC_SOURCE_FILE_AND_LINE` MP UI imports syscall ABI token.
///
/// Raven wrapper: `int trap_PC_SourceFileAndLine( int handle, char *filename, int *line ) { return syscall( UI_PC_SOURCE_FILE_AND_LINE, handle, filename, line ); }`
/// Raven transport: `return botlib_export->PC_SourceFileAndLine( args[1], (char *)VMA(2), (int *)VMA(3) );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:88`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:82-90`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:378-379`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1165-1166`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1165-1166`
pub struct UiPcSourceFileAndLine;

impl OutboundSysCall for UiPcSourceFileAndLine {
    type Import = MpUiImport;
    type Args = UiPcSourceFileAndLineArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_PC_SOURCE_FILE_AND_LINE;
}

impl EncodeSysCall for UiPcSourceFileAndLine {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.handle() as isize,
            ptr_to_word(args.filename()),
            ptr_to_word(args.line()),
        ])
    }
}

impl DecodeSysCallReturn for UiPcSourceFileAndLine {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
