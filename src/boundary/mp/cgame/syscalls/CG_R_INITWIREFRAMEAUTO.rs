use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_INITWIREFRAMEAUTO` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:175`
pub struct CgRInitwireframeauto;

impl OutboundSysCall for CgRInitwireframeauto {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_INITWIREFRAMEAUTO;
}
