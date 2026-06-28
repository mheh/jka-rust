use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CIN_STOPCINEMATIC` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:211`
pub struct CgCinStopcinematic;

impl OutboundSysCall for CgCinStopcinematic {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CIN_STOPCINEMATIC;
}
