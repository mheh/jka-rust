use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_ACOS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:146`
pub struct CgameAcos;

impl OutboundSysCall for CgameAcos {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_ACOS;
}
