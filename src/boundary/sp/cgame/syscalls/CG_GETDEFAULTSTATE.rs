use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETDEFAULTSTATE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:155`
pub struct CgGetdefaultstate;

impl OutboundSysCall for CgGetdefaultstate {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETDEFAULTSTATE;
}
