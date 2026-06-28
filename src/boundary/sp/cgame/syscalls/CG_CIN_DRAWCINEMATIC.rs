use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CIN_DRAWCINEMATIC` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:188`
pub struct CgCinDrawcinematic;

impl OutboundSysCall for CgCinDrawcinematic {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_CIN_DRAWCINEMATIC;
}
