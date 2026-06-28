use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_PC_LOAD_GLOBAL_DEFINES` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:204`
pub struct CgPcLoadGlobalDefines;

impl OutboundSysCall for CgPcLoadGlobalDefines {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_LOAD_GLOBAL_DEFINES;
}
