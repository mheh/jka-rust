use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_SENDCLIENTCOMMAND` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:76`
pub struct CgSendclientcommand;

impl OutboundSysCall for CgSendclientcommand {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_SENDCLIENTCOMMAND;
}
