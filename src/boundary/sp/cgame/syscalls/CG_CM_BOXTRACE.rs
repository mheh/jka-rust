use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CM_BOXTRACE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:87`
pub struct CgCmBoxtrace;

impl OutboundSysCall for CgCmBoxtrace {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_BOXTRACE;
}
