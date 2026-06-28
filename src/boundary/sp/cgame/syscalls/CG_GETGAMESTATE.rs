use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETGAMESTATE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:151`
pub struct CgGetgamestate;

impl OutboundSysCall for CgGetgamestate {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETGAMESTATE;
}
