use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_UPDATEENTITYPOSITION` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:96`
pub struct CgSUpdateentityposition;

impl OutboundSysCall for CgSUpdateentityposition {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_UPDATEENTITYPOSITION;
}
