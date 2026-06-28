use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FX_RESET` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:232`
pub struct CgFxReset;

impl OutboundSysCall for CgFxReset {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_RESET;
}
