use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_SETRANGEFOG` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:147`
pub struct CgRSetrangefog;

impl OutboundSysCall for CgRSetrangefog {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_SETRANGEFOG;
}
