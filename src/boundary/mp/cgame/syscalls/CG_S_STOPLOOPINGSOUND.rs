use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_STOPLOOPINGSOUND` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:103`
pub struct CgSStoploopingsound;

impl OutboundSysCall for CgSStoploopingsound {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_STOPLOOPINGSOUND;
}
