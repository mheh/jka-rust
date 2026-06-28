use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_REMOVECOMMAND` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:80`
pub struct CgRemovecommand;

impl OutboundSysCall for CgRemovecommand {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_REMOVECOMMAND;
}
