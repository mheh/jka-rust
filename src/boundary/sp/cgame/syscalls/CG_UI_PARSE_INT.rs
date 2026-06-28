use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_UI_PARSE_INT` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:196`
pub struct CgUiParseInt;

impl OutboundSysCall for CgUiParseInt {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_PARSE_INT;
}
