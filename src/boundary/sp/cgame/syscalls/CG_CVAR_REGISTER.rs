use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CVAR_REGISTER` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:64`
pub struct CgCvarRegister;

impl OutboundSysCall for CgCvarRegister {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_CVAR_REGISTER;
}
