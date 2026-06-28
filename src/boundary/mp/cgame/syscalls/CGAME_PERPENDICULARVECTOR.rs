use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_PERPENDICULARVECTOR` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:139`
pub struct CgamePerpendicularvector;

impl OutboundSysCall for CgamePerpendicularvector {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_PERPENDICULARVECTOR;
}
