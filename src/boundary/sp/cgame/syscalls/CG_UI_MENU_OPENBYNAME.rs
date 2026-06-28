use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_UI_MENU_OPENBYNAME` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:195`
pub struct CgUiMenuOpenbyname;

impl OutboundSysCall for CgUiMenuOpenbyname {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_MENU_OPENBYNAME;
}
