use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CIN_DRAWCINEMATIC`.
///
/// Raven: draws the current frame.
/// Raven wrapper: `syscall(CG_CIN_DRAWCINEMATIC, handle)`.
/// Raven transport: `CIN_DrawCinematic(args[1]); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:602-603`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2384`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1034-1036`
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

/// `CG_CIN_DRAWCINEMATIC` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:213`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:602-603`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1034-1036`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1034-1036`
pub struct CgCinDrawcinematic;

impl OutboundSysCall for CgCinDrawcinematic {
    type Import = MpCgameImport;
    type Args = CgCinDrawcinematicArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_CIN_DRAWCINEMATIC;
}

impl EncodeSysCall for CgCinDrawcinematic {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle() as isize])
    }
}

impl DecodeSysCallReturn for CgCinDrawcinematic {
    fn decode_return(_word: isize) -> Self::Output {}
}
