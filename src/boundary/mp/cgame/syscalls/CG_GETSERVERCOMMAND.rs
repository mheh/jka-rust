use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETSERVERCOMMAND` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:184`
pub struct CgGetservercommand;

impl OutboundSysCall for CgGetservercommand {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETSERVERCOMMAND;
}
