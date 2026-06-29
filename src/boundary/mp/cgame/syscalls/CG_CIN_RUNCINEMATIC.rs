use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CIN_RUNCINEMATIC`.
///
/// Raven: will run a frame of the cinematic but will not draw it. Will return
/// FMV_EOF if the end of the cinematic has been reached.
/// Raven wrapper: `syscall(CG_CIN_RUNCINEMATIC, handle)`.
/// Raven transport: `return CIN_RunCinematic(args[1]);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:596-597`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2383`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1031-1032`
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

/// `CG_CIN_RUNCINEMATIC` MP cgame imports syscall boundary token.
///
/// Raven `e_status` is an integer transport value.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:212`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:596-597`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2383`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1031-1032`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1031-1032`
pub struct CgCinRuncinematic;

impl OutboundSysCall for CgCinRuncinematic {
    type Import = MpCgameImport;
    type Args = CgCinRuncinematicArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CIN_RUNCINEMATIC;
}

impl EncodeSysCall for CgCinRuncinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for CgCinRuncinematic {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
