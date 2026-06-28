use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_ANGLEVECTORS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:138`
pub struct CgameAnglevectors;

impl OutboundSysCall for CgameAnglevectors {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_ANGLEVECTORS;
}
