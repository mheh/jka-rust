use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_UI_GETITEMTEXT` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:207`
pub struct CgUiGetitemtext;

impl OutboundSysCall for CgUiGetitemtext {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_GETITEMTEXT;
}
