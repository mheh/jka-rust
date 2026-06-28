use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CGAME_FLOOR` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:140`
pub struct CgameFloor;

impl OutboundSysCall for CgameFloor {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_FLOOR;
}
