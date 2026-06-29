use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FF_STOPFX`.
///
/// Raven wrapper: `syscall( CG_FF_STOPFX, iFX );`
/// Raven transport: `FFFX_STOP((ffFX_e) args[1]);`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:277-278`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:637-639`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFfStopfxArgs {
    i_fx: c_int,
}

impl CgFfStopfxArgs {
    pub const fn new(i_fx: c_int) -> Self {
        Self { i_fx }
    }

    pub const fn i_fx(&self) -> c_int {
        self.i_fx
    }
}

/// `CG_FF_STOPFX` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:110`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:277-278`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:637-639`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:637-639`
pub struct CgFfStopfx;

impl OutboundSysCall for CgFfStopfx {
    type Import = SpCgameImport;
    type Args = CgFfStopfxArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_FF_STOPFX;
}

impl EncodeSysCall for CgFfStopfx {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.i_fx() as isize])
    }
}

impl DecodeSysCallReturn for CgFfStopfx {
    fn decode_return(_word: isize) -> Self::Output {}
}
