use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_SETCOLOR` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:159`
pub struct CgRSetcolor;

impl OutboundSysCall for CgRSetcolor {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_SETCOLOR;
}
