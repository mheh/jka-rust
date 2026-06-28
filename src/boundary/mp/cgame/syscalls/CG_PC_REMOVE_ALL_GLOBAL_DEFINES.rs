use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_PC_REMOVE_ALL_GLOBAL_DEFINES` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:205`
pub struct CgPcRemoveAllGlobalDefines;

impl OutboundSysCall for CgPcRemoveAllGlobalDefines {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_REMOVE_ALL_GLOBAL_DEFINES;
}
