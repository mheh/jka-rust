use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_REGISTERSHADERNOMIP`.
///
/// Raven wrapper: `return syscall( CG_R_REGISTERSHADERNOMIP, name );`
/// Raven transport: `return re.RegisterShaderNoMip( (const char *)VMA(1) );`
///
/// Raven comment: `returns all white if not found`.
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:278-279`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2252`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:869-870`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRRegistershadernomipArgs {
    name: *const c_char,
}

impl CgRRegistershadernomipArgs {
    pub const fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `CG_R_REGISTERSHADERNOMIP` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:120`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:278-279`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2252`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:869-870`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:869-870`
pub struct CgRRegistershadernomip;

impl OutboundSysCall for CgRRegistershadernomip {
    type Import = MpCgameImport;
    type Args = CgRRegistershadernomipArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_REGISTERSHADERNOMIP;
}

impl EncodeSysCall for CgRRegistershadernomip {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name)])
    }
}

impl DecodeSysCallReturn for CgRRegistershadernomip {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
