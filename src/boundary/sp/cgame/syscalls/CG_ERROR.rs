use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ERROR` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:62`
pub struct CgError;

impl OutboundSysCall for CgError {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_ERROR;
}
