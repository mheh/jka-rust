use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::cgame::types::e_status;

/// Arguments for `CG_CIN_STOPCINEMATIC`.
///
/// Raven: stops playing the cinematic and ends it. should always return
/// FMV_EOF. cinematics must be stopped in reverse order of when they are
/// started.
/// Raven wrapper: `syscall(CG_CIN_STOPCINEMATIC, handle)`.
/// Raven transport: `return CIN_StopCinematic(args[1]);`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:524-527`
/// Args source: `oracle/code/cgame/cg_local.h:1199`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:820-821`
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

/// `CG_CIN_STOPCINEMATIC` SP cgame imports syscall ABI token.
///
/// Raven `e_status` is an integer transport value.
/// Enum value source: `oracle/code/cgame/cg_public.h:186`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:524-527`
/// Output source: `oracle/code/cgame/cg_syscalls.cpp:526-528`
/// Output source: `oracle/code/cgame/cg_local.h:1199`
/// Output source: `oracle/code/client/cl_cgame.cpp:820-821`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:820-821`
/// Type definition source: `oracle/code/game/q_shared.h:2670-2679`
pub struct CgCinStopcinematic;

impl OutboundSysCall for CgCinStopcinematic {
    type Import = SpCgameImport;
    type Args = CgCinStopcinematicArgs;
    type Output = e_status;

    const IMPORT: SpCgameImport = SpCgameImport::CG_CIN_STOPCINEMATIC;
}

impl EncodeSysCall for CgCinStopcinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for CgCinStopcinematic {
    fn decode_return(word: isize) -> Self::Output {
        e_status::from_wire(word as c_int)
    }
}
