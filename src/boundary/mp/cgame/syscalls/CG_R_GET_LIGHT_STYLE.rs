use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_GET_LIGHT_STYLE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:168`
pub struct CgRGetLightStyle;

impl OutboundSysCall for CgRGetLightStyle {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_GET_LIGHT_STYLE;
}
