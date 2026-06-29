use super::super::SpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CG_R_SETRANGEFOG`.
///
/// Raven: linear fogging, with settable range -rww.
/// Raven wrapper: `syscall(CG_R_SETRANGEFOG, PASSFLOAT(range));`
/// Raven transport: `tr.rangedFog = VMF(1);`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:426-429`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:732-742`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRSetrangefogArgs {
    range: f32,
}

impl CgRSetrangefogArgs {
    pub const fn new(range: f32) -> Self {
        Self { range }
    }

    pub const fn range(&self) -> f32 {
        self.range
    }
}

/// `CG_R_SETRANGEFOG` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:147`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:426-429`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:732-742`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:732-742`
pub struct CgRSetrangefog;

impl OutboundSysCall for CgRSetrangefog {
    type Import = SpCgameImport;
    type Args = CgRSetrangefogArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_SETRANGEFOG;
}

impl EncodeSysCall for CgRSetrangefog {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.range())])
    }
}

impl DecodeSysCallReturn for CgRSetrangefog {
    fn decode_return(_word: isize) -> Self::Output {}
}
