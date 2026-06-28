use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CIN_PLAYCINEMATIC` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:185`
pub struct CgCinPlaycinematic;

impl OutboundSysCall for CgCinPlaycinematic {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_CIN_PLAYCINEMATIC;
}
