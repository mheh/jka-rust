use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_FX_SET_REFDEF` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:228`
pub struct CgFxSetRefdef;

impl OutboundSysCall for CgFxSetRefdef {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_SET_REFDEF;
}
