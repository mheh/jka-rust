use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_PC_SOURCE_FILE_AND_LINE`.
///
/// Raven wrapper: `return syscall( CG_PC_SOURCE_FILE_AND_LINE, handle, filename, line );`
/// Raven transport: `return botlib_export->PC_SourceFileAndLine( args[1], (char *)VMA(2), (int *)VMA(3) );`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:557-558`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1007-1008`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgPcSourceFileAndLineArgs {
    handle: c_int,
    filename: *mut c_char,
    line: *mut c_int,
}

impl CgPcSourceFileAndLineArgs {
    pub const fn new(handle: c_int, filename: *mut c_char, line: *mut c_int) -> Self {
        Self {
            handle,
            filename,
            line,
        }
    }
}

/// `CG_PC_SOURCE_FILE_AND_LINE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:203`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:557-558`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1007-1008`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1007-1008`
pub struct CgPcSourceFileAndLine;

impl OutboundSysCall for CgPcSourceFileAndLine {
    type Import = MpCgameImport;
    type Args = CgPcSourceFileAndLineArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_SOURCE_FILE_AND_LINE;
}

impl EncodeSysCall for CgPcSourceFileAndLine {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.handle as isize,
            ptr_to_word(args.filename),
            ptr_to_word(args.line),
        ])
    }
}

impl DecodeSysCallReturn for CgPcSourceFileAndLine {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
