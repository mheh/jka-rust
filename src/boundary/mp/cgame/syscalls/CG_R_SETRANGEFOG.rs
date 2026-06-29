use super::super::MpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CG_R_SETRANGEFOG`.
///
/// Raven: linear fogging, with settable range -rww.
/// Raven wrapper: `syscall(CG_R_SETRANGEFOG, PASSFLOAT(range));`
/// Raven transport: `tr.rangedFog = VMF(1); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:394-397`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2290`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:943-945`
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

/// `CG_R_SETRANGEFOG` MP cgame imports syscall boundary token.
///
/// Raven: linear fogging, with settable range -rww
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:165`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:394-397`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:943-945`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:943-945`
pub struct CgRSetrangefog;

impl OutboundSysCall for CgRSetrangefog {
    type Import = MpCgameImport;
    type Args = CgRSetrangefogArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_SETRANGEFOG;
}

impl EncodeSysCall for CgRSetrangefog {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.range())])
    }
}

impl DecodeSysCallReturn for CgRSetrangefog {
    fn decode_return(_word: isize) -> Self::Output {}
}
