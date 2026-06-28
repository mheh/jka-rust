use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CIN_STOPCINEMATIC` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:186`
pub struct CgCinStopcinematic;

impl OutboundSysCall for CgCinStopcinematic {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_CIN_STOPCINEMATIC;
}
