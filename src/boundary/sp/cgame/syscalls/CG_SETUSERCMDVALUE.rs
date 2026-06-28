use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_SETUSERCMDVALUE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:160`
pub struct CgSetusercmdvalue;

impl OutboundSysCall for CgSetusercmdvalue {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_SETUSERCMDVALUE;
}
