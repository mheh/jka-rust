use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CIN_SETEXTENTS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:189`
pub struct CgCinSetextents;

impl OutboundSysCall for CgCinSetextents {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_CIN_SETEXTENTS;
}
