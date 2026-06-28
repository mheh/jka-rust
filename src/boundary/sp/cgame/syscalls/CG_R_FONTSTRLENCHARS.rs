use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_FONTSTRLENCHARS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:124`
pub struct CgRFontstrlenchars;

impl OutboundSysCall for CgRFontstrlenchars {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_FONTSTRLENCHARS;
}
