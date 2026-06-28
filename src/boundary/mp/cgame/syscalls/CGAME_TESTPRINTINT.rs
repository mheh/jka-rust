use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_TESTPRINTINT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:143`
pub struct CgameTestprintint;

impl OutboundSysCall for CgameTestprintint {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_TESTPRINTINT;
}
