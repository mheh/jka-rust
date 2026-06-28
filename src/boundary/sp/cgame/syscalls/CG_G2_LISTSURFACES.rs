use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_LISTSURFACES` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:173`
pub struct CgG2Listsurfaces;

impl OutboundSysCall for CgG2Listsurfaces {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_G2_LISTSURFACES;
}
