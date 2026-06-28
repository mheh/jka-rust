use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ROFF_CLEAN` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:242`
pub struct CgRoffClean;

impl OutboundSysCall for CgRoffClean {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_ROFF_CLEAN;
}
