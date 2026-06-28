use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_SENDCLIENTCOMMAND` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:81`
pub struct CgSendclientcommand;

impl OutboundSysCall for CgSendclientcommand {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_SENDCLIENTCOMMAND;
}
