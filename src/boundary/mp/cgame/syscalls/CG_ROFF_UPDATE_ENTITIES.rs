use super::super::MpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_ROFF_UPDATE_ENTITIES`.
///
/// Raven wrapper: `syscall( CG_ROFF_UPDATE_ENTITIES );`
/// Raven transport: `theROFFSystem.UpdateEntities(qtrue); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:735-737`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2431`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1271-1273`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgRoffUpdateEntitiesArgs;

impl CgRoffUpdateEntitiesArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_ROFF_UPDATE_ENTITIES` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:243`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:735-737`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1271-1273`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1271-1273`
pub struct CgRoffUpdateEntities;

impl OutboundSysCall for CgRoffUpdateEntities {
    type Import = MpCgameImport;
    type Args = CgRoffUpdateEntitiesArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_ROFF_UPDATE_ENTITIES;
}

impl EncodeSysCall for CgRoffUpdateEntities {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgRoffUpdateEntities {
    fn decode_return(_word: isize) -> Self::Output {}
}
