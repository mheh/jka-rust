use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_Z_FREE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:191`
pub struct CgZFree;

impl OutboundSysCall for CgZFree {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_Z_FREE;
}
