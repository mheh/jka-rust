use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `CG_MILLISECONDS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:63`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:54-56`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:443-444`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:443-444`
pub struct CgMilliseconds;

impl OutboundSysCall for CgMilliseconds {
    type Import = SpCgameImport;
    type Args = ();
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_MILLISECONDS;
}

impl EncodeSysCall for CgMilliseconds {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgMilliseconds {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
