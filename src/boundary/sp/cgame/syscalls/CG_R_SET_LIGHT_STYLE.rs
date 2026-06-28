use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_SET_LIGHT_STYLE` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:181`
pub struct CgRSetLightStyle;

impl OutboundSysCall for CgRSetLightStyle {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_SET_LIGHT_STYLE;
}
