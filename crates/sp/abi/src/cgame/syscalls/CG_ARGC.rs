use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `CG_ARGC` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:67`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:70-72`
/// Output source: `oracle/code/client/cl_cgame.cpp:454-455`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:454-455`
pub struct CgArgc;

impl OutboundSysCall for CgArgc {
    type Import = SpCgameImport;
    type Args = ();
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_ARGC;
}

impl EncodeSysCall for CgArgc {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([])
    }
}

impl DecodeSysCallReturn for CgArgc {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
