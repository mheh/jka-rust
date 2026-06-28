use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CVAR_UPDATE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:65`
pub struct CgCvarUpdate;

impl OutboundSysCall for CgCvarUpdate {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_CVAR_UPDATE;
}
