use super::super::MpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_AS_PARSESETS`.
///
/// Raven wrapper: `syscall(CG_AS_PARSESETS);`
/// Raven transport: `AS_ParseSets(); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:242-244`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2240`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:849-851`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgAsParsesetsArgs;

impl CgAsParsesetsArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_AS_PARSESETS` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:111`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:242-244`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:849-851`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:849-851`
pub struct CgAsParsesets;

impl OutboundSysCall for CgAsParsesets {
    type Import = MpCgameImport;
    type Args = CgAsParsesetsArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_AS_PARSESETS;
}

impl EncodeSysCall for CgAsParsesets {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgAsParsesets {
    fn decode_return(_word: isize) -> Self::Output {}
}
