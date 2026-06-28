use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_STOPSOUNDS` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:95`
pub struct CgSStopsounds;

impl OutboundSysCall for CgSStopsounds {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_STOPSOUNDS;
}
