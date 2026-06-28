use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `COM_SETORGANGLES` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:168`
pub struct ComSetorgangles;

impl OutboundSysCall for ComSetorgangles {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::COM_SETORGANGLES;
}
