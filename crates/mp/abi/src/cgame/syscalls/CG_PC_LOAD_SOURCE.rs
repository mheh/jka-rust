use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_PC_LOAD_SOURCE`.
///
/// Raven wrapper: `return syscall( CG_PC_LOAD_SOURCE, filename );`
/// Raven transport: `return botlib_export->PC_LoadSourceHandle( (const char *)VMA(1) );`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:545-546`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1001-1002`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgPcLoadSourceArgs {
    filename: *const c_char,
}

impl CgPcLoadSourceArgs {
    pub const fn new(filename: *const c_char) -> Self {
        Self { filename }
    }
}

/// `CG_PC_LOAD_SOURCE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:200`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:545-546`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1001-1002`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1001-1002`
pub struct CgPcLoadSource;

impl OutboundSysCall for CgPcLoadSource {
    type Import = MpCgameImport;
    type Args = CgPcLoadSourceArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_LOAD_SOURCE;
}

impl EncodeSysCall for CgPcLoadSource {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.filename)])
    }
}

impl DecodeSysCallReturn for CgPcLoadSource {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
