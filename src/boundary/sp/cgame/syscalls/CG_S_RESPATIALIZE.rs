use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_RESPATIALIZE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:97`
pub struct CgSRespatialize;

impl OutboundSysCall for CgSRespatialize {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_RESPATIALIZE;
}
