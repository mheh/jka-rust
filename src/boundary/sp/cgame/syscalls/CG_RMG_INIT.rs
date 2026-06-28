use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_RMG_INIT` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:78`
pub struct CgRmgInit;

impl OutboundSysCall for CgRmgInit {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_RMG_INIT;
}
