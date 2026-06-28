use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_STARTSOUND` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:91`
pub struct CgSStartsound;

impl OutboundSysCall for CgSStartsound {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_STARTSOUND;
}
