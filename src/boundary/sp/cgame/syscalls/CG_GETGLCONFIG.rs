use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETGLCONFIG` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:150`
pub struct CgGetglconfig;

impl OutboundSysCall for CgGetglconfig {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETGLCONFIG;
}
