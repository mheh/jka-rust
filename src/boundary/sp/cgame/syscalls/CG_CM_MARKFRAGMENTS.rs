use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CM_MARKFRAGMENTS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:89`
pub struct CgCmMarkfragments;

impl OutboundSysCall for CgCmMarkfragments {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_MARKFRAGMENTS;
}
