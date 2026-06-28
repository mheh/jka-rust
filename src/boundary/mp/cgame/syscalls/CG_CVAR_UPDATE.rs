use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CVAR_UPDATE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:66`
pub struct CgCvarUpdate;

impl OutboundSysCall for CgCvarUpdate {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CVAR_UPDATE;
}
