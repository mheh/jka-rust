use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_HAVEWEGHOULMODELS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:174`
pub struct CgG2Haveweghoulmodels;

impl OutboundSysCall for CgG2Haveweghoulmodels {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_G2_HAVEWEGHOULMODELS;
}
