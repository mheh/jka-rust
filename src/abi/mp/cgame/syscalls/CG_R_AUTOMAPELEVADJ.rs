use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CG_R_AUTOMAPELEVADJ`.
///
/// Raven: automap elevation setting -rww.
/// Raven wrapper: `syscall( CG_R_AUTOMAPELEVADJ, PASSFLOAT(newHeight) );`
/// Raven transport: `R_AutomapElevationAdjustment(VMF(1)); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:438-441`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2304`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1075-1077`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRAutomapelevadjArgs {
    new_height: f32,
}

impl CgRAutomapelevadjArgs {
    pub const fn new(new_height: f32) -> Self {
        Self { new_height }
    }

    pub const fn new_height(&self) -> f32 {
        self.new_height
    }
}

/// `CG_R_AUTOMAPELEVADJ` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:174`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:438-441`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1075-1077`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1075-1077`
pub struct CgRAutomapelevadj;

impl OutboundSysCall for CgRAutomapelevadj {
    type Import = MpCgameImport;
    type Args = CgRAutomapelevadjArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_AUTOMAPELEVADJ;
}

impl EncodeSysCall for CgRAutomapelevadj {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.new_height())])
    }
}

impl DecodeSysCallReturn for CgRAutomapelevadj {
    fn decode_return(_word: isize) -> Self::Output {}
}
