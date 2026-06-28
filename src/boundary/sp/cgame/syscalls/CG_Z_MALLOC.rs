use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_Z_MALLOC` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:190`
pub struct CgZMalloc;

impl OutboundSysCall for CgZMalloc {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_Z_MALLOC;
}
