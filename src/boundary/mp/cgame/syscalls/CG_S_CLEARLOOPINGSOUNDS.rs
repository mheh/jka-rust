use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_CLEARLOOPINGSOUNDS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:99`
pub struct CgSClearloopingsounds;

impl OutboundSysCall for CgSClearloopingsounds {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_CLEARLOOPINGSOUNDS;
}
