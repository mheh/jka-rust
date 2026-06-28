use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_SETCOLOR` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:140`
pub struct CgRSetcolor;

impl OutboundSysCall for CgRSetcolor {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_SETCOLOR;
}
