use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_ADDLOOPINGSOUND` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:100`
pub struct CgSAddloopingsound;

impl OutboundSysCall for CgSAddloopingsound {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_ADDLOOPINGSOUND;
}
