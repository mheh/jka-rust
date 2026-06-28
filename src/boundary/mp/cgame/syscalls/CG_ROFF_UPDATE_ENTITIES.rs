use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ROFF_UPDATE_ENTITIES` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:243`
pub struct CgRoffUpdateEntities;

impl OutboundSysCall for CgRoffUpdateEntities {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_ROFF_UPDATE_ENTITIES;
}
