use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_AS_ADDENTRY` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:165`
pub struct CgAsAddentry;

impl OutboundSysCall for CgAsAddentry {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_AS_ADDENTRY;
}
