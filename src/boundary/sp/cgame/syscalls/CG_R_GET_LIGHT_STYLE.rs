use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_GET_LIGHT_STYLE` SP cgame imports syscall boundary token.
///
/// Raven: Ghoul2 Insert End
/// Source: `oracle/oracle/code/cgame/cg_public.h:180`
pub struct CgRGetLightStyle;

impl OutboundSysCall for CgRGetLightStyle {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_GET_LIGHT_STYLE;
}
