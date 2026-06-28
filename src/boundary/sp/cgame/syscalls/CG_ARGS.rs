use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ARGS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:69`
pub struct CgArgs;

impl OutboundSysCall for CgArgs {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_ARGS;
}
