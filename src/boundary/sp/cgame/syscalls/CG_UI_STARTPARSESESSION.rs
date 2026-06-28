use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_UI_STARTPARSESESSION` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:199`
pub struct CgUiStartparsesession;

impl OutboundSysCall for CgUiStartparsesession {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_STARTPARSESESSION;
}
