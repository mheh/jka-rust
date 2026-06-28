use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_CLEARLOOPINGSOUNDS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:93`
pub struct CgSClearloopingsounds;

impl OutboundSysCall for CgSClearloopingsounds {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_CLEARLOOPINGSOUNDS;
}
