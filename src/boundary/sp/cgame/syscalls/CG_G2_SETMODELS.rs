use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_SETMODELS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:175`
pub struct CgG2Setmodels;

impl OutboundSysCall for CgG2Setmodels {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_G2_SETMODELS;
}
