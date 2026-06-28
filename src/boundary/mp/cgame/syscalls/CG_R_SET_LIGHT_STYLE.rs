use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_SET_LIGHT_STYLE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:169`
pub struct CgRSetLightStyle;

impl OutboundSysCall for CgRSetLightStyle {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_SET_LIGHT_STYLE;
}
