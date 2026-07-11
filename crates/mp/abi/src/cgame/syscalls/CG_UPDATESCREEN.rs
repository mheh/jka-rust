use super::super::MpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_UPDATESCREEN`.
///
/// Raven wrapper: `syscall( CG_UPDATESCREEN );`
/// Raven transport: `SCR_UpdateScreen(); return 0;`
///
/// Raven: this is used during lengthy level loading, so pump message loop.
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:119-120`
/// Args source: `oracle/codemp/client/cl_cgame.cpp:763-771`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:763-771`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgUpdatescreenArgs;

impl CgUpdatescreenArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_UPDATESCREEN` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:82`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:119-120`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:763-771`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:763-771`
pub struct CgUpdatescreen;

impl OutboundSysCall for CgUpdatescreen {
    type Import = MpCgameImport;
    type Args = CgUpdatescreenArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_UPDATESCREEN;
}

impl EncodeSysCall for CgUpdatescreen {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgUpdatescreen {
    fn decode_return(_word: isize) -> Self::Output {}
}
