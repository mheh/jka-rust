use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETSNAPSHOT` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:153`
pub struct CgGetsnapshot;

impl OutboundSysCall for CgGetsnapshot {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETSNAPSHOT;
}
