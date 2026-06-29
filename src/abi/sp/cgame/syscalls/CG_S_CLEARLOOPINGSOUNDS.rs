use super::super::SpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_S_CLEARLOOPINGSOUNDS`.
///
/// Raven wrapper: `syscall(CG_S_CLEARLOOPINGSOUNDS);`
/// Raven transport: `S_ClearLoopingSounds();`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:213-214`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:588-590`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSClearloopingsoundsArgs;

/// `CG_S_CLEARLOOPINGSOUNDS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:93`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:213-214`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:588-590`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:588-590`
pub struct CgSClearloopingsounds;

impl OutboundSysCall for CgSClearloopingsounds {
    type Import = SpCgameImport;
    type Args = CgSClearloopingsoundsArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_CLEARLOOPINGSOUNDS;
}

impl EncodeSysCall for CgSClearloopingsounds {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgSClearloopingsounds {
    fn decode_return(_word: isize) -> Self::Output {}
}
