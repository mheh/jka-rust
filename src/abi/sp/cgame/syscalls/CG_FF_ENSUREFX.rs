use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FF_ENSUREFX`.
///
/// Raven wrapper: `syscall( CG_FF_ENSUREFX, iFX );`
/// Raven transport: `FFFX_ENSURE((ffFX_e) args[1]);`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:273-274`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:634-636`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFfEnsurefxArgs {
    i_fx: c_int,
}

impl CgFfEnsurefxArgs {
    pub const fn new(i_fx: c_int) -> Self {
        Self { i_fx }
    }

    pub const fn i_fx(&self) -> c_int {
        self.i_fx
    }
}

/// `CG_FF_ENSUREFX` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:109`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:273-274`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:634-636`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:634-636`
pub struct CgFfEnsurefx;

impl OutboundSysCall for CgFfEnsurefx {
    type Import = SpCgameImport;
    type Args = CgFfEnsurefxArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_FF_ENSUREFX;
}

impl EncodeSysCall for CgFfEnsurefx {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.i_fx() as isize])
    }
}

impl DecodeSysCallReturn for CgFfEnsurefx {
    fn decode_return(_word: isize) -> Self::Output {}
}
