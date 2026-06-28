use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ADDCOMMAND` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:75`
pub struct CgAddcommand;

impl OutboundSysCall for CgAddcommand {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_ADDCOMMAND;
}
