use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_DRAWROTATEPIC` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:163`
pub struct CgRDrawrotatepic;

impl OutboundSysCall for CgRDrawrotatepic {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_DRAWROTATEPIC;
}
