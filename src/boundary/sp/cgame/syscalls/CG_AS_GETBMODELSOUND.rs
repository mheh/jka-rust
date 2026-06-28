use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_AS_GETBMODELSOUND` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:166`
pub struct CgAsGetbmodelsound;

impl OutboundSysCall for CgAsGetbmodelsound {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_AS_GETBMODELSOUND;
}
