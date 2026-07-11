use super::super::SpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `CG_UPDATESCREEN` SP cgame imports syscall ABI token.
///
/// Raven: used during lengthy level loading, so pump message loop.
/// Enum value source: `oracle/code/cgame/cg_public.h:77`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:110-112`
/// Output source: `oracle/code/client/cl_cgame.cpp:482-486`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:482-486`
pub struct CgUpdatescreen;

impl OutboundSysCall for CgUpdatescreen {
    type Import = SpCgameImport;
    type Args = ();
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UPDATESCREEN;
}

impl EncodeSysCall for CgUpdatescreen {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgUpdatescreen {
    fn decode_return(_word: isize) -> Self::Output {}
}
