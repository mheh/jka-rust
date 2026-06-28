use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_CLEARSCENE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:131`
pub struct CgRClearscene;

impl OutboundSysCall for CgRClearscene {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_CLEARSCENE;
}
