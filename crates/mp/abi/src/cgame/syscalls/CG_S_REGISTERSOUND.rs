use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_S_REGISTERSOUND`.
///
/// Raven wrapper: `return syscall( CG_S_REGISTERSOUND, sample );`
/// Raven transport: `return S_RegisterSound( (const char *)VMA(1) );`
///
/// Raven comment: `returns buzz if not found`.
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:229-230`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2235`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:840-841`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSRegistersoundArgs {
    sample: *const c_char,
}

impl CgSRegistersoundArgs {
    pub const fn new(sample: *const c_char) -> Self {
        Self { sample }
    }
}

/// `CG_S_REGISTERSOUND` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:106`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:229-230`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2235`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:840-841`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:840-841`
pub struct CgSRegistersound;

impl OutboundSysCall for CgSRegistersound {
    type Import = MpCgameImport;
    type Args = CgSRegistersoundArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_REGISTERSOUND;
}

impl EncodeSysCall for CgSRegistersound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.sample)])
    }
}

impl DecodeSysCallReturn for CgSRegistersound {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
