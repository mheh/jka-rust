use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_CEIL` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:141`
pub struct CgameCeil;

impl OutboundSysCall for CgameCeil {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_CEIL;
}
