use super::super::SpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `CG_UPDATESCREEN` SP cgame imports syscall boundary token.
///
/// Raven: used during lengthy level loading, so pump message loop.
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:77`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:110-112`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:482-486`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:482-486`
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
