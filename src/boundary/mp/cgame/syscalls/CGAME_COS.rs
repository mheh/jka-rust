use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_COS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:134`
pub struct CgameCos;

impl OutboundSysCall for CgameCos {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_COS;
}
