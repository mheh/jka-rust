use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_MILLISECONDS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:63`
pub struct CgMilliseconds;

impl OutboundSysCall for CgMilliseconds {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_MILLISECONDS;
}
