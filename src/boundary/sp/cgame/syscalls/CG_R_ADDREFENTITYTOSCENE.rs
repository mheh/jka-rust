use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_ADDREFENTITYTOSCENE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:132`
pub struct CgRAddrefentitytoscene;

impl OutboundSysCall for CgRAddrefentitytoscene {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_ADDREFENTITYTOSCENE;
}
