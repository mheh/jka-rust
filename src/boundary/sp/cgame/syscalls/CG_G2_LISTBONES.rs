use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_LISTBONES` SP cgame imports syscall boundary token.
///
/// Raven: Ghoul2 Insert Start
/// Source: `oracle/oracle/code/cgame/cg_public.h:172`
pub struct CgG2Listbones;

impl OutboundSysCall for CgG2Listbones {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_G2_LISTBONES;
}
