use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_ASIN` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:147`
pub struct CgameAsin;

impl OutboundSysCall for CgameAsin {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_ASIN;
}
