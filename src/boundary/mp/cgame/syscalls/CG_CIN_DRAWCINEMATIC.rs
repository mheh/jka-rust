use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CIN_DRAWCINEMATIC` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:213`
pub struct CgCinDrawcinematic;

impl OutboundSysCall for CgCinDrawcinematic {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CIN_DRAWCINEMATIC;
}
