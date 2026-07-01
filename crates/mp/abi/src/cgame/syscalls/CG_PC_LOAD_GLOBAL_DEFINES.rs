use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_PC_LOAD_GLOBAL_DEFINES`.
///
/// Raven wrapper: `return syscall ( CG_PC_LOAD_GLOBAL_DEFINES, filename );`
/// Raven transport: `return botlib_export->PC_LoadGlobalDefines ( (char *)VMA(1) );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:561-563`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1009-1010`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgPcLoadGlobalDefinesArgs {
    filename: *const c_char,
}

impl CgPcLoadGlobalDefinesArgs {
    pub const fn new(filename: *const c_char) -> Self {
        Self { filename }
    }
}

/// `CG_PC_LOAD_GLOBAL_DEFINES` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:204`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:561-563`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1009-1010`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1009-1010`
pub struct CgPcLoadGlobalDefines;

impl OutboundSysCall for CgPcLoadGlobalDefines {
    type Import = MpCgameImport;
    type Args = CgPcLoadGlobalDefinesArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_LOAD_GLOBAL_DEFINES;
}

impl EncodeSysCall for CgPcLoadGlobalDefines {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.filename)])
    }
}

impl DecodeSysCallReturn for CgPcLoadGlobalDefines {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
