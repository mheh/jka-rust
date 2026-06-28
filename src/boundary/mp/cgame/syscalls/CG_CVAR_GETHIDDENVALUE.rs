use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CVAR_GETHIDDENVALUE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:69`
pub struct CgCvarGethiddenvalue;

impl OutboundSysCall for CgCvarGethiddenvalue {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CVAR_GETHIDDENVALUE;
}
