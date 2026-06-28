use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CIN_RUNCINEMATIC` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:187`
pub struct CgCinRuncinematic;

impl OutboundSysCall for CgCinRuncinematic {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_CIN_RUNCINEMATIC;
}
