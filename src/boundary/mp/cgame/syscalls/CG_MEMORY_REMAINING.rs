use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_MEMORY_REMAINING` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:193`
pub struct CgMemoryRemaining;

impl OutboundSysCall for CgMemoryRemaining {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_MEMORY_REMAINING;
}
