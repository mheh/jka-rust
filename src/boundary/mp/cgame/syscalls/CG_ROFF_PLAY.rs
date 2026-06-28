use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ROFF_PLAY` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:245`
pub struct CgRoffPlay;

impl OutboundSysCall for CgRoffPlay {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_ROFF_PLAY;
}
