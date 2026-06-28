use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CIN_RUNCINEMATIC` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:212`
pub struct CgCinRuncinematic;

impl OutboundSysCall for CgCinRuncinematic {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CIN_RUNCINEMATIC;
}
