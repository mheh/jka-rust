use super::super::MpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_S_STOPBACKGROUNDTRACK`.
///
/// Raven wrapper: `syscall( CG_S_STOPBACKGROUNDTRACK );`
/// Raven transport: `S_StopBackgroundTrack(); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:571-572`
/// Args source: `oracle/codemp/cgame/cg_local.h:2237`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1015-1017`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgSStopbackgroundtrackArgs;

impl CgSStopbackgroundtrackArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_S_STOPBACKGROUNDTRACK` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:207`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:571-572`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1015-1017`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1015-1017`
pub struct CgSStopbackgroundtrack;

impl OutboundSysCall for CgSStopbackgroundtrack {
    type Import = MpCgameImport;
    type Args = CgSStopbackgroundtrackArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_STOPBACKGROUNDTRACK;
}

impl EncodeSysCall for CgSStopbackgroundtrack {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgSStopbackgroundtrack {
    fn decode_return(_word: isize) -> Self::Output {}
}
