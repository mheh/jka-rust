use super::super::SpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_S_STOPSOUNDS`.
///
/// Raven wrapper: `syscall(CG_S_STOPSOUNDS);`
/// Raven transport: `S_StopSounds();`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:180-182`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:550-551`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSStopsoundsArgs;

/// `CG_S_STOPSOUNDS` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:95`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:180-182`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:550-551`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:550-551`
pub struct CgSStopsounds;

impl OutboundSysCall for CgSStopsounds {
    type Import = SpCgameImport;
    type Args = CgSStopsoundsArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_STOPSOUNDS;
}

impl EncodeSysCall for CgSStopsounds {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgSStopsounds {
    fn decode_return(_word: isize) -> Self::Output {}
}
