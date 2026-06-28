use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ROFF_PURGE_ENT` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:246`
pub struct CgRoffPurgeEnt;

impl OutboundSysCall for CgRoffPurgeEnt {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_ROFF_PURGE_ENT;
}
