use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::cgame::types::e_status;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CIN_RUNCINEMATIC`.
///
/// Raven: will run a frame of the cinematic but will not draw it. Will return
/// FMV_EOF if the end of the cinematic has been reached.
/// Raven wrapper: `syscall(CG_CIN_RUNCINEMATIC, handle)`.
/// Raven transport: `return CIN_RunCinematic(args[1]);`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:531-533`
/// Args source: `oracle/code/cgame/cg_local.h:1200`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:823-824`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCinRuncinematicArgs {
    handle: c_int,
}

impl CgCinRuncinematicArgs {
    pub const fn new(handle: c_int) -> Self {
        Self { handle }
    }

    pub const fn handle(&self) -> c_int {
        self.handle
    }
}

/// `CG_CIN_RUNCINEMATIC` SP cgame imports syscall ABI token.
///
/// Raven `e_status` is an integer transport value.
/// Enum value source: `oracle/code/cgame/cg_public.h:187`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:531-533`
/// Output source: `oracle/code/cgame/cg_syscalls.cpp:531-533`
/// Output source: `oracle/code/cgame/cg_local.h:1200`
/// Output source: `oracle/code/client/cl_cgame.cpp:823-824`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:823-824`
/// Type definition source: `oracle/code/game/q_shared.h:2670-2679`
pub struct CgCinRuncinematic;

impl OutboundSysCall for CgCinRuncinematic {
    type Import = SpCgameImport;
    type Args = CgCinRuncinematicArgs;
    type Output = e_status;

    const IMPORT: SpCgameImport = SpCgameImport::CG_CIN_RUNCINEMATIC;
}

impl EncodeSysCall for CgCinRuncinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for CgCinRuncinematic {
    fn decode_return(word: isize) -> Self::Output {
        e_status::from_wire(word as c_int)
    }
}
