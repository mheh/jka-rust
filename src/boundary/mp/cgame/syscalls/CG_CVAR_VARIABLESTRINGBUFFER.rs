use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CVAR_VARIABLESTRINGBUFFER` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:68`
pub struct CgCvarVariablestringbuffer;

impl OutboundSysCall for CgCvarVariablestringbuffer {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CVAR_VARIABLESTRINGBUFFER;
}
