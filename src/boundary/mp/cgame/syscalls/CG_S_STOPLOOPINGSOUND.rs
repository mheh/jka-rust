use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_S_STOPLOOPINGSOUND`.
///
/// Raven wrapper: `syscall( CG_S_STOPLOOPINGSOUND, entityNum );`
/// Raven transport: `S_StopLoopingSound( args[1] ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:216-217`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2222`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:828-830`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSStoploopingsoundArgs {
    entity_num: c_int,
}

impl CgSStoploopingsoundArgs {
    pub const fn new(entity_num: c_int) -> Self {
        Self { entity_num }
    }
}

/// `CG_S_STOPLOOPINGSOUND` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:103`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:216-217`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:828-830`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:828-830`
pub struct CgSStoploopingsound;

impl OutboundSysCall for CgSStoploopingsound {
    type Import = MpCgameImport;
    type Args = CgSStoploopingsoundArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_STOPLOOPINGSOUND;
}

impl EncodeSysCall for CgSStoploopingsound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.entity_num as isize])
    }
}

impl DecodeSysCallReturn for CgSStoploopingsound {
    fn decode_return(_word: isize) -> Self::Output {}
}
