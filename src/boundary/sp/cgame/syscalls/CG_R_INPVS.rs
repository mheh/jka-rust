use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_INPVS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:134`
pub struct CgRInpvs;

impl OutboundSysCall for CgRInpvs {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_INPVS;
}
