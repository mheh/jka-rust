use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_SETUSERCMDANGLES` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:161`
pub struct CgSetusercmdangles;

impl OutboundSysCall for CgSetusercmdangles {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_SETUSERCMDANGLES;
}
