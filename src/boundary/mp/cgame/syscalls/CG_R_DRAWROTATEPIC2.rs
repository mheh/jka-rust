use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_DRAWROTATEPIC2` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:164`
pub struct CgRDrawrotatepic2;

impl OutboundSysCall for CgRDrawrotatepic2 {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_DRAWROTATEPIC2;
}
