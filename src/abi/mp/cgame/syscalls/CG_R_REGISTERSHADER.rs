use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_REGISTERSHADER`.
///
/// Raven wrapper: `return syscall( CG_R_REGISTERSHADER, name );`
/// Raven transport: `return re.RegisterShader( (const char *)VMA(1) );`
///
/// Raven comment: `returns all white if not found`.
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:274-275`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2251`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:867-868`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRegistershaderArgs {
    name: *const c_char,
}

impl CgRRegistershaderArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `CG_R_REGISTERSHADER` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:119`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:274-275`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2251`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:867-868`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:867-868`
pub struct CgRRegistershader;

impl OutboundSysCall for CgRRegistershader {
    type Import = MpCgameImport;
    type Args = CgRRegistershaderArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_REGISTERSHADER;
}

impl EncodeSysCall for CgRRegistershader {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name)])
    }
}

impl DecodeSysCallReturn for CgRRegistershader {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
