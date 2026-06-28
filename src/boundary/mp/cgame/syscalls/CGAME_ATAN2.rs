use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_ATAN2` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:135`
pub struct CgameAtan2;

impl OutboundSysCall for CgameAtan2 {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_ATAN2;
}
