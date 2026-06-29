use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FF_STARTFX`.
///
/// Raven wrapper: `syscall( CG_FF_STARTFX, iFX );`
/// Raven transport: `FFFX_START((ffFX_e) args[1]);`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:269-270`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:631-633`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFfStartfxArgs {
    i_fx: c_int,
}

impl CgFfStartfxArgs {
    pub const fn new(i_fx: c_int) -> Self {
        Self { i_fx }
    }

    pub const fn i_fx(&self) -> c_int {
        self.i_fx
    }
}

/// `CG_FF_STARTFX` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:108`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:269-270`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:631-633`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:631-633`
pub struct CgFfStartfx;

impl OutboundSysCall for CgFfStartfx {
    type Import = SpCgameImport;
    type Args = CgFfStartfxArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_FF_STARTFX;
}

impl EncodeSysCall for CgFfStartfx {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.i_fx() as isize])
    }
}

impl DecodeSysCallReturn for CgFfStartfx {
    fn decode_return(_word: isize) -> Self::Output {}
}
