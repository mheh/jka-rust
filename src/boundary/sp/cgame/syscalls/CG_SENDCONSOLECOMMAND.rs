use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_SENDCONSOLECOMMAND` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:74`
pub struct CgSendconsolecommand;

impl OutboundSysCall for CgSendconsolecommand {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_SENDCONSOLECOMMAND;
}
