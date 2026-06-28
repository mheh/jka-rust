use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_SETREFRACTIONPROP` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:130`
pub struct CgRSetrefractionprop;

impl OutboundSysCall for CgRSetrefractionprop {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_SETREFRACTIONPROP;
}
