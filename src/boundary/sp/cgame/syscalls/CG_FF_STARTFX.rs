use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FF_STARTFX` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:108`
pub struct CgFfStartfx;

impl OutboundSysCall for CgFfStartfx {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_FF_STARTFX;
}
