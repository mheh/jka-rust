use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_DRAWROTATEPIC2` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:146`
pub struct CgRDrawrotatepic2;

impl OutboundSysCall for CgRDrawrotatepic2 {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_DRAWROTATEPIC2;
}
