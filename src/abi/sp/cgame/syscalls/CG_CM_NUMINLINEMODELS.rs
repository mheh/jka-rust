use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `CG_CM_NUMINLINEMODELS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:82`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:135-137`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:529-530`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:529-530`
pub struct CgCmNuminlinemodels;

impl OutboundSysCall for CgCmNuminlinemodels {
    type Import = SpCgameImport;
    type Args = ();
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_NUMINLINEMODELS;
}

impl EncodeSysCall for CgCmNuminlinemodels {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgCmNuminlinemodels {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
