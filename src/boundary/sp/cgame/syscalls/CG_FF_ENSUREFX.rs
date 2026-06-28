use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FF_ENSUREFX` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:109`
pub struct CgFfEnsurefx;

impl OutboundSysCall for CgFfEnsurefx {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_FF_ENSUREFX;
}
