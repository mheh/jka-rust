use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CVAR_SET` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:67`
pub struct CgCvarSet;

impl OutboundSysCall for CgCvarSet {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CVAR_SET;
}
