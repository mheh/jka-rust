use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_PRINT` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:61`
pub struct CgPrint;

impl OutboundSysCall for CgPrint {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_PRINT;
}
