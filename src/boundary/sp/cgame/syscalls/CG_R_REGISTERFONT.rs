use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_REGISTERFONT` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:122`
pub struct CgRRegisterfont;

impl OutboundSysCall for CgRRegisterfont {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_REGISTERFONT;
}
