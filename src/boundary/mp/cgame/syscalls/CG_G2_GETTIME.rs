use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_GETTIME`.
///
/// Raven wrapper: `return syscall(CG_G2_GETTIME);`
/// Raven transport: `return G2API_GetTime(0);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:981-983`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1520-1521`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgG2GettimeArgs;

impl CgG2GettimeArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_G2_GETTIME` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:292`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:981-983`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:981-983`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1520-1521`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1520-1521`
pub struct CgG2Gettime;

impl OutboundSysCall for CgG2Gettime {
    type Import = MpCgameImport;
    type Args = CgG2GettimeArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETTIME;
}

impl EncodeSysCall for CgG2Gettime {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgG2Gettime {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
