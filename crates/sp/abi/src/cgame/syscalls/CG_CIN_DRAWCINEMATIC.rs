use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CIN_DRAWCINEMATIC`.
///
/// Raven: draws the current frame.
/// Raven wrapper: `syscall(CG_CIN_DRAWCINEMATIC, handle)`.
/// Raven transport: `CIN_DrawCinematic(args[1]); return 0;`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:537-539`
/// Args source: `oracle/code/cgame/cg_local.h:1201`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:826-828`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCinDrawcinematicArgs {
    handle: c_int,
}

impl CgCinDrawcinematicArgs {
    pub const fn new(handle: c_int) -> Self {
        Self { handle }
    }

    pub const fn handle(&self) -> c_int {
        self.handle
    }
}

/// `CG_CIN_DRAWCINEMATIC` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:188`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:537-539`
/// Output source: `oracle/code/client/cl_cgame.cpp:826-828`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:826-828`
pub struct CgCinDrawcinematic;

impl OutboundSysCall for CgCinDrawcinematic {
    type Import = SpCgameImport;
    type Args = CgCinDrawcinematicArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_CIN_DRAWCINEMATIC;
}

impl EncodeSysCall for CgCinDrawcinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for CgCinDrawcinematic {
    fn decode_return(_word: isize) -> Self::Output {}
}
