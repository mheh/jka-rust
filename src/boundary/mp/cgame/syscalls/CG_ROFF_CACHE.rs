use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ROFF_CACHE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:244`
pub struct CgRoffCache;

impl OutboundSysCall for CgRoffCache {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_ROFF_CACHE;
}
