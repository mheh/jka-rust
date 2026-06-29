use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_GETDISTANCECULL`.
///
/// Raven wrapper: `syscall(CG_R_GETDISTANCECULL, f);`
/// Raven transport: writes `tr.distanceCull` to `(float *)VMA(1)`, then returns
/// 0.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:426-428`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2301`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1058-1064`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRGetdistancecullArgs {
    f: *mut f32,
}

impl CgRGetdistancecullArgs {
    pub const fn new(f: *mut f32) -> Self {
        Self { f }
    }
}

/// `CG_R_GETDISTANCECULL` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:171`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:426-428`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1058-1064`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1058-1064`
pub struct CgRGetdistancecull;

impl OutboundSysCall for CgRGetdistancecull {
    type Import = MpCgameImport;
    type Args = CgRGetdistancecullArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_GETDISTANCECULL;
}

impl EncodeSysCall for CgRGetdistancecull {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.f)])
    }
}

impl DecodeSysCallReturn for CgRGetdistancecull {
    fn decode_return(_word: isize) -> Self::Output {}
}
