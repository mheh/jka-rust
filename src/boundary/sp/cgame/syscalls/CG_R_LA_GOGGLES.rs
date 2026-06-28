use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_LA_GOGGLES` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:148`
pub struct CgRLaGoggles;

impl OutboundSysCall for CgRLaGoggles {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_LA_GOGGLES;
}
