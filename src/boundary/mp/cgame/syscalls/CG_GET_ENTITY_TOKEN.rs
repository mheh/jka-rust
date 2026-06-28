use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GET_ENTITY_TOKEN` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:216`
pub struct CgGetEntityToken;

impl OutboundSysCall for CgGetEntityToken {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_GET_ENTITY_TOKEN;
}
