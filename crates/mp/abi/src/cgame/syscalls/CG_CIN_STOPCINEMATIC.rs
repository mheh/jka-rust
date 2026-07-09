use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_CIN_STOPCINEMATIC`.
///
/// Raven: stops playing the cinematic and ends it. should always return
/// FMV_EOF. cinematics must be stopped in reverse order of when they are
/// started.
/// Raven wrapper: `syscall(CG_CIN_STOPCINEMATIC, handle)`.
/// Raven transport: `return CIN_StopCinematic(args[1]);`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:590-591`
/// Args source: `oracle/codemp/cgame/cg_local.h:2382`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1028-1029`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCinStopcinematicArgs {
    handle: c_int,
}

impl CgCinStopcinematicArgs {
    pub const fn new(handle: c_int) -> Self {
        Self { handle }
    }

    pub const fn handle(&self) -> c_int {
        self.handle
    }
}

/// `CG_CIN_STOPCINEMATIC` MP cgame imports syscall ABI token.
///
/// Raven `e_status` is an integer transport value.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:211`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:590-591`
/// Output source: `oracle/codemp/cgame/cg_local.h:2382`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1028-1029`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1028-1029`
pub struct CgCinStopcinematic;

impl OutboundSysCall for CgCinStopcinematic {
    type Import = MpCgameImport;
    type Args = CgCinStopcinematicArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CIN_STOPCINEMATIC;
}

impl EncodeSysCall for CgCinStopcinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for CgCinStopcinematic {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
