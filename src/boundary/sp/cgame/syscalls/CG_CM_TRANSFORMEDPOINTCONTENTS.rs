use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CM_TRANSFORMEDPOINTCONTENTS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:86`
pub struct CgCmTransformedpointcontents;

impl OutboundSysCall for CgCmTransformedpointcontents {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_TRANSFORMEDPOINTCONTENTS;
}
