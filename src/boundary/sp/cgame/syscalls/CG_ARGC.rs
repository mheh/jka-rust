use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ARGC` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:67`
pub struct CgArgc;

impl OutboundSysCall for CgArgc {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_ARGC;
}
