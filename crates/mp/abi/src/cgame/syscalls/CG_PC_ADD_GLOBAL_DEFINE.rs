use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_PC_ADD_GLOBAL_DEFINE`.
///
/// Raven wrapper: `return syscall( CG_PC_ADD_GLOBAL_DEFINE, define );`
/// Raven transport: `return botlib_export->PC_AddGlobalDefine( (char *)VMA(1) );`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:541-542`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:999-1000`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgPcAddGlobalDefineArgs {
    define: *mut c_char,
}

impl CgPcAddGlobalDefineArgs {
    pub const fn new(define: *mut c_char) -> Self {
        Self { define }
    }
}

/// `CG_PC_ADD_GLOBAL_DEFINE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:199`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:541-542`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:999-1000`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:999-1000`
pub struct CgPcAddGlobalDefine;

impl OutboundSysCall for CgPcAddGlobalDefine {
    type Import = MpCgameImport;
    type Args = CgPcAddGlobalDefineArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_ADD_GLOBAL_DEFINE;
}

impl EncodeSysCall for CgPcAddGlobalDefine {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.define)])
    }
}

impl DecodeSysCallReturn for CgPcAddGlobalDefine {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
